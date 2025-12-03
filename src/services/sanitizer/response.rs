// SCG Governance: Deterministic | ESV-Compliant | Drift ≤1e-10
// Lineage: MCP_BOUNDARY_V2.0
// Generated under SCG_Governance_v1.0

//! Response Sanitizer Module
//!
//! This module ensures that no substrate internals leak through the MCP boundary.
//! All responses are sanitized to remove:
//! - DAG topology information
//! - Raw ESV values
//! - Internal energy matrices
//! - Lineage chain details (only hashes are exposed)
//!
//! Zero-Touch Zones (from Hardening Directive v2.0):
//! - ❌ No substrate introspection endpoints
//! - ❌ No DAG topology logging
//! - ❌ No ESV/energy matrix exposure
//! - ❌ No debug interfaces that leak substrate internals

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::forbidden::{is_forbidden, normalize_for_matching, FORBIDDEN_PATTERNS};

/// Sanitization result
#[derive(Debug, Clone)]
pub struct SanitizationResult {
    pub sanitized: Value,
    pub fields_removed: Vec<String>,
    pub warnings: Vec<String>,
}

/// Sanitizer for MCP responses
pub struct ResponseSanitizer {
    allow_belief_values: bool,
    allow_energy_summary: bool,
    max_lineage_entries: usize,
}

impl Default for ResponseSanitizer {
    fn default() -> Self {
        Self::new()
    }
}

impl ResponseSanitizer {
    /// Create a new sanitizer with default settings
    pub fn new() -> Self {
        Self {
            allow_belief_values: true,  // Individual beliefs are OK
            allow_energy_summary: true, // Aggregate energy metrics are OK
            max_lineage_entries: 10,    // Limit exposed lineage depth
        }
    }

    /// Create a strict sanitizer that exposes minimal information
    pub fn strict() -> Self {
        Self {
            allow_belief_values: false,
            allow_energy_summary: false,
            max_lineage_entries: 1,
        }
    }

    /// Sanitize a JSON response
    pub fn sanitize(&self, value: Value) -> SanitizationResult {
        let mut fields_removed = Vec::new();
        let mut warnings = Vec::new();

        let sanitized = self.sanitize_value(value, &mut fields_removed, &mut warnings);

        SanitizationResult {
            sanitized,
            fields_removed,
            warnings,
        }
    }

    fn sanitize_value(
        &self,
        value: Value,
        removed: &mut Vec<String>,
        warnings: &mut Vec<String>,
    ) -> Value {
        match value {
            Value::Object(map) => {
                let mut new_map = serde_json::Map::new();
                
                for (key, val) in map {
                    // Check if field is forbidden (with Unicode normalization)
                    if is_forbidden(&key) {
                        removed.push(key.clone());
                        continue;
                    }

                    // Apply additional restrictions
                    if !self.allow_belief_values && key.to_lowercase().contains("belief") {
                        removed.push(key.clone());
                        continue;
                    }

                    if !self.allow_energy_summary && key.to_lowercase().contains("energy") {
                        removed.push(key.clone());
                        continue;
                    }

                    // Recursively sanitize nested values
                    let sanitized_val = self.sanitize_value(val, removed, warnings);
                    new_map.insert(key, sanitized_val);
                }

                Value::Object(new_map)
            }
            Value::Array(arr) => {
                let sanitized: Vec<Value> = arr
                    .into_iter()
                    .map(|v| self.sanitize_value(v, removed, warnings))
                    .collect();
                
                // Limit array size for lineage-like data
                if sanitized.len() > self.max_lineage_entries {
                    warnings.push(format!(
                        "Array truncated from {} to {} entries",
                        sanitized.len(),
                        self.max_lineage_entries
                    ));
                    Value::Array(sanitized.into_iter().take(self.max_lineage_entries).collect())
                } else {
                    Value::Array(sanitized)
                }
            }
            // For string values, check if they contain forbidden patterns
            Value::String(s) => {
                // Check if the string value itself contains forbidden patterns
                // This catches cases like {"prompt": "dag_topology"}
                let violations = super::forbidden::contains_forbidden(&s);
                if !violations.is_empty() {
                    warnings.push(format!(
                        "String value contained forbidden patterns: {:?}",
                        violations
                    ));
                }
                Value::String(s)
            }
            // Primitives pass through unchanged
            other => other,
        }
    }

    /// Check if a response contains any forbidden fields
    pub fn check_for_leakage(&self, value: &Value) -> Vec<String> {
        let mut leaks = Vec::new();
        self.find_leaks(value, "", &mut leaks);
        leaks
    }

