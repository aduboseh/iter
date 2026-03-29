use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

use reqwest::blocking::Client;
use scg_governance_bridge::contract::{
    Decision as ScgDecision, GovernanceOutcome as ScgGovernanceOutcome, GovernanceRequest,
    CONTRACT_VERSION_STR,
};

use crate::audit::{AuditLog, DecisionPacket};
use crate::contracts::{PolicyDecision, PolicyEnvelope, SystemState};
use crate::runtime::{
    GovernanceMode, GovernanceOutcome, GovernanceRuntime, GovernanceRuntimeError,
    GovernanceRuntimeMeta, GovernanceVerdict, ReasonCode,
};
use crate::substrate::stub::{
    AuditSearchFilter, AuditSearchResult, DecisionSummary, GovernanceProposal, StubRuntime,
};

/// SCG-backed implementation of Iter's governance runtime boundary.
pub struct ScgRuntime {
    endpoint: String,
    boot_governance_hash: Arc<String>,
    http_client: Client,
    graph: StubRuntime,
    audit_log: AuditLog,
}

impl ScgRuntime {
    /// Construct a fail-closed connector to the live SCG governance endpoint.
    pub fn connect(endpoint: String, boot_hash: String) -> Result<Self, GovernanceRuntimeError> {
        let endpoint = endpoint.trim().trim_end_matches('/').to_string();
        let boot_hash = boot_hash.trim().to_string();
        if endpoint.is_empty() {
            return Err(GovernanceRuntimeError::ConfigMissing(
                "SCG_ENDPOINT".to_string(),
            ));
        }
        if boot_hash.is_empty() {
            return Err(GovernanceRuntimeError::ConfigMissing(
                "governance_hash".to_string(),
            ));
        }

        let http_client = Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| GovernanceRuntimeError::ScgUnavailable(e.to_string()))?;

