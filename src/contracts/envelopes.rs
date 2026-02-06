//! ITER-PAR-01: Contract Envelopes
//!
//! Typed envelopes for cognitive and learning contracts.
//! Maps directly to SCG-CTX-03/INT-04 audit surfaces.

use serde::{Deserialize, Serialize};

use super::validation::{validate_bounded_float, validate_hash, ContractError};

/// Energy envelope - thermodynamic state summary.
///
/// # Fields
/// All energy values are non-negative. Integrity in [0.0, 1.0].
#[cfg_attr(feature = "schema-gen", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EnergyEnvelope {
    /// Total energy across all nodes
    pub nodes: f64,
    /// Energy in reservoir
    pub reservoir: f64,
    /// System integrity ratio [0.0, 1.0]
    pub integrity: f64,
}

impl EnergyEnvelope {
    /// Construct with validation. Rejects NaN/Inf.
    pub fn new(nodes: f64, reservoir: f64, integrity: f64) -> Result<Self, ContractError> {
        let nodes = validate_bounded_float(nodes, 0.0, f64::MAX, "energy.nodes")?;
        let reservoir = validate_bounded_float(reservoir, 0.0, f64::MAX, "energy.reservoir")?;
        let integrity = validate_bounded_float(integrity, 0.0, 1.0, "energy.integrity")?;
        Ok(Self {
            nodes,
            reservoir,
            integrity,
        })
    }
}

/// Reasoning envelope - cortex output summary.
///
/// # Fields
/// All signals in [0.0, 1.0].
#[cfg_attr(feature = "schema-gen", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReasoningEnvelope {
    /// Reasoning quality [0.0, 1.0] - payment ratio
    pub quality: f64,
    /// Value signal from cortex [0.0, 1.0]
    pub value_signal: f64,
    /// Conflict detection signal [0.0, 1.0]
    pub conflict_signal: f64,
    /// Control signal [0.0, 1.0]
    pub control_signal: f64,
}

impl ReasoningEnvelope {
    /// Construct with validation. Rejects NaN/Inf.
    pub fn new(
        quality: f64,
        value_signal: f64,
        conflict_signal: f64,
        control_signal: f64,
    ) -> Result<Self, ContractError> {
        let quality = validate_bounded_float(quality, 0.0, 1.0, "reasoning.quality")?;
        let value_signal =
            validate_bounded_float(value_signal, 0.0, 1.0, "reasoning.value_signal")?;
        let conflict_signal =
            validate_bounded_float(conflict_signal, 0.0, 1.0, "reasoning.conflict_signal")?;
        let control_signal =
            validate_bounded_float(control_signal, 0.0, 1.0, "reasoning.control_signal")?;
        Ok(Self {
            quality,
            value_signal,
            conflict_signal,
            control_signal,
        })
    }

    /// Create a starved reasoning envelope (quality degraded, signals zeroed).
    pub fn starved(quality: f64) -> Result<Self, ContractError> {
        Self::new(quality, 0.0, 0.0, 0.0)
    }
}

/// Learning update status - closed enum per SCG-INT-04.
///
/// Unknown values fail closed.
#[cfg_attr(feature = "schema-gen", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LearningStatus {
    /// Update was successfully committed
    Committed,
    /// No proposal generated (value delta below threshold)
    NoProposalNoDelta,
    /// Rejected: cortex input quality below threshold
    RejectedInputQuality,
    /// Rejected: insufficient energy to fund update
    RejectedScarcity,
    /// Rejected: hash verification or arithmetic error
    RejectedIntegrity,
}

impl LearningStatus {
    /// Parse from string, fail closed on unknown.
    pub fn from_str_closed(s: &str) -> Result<Self, ContractError> {
        match s {
            "COMMITTED" => Ok(Self::Committed),
            "NO_PROPOSAL_NO_DELTA" => Ok(Self::NoProposalNoDelta),
            "REJECTED_INPUT_QUALITY" => Ok(Self::RejectedInputQuality),
            "REJECTED_SCARCITY" => Ok(Self::RejectedScarcity),
            "REJECTED_INTEGRITY" => Ok(Self::RejectedIntegrity),
            _ => Err(ContractError::UnknownEnum {
                field: "learning.status".to_string(),
                value: s.to_string(),
            }),
        }
    }