    fn find_leaks(&self, value: &Value, path: &str, leaks: &mut Vec<String>) {
        match value {
            Value::Object(map) => {
                for (key, val) in map {
                    let full_path = if path.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", path, key)
                    };

                    if is_forbidden(key) {
                        leaks.push(full_path.clone());
                    }

                    self.find_leaks(val, &full_path, leaks);
                }
            }
            Value::Array(arr) => {
                for (i, val) in arr.iter().enumerate() {
                    let full_path = format!("{}[{}]", path, i);
                    self.find_leaks(val, &full_path, leaks);
                }
            }
            _ => {}
        }
    }

    /// Check raw text for forbidden patterns (with Unicode normalization)
    pub fn check_raw_text(&self, text: &str) -> Vec<String> {
        let normalized = normalize_for_matching(text);
        FORBIDDEN_PATTERNS
            .iter()
            .filter(|pattern| normalized.contains(&normalize_for_matching(pattern)))
            .map(|s| s.to_string())
            .collect()
    }
}

/// Sanitized node state for external exposure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanitizedNodeState {
    pub id: String,
    pub belief: f64,
    pub esv_valid: bool,  // Only validity, not raw values
}

/// Sanitized governor status for external exposure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanitizedGovernorStatus {
    pub stable: bool,
    pub drift_ok: bool,
    pub coherence_ok: bool,
    // No raw drift values, no raw coherence values
}

/// Sanitized trace summary for external exposure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanitizedTraceSummary {
    pub trace_id: String,
    pub lineage_hash: String,  // Only the hash, not the chain
    pub verified: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_sanitizer_removes_forbidden_fields() {
        let sanitizer = ResponseSanitizer::new();
        
        let input = json!({
            "id": "node-1",
            "belief": 0.5,
            "adjacency_list": [[1, 2], [2, 3]],
            "esv_raw": [0.8, 0.1, 0.9],
            "esv_valid": true
        });

        let result = sanitizer.sanitize(input);
        
        assert!(result.fields_removed.contains(&"adjacency_list".to_string()));
        assert!(result.fields_removed.contains(&"esv_raw".to_string()));
        assert!(!result.sanitized.get("adjacency_list").is_some());
        assert!(!result.sanitized.get("esv_raw").is_some());
        assert!(result.sanitized.get("id").is_some());
        assert!(result.sanitized.get("belief").is_some());
        assert!(result.sanitized.get("esv_valid").is_some());
    }

    #[test]
    fn test_strict_sanitizer() {
        let sanitizer = ResponseSanitizer::strict();
        
        let input = json!({
            "id": "node-1",
            "belief": 0.5,
            "energy_total": 100.0
        });

        let result = sanitizer.sanitize(input);
        
        assert!(result.fields_removed.contains(&"belief".to_string()));
        assert!(result.fields_removed.contains(&"energy_total".to_string()));
    }

    #[test]
    fn test_leakage_detection() {
        let sanitizer = ResponseSanitizer::new();
        
        let value = json!({
            "data": {
                "node": {
                    "adjacency": [1, 2, 3]
                }
            }
        });

        let leaks = sanitizer.check_for_leakage(&value);
        assert!(!leaks.is_empty());
        assert!(leaks[0].contains("adjacency"));
    }

    #[test]
    fn test_nested_sanitization() {
        let sanitizer = ResponseSanitizer::new();
        
        let input = json!({
            "response": {
                "node": {
                    "id": "test",
                    "internal_state": {"secret": "data"}
                }
            }
        });

        let result = sanitizer.sanitize(input);
        assert!(result.fields_removed.contains(&"internal_state".to_string()));
    }

    #[test]
    fn test_unicode_obfuscated_field_removal() {
        let sanitizer = ResponseSanitizer::new();
        
        // Using zero-width space in field name
        let input = json!({
            "id": "test",
            "dag\u{200B}_topology": {"nodes": [1, 2, 3]}
        });

        let result = sanitizer.sanitize(input);
        assert!(result.fields_removed.contains(&"dag\u{200B}_topology".to_string()));
        assert!(!result.sanitized.get("dag\u{200B}_topology").is_some());
    }

    #[test]
    fn test_check_raw_text() {
        let sanitizer = ResponseSanitizer::new();
        
        let text = r#"{"dag_topology": [], "energy_matrix": [[1,2]]}"#;
        let violations = sanitizer.check_raw_text(text);
        
        assert!(violations.contains(&"dag_topology".to_string()));
        assert!(violations.contains(&"energy_matrix".to_string()));
    }

    #[test]
    fn test_check_raw_text_with_unicode_obfuscation() {
        let sanitizer = ResponseSanitizer::new();
        
        let text = r#"{"dаg_topology": []}"#; // Cyrillic 'а'
        let violations = sanitizer.check_raw_text(text);
        
        assert!(violations.contains(&"dag_topology".to_string()));
    }
}
