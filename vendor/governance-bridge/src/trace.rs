use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraceStep {
    pub region_id: String,
    pub operation: String,
    pub input_hash: String,
    pub output_hash: String,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ExecutionTrace(Vec<TraceStep>);

impl ExecutionTrace {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn push(&mut self, step: TraceStep) {
        self.0.push(step);
    }

    pub fn from_steps(steps: Vec<TraceStep>) -> Self {
        Self(steps)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn steps(&self) -> &[TraceStep] {
        &self.0
    }

    pub fn into_steps(self) -> Vec<TraceStep> {
        self.0
    }

    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(&self.0).expect("ExecutionTrace serialization must not fail")
    }
}
