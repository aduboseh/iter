use serde::{Deserialize, Serialize};

// TRACE_SCHEMA_VERSION: bump when TraceStep fields change.
pub const TRACE_SCHEMA_VERSION: &str = "trace.v1";

// Stable typed operation taxonomy for execution traces.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationType {
    PolicyEval,
    HashVerify,
    StateCheck,
    DecisionEmit,
    TraceFinalize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraceStep {
    pub region_id: String,
    pub operation: String,
    pub input_hash: String,
    pub output_hash: String,
    pub operation_type: OperationType,
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
        serde_json::to_vec(self).expect("ExecutionTrace serialization must not fail")
    }
}
