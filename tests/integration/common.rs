// SCG MCP Integration Test Suite - Common Utilities
// Validates MCP boundary sanitization under real request/response flows

use scg_mcp_server::mcp_handler::handle_rpc;
use scg_mcp_server::services::sanitizer::{normalize_for_matching, FORBIDDEN_PATTERNS};
use scg_mcp_server::types::{RpcRequest, RpcResponse};
use scg_mcp_server::SubstrateRuntime;
use serde_json::{json, Value};
use std::collections::HashSet;

/// Test runtime builder for integration tests
pub fn create_test_runtime() -> SubstrateRuntime {
    SubstrateRuntime::with_defaults().expect("Failed to create test runtime")
}

/// Build an RPC request for testing
pub fn build_rpc_request(method: &str, params: Value) -> RpcRequest {
    RpcRequest {
        jsonrpc: "2.0".to_string(),
        method: method.to_string(),
        params,
        id: Some(json!(1)),
    }
}

/// Build a tools/call RPC request
pub fn build_tool_call_request(tool_name: &str, arguments: Value) -> RpcRequest {
    RpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "tools/call".to_string(),
        params: json!({
            "name": tool_name,
            "arguments": arguments
        }),
        id: Some(json!(1)),
    }
}

/// Test response wrapper with sanitization validation
pub struct TestResponse {
    pub response: RpcResponse,
    pub raw_json: String,
}

impl TestResponse {
    pub fn from_rpc(response: RpcResponse) -> Self {
        let raw_json = serde_json::to_string(&response).unwrap_or_default();
        Self { response, raw_json }
    }

    /// Check if the response indicates success
    pub fn is_success(&self) -> bool {
        self.response.error.is_none() && self.response.result.is_some()
    }

    /// Check if the response indicates an error
    pub fn is_error(&self) -> bool {
        self.response.error.is_some()
    }

    /// Get the error code if present
    pub fn error_code(&self) -> Option<i32> {
        self.response.error.as_ref().map(|e| e.code)
    }

    /// Get the result value
    pub fn result(&self) -> Option<&Value> {
        self.response.result.as_ref()
    }

    /// Detect forbidden patterns with Unicode normalization
    /// 
    /// This checks for forbidden patterns appearing as JSON field keys,
    /// not as string values or in arrays (which may legitimately describe
    /// tool capabilities like sideEffects).
    pub fn detect_forbidden_patterns(&self) -> HashSet<String> {
        let mut violations = HashSet::new();
        
        // Parse as JSON to check field keys specifically
        if let Ok(parsed) = serde_json::from_str::<Value>(&self.raw_json) {
            Self::find_forbidden_keys(&parsed, &mut violations);
        } else {
            // Fallback to raw text matching for non-JSON responses
            let normalized_text = normalize_for_matching(&self.raw_json);
            for pattern in FORBIDDEN_PATTERNS {
                let normalized_pattern = normalize_for_matching(pattern);
                if normalized_text.contains(&normalized_pattern) {
                    violations.insert(pattern.to_string());
                }
            }
        }

        violations
    }

    /// Recursively find forbidden patterns in JSON field keys (not values)
    fn find_forbidden_keys(value: &Value, violations: &mut HashSet<String>) {
        match value {
            Value::Object(map) => {
                for (key, val) in map {
                    // Check if the KEY matches a forbidden pattern
                    let normalized_key = normalize_for_matching(key);
                    for pattern in FORBIDDEN_PATTERNS {
                        let normalized_pattern = normalize_for_matching(pattern);
                        if normalized_key.contains(&normalized_pattern) {
                            violations.insert(pattern.to_string());
                        }
                    }
                    // Recursively check nested objects
                    Self::find_forbidden_keys(val, violations);
                }
            }
            Value::Array(arr) => {
                // Check nested objects in arrays, but NOT string values
                // (string values like sideEffects are legitimate descriptions)
                for item in arr {
                    if item.is_object() || item.is_array() {
                        Self::find_forbidden_keys(item, violations);
                    }
                }
            }
            _ => {}
        }
    }

    /// Assert no forbidden patterns appear in response
    pub fn assert_no_forbidden_fields(&self) -> &Self {
        let violations = self.detect_forbidden_patterns();
        assert!(
            violations.is_empty(),
            "Found forbidden patterns in response: {:?}\nFull response: {}",
            violations,
            self.raw_json
        );
        self
    }

    /// Assert specific field exists in result
    pub fn assert_result_field_exists(&self, field_path: &str) -> &Self {
        let exists = self.result_field_exists(field_path);
        assert!(
            exists,
            "Expected field '{}' not found in response result: {}",
            field_path,
            self.raw_json
        );
        self
    }

    /// Assert specific field does NOT exist in result
    pub fn assert_result_field_absent(&self, field_path: &str) -> &Self {
        let exists = self.result_field_exists(field_path);
        assert!(
            !exists,
            "Field '{}' should not exist in response result: {}",
            field_path,
            self.raw_json
        );
        self
    }

