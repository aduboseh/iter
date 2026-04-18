use crate::errors::BridgeError;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use unicode_normalization::UnicodeNormalization;

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
        let value = serde_json::to_value(self).expect("ExecutionTrace serialization must not fail");
        let canonical = canonicalize(&value);
        serde_json::to_vec(&canonical).expect("canonical serialization must not fail")
    }
}

/// Assert a string field is NFC-normalized.
/// CANON.md Rule 9: NFC normalization is a boundary contract.
/// Returns Err on violation — never panics.
fn assert_nfc(field: &str, value: &str) -> Result<(), BridgeError> {
    let normalized: String = value.nfc().collect();
    if normalized.as_bytes() != value.as_bytes() {
        return Err(BridgeError::TraceDeterminismViolation(format!(
            "NON_CANONICAL_INPUT: field '{}' is not NFC-normalized",
            field
        )));
    }
    Ok(())
}

fn payload_context(payload: &str) -> String {
    const PREVIEW_CHARS: usize = 64;

    let preview: String = payload.chars().take(PREVIEW_CHARS).collect();
    let truncated = payload.chars().count() > PREVIEW_CHARS;

    format!(
        "payload_len={} snippet={:?}{}",
        payload.len(),
        preview,
        if truncated { "..." } else { "" }
    )
}

fn assert_nfc_in_value(field: &str, value: &serde_json::Value) -> Result<(), BridgeError> {
    match value {
        serde_json::Value::String(text) => assert_nfc(field, text),
        serde_json::Value::Array(items) => {
            for (index, item) in items.iter().enumerate() {
                assert_nfc_in_value(&format!("{field}[{index}]"), item)?;
            }
            Ok(())
        }
        serde_json::Value::Object(map) => {
            for (key, nested) in map {
                assert_nfc(
                    &format!("{field} object key {:?} (bytes={})", key, key.len()),
                    key,
                )?;
                assert_nfc_in_value(&format!("{field}.{key}"), nested)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn validate_payload_json_strings(field: &str, payload: &str) -> Result<(), BridgeError> {
    let value: serde_json::Value = serde_json::from_str(payload).map_err(|err| {
        BridgeError::TraceDeterminismViolation(format!(
            "{field} invalid JSON: {err} ({})",
            payload_context(payload)
        ))
    })?;
    assert_nfc_in_value(field, &value)
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
        let value = serde_json::to_value(self).expect("TraceStep serialization must not fail");
        let canonical = canonicalize(&value);
        serde_json::to_vec(&canonical).expect("canonical serialization must not fail")
    }

    /// Validate string metadata and decoded payload strings are NFC-normalized.
    /// Must be called at ingress before any hash computation.
    /// Returns Err on first violation — fail-closed.
    pub fn validate_canonical(&self) -> Result<(), BridgeError> {
        assert_nfc("region_id", &self.region_id)?;
        assert_nfc("operation", &self.operation)?;
        assert_nfc("input_hash", &self.input_hash)?;
        assert_nfc("output_hash", &self.output_hash)?;

        if !self.input_payload.is_empty() {
            validate_payload_json_strings("input_payload", &self.input_payload)?;
        }
        if !self.output_payload.is_empty() {
            validate_payload_json_strings("output_payload", &self.output_payload)?;
        }

        Ok(())
    }

    pub fn verify_hash_binding(&self) -> Result<(), BridgeError> {
        self.validate_canonical()?;

        if self.input_payload.is_empty() {
            return Err(BridgeError::TraceDeterminismViolation(format!(
                "input_payload missing for operation '{}' ({})",
                self.operation,
                self.operation_type.as_str()
            )));
        }

        let expected_input_hash = payload_hash(&self.input_payload).map_err(|err| {
            BridgeError::TraceDeterminismViolation(format!(
                "input_payload invalid JSON for operation '{}' ({}): {err}",
                self.operation,
                self.operation_type.as_str()
            ))
        })?;
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

        let expected_output_hash = payload_hash(&self.output_payload).map_err(|err| {
            BridgeError::TraceDeterminismViolation(format!(
                "output_payload invalid JSON for operation '{}' ({}): {err}",
                self.operation,
                self.operation_type.as_str()
            ))
        })?;
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

pub fn payload_hash(payload: &str) -> Result<String, serde_json::Error> {
    let value: serde_json::Value = serde_json::from_str(payload)?;
    let canonical = canonicalize(&value);
    let bytes = serde_json::to_vec(&canonical).expect("canonical serialization must not fail");
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Ok(hex::encode_upper(hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use super::{assert_nfc_in_value, validate_payload_json_strings};
    use crate::errors::BridgeError;

    #[test]
    fn nfc_error_identifies_object_key() {
        let value = serde_json::json!({ "e\u{301}": "value" });

        let err = assert_nfc_in_value("input_payload", &value).unwrap_err();
        let message = match err {
            BridgeError::TraceDeterminismViolation(message) => message,
            other => panic!("expected TraceDeterminismViolation, got: {other:?}"),
        };

        assert!(message.contains("object key"));
        assert!(message.contains("bytes=3"));
        assert!(message.contains("NON_CANONICAL_INPUT"));
    }

    #[test]
    fn invalid_json_error_includes_payload_context() {
        let err = validate_payload_json_strings("input_payload", "{\"unterminated\"")
            .unwrap_err();
        let message = match err {
            BridgeError::TraceDeterminismViolation(message) => message,
            other => panic!("expected TraceDeterminismViolation, got: {other:?}"),
        };

        assert!(message.contains("input_payload invalid JSON"));
        assert!(message.contains("payload_len=15"));
        assert!(message.contains("snippet"));
    }
}
