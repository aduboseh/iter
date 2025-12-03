// SCG MCP Integration Test Suite - Error Handling Tests
// Validates error response sanitization

use super::common::*;
use serde_json::json;

#[test]
fn test_malformed_json_params_error() {
    let runtime = create_test_runtime();

    // Missing required params
    let response = execute_tool(&runtime, "node.create", json!({}));

    assert!(response.is_error());
    response.assert_no_forbidden_fields();
}

#[test]
fn test_invalid_uuid_error() {
    let runtime = create_test_runtime();

    let response = execute_tool(&runtime, "node.query", json!({
        "node_id": "not-a-valid-uuid"
    }));

    assert!(response.is_error());
    response.assert_no_forbidden_fields();

    // Error should not expose stack traces
    let raw = &response.raw_json;
    assert!(!raw.contains("stack_trace"));
    assert!(!raw.contains("backtrace"));
    assert!(!raw.contains("panic"));
}

#[test]
fn test_not_found_error() {
    let runtime = create_test_runtime();

    let response = execute_tool(&runtime, "node.query", json!({
        "node_id": "00000000-0000-0000-0000-000000000000"
    }));

    assert!(response.is_error());
    assert_eq!(response.error_code(), Some(4004));
    response.assert_no_forbidden_fields();
}

#[test]
fn test_unknown_method_error() {
    let runtime = create_test_runtime();
    let request = build_rpc_request("nonexistent.method", json!({}));
    let response = execute_rpc(&runtime, request);

    assert!(response.is_error());
    response.assert_no_forbidden_fields();
}

#[test]
fn test_unknown_tool_error() {
    let runtime = create_test_runtime();

    let response = execute_tool(&runtime, "nonexistent.tool", json!({}));

    assert!(response.is_error());
    response.assert_no_forbidden_fields();
}

#[test]
fn test_invalid_tool_arguments_error() {
    let runtime = create_test_runtime();

    // String where number expected
    let response = execute_tool(&runtime, "node.create", json!({
        "belief": "not-a-number",
        "energy": 100.0
    }));

    assert!(response.is_error());
    response.assert_no_forbidden_fields();
}

#[test]
fn test_edge_bind_missing_nodes_error() {
    let runtime = create_test_runtime();

    let response = execute_tool(&runtime, "edge.bind", json!({
        "src": "00000000-0000-0000-0000-000000000000",
        "dst": "00000000-0000-0000-0000-000000000001",
        "weight": 0.5
    }));

    assert!(response.is_error());
    response.assert_no_forbidden_fields();

    // Error should not expose internal DAG state
    let raw = &response.raw_json;
    assert!(!raw.contains("adjacency"));
    assert!(!raw.contains("dag_topology"));
    assert!(!raw.contains("internal_edges"));
}

#[test]
fn test_esv_error_no_raw_values() {
    let runtime = create_test_runtime();

    // Create a node
    let create_resp = execute_tool(&runtime, "node.create", json!({
        "belief": 0.5,
        "energy": 100.0
    }));
    let node: serde_json::Value = serde_json::from_str(&create_resp.get_content_text().unwrap()).unwrap();
    let node_id = node["id"].as_str().unwrap();

    // Mutate with extreme delta (ESV validation should trigger)
    let mutate_resp = execute_tool(&runtime, "node.mutate", json!({
        "node_id": node_id,
        "delta": 1000.0  // Extreme value
    }));

    // Whether success (clamped) or error, no raw ESV values should leak
    mutate_resp.assert_no_forbidden_fields();
    
    let raw = &mutate_resp.raw_json;
    assert!(!raw.contains("esv_raw"));
    assert!(!raw.contains("ethical_gradient"));
    assert!(!raw.contains("harm_potential_raw"));
}

#[test]
fn test_propagate_error_no_internal_leakage() {
    let runtime = create_test_runtime();

    let response = execute_tool(&runtime, "edge.propagate", json!({
        "edge_id": "00000000-0000-0000-0000-000000000000"
    }));

    assert!(response.is_error());
    response.assert_no_forbidden_fields();

    // Should not expose propagation internals
    let raw = &response.raw_json;
    assert!(!raw.contains("propagation_path"));
    assert!(!raw.contains("energy_redistribution"));
    assert!(!raw.contains("belief_vector"));
}

#[test]
fn test_consecutive_errors_no_state_leakage() {
    let runtime = create_test_runtime();

    // Multiple error requests should not accumulate or leak state
    for _ in 0..5 {
        let response = execute_tool(&runtime, "node.query", json!({
            "node_id": "00000000-0000-0000-0000-000000000000"
        }));
        assert!(response.is_error());
        response.assert_no_forbidden_fields();
    }

    // Governor status should still be sanitized
    let gov_resp = execute_tool(&runtime, "governor.status", json!({}));
    gov_resp.assert_no_forbidden_fields();
}

#[test]
fn test_partial_params_error() {
    let runtime = create_test_runtime();

    // Only belief, missing energy
    let response = execute_tool(&runtime, "node.create", json!({
        "belief": 0.5
    }));

    assert!(response.is_error());
    response.assert_no_forbidden_fields();
}

#[test]
fn test_null_params_error() {
    let runtime = create_test_runtime();

    let response = execute_tool(&runtime, "node.create", json!({
        "belief": null,
        "energy": null
    }));

    assert!(response.is_error());
    response.assert_no_forbidden_fields();
}

#[test]
fn test_empty_string_uuid_error() {
    let runtime = create_test_runtime();

    let response = execute_tool(&runtime, "node.query", json!({
        "node_id": ""
    }));

    assert!(response.is_error());
    response.assert_no_forbidden_fields();
}

#[test]
fn test_negative_energy_handling() {
    let runtime = create_test_runtime();

    // Negative energy - should be handled gracefully
    let response = execute_tool(&runtime, "node.create", json!({
        "belief": 0.5,
        "energy": -100.0
    }));

    // Whether success or error, must be sanitized
    response.assert_no_forbidden_fields();
}

#[test]
fn test_nan_value_handling() {
    let runtime = create_test_runtime();

    // NaN belief (as string since JSON doesn't support NaN)
    let response = execute_tool(&runtime, "node.create", json!({
        "belief": "NaN",
        "energy": 100.0
    }));

    // Should error but not expose internals
    response.assert_no_forbidden_fields();
}

#[test]
fn test_infinity_value_handling() {
    let runtime = create_test_runtime();

    // Infinity as string
    let response = execute_tool(&runtime, "node.create", json!({
        "belief": 0.5,
        "energy": "Infinity"
    }));

    // Should error but not expose internals
    response.assert_no_forbidden_fields();
}