    /// Check if field exists in result (supports nested paths with dots)
    fn result_field_exists(&self, field_path: &str) -> bool {
        let Some(result) = self.result() else {
            return false;
        };

        let parts: Vec<&str> = field_path.split('.').collect();
        let mut current = result;

        for part in parts {
            match current.get(part) {
                Some(value) => current = value,
                None => return false,
            }
        }

        true
    }

    /// Get the text content from an MCP tool response
    pub fn get_content_text(&self) -> Option<String> {
        self.result()
            .and_then(|r| r.get("content"))
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.first())
            .and_then(|item| item.get("text"))
            .and_then(|t| t.as_str())
            .map(|s| s.to_string())
    }

    /// Assert content text exists and contains substring
    pub fn assert_content_contains(&self, substring: &str) -> &Self {
        let content = self.get_content_text();
        assert!(
            content.is_some(),
            "No content text found in response: {}",
            self.raw_json
        );
        assert!(
            content.as_ref().unwrap().contains(substring),
            "Content does not contain '{}': {}",
            substring,
            content.unwrap()
        );
        self
    }
}

/// Execute an RPC request and return wrapped response
pub fn execute_rpc(runtime: &mut SubstrateRuntime, request: RpcRequest) -> TestResponse {
    let response = handle_rpc(runtime, request);
    TestResponse::from_rpc(response)
}

/// Execute a tool call and return wrapped response
pub fn execute_tool(runtime: &mut SubstrateRuntime, tool_name: &str, arguments: Value) -> TestResponse {
    let request = build_tool_call_request(tool_name, arguments);
    execute_rpc(runtime, request)
}

/// Extract node ID from a node response as a string suitable for API calls.
/// Node IDs are now u64 integers in JSON, so we convert them to strings.
pub fn extract_node_id(response: &TestResponse) -> String {
    let content = response.get_content_text().expect("Response should have content");
    let parsed: Value = serde_json::from_str(&content).expect("Content should be valid JSON");
    
    // ID can be either a number or string in JSON
    if let Some(id) = parsed["id"].as_u64() {
        id.to_string()
    } else if let Some(id) = parsed["id"].as_str() {
        id.to_string()
    } else {
        panic!("Node ID not found or invalid type in response: {}", content)
    }
}

/// Extract edge ID from an edge response as a string suitable for API calls.
pub fn extract_edge_id(response: &TestResponse) -> String {
    let content = response.get_content_text().expect("Response should have content");
    let parsed: Value = serde_json::from_str(&content).expect("Content should be valid JSON");
    
    if let Some(id) = parsed["id"].as_u64() {
        id.to_string()
    } else if let Some(id) = parsed["id"].as_str() {
        id.to_string()
    } else {
        panic!("Edge ID not found or invalid type in response: {}", content)
    }
}

/// Parse a node ID string to u64 for verification.
pub fn parse_node_id(id: &str) -> u64 {
    id.parse::<u64>().expect("Node ID must be a valid u64 string")
}

/// Generate adversarial payloads designed to trigger sanitizer bypass
pub fn adversarial_payloads() -> Vec<Value> {
    vec![
        // Attempt to request internal fields explicitly
        json!({
            "include_internals": true,
            "show_dag_topology": true,
            "expose_esv": true,
        }),
        // SQL-injection-style field injection
        json!({
            "node_id": "'; DROP TABLE nodes; SELECT * FROM esv_raw WHERE '1'='1",
        }),
        // Path traversal in field names
        json!({
            "node_id": "../../../esv_raw",
        }),
        // JSON deserialization confusion
        json!({
            "node_id": "{\"dag_topology\": {\"nodes\": [1,2,3]}}",
        }),
        // Unicode obfuscation (zero-width characters)
        json!({
            "field": "d\u{200B}ag_topology",
        }),
        // Lookalike Unicode characters
        json!({
            "field": "dаg_topology",  // Cyrillic 'a'
        }),
        // Large payload
        json!({
            "data": "A".repeat(10000),
        }),
    ]
}

/// Create snapshot of response for regression detection
pub fn snapshot_response(endpoint: &str, response: &TestResponse) -> Value {
    json!({
        "endpoint": endpoint,
        "is_success": response.is_success(),
        "error_code": response.error_code(),
        "forbidden_check_passed": response.detect_forbidden_patterns().is_empty(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_rpc_request() {
        let req = build_rpc_request("test_method", json!({"key": "value"}));
        assert_eq!(req.jsonrpc, "2.0");
        assert_eq!(req.method, "test_method");
    }

    #[test]
    fn test_build_tool_call_request() {
        let req = build_tool_call_request("node.create", json!({"belief": 0.5, "energy": 1.0}));
        assert_eq!(req.method, "tools/call");
    }

    #[test]
    fn test_adversarial_payloads_generated() {
        let payloads = adversarial_payloads();
        assert!(!payloads.is_empty());
    }
}
