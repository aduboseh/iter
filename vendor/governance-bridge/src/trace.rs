use crate::errors::BridgeError;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

// TRACE_SCHEMA_VERSION: bump when TraceStep fields change.
pub const TRACE_SCHEMA_VERSION: &str = "trace.v1";

// Stable typed operation taxonomy for execution traces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationType {
    PolicyEval,
    HashVerify,
    StateCheck,
    DecisionEmit,
    TraceFinalize,
}

impl OperationType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PolicyEval => "policy_eval",
            Self::HashVerify => "hash_verify",
            Self::StateCheck => "state_check",
            Self::DecisionEmit => "decision_emit",
            Self::TraceFinalize => "trace_finalize",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraceStep {
    pub region_id: String,
    pub operation: String,
    pub input_hash: String,
    pub output_hash: String,
    pub operation_type: OperationType,
    #[serde(default)]
    pub input_payload: String,
    #[serde(default)]
    pub output_payload: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionTrace {
    pub trace_version: String,
    steps: Vec<TraceStep>,
}

impl Default for ExecutionTrace {
    fn default() -> Self {
        Self::new()
    }
}

impl ExecutionTrace {
    pub fn new() -> Self {
        Self {
            trace_version: TRACE_SCHEMA_VERSION.to_string(),
            steps: Vec::new(),
        }
    }

    pub fn push(&mut self, step: TraceStep) {
        self.steps.push(step);
    }

    pub fn from_steps(steps: Vec<TraceStep>) -> Self {
        Self {
            trace_version: TRACE_SCHEMA_VERSION.to_string(),
            steps,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    pub fn len(&self) -> usize {
        self.steps.len()
    }

    pub fn steps(&self) -> &[TraceStep] {
        &self.steps
    }

    pub fn into_steps(self) -> Vec<TraceStep> {
        self.steps
    }

    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_string(self)
            .expect("ExecutionTrace serialization must not fail")
            .into_bytes()
    }
}

impl TraceStep {
    pub fn verify_hash_binding(&self) -> Result<(), BridgeError> {
        if self.input_payload.is_empty() {
            return Err(BridgeError::TraceDeterminismViolation(format!(
                "input_payload missing for operation '{}' ({})",
                self.operation,
                self.operation_type.as_str()
            )));
        }

        let expected_input_hash = payload_hash(&self.input_payload);
        if self.input_hash != expected_input_hash {
            return Err(BridgeError::TraceDeterminismViolation(format!(
                "input_hash mismatch for operation '{}' ({}): expected {}, got {}",
                self.operation,
                self.operation_type.as_str(),
                expected_input_hash,
                self.input_hash
            )));
        }

        if self.output_payload.is_empty() {
            return Err(BridgeError::TraceDeterminismViolation(format!(
                "output_payload missing for operation '{}' ({})",
                self.operation,
                self.operation_type.as_str()
            )));
        }

        let expected_output_hash = payload_hash(&self.output_payload);
        if self.output_hash != expected_output_hash {
            return Err(BridgeError::TraceDeterminismViolation(format!(
                "output_hash mismatch for operation '{}' ({}): expected {}, got {}",
                self.operation,
                self.operation_type.as_str(),
                expected_output_hash,
                self.output_hash
            )));
        }

        Ok(())
    }
}

fn payload_hash(payload: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(payload.as_bytes());
    hex::encode_upper(hasher.finalize())
}
