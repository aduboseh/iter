use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    errors::BridgeError,
    trace::{canonicalize, ExecutionTrace, OperationType, TRACE_SCHEMA_VERSION},
};

pub const CONTRACT_VERSION_STR: &str = "scg.v1";
pub const STATE_ENVELOPE_SCHEMA: &str = "scg.gateway.state_envelope.v1";
const TRACE_V1_REQUIRED_SEQUENCE: [OperationType; 5] = [
    OperationType::HashVerify,
    OperationType::PolicyEval,
    OperationType::StateCheck,
    OperationType::DecisionEmit,
    OperationType::TraceFinalize,
];

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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GovernanceStateEnvelope {
    pub schema: String,
    pub state_snapshot_hash: String,
    pub total_energy: f64,
    pub global_energy_drift: f64,
    pub active_simulations: u64,
    pub violation_count: u64,
}

impl GovernanceStateEnvelope {
    pub fn new(
        state_snapshot_hash: String,
        total_energy: f64,
        global_energy_drift: f64,
        active_simulations: u64,
        violation_count: u64,
    ) -> Self {
        Self {
            schema: STATE_ENVELOPE_SCHEMA.to_string(),
            state_snapshot_hash,
            total_energy,
            global_energy_drift,
            active_simulations,
            violation_count,
        }
    }

    pub fn canonical_bytes(&self) -> Vec<u8> {
        let value = serde_json::to_value(self)
            .expect("GovernanceStateEnvelope serialization must not fail");
        let canonical = canonicalize(&value);
        serde_json::to_vec(&canonical).expect("canonical serialization must not fail")
    }

    pub fn compute_hash(&self) -> String {
        sha256_hex(&self.canonical_bytes())
    }