        Ok(Self {
            endpoint,
            boot_governance_hash: Arc::new(boot_hash),
            http_client,
            graph: StubRuntime::new(),
            audit_log: AuditLog::new(),
        })
    }

    /// Read access to the local graph used for non-governance tool surfaces.
    pub fn graph(&self) -> &StubRuntime {
        &self.graph
    }

    /// Mutable graph access for legacy tool compatibility.
    pub fn graph_mut(&mut self) -> &mut StubRuntime {
        &mut self.graph
    }

    fn build_request(
        proposal: &GovernanceProposal,
    ) -> Result<GovernanceRequest, GovernanceRuntimeError> {
        if proposal.proposal_id.trim().is_empty() {
            return Err(GovernanceRuntimeError::ProposalInvalid {
                reason: "proposal_id must not be empty".to_string(),
            });
        }
        if proposal.state_snapshot_hash.trim().is_empty() {
            return Err(GovernanceRuntimeError::ProposalInvalid {
                reason: "state_snapshot_hash must not be empty".to_string(),
            });
        }
        if proposal.requested_action.trim().is_empty() {
            return Err(GovernanceRuntimeError::ProposalInvalid {
                reason: "requested_action must not be empty".to_string(),
            });
        }

        Ok(GovernanceRequest {
            proposal_id: proposal.proposal_id.clone(),
            state_snapshot_hash: proposal.state_snapshot_hash.clone(),
            requested_action: proposal.requested_action.clone(),
            constraints: Self::constraints_from_value(&proposal.constraints)?,
        })
    }

    fn constraints_from_value(
        value: &serde_json::Value,
    ) -> Result<BTreeMap<String, String>, GovernanceRuntimeError> {
        match value {
            serde_json::Value::Null => Ok(BTreeMap::new()),
            serde_json::Value::Object(map) => {
                let mut constraints = BTreeMap::new();
                for (key, value) in map {
                    let value = match value {
                        serde_json::Value::String(s) => s.clone(),
                        other => serde_json::to_string(other).map_err(|e| {
                            GovernanceRuntimeError::ProposalInvalid {
                                reason: format!("constraints serialization failed: {}", e),
                            }
                        })?,
                    };
                    constraints.insert(key.clone(), value);
                }
                Ok(constraints)
            }
            _ => Err(GovernanceRuntimeError::ProposalInvalid {
                reason: "constraints must be an object when present".to_string(),
            }),
        }
    }

    fn fetch_scg_outcome(
        &self,
        proposal: &GovernanceProposal,
    ) -> Result<ScgGovernanceOutcome, GovernanceRuntimeError> {
        let request = Self::build_request(proposal)?;
        let url = format!("{}/governance/evaluate", self.endpoint);
        let response = self
            .http_client
            .post(&url)
            .json(&request)
            .send()
            .map_err(|e| GovernanceRuntimeError::ScgUnavailable(e.to_string()))?;

        if !response.status().is_success() {
            return Err(GovernanceRuntimeError::ScgUnavailable(format!(
                "SCG returned HTTP {}",
                response.status()
            )));
        }

        let outcome: ScgGovernanceOutcome =
            response
                .json()
                .map_err(|e| GovernanceRuntimeError::EvaluationFailed {
                    reason: format!("SCG response deserialization failed: {}", e),
                })?;

        if outcome.contract_version != CONTRACT_VERSION_STR {
            return Err(GovernanceRuntimeError::ContractVersionMismatch(format!(
                "expected {}, got {}",
                CONTRACT_VERSION_STR, outcome.contract_version
            )));
        }

        outcome
            .verify_replay_id()
            .map_err(|e| GovernanceRuntimeError::ReplayIntegrityViolation(e.to_string()))?;

        if outcome.governance_hash != *self.boot_governance_hash {
            return Err(GovernanceRuntimeError::GovernanceHashMismatch(format!(
                "boot hash: {}, response hash: {}",
                self.boot_governance_hash, outcome.governance_hash
            )));
        }

        if outcome.execution_trace.is_empty() {
            return Err(GovernanceRuntimeError::EvaluationFailed {
                reason: "SCG response missing execution_trace".to_string(),
            });
        }

        Ok(outcome)
    }

    fn trace_strings(
        outcome: &ScgGovernanceOutcome,
    ) -> Result<Vec<String>, GovernanceRuntimeError> {
        outcome
            .execution_trace
            .steps()
            .iter()
            .map(|step| {
                serde_json::to_string(step).map_err(|e| GovernanceRuntimeError::EvaluationFailed {
                    reason: format!("execution_trace serialization failed: {}", e),
                })
            })
            .collect()
    }

    fn evaluated_rules(outcome: &ScgGovernanceOutcome) -> Vec<String> {
        outcome
            .execution_trace
            .steps()
            .iter()
            .map(|step| format!("{}:{}", step.region_id, step.operation))
            .collect()
    }

    fn policy_decision(decision: ScgDecision) -> PolicyDecision {
        match decision {
            ScgDecision::Allow => PolicyDecision::Allow,
            ScgDecision::Deny => PolicyDecision::Deny,
            ScgDecision::Escalate => PolicyDecision::RequireReview,
        }
    }

    fn runtime_verdict(decision: ScgDecision) -> GovernanceVerdict {
        match decision {
            ScgDecision::Allow => GovernanceVerdict::Allow,
            ScgDecision::Deny => GovernanceVerdict::Block,
            ScgDecision::Escalate => GovernanceVerdict::Review,
        }
    }

    fn runtime_reason_codes(decision: ScgDecision) -> Vec<ReasonCode> {
        let suffix = match decision {
            ScgDecision::Allow => "scg_allow",
            ScgDecision::Deny => "scg_deny",
            ScgDecision::Escalate => "scg_escalate",
        };
        vec![ReasonCode::policy(suffix)]
    }

    fn packet_reason_codes(decision: ScgDecision) -> Vec<String> {
        let reason = match decision {
            ScgDecision::Allow => "policy.scg_allow",
            ScgDecision::Deny => "policy.scg_deny",
            ScgDecision::Escalate => "policy.scg_escalate",
        };
        vec![reason.to_string()]
    }

    fn build_packet(
        &self,
        outcome: &ScgGovernanceOutcome,
    ) -> Result<DecisionPacket, GovernanceRuntimeError> {
        let base_state =
            self.graph
                .system_state()
                .map_err(|e| GovernanceRuntimeError::EvaluationFailed {
                    reason: e.to_string(),
                })?;

        let policy = PolicyEnvelope::new(
            outcome.governance_hash.clone(),
            Self::policy_decision(outcome.decision.clone()),
            Self::packet_reason_codes(outcome.decision.clone()),
        )
        .map_err(|e| GovernanceRuntimeError::EvaluationFailed {
            reason: e.to_string(),
        })?;

        let state = SystemState::new(
            base_state.tick,
            base_state.energy,
            base_state.reasoning,
            base_state.learning,
            policy,
        );

        // decision_id is content-addressed: identical inputs produce identical IDs.
        // That is the replay guarantee, not a collision bug.
        let mut packet = DecisionPacket::new(
            env!("CARGO_PKG_VERSION").to_string(),
            outcome.governance_hash.clone(),
            &state,
            None,
            self.graph.economics_config().compute_hash(),
            Self::evaluated_rules(outcome),
        )
        .map_err(|e| GovernanceRuntimeError::EvaluationFailed {
            reason: e.to_string(),
        })?;

        packet.bind_governance_context(
            outcome.governance_hash.clone(),
            Self::trace_strings(outcome)?,
        );
        packet
            .verify_checksum()
            .map_err(|e| GovernanceRuntimeError::EvaluationFailed {
                reason: format!("packet checksum verification failed: {}", e),
            })?;

        Ok(packet)
    }

    fn build_runtime_outcome(
        outcome: &ScgGovernanceOutcome,
        packet: Option<DecisionPacket>,
        replay_sufficient: bool,
    ) -> GovernanceOutcome {
        GovernanceOutcome {
            verdict: Self::runtime_verdict(outcome.decision.clone()),
            reason_codes: Self::runtime_reason_codes(outcome.decision.clone()),
            mode: GovernanceMode::Governed,
            policy_id: Some(outcome.governance_hash.clone()),
            policy_version: Some(format!("sha256:{}", outcome.governance_hash)),
            schema_version: "decision_packet:v1".to_string(),
            packet,
            trace_available: true,
            authoritative_pdp: true,
            replay_sufficient,
        }
    }
}

