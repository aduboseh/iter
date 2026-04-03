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
        let value = serde_json::to_value(self)
            .expect("ExecutionTrace serialization must not fail");
        let canonical = canonicalize(&value);
        serde_json::to_vec(&canonical)
            .expect("canonical serialization must not fail")
    }
}

pub fn canonicalize(value: &serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let mut sorted = serde_json::Map::new();
            let mut keys: Vec<_> = map.keys().collect();
            keys.sort();
            for key in keys {
                sorted.insert(key.clone(), canonicalize(&map[key]));
            }
            serde_json::Value::Object(sorted)
        }
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(canonicalize).collect())
        }
        _ => value.clone(),
    }
}

impl TraceStep {
    pub fn canonical_payload(&self) -> Vec<u8> {
        let value = serde_json::to_value(self)
            .expect("TraceStep serialization must not fail");
        let canonical = canonicalize(&value);
        serde_json::to_vec(&canonical)
            .expect("canonical serialization must not fail")
    }

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

pub fn payload_hash(payload: &str) -> String {
    let value: serde_json::Value = serde_json::from_str(payload)
        .expect("payload must be valid JSON — fail-closed");
    let canonical = canonicalize(&value);
    let bytes = serde_json::to_vec(&canonical)
        .expect("canonical serialization must not fail");
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    hex::encode_upper(hasher.finalize())
}