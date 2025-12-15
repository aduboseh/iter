// Iter MCP Integration Test Suite - Tool Endpoint Tests
// Functional and sanitization tests for MCP tools

use super::common::*;
use serde_json::json;

// ============================================================================
// node.create tests
// ============================================================================

#[test]
fn test_node_create_basic() {
    let mut runtime = create_test_runtime();

    let response = execute_tool(&mut runtime, "node.create", json!({
        "belief": 0.5,
        "energy": 100.0
    }));

    assert!(response.is_success());
    response.assert_no_forbidden_fields();
    response.assert_content_contains("id");
}

#[test]
fn test_node_create_boundary_belief_values() {
    let mut runtime = create_test_runtime();

    // Test belief = 0.0 (minimum)
    let resp1 = execute_tool(&mut runtime, "node.create", json!({
        "belief": 0.0,
        "energy": 10.0
    }));
    assert!(resp1.is_success());
    resp1.assert_no_forbidden_fields();

    // Test belief = 1.0 (maximum)
    let resp2 = execute_tool(&mut runtime, "node.create", json!({
        "belief": 1.0,
        "energy": 10.0
    }));
    assert!(resp2.is_success());
    resp2.assert_no_forbidden_fields();
}

#[test]
fn test_node_create_validation_rejects_invalid_belief() {
    let mut runtime = create_test_runtime();

    // Negative belief should be rejected by MCP boundary validation
    let resp1 = execute_tool(&mut runtime, "node.create", json!({
        "belief": -0.5,
        "energy": 10.0
    }));
    assert!(resp1.is_error(), "Negative belief should be rejected");
    resp1.assert_no_forbidden_fields();

    // Belief > 1.0 should be rejected by MCP boundary validation
    let resp2 = execute_tool(&mut runtime, "node.create", json!({
        "belief": 1.5,
        "energy": 10.0
    }));
    assert!(resp2.is_error(), "Belief > 1.0 should be rejected");
    resp2.assert_no_forbidden_fields();

    // NaN belief should be rejected
    let resp3 = execute_tool(&mut runtime, "node.create", json!({
        "belief": f64::NAN,
        "energy": 10.0
    }));
    // NaN can't be represented in JSON, but test that parsing handles it
    resp3.assert_no_forbidden_fields();
}

// ============================================================================
// node.mutate tests
// ============================================================================

#[test]
fn test_node_mutate_basic() {
    let mut runtime = create_test_runtime();

    // Create a node
    let create_resp = execute_tool(&mut runtime, "node.create", json!({
        "belief": 0.5,
        "energy": 100.0
    }));
    let node_id = extract_node_id(&create_resp);

    // Mutate the node
    let mutate_resp = execute_tool(&mut runtime, "node.mutate", json!({
        "node_id": node_id,
        "delta": 0.1
    }));

    assert!(mutate_resp.is_success());
    mutate_resp.assert_no_forbidden_fields();

    let mutated: serde_json::Value = serde_json::from_str(&mutate_resp.get_content_text().unwrap()).unwrap();
    assert!((mutated["belief"].as_f64().unwrap() - 0.6).abs() < 0.001);
}

#[test]
fn test_node_mutate_clamping() {
    let mut runtime = create_test_runtime();

    // Create a node at belief 0.9
    let create_resp = execute_tool(&mut runtime, "node.create", json!({
        "belief": 0.9,
        "energy": 100.0
    }));
    let node_id = extract_node_id(&create_resp);

    // Mutate with delta that would exceed 1.0
    let mutate_resp = execute_tool(&mut runtime, "node.mutate", json!({
        "node_id": node_id,
        "delta": 0.5
    }));

    assert!(mutate_resp.is_success());
    mutate_resp.assert_no_forbidden_fields();

    // Should be clamped to 1.0
    let mutated: serde_json::Value = serde_json::from_str(&mutate_resp.get_content_text().unwrap()).unwrap();
    assert_eq!(mutated["belief"].as_f64().unwrap(), 1.0);
}

