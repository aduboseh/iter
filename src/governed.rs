//! GovernedRuntime — authoritative PDP governance path.
//!
//! PolicyEvaluator is the sole verdict source. DecisionPackets are emitted
//! for every evaluate() call. AuditLog records all decisions. Preview is
//! read-only (no packet, no audit mutation).
//!
//! # Mode invariants
//!
//! - `authoritative_pdp = true`
//! - `replay_sufficient = true` (for evaluate only; preview is false)
//! - `packet = Some(DecisionPacket)` (for evaluate only)
//! - Reason codes use `policy.*` namespace

use crate::audit::{AuditLog, DecisionPacket};
use crate::contracts::{PolicyDecision, SystemState};
use crate::economics::EconomicsConfig;
use crate::policy::{PolicyConfig, PolicyEvaluator};
use crate::runtime::{
    GovernanceMode, GovernanceOutcome, GovernanceRuntime, GovernanceRuntimeError,
    GovernanceRuntimeMeta, GovernanceVerdict, ReasonCode,
};
use crate::substrate::stub::{
    AuditSearchFilter, AuditSearchResult, DecisionSummary, GovernanceProposal, StubRuntime,
};

const GOVERNANCE_HASH: &str = include_str!("../governance/governance.hash");

/// Governed-mode runtime wrapping graph state + PolicyEvaluator + AuditLog.
///
/// PolicyEvaluator is the sole verdict source. All evaluate() calls produce
/// a DecisionPacket recorded in the AuditLog. Preview is read-only.
pub struct GovernedRuntime {
    graph: StubRuntime,
    evaluator: PolicyEvaluator,
    audit_log: AuditLog,
    economics_config: EconomicsConfig,
    governance_hash: String,
}

impl GovernedRuntime {
    /// Create a governed runtime wrapping a graph, policy config, and economics config.
    pub fn new(
        graph: StubRuntime,
        policy_config: PolicyConfig,
        economics_config: EconomicsConfig,
    ) -> Self {
        Self {
            graph,
            evaluator: PolicyEvaluator::new(policy_config),
            audit_log: AuditLog::new(),
            economics_config,
            governance_hash: GOVERNANCE_HASH.trim().to_string(),
        }
    }

    /// Mutable access to the underlying graph for node/edge operations.
    pub fn graph_mut(&mut self) -> &mut StubRuntime {
        &mut self.graph
    }

    /// Read access to the underlying graph.
    pub fn graph(&self) -> &StubRuntime {
        &self.graph
    }

    /// Read access to the audit log.
    pub fn audit_log(&self) -> &AuditLog {
        &self.audit_log
    }

    /// Map PolicyDecision to GovernanceVerdict.
    ///
    /// Allow → Allow. Deny → Block. All others → Review.
    fn map_policy_verdict(decision: PolicyDecision) -> GovernanceVerdict {
        match decision {
            PolicyDecision::Allow => GovernanceVerdict::Allow,
            PolicyDecision::Deny => GovernanceVerdict::Block,
            PolicyDecision::DegradedMode
            | PolicyDecision::FreezeLearning
            | PolicyDecision::RequireReview => GovernanceVerdict::Review,
        }
    }

    /// Build policy-namespace reason codes from PolicyResult.
    ///
    /// If no terminal rule fired (all gates passed), emits `policy.all_gates_pass`.
    fn build_reason_codes(reason_codes: &[String]) -> Vec<ReasonCode> {
        if reason_codes.is_empty() {
            return vec![ReasonCode::policy("all_gates_pass")];
        }
        reason_codes.iter().map(|r| ReasonCode::policy(r)).collect()
    }

    /// Compute strong policy version identifier.
    ///
    /// Format: `sha256:{config_hash}`
    fn policy_version(&self) -> String {
        format!("sha256:{}", self.evaluator.config().compute_hash())
    }

    fn execution_trace(policy_result: &crate::policy::PolicyResult) -> Vec<String> {
        let mut trace = Vec::with_capacity(policy_result.evaluated_rules.len() + 3);
        trace.push("governance.hash.bound".to_string());
        trace.push("policy.evaluate.start".to_string());
        trace.extend(
            policy_result
                .evaluated_rules
                .iter()
                .map(|rule| format!("policy.rule.{}", rule.to_ascii_lowercase())),
        );
        trace.push(format!(
            "policy.decision.{}",
            match policy_result.decision {
                PolicyDecision::Allow => "allow",
                PolicyDecision::Deny => "deny",
                PolicyDecision::DegradedMode => "degraded_mode",
                PolicyDecision::FreezeLearning => "freeze_learning",
                PolicyDecision::RequireReview => "require_review",
            }
        ));
        trace
    }

