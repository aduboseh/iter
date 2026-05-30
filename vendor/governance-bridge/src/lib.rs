pub mod contract;
pub mod errors;
pub mod trace;

pub use contract::{
    Decision, GovernanceOutcome, GovernanceRequest, GovernanceStateEnvelope, CONTRACT_VERSION_STR,
    STATE_ENVELOPE_SCHEMA,
};
pub use errors::BridgeError;
pub use trace::{ExecutionTrace, OperationType, TraceStep, TRACE_SCHEMA_VERSION};

pub trait GovernanceBridge: Send + Sync {
    fn evaluate(&self, request: GovernanceRequest) -> Result<GovernanceOutcome, BridgeError>;
}

#[cfg(any(test, feature = "test-fixtures"))]
pub struct StubBridge {
    pub governance_hash: String,
}

#[cfg(any(test, feature = "test-fixtures"))]
fn stub_hash(value: &str) -> String {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    hex::encode_upper(hasher.finalize())
}

#[cfg(any(test, feature = "test-fixtures"))]
fn stub_payload<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_string(value).expect("stub trace payload serialization must not fail")
}

#[cfg(any(test, feature = "test-fixtures"))]
fn stub_trace_step<TInput: serde::Serialize, TOutput: serde::Serialize>(
    region_id: &str,
    operation: &str,
    operation_type: OperationType,
    input: &TInput,
    output: &TOutput,
) -> TraceStep {
    let input_payload = stub_payload(input);
    let output_payload = stub_payload(output);

    TraceStep {
        region_id: region_id.into(),
        operation: operation.into(),
        input_hash: stub_hash(&input_payload),
        output_hash: stub_hash(&output_payload),
        operation_type,
        input_payload,
        output_payload,
    }
}

#[cfg(any(test, feature = "test-fixtures"))]
impl GovernanceBridge for StubBridge {
    fn evaluate(&self, request: GovernanceRequest) -> Result<GovernanceOutcome, BridgeError> {
        if self.governance_hash.is_empty() {
            return Err(BridgeError::GovernanceHashInvalid(
                "governance_hash must not be empty".into(),
            ));
        }

        let decision = Decision::Allow;

        let mut trace = ExecutionTrace::new();
        let hash_input = (
            request.proposal_id.clone(),
            request.state_snapshot_hash.clone(),
            request.requested_action.clone(),
        );
        let hash_output = (request.state_snapshot_hash.clone(), true);
        trace.push(stub_trace_step(
            "stub",
            "snapshot-hash-compare",
            OperationType::HashVerify,
            &hash_input,
            &hash_output,
        ));

        let policy_input = hash_output.clone();
        let policy_output = (
            policy_input.clone(),
            request.requested_action.clone(),
            decision.clone(),
        );
        trace.push(stub_trace_step(
            "stub",
            "policy-eval",
            OperationType::PolicyEval,
            &policy_input,
            &policy_output,
        ));

        let state_input = policy_output.clone();
        let state_output = (state_input.clone(), "state-ok", false);
        trace.push(stub_trace_step(
            "stub",
            "state-check",
            OperationType::StateCheck,
            &state_input,
            &state_output,
        ));

        let decision_input = state_output.clone();
        let decision_output = (
            decision.clone(),
            request.requested_action.clone(),
            request.proposal_id.clone(),
        );
        trace.push(stub_trace_step(
            "stub",
            "decision-emit",
            OperationType::DecisionEmit,
            &decision_input,
            &decision_output,
        ));

        let finalize_input = decision_output.clone();
        let finalize_output = (
            self.governance_hash.as_str(),
            CONTRACT_VERSION_STR,
            request.proposal_id.as_str(),
            "trace-sealed",
        );
        trace.push(stub_trace_step(
            "stub",
            "trace-finalize",
            OperationType::TraceFinalize,
            &finalize_input,
            &finalize_output,
        ));

        let state_envelope =
            GovernanceStateEnvelope::new(request.state_snapshot_hash.clone(), 0.0, 0.0, 0, 0);
        let state_envelope_hash = state_envelope.compute_hash();
        let replay_id = GovernanceOutcome::compute_replay_id(
            CONTRACT_VERSION_STR,
            &decision,
            &self.governance_hash,
            &request.state_snapshot_hash,
            STATE_ENVELOPE_SCHEMA,
            &state_envelope_hash,
            &trace,
        );

        Ok(GovernanceOutcome {
            contract_version: CONTRACT_VERSION_STR.to_string(),
            decision,
            governance_hash: self.governance_hash.clone(),
            state_snapshot_hash: request.state_snapshot_hash,
            state_envelope_schema: STATE_ENVELOPE_SCHEMA.to_string(),
            state_envelope_hash,
            state_envelope,
            execution_trace: trace,
            replay_id,
        })
    }
}
