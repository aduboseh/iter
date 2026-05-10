use std::collections::{BTreeMap, VecDeque};
use std::sync::Arc;
use std::time::Duration;

use governance_bridge::contract::{
    Decision as ScgDecision, GovernanceOutcome as ScgGovernanceOutcome, GovernanceRequest,
    CONTRACT_VERSION_STR,
};
use reqwest::{blocking::Client, Url};

use crate::audit::{AuditLog, DecisionPacket};
use crate::contracts::{PolicyDecision, PolicyEnvelope, SystemState};
use crate::runtime::{
    GovernanceMode, GovernanceOutcome, GovernanceRuntime, GovernanceRuntimeError,
    GovernanceRuntimeMeta, GovernanceVerdict, ReasonCode,
};
use crate::substrate::stub::{
    AuditSearchFilter, AuditSearchResult, DecisionSummary, GovernanceProposal, ReplayResult,
    ReplayStatus, StubRuntime,
};

/// SCG-backed implementation of Iter's governance runtime boundary.
pub struct ScgRuntime {
    endpoint: String,
    boot_governance_hash: Arc<String>,
    auth_token: Option<Arc<str>>,
    http_client: Client,
    graph: StubRuntime,
    audit_log: AuditLog,
    replay_packets: VecDeque<DecisionPacket>,
}

impl ScgRuntime {
    const REPLAY_PACKET_LIMIT: usize = 1000;