    pub fn validate(&self, expected_snapshot_hash: &str) -> Result<(), BridgeError> {
        if self.schema != STATE_ENVELOPE_SCHEMA {
            return Err(BridgeError::TraceDeterminismViolation(format!(
                "state envelope schema mismatch: expected {}, got {}",
                STATE_ENVELOPE_SCHEMA, self.schema
            )));
        }
        if self.state_snapshot_hash != expected_snapshot_hash {
            return Err(BridgeError::ReplayIntegrityViolation(format!(
                "state envelope snapshot mismatch: expected {}, got {}",
                expected_snapshot_hash, self.state_snapshot_hash
            )));
        }
        if !self.total_energy.is_finite() || self.total_energy < 0.0 {
            return Err(BridgeError::TraceDeterminismViolation(
                "state envelope total_energy must be finite and non-negative".to_string(),
            ));
        }
        if !self.global_energy_drift.is_finite() || self.global_energy_drift < 0.0 {
            return Err(BridgeError::TraceDeterminismViolation(
                "state envelope global_energy_drift must be finite and non-negative".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Serialize)]
struct GovernanceOutcomeDigest<'a> {
    contract_version: &'a str,
    decision: &'a Decision,
    governance_hash: &'a str,
    state_snapshot_hash: &'a str,
    state_envelope_schema: &'a str,
    state_envelope_hash: &'a str,
    execution_trace: &'a ExecutionTrace,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GovernanceOutcome {
    pub contract_version: String,
    pub decision: Decision,
    pub governance_hash: String,
    pub state_snapshot_hash: String,
    pub state_envelope_schema: String,
    pub state_envelope_hash: String,
    pub state_envelope: GovernanceStateEnvelope,
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
        state_snapshot_hash: &str,
        state_envelope_schema: &str,
        state_envelope_hash: &str,
        execution_trace: &ExecutionTrace,
    ) -> String {
        let digest = GovernanceOutcomeDigest {
            contract_version,
            decision,
            governance_hash,
            state_snapshot_hash,
            state_envelope_schema,
            state_envelope_hash,
            execution_trace,
        };
        let bytes = serde_json::to_vec(&digest)
            .expect("GovernanceOutcomeDigest serialization must not fail");
        sha256_hex(&bytes)
    }

    pub fn verify_replay_id(&self) -> Result<(), BridgeError> {
        if self.contract_version != CONTRACT_VERSION_STR {
            return Err(BridgeError::ContractVersionMismatch {
                expected: CONTRACT_VERSION_STR.to_string(),
                got: self.contract_version.clone(),
            });
        }
        self.verify_state_envelope()?;
        self.execution_trace.validate_semantics()?;
        let expected = Self::compute_replay_id(
            &self.contract_version,
            &self.decision,
            &self.governance_hash,
            &self.state_snapshot_hash,
            &self.state_envelope_schema,
            &self.state_envelope_hash,
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

    pub fn verify_state_envelope(&self) -> Result<(), BridgeError> {
        self.state_envelope.validate(&self.state_snapshot_hash)?;
        if self.state_envelope_schema != self.state_envelope.schema {
            return Err(BridgeError::ReplayIntegrityViolation(format!(
                "state envelope schema binding mismatch: expected {}, got {}",
                self.state_envelope.schema, self.state_envelope_schema
            )));
        }
        let expected = self.state_envelope.compute_hash();
        if self.state_envelope_hash != expected {
            return Err(BridgeError::ReplayIntegrityViolation(format!(
                "state envelope hash mismatch: expected {}, got {}",
                expected, self.state_envelope_hash
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

    pub fn validate_sequence(&self) -> Result<(), BridgeError> {
        match self.trace_version.as_str() {
            TRACE_SCHEMA_VERSION => {
                let steps = self.steps();
                if steps.len() < 2 {
                    return Ok(());
                }

                for i in 1..steps.len() {
                    let from = steps[i - 1].operation_type;
                    let to = steps[i].operation_type;
                    if !matches!(
                        (from, to),
                        (OperationType::HashVerify, OperationType::PolicyEval)
                            | (OperationType::PolicyEval, OperationType::StateCheck)
                            | (OperationType::StateCheck, OperationType::DecisionEmit)
                            | (OperationType::DecisionEmit, OperationType::TraceFinalize)
                    ) {
                        return Err(BridgeError::TraceDeterminismViolation(format!(
                            "invalid trace transition at step {}: {} -> {}",
                            i,
                            from.as_str(),
                            to.as_str()
                        )));
                    }
                }

                Ok(())
            }
            _ => self.validate_schema_version(),
        }
    }

    pub fn validate_completeness(&self) -> Result<(), BridgeError> {
        match self.trace_version.as_str() {
            TRACE_SCHEMA_VERSION => {
                let steps = self.steps();
                if steps.len() != TRACE_V1_REQUIRED_SEQUENCE.len() {
                    return Err(BridgeError::TraceDeterminismViolation(format!(
                        "trace.v1 governed traces require exactly {} steps, got {}",
                        TRACE_V1_REQUIRED_SEQUENCE.len(),
                        steps.len()
                    )));
                }

                let mut counts = BTreeMap::new();
                for step in steps {
                    *counts.entry(step.operation_type).or_insert(0usize) += 1;
                }

                for operation in TRACE_V1_REQUIRED_SEQUENCE {
                    match counts.get(&operation).copied().unwrap_or(0) {
                        0 => {
                            return Err(BridgeError::TraceDeterminismViolation(format!(
                                "trace.v1 governed traces require exactly one {} step, got 0",
                                operation.as_str()
                            )))
                        }
                        1 => {}
                        count => {
                            return Err(BridgeError::TraceDeterminismViolation(format!(
                                "trace.v1 governed traces require exactly one {} step, got {}",
                                operation.as_str(),
                                count
                            )))
                        }
                    }
                }

                Ok(())
            }
            _ => self.validate_schema_version(),
        }
    }

    pub fn validate_semantics(&self) -> Result<(), BridgeError> {
        self.validate_schema_version()?;
        self.validate_chain()?;
        self.validate_sequence()?;
        self.validate_completeness()?;
        self.verify_hash_bindings_for_all_steps()?;
        Ok(())
    }

    fn verify_hash_bindings_for_all_steps(&self) -> Result<(), BridgeError> {
        for (i, step) in self.steps().iter().enumerate() {
            step.verify_hash_binding().map_err(|error| match error {
                BridgeError::TraceDeterminismViolation(message) => {
                    BridgeError::TraceDeterminismViolation(format!(
                        "hash binding mismatch at step {}: {}",
                        i, message
                    ))
                }
                other => other,
            })?;
        }

        Ok(())
    }
}
