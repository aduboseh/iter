//! GovernanceRuntime trait and outcome types.
//!
//! Defines the single authoritative dispatch path for all governance
//! decision endpoints. No MCP handler may compute a verdict except
//! through `GovernanceRuntime::evaluate`.
//!
//! # Modes
//!
//! - `Demo`: Threshold-based, non-authoritative, no DecisionPacket.
//! - `Governed`: PolicyEvaluator-based, authoritative, replay-sufficient.

use serde::{Deserialize, Serialize};

use crate::audit::DecisionPacket;
use crate::substrate::stub::{AuditSearchFilter, AuditSearchResult, GovernanceProposal};

/// Runtime governance mode.
///
/// Determines which semantics engine handles governance evaluation
/// and what claims are permitted at the MCP edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GovernanceMode {
    /// Threshold-based, non-authoritative, no DecisionPacket at edge.
    Demo,
    /// PolicyEvaluator-based, authoritative PDP, replay-sufficient.
    Governed,
}

impl GovernanceMode {
    /// Parse from string, fail-closed on unknown.
    pub fn from_str_closed(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "demo" => Ok(Self::Demo),
            "governed" => Ok(Self::Governed),
            _ => Err(format!(
                "unknown governance mode: '{}' (expected 'demo' or 'governed')",
                s
            )),
        }
    }
}

/// Verdict produced by governance evaluation.
///
/// Shared across demo and governed modes.
/// Serialization is uppercase for MCP responses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GovernanceVerdict {
    /// Action is admissible under current governance constraints.
    Allow,
    /// Action is blocked due to governance violation.
    Block,
    /// Action requires human review.
    Review,
}

/// Reason code with namespace enforcement.
///
/// Demo reason codes MUST use `demo.*` prefix.
/// Governed reason codes MUST use `policy.*`, `economics.*`, or `safety.*`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasonCode(String);

impl ReasonCode {
    /// Create a demo-namespace reason code.
    pub fn demo(suffix: &str) -> Self {
        Self(format!("demo.{}", suffix))
    }

    /// Create a governed-namespace reason code.
    pub fn policy(suffix: &str) -> Self {
        Self(format!("policy.{}", suffix))
    }

    /// Raw string value.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Runtime metadata exposed at the MCP edge.
///
/// Canonical source of truth for external clients about
/// what Iter is actually doing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceRuntimeMeta {
    /// Current governance mode.
    pub mode: GovernanceMode,
    /// Whether this runtime is an authoritative PDP.
    /// MUST be false in demo mode.
    pub authoritative_pdp: bool,
    /// Whether decision artifacts are replay-sufficient.
    /// MUST be false in demo mode.
    pub replay_sufficient: bool,
}

impl GovernanceRuntimeMeta {
    /// Demo mode metadata (non-authoritative, not replay-sufficient).
    pub fn demo() -> Self {
        Self {
            mode: GovernanceMode::Demo,
            authoritative_pdp: false,
            replay_sufficient: false,
        }
    }

    /// Governed mode metadata (authoritative, replay-sufficient).
    pub fn governed() -> Self {
        Self {
            mode: GovernanceMode::Governed,
            authoritative_pdp: true,
            replay_sufficient: true,
        }
    }
}

/// Canonical result type from governance evaluation.
///
/// In demo mode: packet is None, authoritative_pdp is false.
/// In governed mode: packet is Some(DecisionPacket), authoritative_pdp is true.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceOutcome {
    /// Governance verdict.
    pub verdict: GovernanceVerdict,
    /// Reason codes (namespaced by mode).
    pub reason_codes: Vec<ReasonCode>,
    /// Runtime mode that produced this outcome.
    pub mode: GovernanceMode,
    /// Policy identifier (None in demo mode).
    pub policy_id: Option<String>,
    /// Strong policy version identifier (None in demo mode).
    /// Format: "{timestamp}+sha256:{config_hash}"
    pub policy_version: Option<String>,
    /// Schema version governing packet/outcome shape.
    pub schema_version: String,
    /// DecisionPacket (None in demo mode, Some in governed mode).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub packet: Option<DecisionPacket>,
    /// Whether a policy trace is available.
    pub trace_available: bool,
    /// Whether this runtime is an authoritative PDP.
    pub authoritative_pdp: bool,
    /// Whether this outcome is replay-sufficient.
    pub replay_sufficient: bool,
}