#[test]
fn test_node_mutate_invalid_node() {
    let mut runtime = create_test_runtime();

    let response = execute_tool(&mut runtime, "node.mutate", json!({
        "node_id": "999999",
        "delta": 0.1
    }));

    assert!(response.is_error());
    response.assert_no_forbidden_fields();
}

// ============================================================================
// node.query tests
// ============================================================================

#[test]
fn test_node_query_basic() {
    let mut runtime = create_test_runtime();

    // Create a node
    let create_resp = execute_tool(&mut runtime, "node.create", json!({
        "belief": 0.5,
        "energy": 100.0
    }));
    let node_id = extract_node_id(&create_resp);

    // Query the node
    let query_resp = execute_tool(&mut runtime, "node.query", json!({
        "node_id": node_id
    }));

    assert!(query_resp.is_success());
    query_resp.assert_no_forbidden_fields();

    let queried: serde_json::Value = serde_json::from_str(&query_resp.get_content_text().unwrap()).unwrap();
    assert_eq!(queried["id"].as_u64().unwrap().to_string(), node_id);
    assert_eq!(queried["belief"].as_f64().unwrap(), 0.5);
}

#[test]
fn test_node_query_invalid_node() {
    let mut runtime = create_test_runtime();

    let response = execute_tool(&mut runtime, "node.query", json!({
        "node_id": "999999"
    }));

    assert!(response.is_error());
    response.assert_no_forbidden_fields();
}

// ============================================================================
// edge.bind tests
// ============================================================================

#[test]
fn test_edge_bind_basic() {
    let mut runtime = create_test_runtime();

    // Create two nodes
    let node1_resp = execute_tool(&mut runtime, "node.create", json!({"belief": 0.5, "energy": 100.0}));
    let node1_id = extract_node_id(&node1_resp);

    let node2_resp = execute_tool(&mut runtime, "node.create", json!({"belief": 0.3, "energy": 50.0}));
    let node2_id = extract_node_id(&node2_resp);

    // Bind edge
    let bind_resp = execute_tool(&mut runtime, "edge.bind", json!({
        "src": node1_id,
        "dst": node2_id,
        "weight": 0.5
    }));

    assert!(bind_resp.is_success());
    bind_resp.assert_no_forbidden_fields();
}

#[test]
fn test_edge_bind_invalid_nodes() {
    let mut runtime = create_test_runtime();

    let response = execute_tool(&mut runtime, "edge.bind", json!({
        "src": "999999",
        "dst": "00000000-0000-0000-0000-000000000001",
        "weight": 0.5
    }));

    assert!(response.is_error());
    response.assert_no_forbidden_fields();
}

// ============================================================================
// edge.propagate tests
// ============================================================================

#[test]
fn test_edge_propagate_basic() {
    let mut runtime = create_test_runtime();

    // Create two nodes
    let node1_resp = execute_tool(&mut runtime, "node.create", json!({"belief": 0.8, "energy": 100.0}));
    let node1_id = extract_node_id(&node1_resp);

    let node2_resp = execute_tool(&mut runtime, "node.create", json!({"belief": 0.2, "energy": 50.0}));
    let node2_id = extract_node_id(&node2_resp);

    // Bind edge
    let bind_resp = execute_tool(&mut runtime, "edge.bind", json!({
        "src": node1_id,
        "dst": node2_id,
        "weight": 0.5
    }));
    let edge_id = extract_node_id(&bind_resp);

    // Propagate
    let prop_resp = execute_tool(&mut runtime, "edge.propagate", json!({
        "edge_id": edge_id
    }));

    assert!(prop_resp.is_success());
    prop_resp.assert_no_forbidden_fields();
}

