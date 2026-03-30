use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::trace::ExecutionTrace;

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

    pub fn verify_replay_id(&self) -> Result<(), crate::errors::BridgeError> {
        if self.contract_version != CONTRACT_VERSION_STR {
            return Err(crate::errors::BridgeError::ContractVersionMismatch {
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
            return Err(crate::errors::BridgeError::ReplayIntegrityViolation(
                format!("expected {}, got {}", expected, self.replay_id),
            ));
        }
        Ok(())
    }
}