    /// Reason code for audit trail.
    pub fn reason_code(&self) -> &'static str {
        match self {
            Self::Committed => "COMMITTED",
            Self::NoProposalNoDelta => "NO_PROPOSAL_NO_DELTA",
            Self::RejectedInputQuality => "REJECTED_INPUT_QUALITY",
            Self::RejectedScarcity => "REJECTED_SCARCITY",
            Self::RejectedIntegrity => "REJECTED_INTEGRITY",
        }
    }
}

/// Learning envelope - capsule state and update audit.
///
/// # Fields
/// - capsule_id: Unique capsule identifier
/// - epoch: Update count
/// - version_hash: SHA-256 of capsule state (hex-encoded)
/// - update_cost/paid/quality: Economics
/// - status: Outcome of learning tick
/// - scarcity_streak: Consecutive scarcity rejections
#[cfg_attr(feature = "schema-gen", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LearningEnvelope {
    /// Capsule identifier
    pub capsule_id: String,
    /// Current epoch (update count)
    pub epoch: u64,
    /// SHA-256 hash of capsule state (hex-encoded, 64 chars)
    pub version_hash: String,
    /// Cost of proposed update
    pub update_cost: f64,
    /// Amount actually paid
    pub update_paid: f64,
    /// Payment quality (paid / cost)
    pub update_quality: f64,
    /// Outcome of learning tick
    pub status: LearningStatus,
    /// Consecutive scarcity rejections
    pub scarcity_streak: u64,
}

impl LearningEnvelope {
    /// Construct with validation. Rejects NaN/Inf, invalid hash.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        capsule_id: String,
        epoch: u64,
        version_hash: String,
        update_cost: f64,
        update_paid: f64,
        update_quality: f64,
        status: LearningStatus,
        scarcity_streak: u64,
    ) -> Result<Self, ContractError> {
        validate_hash(&version_hash, "learning.version_hash")?;
        let update_cost =
            validate_bounded_float(update_cost, 0.0, f64::MAX, "learning.update_cost")?;
        let update_paid =
            validate_bounded_float(update_paid, 0.0, f64::MAX, "learning.update_paid")?;
        let update_quality =
            validate_bounded_float(update_quality, 0.0, 1.0, "learning.update_quality")?;

        Ok(Self {
            capsule_id,
            epoch,
            version_hash,
            update_cost,
            update_paid,
            update_quality,
            status,
            scarcity_streak,
        })
    }

    /// Create default (no learning) envelope.
    pub fn default_no_learning() -> Self {
        Self {
            capsule_id: String::new(),
            epoch: 0,
            version_hash: "0".repeat(64),
            update_cost: 0.0,
            update_paid: 0.0,
            update_quality: 0.0,
            status: LearningStatus::NoProposalNoDelta,
            scarcity_streak: 0,
        }
    }
}

/// Policy decision - closed enum.
#[cfg_attr(feature = "schema-gen", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PolicyDecision {
    /// Action is allowed
    Allow,
    /// Action is denied
    Deny,
    /// Learning is frozen
    FreezeLearning,
    /// Degraded mode (quality below threshold)
    DegradedMode,
    /// Requires external review
    RequireReview,
}

impl PolicyDecision {
    /// Parse from string, fail closed on unknown.
    pub fn from_str_closed(s: &str) -> Result<Self, ContractError> {
        match s {
            "ALLOW" => Ok(Self::Allow),
            "DENY" => Ok(Self::Deny),
            "FREEZE_LEARNING" => Ok(Self::FreezeLearning),
            "DEGRADED_MODE" => Ok(Self::DegradedMode),
            "REQUIRE_REVIEW" => Ok(Self::RequireReview),
            _ => Err(ContractError::UnknownEnum {
                field: "policy.decision".to_string(),
                value: s.to_string(),
            }),
        }
    }
}

