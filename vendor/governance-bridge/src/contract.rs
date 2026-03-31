use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    errors::BridgeError,
    trace::{ExecutionTrace, OperationType, TRACE_SCHEMA_VERSION},
};

pub const CONTRACT_VERSION_STR: &str = "scg.v1";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GovernanceRequest {
    pub proposal_id: String,
    pub state_snapshot_hash: String,
    pub requested_action: String,
    pub constraints: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Decision {
    Allow,
    Deny,
    Escalate,
}

#[derive(Serialize)]
struct GovernanceOutcomeDigest<'a> {
    contract_version: &'a str,
    decision: &'a Decision,
    governance_hash: &'a str,
    execution_trace: &'a ExecutionTrace,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GovernanceOutcome {
    pub contract_version: String,
    pub decision: Decision,
    pub governance_hash: String,
    pub execution_trace: ExecutionTrace,
    pub replay_id: String,
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

impl GovernanceOutcome {
    pub fn compute_replay_id(
        contract_version: &str,
        decision: &Decision,
        governance_hash: &str,
        execution_trace: &ExecutionTrace,
    ) -> String {
        let digest = GovernanceOutcomeDigest {
            contract_version,
            decision,
            governance_hash,
            execution_trace,
        };
        let bytes = serde_json::to_vec(&digest)
            .expect("GovernanceOutcomeDigest serialization must not fail");
        sha256_hex(&bytes)
    }

    pub fn verify_replay_id(&self) -> Result<(), BridgeError> {
        self.execution_trace.validate_schema_version()?;
        self.execution_trace.validate_chain()?;
        if self.contract_version != CONTRACT_VERSION_STR {
            return Err(BridgeError::ContractVersionMismatch {
                expected: CONTRACT_VERSION_STR.to_string(),
                got: self.contract_version.clone(),
            });
        }
        let expected = Self::compute_replay_id(
            &self.contract_version,
            &self.decision,
            &self.governance_hash,
            &self.execution_trace,
        );
        if self.replay_id != expected {
            return Err(BridgeError::ReplayIntegrityViolation(format!(
                "expected {}, got {}",
                expected, self.replay_id
            )));
        }
        Ok(())
    }
}

impl ExecutionTrace {
    pub fn validate_chain(&self) -> Result<(), BridgeError> {
        let steps = self.steps();
        if steps.is_empty() {
            return Ok(());
        }

        for (i, step) in steps.iter().enumerate().take(steps.len().saturating_sub(1)) {
            if step.operation_type == OperationType::TraceFinalize {
                return Err(BridgeError::TraceDeterminismViolation(format!(
                    "TraceFinalize must be last step, found at step {}",
                    i
                )));
            }
        }

        if steps
            .last()
            .is_some_and(|step| step.operation_type != OperationType::TraceFinalize)
        {
            return Err(BridgeError::TraceDeterminismViolation(
                "TraceFinalize must be the last step".to_string(),
            ));
        }

        if steps.len() < 2 {
            return Ok(());
        }

        for i in 1..steps.len() {
            if steps[i].input_hash != steps[i - 1].output_hash {
                return Err(BridgeError::TraceDeterminismViolation(format!(
                    "chain broken at step {}: input_hash '{}' != previous output_hash '{}'",
                    i,
                    steps[i].input_hash,
                    steps[i - 1].output_hash
                )));
            }
        }
        Ok(())
    }

    pub fn validate_schema_version(&self) -> Result<(), BridgeError> {
        if self.trace_version != TRACE_SCHEMA_VERSION {
            return Err(BridgeError::TraceDeterminismViolation(format!(
                "trace schema version mismatch: expected {}, got {}",
                TRACE_SCHEMA_VERSION, self.trace_version
            )));
        }
        Ok(())
    }
}