    /// Evaluate policy against graph state. Returns PolicyResult + governed SystemState.
    ///
    /// Extracts envelopes from graph, re-evaluates with governed PolicyEvaluator,
    /// builds a SystemState with the governed policy envelope.
    fn evaluate_state(
        &self,
    ) -> Result<(crate::policy::PolicyResult, SystemState), GovernanceRuntimeError> {
        let base_state =
            self.graph
                .system_state()
                .map_err(|e| GovernanceRuntimeError::EvaluationFailed {
                    reason: e.to_string(),
                })?;

        let policy_result = self.evaluator.evaluate(
            &base_state.reasoning,
            &base_state.learning,
            &base_state.energy,
        );

        let policy_envelope = self.evaluator.build_envelope(&policy_result).map_err(|e| {
            GovernanceRuntimeError::EvaluationFailed {
                reason: e.to_string(),
            }
        })?;

        let governed_state = SystemState::new(
            base_state.tick,
            base_state.energy,
            base_state.reasoning,
            base_state.learning,
            policy_envelope,
        );

        Ok((policy_result, governed_state))
    }
}

impl GovernanceRuntime for GovernedRuntime {
    fn evaluate(
        &mut self,
        _proposal: &GovernanceProposal,
    ) -> Result<GovernanceOutcome, GovernanceRuntimeError> {
        let (policy_result, governed_state) = self.evaluate_state()?;
        let execution_trace = Self::execution_trace(&policy_result);

        let mut packet = DecisionPacket::new(
            env!("CARGO_PKG_VERSION").to_string(),
            "stub".to_string(),
            &governed_state,
            None,
            self.economics_config.compute_hash(),
            policy_result
                .evaluated_rules
                .iter()
                .map(|s| s.to_string())
                .collect(),
        )
        .map_err(|e| GovernanceRuntimeError::EvaluationFailed {
            reason: e.to_string(),
        })?;
        packet.bind_governance_context(self.governance_hash.clone(), execution_trace);

        self.audit_log.append(&packet);
        self.graph.advance_tick();

        let verdict = Self::map_policy_verdict(policy_result.decision);
        let reason_codes = Self::build_reason_codes(&policy_result.reason_codes);
        let config_hash = self.evaluator.config().compute_hash();

        Ok(GovernanceOutcome {
            verdict,
            reason_codes,
            mode: GovernanceMode::Governed,
            policy_id: Some(config_hash),
            policy_version: Some(self.policy_version()),
            schema_version: "decision_packet:v1".to_string(),
            packet: Some(packet),
            trace_available: true,
            authoritative_pdp: true,
            replay_sufficient: true,
        })
    }

    fn preview(
        &self,
        _proposal: &GovernanceProposal,
    ) -> Result<GovernanceOutcome, GovernanceRuntimeError> {
        let (policy_result, _governed_state) = self.evaluate_state()?;

        let verdict = Self::map_policy_verdict(policy_result.decision);
        let reason_codes = Self::build_reason_codes(&policy_result.reason_codes);
        let config_hash = self.evaluator.config().compute_hash();

        Ok(GovernanceOutcome {
            verdict,
            reason_codes,
            mode: GovernanceMode::Governed,
            policy_id: Some(config_hash),
            policy_version: Some(self.policy_version()),
            schema_version: "decision_packet:v1".to_string(),
            packet: None,
            trace_available: true,
            authoritative_pdp: true,
            replay_sufficient: false,
        })
    }

    fn search_decisions(&self, filter: &AuditSearchFilter) -> AuditSearchResult {
        let limit = filter.limit.unwrap_or(100).clamp(1, 1000) as usize;

        let results: Vec<DecisionSummary> = self
            .audit_log
            .events()
            .iter()
            .filter(|event| {
                if let Some(ref decision_filter) = filter.decision {
                    let status_upper = event.learning_status.to_uppercase();
                    let filter_upper = decision_filter.to_uppercase();
                    status_upper.contains(&filter_upper)
                } else {
                    true
                }
            })
            .take(limit)
            .map(|event| DecisionSummary {
                decision_id: event.decision_id.clone(),
                principal: "policy".to_string(),
                action: "evaluate".to_string(),
                resource: "proposal".to_string(),
                decision: event.learning_status.clone(),
                timestamp: event.created_at.clone(),
            })
            .collect();

        let count = results.len();

        AuditSearchResult {
            results,
            count,
            ordering: "(timestamp_utc,decision_id) ASC".to_string(),
        }
    }

    fn mode(&self) -> GovernanceMode {
        GovernanceMode::Governed
    }

