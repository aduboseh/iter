// SCG MCP Integration Test Suite - Adversarial Tests
// Attack simulation tests for MCP boundary protection

use super::common::*;
use serde_json::json;

// ============================================================================
// Unicode Obfuscation Attacks
// ============================================================================

#[test]
fn test_unicode_zero_width_space_in_param() {
    let mut runtime = create_test_runtime();

    // Zero-width space in field value
    let response = execute_tool(&mut runtime, "node.query", json!({
        "node_id": "dag\u{200B}_topology"  // Zero-width space
    }));

    // Should either reject or sanitize
    response.assert_no_forbidden_fields();
}

#[test]
fn test_unicode_zero_width_non_joiner() {
    let mut runtime = create_test_runtime();

    let response = execute_tool(&mut runtime, "node.query", json!({
        "node_id": "esv\u{200C}_raw"  // Zero-width non-joiner
    }));

    response.assert_no_forbidden_fields();
}

#[test]
fn test_unicode_cyrillic_lookalike() {
    let mut runtime = create_test_runtime();

    // Cyrillic 'а' looks like Latin 'a'
    let response = execute_tool(&mut runtime, "node.query", json!({
        "node_id": "dаg_topology"  // Cyrillic а
    }));

    response.assert_no_forbidden_fields();
}

#[test]
fn test_unicode_mixed_obfuscation() {
    let mut runtime = create_test_runtime();

    // Mix of zero-width chars and lookalikes
    let response = execute_tool(&mut runtime, "node.query", json!({
        "node_id": "e\u{200C}sv\u{200D}_rаw"  // Mixed obfuscation
    }));

    response.assert_no_forbidden_fields();
}

#[test]
fn test_unicode_bom_injection() {
    let mut runtime = create_test_runtime();

    // BOM character
    let response = execute_tool(&mut runtime, "node.query", json!({
        "node_id": "\u{FEFF}adjacency_list"
    }));

    response.assert_no_forbidden_fields();
}

// ============================================================================
// Injection Attacks
// ============================================================================

#[test]
fn test_sql_injection_style_prompt() {
    let mut runtime = create_test_runtime();

    let response = execute_tool(&mut runtime, "node.query", json!({
        "node_id": "'; DROP TABLE nodes; SELECT * FROM esv_raw WHERE '1'='1"
    }));

    // Must not expose internal state even with injection attempt
    response.assert_no_forbidden_fields();
    let raw = &response.raw_json;
    assert!(!raw.contains("esv_raw"));
    assert!(!raw.contains("dag_topology"));
}

#[test]
fn test_path_traversal_attempt() {
    let mut runtime = create_test_runtime();

    let response = execute_tool(&mut runtime, "node.query", json!({
        "node_id": "../../../etc/passwd"
    }));

    response.assert_no_forbidden_fields();
}

#[test]
fn test_json_injection_in_string() {
    let mut runtime = create_test_runtime();

    // Attempt to inject JSON as string value
    let response = execute_tool(&mut runtime, "node.query", json!({
        "node_id": "{\"dag_topology\": {\"nodes\": [1,2,3]}}"
    }));

    response.assert_no_forbidden_fields();
    
    // The response should not echo back the injected JSON structure
    let raw = &response.raw_json;
    assert!(!raw.contains("\"dag_topology\""));
}

#[test]
fn test_nested_json_injection() {
    let mut runtime = create_test_runtime();

    // Attempt deeply nested JSON injection
    let response = execute_tool(&mut runtime, "node.query", json!({
        "node_id": "test",
        "extra": {
            "nested": {
                "deep": {
                    "energy_matrix": [[1,2],[3,4]]
                }
            }
        }
    }));

    response.assert_no_forbidden_fields();
}

// ============================================================================
// Field Request Attacks
// ============================================================================

#[test]
fn test_explicit_internal_field_request() {
    let mut runtime = create_test_runtime();

    let response = execute_tool(&mut runtime, "node.create", json!({
        "belief": 0.5,
        "energy": 100.0,
        "include_internals": true,
        "show_dag_topology": true,
        "expose_esv": true
    }));

    // Even with explicit requests for internals, must not expose them
    response.assert_no_forbidden_fields();
}

#[test]
fn test_extra_field_injection() {
    let mut runtime = create_test_runtime();

    let response = execute_tool(&mut runtime, "node.create", json!({
        "belief": 0.5,
        "energy": 100.0,
        "dag_topology": {"nodes": [1,2,3]},
        "esv_raw": [0.8, 0.1, 0.9],
        "internal_state": {"secret": "data"}
    }));

    // Extra fields should be ignored, not echoed back
    response.assert_no_forbidden_fields();
}

// ============================================================================
// Large Payload Attacks
// ============================================================================

#[test]
fn test_large_string_payload() {
    let mut runtime = create_test_runtime();

    // 10KB string
    let response = execute_tool(&mut runtime, "node.query", json!({
        "node_id": "A".repeat(10000)
    }));

    // Should handle gracefully without crashing or exposing internals
    response.assert_no_forbidden_fields();
}