/// Error from governance evaluation.
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
pub enum GovernanceRuntimeError {
    /// Proposal validation failed.
    #[error("proposal validation failed: {reason}")]
    ProposalInvalid {
        /// Human-readable description of the validation failure.
        reason: String,
    },
    /// Internal evaluation error.
    #[error("evaluation error: {reason}")]
    EvaluationFailed {
        /// Human-readable description of the evaluation failure.
        reason: String,
    },
    /// Mode-specific error.
    #[error("mode error: {reason}")]
    ModeError {
        /// Human-readable description of the mode error.
        reason: String,
    },
    /// SCG endpoint unavailable or returned a non-success transport status.
    #[error("SCG unavailable: {0}")]
    ScgUnavailable(String),
    /// SCG contract version did not match the expected value.
    #[error("contract version mismatch: {0}")]
    ContractVersionMismatch(String),
    /// SCG replay integrity verification failed.
    #[error("replay integrity violation: {0}")]
    ReplayIntegrityViolation(String),
    /// SCG governance hash did not match the boot-loaded canonical hash.
    #[error("governance hash mismatch: {0}")]
    GovernanceHashMismatch(String),
    /// Required runtime configuration was missing.
    #[error("configuration missing: {0}")]
    ConfigMissing(String),
}

/// Replay a DecisionPacket and verify it is still valid.
///
/// Fail-closed: rejects on checksum mismatch, policy_version mismatch,
/// or schema_version mismatch.
pub fn replay_decision(
    packet: &crate::audit::DecisionPacket,
    expected_policy_version: &str,
    expected_schema_version: &str,
) -> Result<GovernanceOutcome, GovernanceRuntimeError> {
    packet
        .verify_checksum()
        .map_err(|e| GovernanceRuntimeError::EvaluationFailed {
            reason: format!("checksum verification failed: {}", e),
        })?;

    let actual_policy_hash = packet.policy.policy_hash.clone();
    let actual_policy_version = format!("sha256:{}", actual_policy_hash);
    if actual_policy_version != expected_policy_version {
        return Err(GovernanceRuntimeError::EvaluationFailed {
            reason: format!(
                "policy_version mismatch: expected={}, actual={}",
                expected_policy_version, actual_policy_version
            ),
        });
    }

    if expected_schema_version != "decision_packet:v1" {
        return Err(GovernanceRuntimeError::EvaluationFailed {
            reason: format!(
                "schema_version mismatch: expected={}, supported=decision_packet:v1",
                expected_schema_version
            ),
        });
    }

    let verdict = match packet.policy.decision {
        crate::contracts::PolicyDecision::Allow => GovernanceVerdict::Allow,
        crate::contracts::PolicyDecision::Deny => GovernanceVerdict::Block,
        _ => GovernanceVerdict::Review,
    };

    let reason_codes = packet
        .policy
        .reason_codes
        .iter()
        .map(|r| ReasonCode::policy(r))
        .collect::<Vec<_>>();

    let reason_codes = if reason_codes.is_empty() {
        vec![ReasonCode::policy("all_gates_pass")]
    } else {
        reason_codes
    };

    Ok(GovernanceOutcome {
        verdict,
        reason_codes,
        mode: GovernanceMode::Governed,
        policy_id: Some(actual_policy_hash),
        policy_version: Some(actual_policy_version),
        schema_version: "decision_packet:v1".to_string(),
        packet: Some(packet.clone()),
        trace_available: true,
        authoritative_pdp: true,
        replay_sufficient: true,
    })
}

/// Single authoritative dispatch path for governance decisions.
///
/// All MCP decision endpoints MUST call methods on this trait.
/// No other verdict computation path is permitted.
pub trait GovernanceRuntime {
    /// Evaluate a governance proposal and return an authoritative (governed)
    /// or non-authoritative (demo) outcome.
    fn evaluate(
        &mut self,
        proposal: &GovernanceProposal,
    ) -> Result<GovernanceOutcome, GovernanceRuntimeError>;

    /// Preview a governance proposal without committing to lineage/audit.
    fn preview(
        &self,
        proposal: &GovernanceProposal,
    ) -> Result<GovernanceOutcome, GovernanceRuntimeError>;

    /// Search governance decision history.
    fn search_decisions(&self, filter: &AuditSearchFilter) -> AuditSearchResult;

    /// Current governance mode.
    fn mode(&self) -> GovernanceMode;

    /// Runtime metadata for edge introspection.
    fn meta(&self) -> GovernanceRuntimeMeta;
}