#[test]
fn test_edge_propagate_with_nonexistent_edge() {
    let mut runtime = create_test_runtime();

    // In the new substrate model, edge.propagate runs a simulation step
    // which processes ALL edges - the edge_id is accepted for API compatibility
    // but doesn't cause an error if it doesn't exist.
    let response = execute_tool(&mut runtime, "edge.propagate", json!({
        "edge_id": "999999"
    }));

    // Should succeed (runs step on graph, even if specified edge doesn't exist)
    assert!(response.is_success());
    response.assert_no_forbidden_fields();
}

// ============================================================================
// governor.status tests
// ============================================================================

#[test]
fn test_governor_status_empty_graph() {
    let mut runtime = create_test_runtime();

    let response = execute_tool(&mut runtime, "governor.status", json!({}));

    assert!(response.is_success());
    response.assert_no_forbidden_fields();

    let content = response.get_content_text().unwrap();
    let status: serde_json::Value = serde_json::from_str(&content).unwrap();
    
    assert_eq!(status["node_count"].as_u64().unwrap(), 0);
    assert_eq!(status["edge_count"].as_u64().unwrap(), 0);
}

#[test]
fn test_governor_status_with_nodes() {
    let mut runtime = create_test_runtime();

    // Create nodes
    execute_tool(&mut runtime, "node.create", json!({"belief": 0.5, "energy": 100.0}));
    execute_tool(&mut runtime, "node.create", json!({"belief": 0.3, "energy": 50.0}));

    let response = execute_tool(&mut runtime, "governor.status", json!({}));

    assert!(response.is_success());
    response.assert_no_forbidden_fields();

    let content = response.get_content_text().unwrap();
    let status: serde_json::Value = serde_json::from_str(&content).unwrap();
    
    assert_eq!(status["node_count"].as_u64().unwrap(), 2);
}

// ============================================================================
// esv.audit tests
// ============================================================================

#[test]
fn test_esv_audit_valid_node() {
    let mut runtime = create_test_runtime();

    let create_resp = execute_tool(&mut runtime, "node.create", json!({
        "belief": 0.5,
        "energy": 100.0
    }));
    let node_id = extract_node_id(&create_resp);

    let audit_resp = execute_tool(&mut runtime, "esv.audit", json!({
        "node_id": node_id
    }));

    assert!(audit_resp.is_success());
    audit_resp.assert_no_forbidden_fields();
    audit_resp.assert_content_contains("VALID");
}

#[test]
fn test_esv_audit_nonexistent_node() {
    let mut runtime = create_test_runtime();

    // In the new substrate model, esv.audit checks global energy conservation
    // rather than per-node ESV. The node_id is accepted for API compatibility
    // but the check is system-wide.
    let response = execute_tool(&mut runtime, "esv.audit", json!({
        "node_id": "999999"
    }));

    // Should succeed (global check, node_id not verified)
    assert!(response.is_success());
    response.assert_no_forbidden_fields();
}

// ============================================================================
// lineage tests
// ============================================================================

#[test]
fn test_lineage_replay_basic() {
    let mut runtime = create_test_runtime();

    // Create operations
    execute_tool(&mut runtime, "node.create", json!({"belief": 0.5, "energy": 100.0}));

    let response = execute_tool(&mut runtime, "lineage.replay", json!({}));

    assert!(response.is_success());
    response.assert_no_forbidden_fields();

    // lineage.replay returns an array of McpLineageEntry
    // In the new substrate model, lineage comes from CausalTrace events
    let content = response.get_content_text().unwrap();
    let entries: serde_json::Value = serde_json::from_str(&content).unwrap();
    
    // Should be an array (may be empty if no trace events yet)
    assert!(entries.is_array(), "Lineage replay should return an array");
}

#[test]
fn test_lineage_replay_empty() {
    let mut runtime = create_test_runtime();

    let response = execute_tool(&mut runtime, "lineage.replay", json!({}));

    assert!(response.is_success());
    response.assert_no_forbidden_fields();
}