#[test]
fn test_deeply_nested_json() {
    let mut runtime = create_test_runtime();

    // Create deeply nested JSON
    let mut nested = json!({"value": 1});
    for _ in 0..50 {
        nested = json!({"nested": nested});
    }

    let response = execute_tool(&mut runtime, "node.query", json!({
        "node_id": "test",
        "data": nested
    }));

    response.assert_no_forbidden_fields();
}

#[test]
fn test_large_array_payload() {
    let mut runtime = create_test_runtime();

    // Large array
    let large_array: Vec<i32> = (0..1000).collect();

    let response = execute_tool(&mut runtime, "node.query", json!({
        "node_id": "test",
        "data": large_array
    }));

    response.assert_no_forbidden_fields();
}

// ============================================================================
// Batch Attack Simulation
// ============================================================================

#[test]
fn test_all_adversarial_payloads() {
    let mut runtime = create_test_runtime();

    for (idx, payload) in adversarial_payloads().iter().enumerate() {
        // Try each payload as node.query params
        let response = execute_tool(&mut runtime, "node.query", payload.clone());

        // All adversarial payloads must be either rejected or sanitized
        assert!(
            response.is_error() || response.is_success(),
            "Adversarial payload {} caused unexpected response",
            idx
        );

        response.assert_no_forbidden_fields();
    }
}

#[test]
fn test_adversarial_payloads_on_multiple_endpoints() {
    let mut runtime = create_test_runtime();

    // First create a valid node for some operations
    let create_resp = execute_tool(&mut runtime, "node.create", json!({
        "belief": 0.5,
        "energy": 100.0
    }));
    assert!(create_resp.is_success());

    let tools = ["node.query", "node.mutate", "esv.audit"];

    for payload in adversarial_payloads() {
        for tool in &tools {
            let response = execute_tool(&mut runtime, tool, payload.clone());
            response.assert_no_forbidden_fields();
        }
    }
}

// ============================================================================
// Timing-Based Probing
// ============================================================================

#[test]
fn test_consistent_response_structure() {
    let mut runtime = create_test_runtime();

    // Create a node
    let create_resp = execute_tool(&mut runtime, "node.create", json!({
        "belief": 0.5,
        "energy": 100.0
    }));
    let node_id = extract_node_id(&create_resp);

    // Query valid node
    let valid_resp = execute_tool(&mut runtime, "node.query", json!({
        "node_id": node_id
    }));
    assert!(valid_resp.is_success());
    valid_resp.assert_no_forbidden_fields();

    // Query invalid node
    let invalid_resp = execute_tool(&mut runtime, "node.query", json!({
        "node_id": "999999"
    }));
    assert!(invalid_resp.is_error());
    invalid_resp.assert_no_forbidden_fields();

    // Both responses should have similar structure (not leak more info in one case)
    // Valid has result, invalid has error - but neither should expose internals
}

// ============================================================================
// State Manipulation Attacks
// ============================================================================

#[test]
fn test_rapid_operation_sequence() {
    let mut runtime = create_test_runtime();

    // Rapid sequence of operations to try to cause state leakage
    for i in 0..20 {
        let resp = execute_tool(&mut runtime, "node.create", json!({
            "belief": (i as f64) * 0.05,
            "energy": 10.0 * (i as f64)
        }));
        
        // Every response must be sanitized
        resp.assert_no_forbidden_fields();
    }

    // Final state check
    let gov_resp = execute_tool(&mut runtime, "governor.status", json!({}));
    gov_resp.assert_no_forbidden_fields();
}

#[test]
fn test_interleaved_valid_invalid_operations() {
    let mut runtime = create_test_runtime();

    for i in 0..10 {
        if i % 2 == 0 {
            // Valid operation
            let resp = execute_tool(&mut runtime, "node.create", json!({
                "belief": 0.5,
                "energy": 100.0
            }));
            assert!(resp.is_success());
            resp.assert_no_forbidden_fields();
        } else {
            // Invalid operation
            let resp = execute_tool(&mut runtime, "node.query", json!({
                "node_id": "invalid"
            }));
            assert!(resp.is_error());
            resp.assert_no_forbidden_fields();
        }
    }
}

// ============================================================================
// Protocol-Level Attacks
// ============================================================================

#[test]
fn test_missing_jsonrpc_version() {
    let mut runtime = create_test_runtime();
    
    // Build a request manually with missing/wrong version
    use scg_mcp_server::types::RpcRequest;
    let request = RpcRequest {
        jsonrpc: "1.0".to_string(),  // Wrong version
        method: "node.create".to_string(),
        params: json!({"belief": 0.5, "energy": 100.0}),
        id: Some(json!(1)),
    };

    let response = execute_rpc(&mut runtime, request);
    response.assert_no_forbidden_fields();
}

#[test]
fn test_null_id_request() {
    let mut runtime = create_test_runtime();
    
    use scg_mcp_server::types::RpcRequest;
    let request = RpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "node.create".to_string(),
        params: json!({"belief": 0.5, "energy": 100.0}),
        id: None,  // No ID
    };

    let response = execute_rpc(&mut runtime, request);
    response.assert_no_forbidden_fields();
}

#[test]
fn test_empty_method() {
    let mut runtime = create_test_runtime();
    let request = build_rpc_request("", json!({}));
    let response = execute_rpc(&mut runtime, request);
    
    // Empty method should error but not expose internals
    response.assert_no_forbidden_fields();
}