impl GovernanceRuntime for ScgRuntime {
    fn evaluate(
        &mut self,
        proposal: &GovernanceProposal,
    ) -> Result<GovernanceOutcome, GovernanceRuntimeError> {
        let outcome = self.fetch_scg_outcome(proposal)?;
        let packet = self.build_packet(&outcome)?;
        self.audit_log.append(&packet);
        Ok(Self::build_runtime_outcome(&outcome, Some(packet), true))
    }

    fn preview(
        &self,
        proposal: &GovernanceProposal,
    ) -> Result<GovernanceOutcome, GovernanceRuntimeError> {
        let outcome = self.fetch_scg_outcome(proposal)?;
        Ok(Self::build_runtime_outcome(&outcome, None, false))
    }

    fn search_decisions(&self, filter: &AuditSearchFilter) -> AuditSearchResult {
        let limit = filter.limit.unwrap_or(100).clamp(1, 1000) as usize;

        // Supported filters: decision, limit.
        // Other AuditSearchFilter fields are deferred to WO-ITER-SURFACE-001.
        let results: Vec<DecisionSummary> = self
            .audit_log
            .events()
            .iter()
            .filter(|event| {
                if let Some(ref decision_filter) = filter.decision {
                    let status_upper = event.decision.to_uppercase();
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
                decision: event.decision.clone(),
                timestamp: event.created_at.clone(),
            })
            .collect();

        let count = results.len();
        AuditSearchResult {
            results,
            count,
            ordering: "created_at desc".to_string(),
        }
    }

    fn mode(&self) -> GovernanceMode {
        GovernanceMode::Governed
    }

    fn meta(&self) -> GovernanceRuntimeMeta {
        GovernanceRuntimeMeta::governed()
    }
}