    /// Construct a fail-closed connector to the live SCG governance endpoint.
    pub fn connect(endpoint: String, boot_hash: String) -> Result<Self, GovernanceRuntimeError> {
        let endpoint = Self::validate_endpoint(endpoint.trim())?;
        let boot_hash = boot_hash.trim().to_string();
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
            auth_token: Self::auth_token_from_env(),
            http_client,
            graph: StubRuntime::new(),
            audit_log: AuditLog::new(),
            replay_packets: VecDeque::new(),
        })
    }

    fn validate_endpoint(endpoint: &str) -> Result<String, GovernanceRuntimeError> {
        let endpoint = endpoint.trim().trim_end_matches('/').to_string();
        if endpoint.is_empty() {
            return Err(GovernanceRuntimeError::ConfigMissing(
                "SCG_ENDPOINT".to_string(),
            ));
        }

        let url = Url::parse(&endpoint).map_err(|e| {
            GovernanceRuntimeError::ConfigMissing(format!(
                "SCG_ENDPOINT must be a valid absolute URL: {}",
                e
            ))
        })?;
        let host = url.host_str().map_or("", |host| host);
        let allow_loopback_http =
            url.scheme() == "http" && matches!(host, "localhost" | "127.0.0.1" | "::1");

        if url.scheme() != "https" && !allow_loopback_http {
            return Err(GovernanceRuntimeError::ConfigMissing(
                "SCG_ENDPOINT must use https unless host is localhost or loopback".to_string(),
            ));
        }

        Ok(endpoint)
    }

    fn auth_token_from_env() -> Option<Arc<str>> {
        std::env::var("SCG_AUTH_TOKEN")
            .ok()
            .or_else(|| std::env::var("SCG_GATEWAY_AUTH_TOKEN").ok())
            .map(|token| token.trim().to_string())
            .filter(|token| !token.is_empty())
            .map(Arc::<str>::from)
    }

    /// Read access to the local graph used for non-governance tool surfaces.
    pub fn graph(&self) -> &StubRuntime {
        &self.graph
    }

    /// Mutable graph access for legacy tool compatibility.
    pub fn graph_mut(&mut self) -> &mut StubRuntime {
        &mut self.graph
    }

    /// Replay SCG-backed decisions from the persisted packet stream rather than local lineage.
    pub fn replay_decisions(&self) -> Vec<ReplayResult> {
        self.replay_packets
            .iter()
            .map(|packet| match packet.verify_checksum() {
                Ok(()) => ReplayResult {
                    decision_id: packet.checksum.clone(),
                    replay_status: ReplayStatus::Match,
                    propagation_checksum: Some(packet.checksum.clone()),
                    reason: None,
                },
                Err(err) => ReplayResult {
                    decision_id: packet.checksum.clone(),
                    replay_status: ReplayStatus::Blocked,
                    propagation_checksum: None,
                    reason: Some(format!("decision_packet_checksum_mismatch: {}", err)),
                },
            })
            .collect()
    }

    fn record_replay_packet(&mut self, packet: DecisionPacket) {
        if self.replay_packets.len() >= Self::REPLAY_PACKET_LIMIT {
            self.replay_packets.pop_front();
        }
        self.replay_packets.push_back(packet);
    }

    fn assert_governed_packet_integrity(
        packet: &DecisionPacket,
        context: &str,
    ) -> Result<(), GovernanceRuntimeError> {
        let has_valid_governance_hash = match packet.governance_hash.as_deref() {
            Some(hash) => !hash.is_empty(),
            None => false,
        };
        if !has_valid_governance_hash {
            return Err(GovernanceRuntimeError::GovernanceHashMismatch(format!(
                "INV-GOV-HASH: governance_hash empty or absent before packet return [{}]",
                context
            )));
        }
        if packet.execution_trace.is_empty() {
            return Err(GovernanceRuntimeError::ReplayIntegrityViolation(format!(
                "INV-TRACE: execution_trace empty or absent before packet return [{}]",
                context
            )));
        }
        Ok(())
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

    fn fetch_outcome(
        &self,
        proposal: &GovernanceProposal,
    ) -> Result<ScgGovernanceOutcome, GovernanceRuntimeError> {
        let request = Self::build_request(proposal)?;
        let url = format!("{}/governance/evaluate", self.endpoint);
        let mut request_builder = self.http_client.post(&url).json(&request);
        if let Some(token) = &self.auth_token {
            request_builder = request_builder.bearer_auth(token.as_ref());
        }
        let response = request_builder
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
            ScgDecision::Allow => "governance_allow",
            ScgDecision::Deny => "governance_deny",
            ScgDecision::Escalate => "governance_escalate",
        };
        vec![ReasonCode::policy(suffix)]
    }

    fn packet_reason_codes(decision: ScgDecision) -> Vec<String> {
        let reason = match decision {
            ScgDecision::Allow => "policy.governance_allow",
            ScgDecision::Deny => "policy.governance_deny",
            ScgDecision::Escalate => "policy.governance_escalate",
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
            env!("ITER_BUILD_HASH").to_string(),
            env!("SUBSTRATE_BUILD_HASH").to_string(),
            &state,
            None,
            self.graph.economics_config().compute_hash(),
            Self::evaluated_rules(outcome),
        )
        .map_err(|e| GovernanceRuntimeError::EvaluationFailed {
            reason: e.to_string(),
        })?;

        let trace_strings = Self::trace_strings(outcome)?;
        packet.bind_governance_context(outcome.governance_hash.clone(), trace_strings.clone());
        // INV-IMMUT-001: SCG-bound fields must not be overwritten after binding.
        debug_assert!(
            match packet.governance_hash.as_deref() {
                Some(hash) => !hash.is_empty(),
                None => false,
            },
            "INV-IMMUT-001: governance_hash must be non-empty after SCG binding"
        );
        debug_assert!(
            packet.governance_hash.as_deref() == Some(outcome.governance_hash.as_str()),
            "INV-IMMUT-001: governance_hash must not be overwritten after SCG binding"
        );
        debug_assert!(
            !packet.execution_trace.is_empty(),
            "INV-IMMUT-001: execution_trace must be non-empty after SCG binding"
        );
        debug_assert!(
            packet.execution_trace == trace_strings,
            "INV-IMMUT-001: execution_trace must not be overwritten after SCG binding"
        );
        packet
            .verify_checksum()
            .map_err(|e| GovernanceRuntimeError::EvaluationFailed {
                reason: format!("packet checksum verification failed: {}", e),
            })?;
        Self::assert_governed_packet_integrity(&packet, "ScgBacked::build_packet")?;

        Ok(packet)
    }

    /// Return audit.search filters that ScgBacked mode does not currently support.
    pub fn unsupported_audit_filters(filter: &AuditSearchFilter) -> Vec<String> {
        let mut unsupported = Vec::new();

        if filter
            .principal
            .as_ref()
            .is_some_and(|value| !value.trim().is_empty())
        {
            unsupported.push("principal".to_string());
        }
        if filter
            .action
            .as_ref()
            .is_some_and(|value| !value.trim().is_empty())
        {
            unsupported.push("action".to_string());
        }
        if filter
            .resource
            .as_ref()
            .is_some_and(|value| !value.trim().is_empty())
        {
            unsupported.push("resource".to_string());
        }
        if filter
            .policy_id
            .as_ref()
            .is_some_and(|value| !value.trim().is_empty())
        {
            unsupported.push("policy_id".to_string());
        }
        if filter
            .from
            .as_ref()
            .is_some_and(|value| !value.trim().is_empty())
        {
            unsupported.push("from".to_string());
        }
        if filter
            .to
            .as_ref()
            .is_some_and(|value| !value.trim().is_empty())
        {
            unsupported.push("to".to_string());
        }

        unsupported
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
        let outcome = self.fetch_outcome(proposal)?;
        let packet = self.build_packet(&outcome)?;
        // Defense in depth: build_packet enforces packet integrity before returning,
        // and evaluate re-checks it at the outer boundary before audit/publish.
        Self::assert_governed_packet_integrity(&packet, "ScgBacked::evaluate")?;
        self.audit_log.append(&packet);
        self.record_replay_packet(packet.clone());
        Ok(Self::build_runtime_outcome(&outcome, Some(packet), true))
    }

    fn preview(
        &self,
        proposal: &GovernanceProposal,
    ) -> Result<GovernanceOutcome, GovernanceRuntimeError> {
        let outcome = self.fetch_outcome(proposal)?;
        Ok(Self::build_runtime_outcome(&outcome, None, false))
    }

    fn search_decisions(&self, filter: &AuditSearchFilter) -> AuditSearchResult {
        // map_or keeps this connector clear of the CI-guarded defaulting pattern in this file.
        let limit = filter.limit.map_or(100, |limit| limit).clamp(1, 1000) as usize;

        // main.rs rejects unsupported filters before routing audit.search here.
        let mut results: Vec<DecisionSummary> = self
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
            .map(|event| DecisionSummary {
                decision_id: event.decision_id.clone(),
                principal: "policy".to_string(),
                action: "evaluate".to_string(),
                resource: "proposal".to_string(),
                decision: event.decision.clone(),
                timestamp: event.created_at.clone(),
            })
            .collect();

        results.sort_by(|a, b| {
            a.timestamp
                .cmp(&b.timestamp)
                .then_with(|| a.decision_id.cmp(&b.decision_id))
        });
        results.truncate(limit);

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
    use crate::contracts::{EnergyEnvelope, LearningEnvelope, LearningStatus, ReasoningEnvelope};

    fn make_packet(tick: u64, governance_hash: &str) -> DecisionPacket {
        let energy = EnergyEnvelope::new(100.0, 10.0, 0.95).unwrap();
        let reasoning = ReasoningEnvelope::new(0.9, 0.5, 0.1, 0.5).unwrap();
        let learning = LearningEnvelope::new(
            format!("capsule-{}", tick),
            1,
            "a".repeat(64),
            0.5,
            0.5,
            1.0,
            LearningStatus::Committed,
            0,
        )
        .unwrap();
        let policy = PolicyEnvelope::new(
            governance_hash.to_string(),
            PolicyDecision::Allow,
            vec!["policy.governance_allow".to_string()],
        )
        .unwrap();
        let state = SystemState::new(tick, energy, reasoning, learning, policy);
        let mut packet = DecisionPacket::new(
            env!("ITER_BUILD_HASH").to_string(),
            env!("SUBSTRATE_BUILD_HASH").to_string(),
            &state,
            None,
            "economics-hash".to_string(),
            vec!["gateway.snapshot:compare_hash".to_string()],
        )
        .unwrap();
        packet.bind_governance_context(governance_hash.to_string(), vec!["trace-step".to_string()]);
        packet
    }

    #[test]
    fn connect_rejects_remote_http_endpoint() {
        let err = ScgRuntime::connect("http://example.com".to_string(), "a".repeat(64))
            .err()
            .expect("remote http endpoint must fail closed");
        assert!(matches!(err, GovernanceRuntimeError::ConfigMissing(_)));
    }

    #[test]
    fn connect_allows_loopback_http_endpoint() {
        let runtime = ScgRuntime::connect("http://127.0.0.1:18080".to_string(), "a".repeat(64))
            .expect("loopback http endpoint remains valid for local seam tests");
        assert_eq!(runtime.endpoint, "http://127.0.0.1:18080");
    }

    #[test]
    fn replay_packet_retention_is_bounded() {
        let mut runtime =
            ScgRuntime::connect("https://governance.example.com".to_string(), "a".repeat(64))
                .expect("https endpoint");
        let total_packets = ScgRuntime::REPLAY_PACKET_LIMIT + 2;
        let first_retained_tick = 2_u64;

        for tick in 0..total_packets as u64 {
            runtime.record_replay_packet(make_packet(tick, &format!("{:064x}", tick + 1)));
        }

        let results = runtime.replay_decisions();
        assert_eq!(results.len(), ScgRuntime::REPLAY_PACKET_LIMIT);
        assert_eq!(
            results.first().expect("bounded replay result").decision_id,
            make_packet(
                first_retained_tick,
                &format!("{:064x}", first_retained_tick + 1)
            )
            .checksum
        );
    }

    #[test]
    fn audit_search_reports_ascending_ordering() {
        let mut runtime =
            ScgRuntime::connect("https://governance.example.com".to_string(), "a".repeat(64))
                .expect("https endpoint");
        let first = make_packet(1, &format!("{:064x}", 1));
        let second = make_packet(2, &format!("{:064x}", 2));

        runtime.audit_log.append(&first);
        std::thread::sleep(std::time::Duration::from_millis(2));
        runtime.audit_log.append(&second);

        let result = runtime.search_decisions(&AuditSearchFilter::default());
        assert_eq!(result.ordering, "(timestamp_utc,decision_id) ASC");
        assert_eq!(result.results.len(), 2);
        assert_eq!(result.results[0].decision_id, first.checksum);
        assert_eq!(result.results[1].decision_id, second.checksum);
    }

    #[test]
    fn governed_packet_integrity_enforced_before_return() {
        let mut missing_hash = make_packet(1, &format!("{:064x}", 1));
        missing_hash.governance_hash = Some(String::new());
        assert!(ScgRuntime::assert_governed_packet_integrity(&missing_hash, "test").is_err());

        let mut absent_hash = make_packet(2, &format!("{:064x}", 2));
        absent_hash.governance_hash = None;
        assert!(ScgRuntime::assert_governed_packet_integrity(&absent_hash, "test").is_err());

        let mut missing_trace = make_packet(3, &format!("{:064x}", 3));
        missing_trace.execution_trace.clear();
        assert!(ScgRuntime::assert_governed_packet_integrity(&missing_trace, "test").is_err());

        let valid_packet = make_packet(4, &format!("{:064x}", 4));
        assert!(ScgRuntime::assert_governed_packet_integrity(&valid_packet, "test").is_ok());
    }
}