    fn meta(&self) -> GovernanceRuntimeMeta {
        GovernanceRuntimeMeta::governed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::PolicyConfig;

    fn make_governed() -> GovernedRuntime {
        let mut graph = StubRuntime::new();
        graph.create_node(0.8, 100.0);
        graph.create_node(0.6, 50.0);
        GovernedRuntime::new(graph, PolicyConfig::default(), EconomicsConfig::default())
    }

    fn test_proposal() -> GovernanceProposal {
        GovernanceProposal {
            proposal_id: "test-governed-001".to_string(),
            state_snapshot_hash: "a".repeat(64),
            constraints: serde_json::json!({}),
            requested_action: "deploy".to_string(),
            proposal_c14n: None,
            proposal_hash: None,
        }
    }

    #[test]
    fn governed_evaluate_returns_packet() {
        let mut rt = make_governed();
        let outcome = rt
            .evaluate(&test_proposal())
            .expect("evaluate must succeed");

        assert_eq!(outcome.mode, GovernanceMode::Governed);
        assert!(outcome.authoritative_pdp);
        assert!(outcome.replay_sufficient);
        assert!(
            outcome.packet.is_some(),
            "governed evaluate must emit packet"
        );
        assert_eq!(outcome.schema_version, "decision_packet:v1");
    }

    #[test]
    fn governed_preview_no_packet() {
        let rt = make_governed();
        let outcome = rt.preview(&test_proposal()).expect("preview must succeed");

        assert_eq!(outcome.mode, GovernanceMode::Governed);
        assert!(outcome.authoritative_pdp);
        assert!(
            !outcome.replay_sufficient,
            "preview is not replay-sufficient"
        );
        assert!(outcome.packet.is_none(), "preview must not emit packet");
    }

    #[test]
    fn governed_evaluate_records_audit() {
        let mut rt = make_governed();
        assert!(rt.audit_log().is_empty());

        let _ = rt.evaluate(&test_proposal()).expect("evaluate");
        assert_eq!(rt.audit_log().len(), 1);

        let _ = rt.evaluate(&test_proposal()).expect("evaluate");
        assert_eq!(rt.audit_log().len(), 2);
    }

    #[test]
    fn governed_preview_does_not_mutate_audit() {
        let rt = make_governed();
        assert!(rt.audit_log().is_empty());

        let _ = rt.preview(&test_proposal()).expect("preview");
        assert!(
            rt.audit_log().is_empty(),
            "preview must not mutate audit log"
        );
    }

    #[test]
    fn governed_packet_checksum_verifies() {
        let mut rt = make_governed();
        let outcome = rt.evaluate(&test_proposal()).expect("evaluate");
        let packet = outcome.packet.expect("must have packet");

        assert!(
            packet.verify_checksum().is_ok(),
            "packet checksum must verify"
        );
    }

    #[test]
    fn governed_verdict_is_deterministic() {
        let mut rt1 = make_governed();
        let mut rt2 = make_governed();

        let o1 = rt1.evaluate(&test_proposal()).expect("evaluate");
        let o2 = rt2.evaluate(&test_proposal()).expect("evaluate");

        assert_eq!(o1.verdict, o2.verdict);
        assert_eq!(o1.policy_version, o2.policy_version);
    }

    #[test]
    fn governed_policy_version_contains_hash() {
        let rt = make_governed();
        let outcome = rt.preview(&test_proposal()).expect("preview");
        let version = outcome.policy_version.expect("must have policy_version");

        assert!(
            version.starts_with("sha256:"),
            "policy_version must start with sha256:"
        );
        assert!(version.len() > 7, "policy_version must contain a hash");
    }

    #[test]
    fn governed_search_after_evaluate() {
        let mut rt = make_governed();
        let _ = rt.evaluate(&test_proposal()).expect("evaluate");

        let result = rt.search_decisions(&AuditSearchFilter::default());
        assert_eq!(result.count, 1);
        assert_eq!(result.results[0].principal, "policy");
    }

    #[test]
    fn governed_reason_codes_use_policy_namespace() {
        let mut rt = make_governed();
        let outcome = rt.evaluate(&test_proposal()).expect("evaluate");

        for code in &outcome.reason_codes {
            assert!(
                code.as_str().starts_with("policy."),
                "governed reason code must use policy.* namespace, got: {}",
                code.as_str()
            );
        }
    }

    #[test]
    fn governed_mode_metadata() {
        let rt = make_governed();
        let meta = rt.meta();

        assert_eq!(meta.mode, GovernanceMode::Governed);
        assert!(meta.authoritative_pdp);
        assert!(meta.replay_sufficient);
    }

    #[test]
    fn governed_packet_binds_governance_hash() {
        let mut rt = make_governed();
        let outcome = rt.evaluate(&test_proposal()).expect("evaluate");
        let packet = outcome.packet.expect("packet");

        assert_eq!(packet.governance_hash(), Some(GOVERNANCE_HASH.trim()));
    }

    #[test]
    fn governed_execution_trace_is_deterministic() {
        let mut rt1 = make_governed();
        let mut rt2 = make_governed();

        let packet1 = rt1
            .evaluate(&test_proposal())
            .expect("evaluate")
            .packet
            .expect("packet");
        let packet2 = rt2
            .evaluate(&test_proposal())
            .expect("evaluate")
            .packet
            .expect("packet");

        assert_eq!(packet1.execution_trace(), packet2.execution_trace());
        assert!(
            !packet1.execution_trace().is_empty(),
            "governed packets must carry an execution trace"
        );
    }
}