/// Policy envelope - governance decision with audit trail.
#[cfg_attr(feature = "schema-gen", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PolicyEnvelope {
    /// SHA-256 hash of policy configuration (hex-encoded)
    pub policy_hash: String,
    /// Final decision
    pub decision: PolicyDecision,
    /// Reason codes for decision (ordered)
    pub reason_codes: Vec<String>,
}

impl PolicyEnvelope {
    /// Construct with validation.
    pub fn new(
        policy_hash: String,
        decision: PolicyDecision,
        reason_codes: Vec<String>,
    ) -> Result<Self, ContractError> {
        validate_hash(&policy_hash, "policy.policy_hash")?;
        Ok(Self {
            policy_hash,
            decision,
            reason_codes,
        })
    }
}

/// System state - complete snapshot for governance.
///
/// # Invariants
/// - INV-ITER-02: Identical SystemState + config => identical decision
#[cfg_attr(feature = "schema-gen", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SystemState {
    /// Current tick
    pub tick: u64,
    /// Energy snapshot
    pub energy: EnergyEnvelope,
    /// Reasoning signature
    pub reasoning: ReasoningEnvelope,
    /// Learning audit
    pub learning: LearningEnvelope,
    /// Policy decision
    pub policy: PolicyEnvelope,
}

impl SystemState {
    /// Construct system state (all sub-envelopes pre-validated).
    pub fn new(
        tick: u64,
        energy: EnergyEnvelope,
        reasoning: ReasoningEnvelope,
        learning: LearningEnvelope,
        policy: PolicyEnvelope,
    ) -> Self {
        Self {
            tick,
            energy,
            reasoning,
            learning,
            policy,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn energy_envelope_rejects_nan() {
        let result = EnergyEnvelope::new(f64::NAN, 0.0, 1.0);
        assert!(result.is_err());
    }

    #[test]
    fn energy_envelope_rejects_inf() {
        let result = EnergyEnvelope::new(0.0, f64::INFINITY, 1.0);
        assert!(result.is_err());
    }

    #[test]
    fn energy_envelope_rejects_negative() {
        let result = EnergyEnvelope::new(-1.0, 0.0, 1.0);
        assert!(result.is_err());
    }

    #[test]
    fn energy_envelope_accepts_valid() {
        let result = EnergyEnvelope::new(100.0, 10.0, 0.95);
        assert!(result.is_ok());
    }

    #[test]
    fn reasoning_envelope_rejects_out_of_range() {
        let result = ReasoningEnvelope::new(1.5, 0.5, 0.5, 0.5);
        assert!(result.is_err());
    }

    #[test]
    fn learning_status_fail_closed() {
        let result = LearningStatus::from_str_closed("UNKNOWN_STATUS");
        assert!(result.is_err());
    }

    #[test]
    fn learning_status_parses_valid() {
        assert_eq!(
            LearningStatus::from_str_closed("COMMITTED").unwrap(),
            LearningStatus::Committed
        );
        assert_eq!(
            LearningStatus::from_str_closed("REJECTED_SCARCITY").unwrap(),
            LearningStatus::RejectedScarcity
        );
    }

    #[test]
    fn learning_envelope_validates_hash() {
        let bad_hash = "not_a_valid_hash";
        let result = LearningEnvelope::new(
            "cap1".to_string(),
            1,
            bad_hash.to_string(),
            0.5,
            0.5,
            1.0,
            LearningStatus::Committed,
            0,
        );
        assert!(result.is_err());
    }

    #[test]
    fn learning_envelope_accepts_valid() {
        let good_hash = "a".repeat(64);
        let result = LearningEnvelope::new(
            "cap1".to_string(),
            1,
            good_hash,
            0.5,
            0.5,
            1.0,
            LearningStatus::Committed,
            0,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn policy_decision_fail_closed() {
        let result = PolicyDecision::from_str_closed("MAYBE");
        assert!(result.is_err());
    }
}
