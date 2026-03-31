pub mod contract;
pub mod errors;
pub mod trace;

pub use contract::{Decision, GovernanceOutcome, GovernanceRequest, CONTRACT_VERSION_STR};
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
    hex::encode(hasher.finalize())
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
        let eval_output_hash = stub_hash(&format!("Allow::{}", request.proposal_id));
        trace.push(TraceStep {
            region_id: "stub".into(),
            operation: "eval".into(),
            input_hash: stub_hash(&request.proposal_id),
            output_hash: eval_output_hash.clone(),
            operation_type: OperationType::PolicyEval,
        });
        trace.push(TraceStep {
            region_id: "stub".into(),
            operation: "trace-finalize".into(),
            input_hash: eval_output_hash,
            output_hash: stub_hash(&format!("sealed::Allow::{}", request.proposal_id)),
            operation_type: OperationType::TraceFinalize,
        });

        let replay_id = GovernanceOutcome::compute_replay_id(
            CONTRACT_VERSION_STR,
            &decision,
            &self.governance_hash,
            &trace,
        );

        Ok(GovernanceOutcome {
            contract_version: CONTRACT_VERSION_STR.to_string(),
            decision,
            governance_hash: self.governance_hash.clone(),
            execution_trace: trace,
            replay_id,
        })
    }
}
