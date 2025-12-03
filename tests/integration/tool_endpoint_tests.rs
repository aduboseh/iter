// SCG MCP Integration Test Suite - Tool Endpoint Tests
// Functional and sanitization tests for MCP tools

use super::common::*;
use serde_json::json;

// ============================================================================
// node.create tests
// ============================================================================

#[test]
fn test_node_create_basic() {
    let runtime = create_test_runtime();

    let response = execute_tool(&runtime, "node.create", json!({
        "belief": 0.5,
        "energy": 100.0
    }));

    assert!(response.is_success());
    response.assert_no_forbidden_fields();
    response.assert_content_contains("id");
}

#[test]
fn test_node_create_boundary_belief_values() {
    let runtime = create_test_runtime();

    // Test belief = 0.0 (minimum)
    let resp1 = execute_tool(&runtime, "node.create", json!({
        "belief": 0.0,
        "energy": 10.0
    }));
    assert!(resp1.is_success());
    resp1.assert_no_forbidden_fields();

    // Test belief = 1.0 (maximum)
    let resp2 = execute_tool(&runtime, "node.create", json!({
        "belief": 1.0,
        "energy": 10.0
    }));
    assert!(resp2.is_success());
    resp2.assert_no_forbidden_fields();
}

#[test]
fn test_node_create_clamping_invalid_belief() {
    let runtime = create_test_runtime();

    // Negative belief should be clamped to 0.0
    let resp1 = execute_tool(&runtime, "node.create", json!({
        "belief": -0.5,
        "energy": 10.0
    }));
    assert!(resp1.is_success());
    resp1.assert_no_forbidden_fields();
    
    let content = resp1.get_content_text().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert_eq!(parsed["belief"].as_f64().unwrap(), 0.0);

    // Belief > 1.0 should be clamped to 1.0
    let resp2 = execute_tool(&runtime, "node.create", json!({
        "belief": 1.5,
        "energy": 10.0
    }));
    assert!(resp2.is_success());
    resp2.assert_no_forbidden_fields();
    
    let content2 = resp2.get_content_text().unwrap();
    let parsed2: serde_json::Value = serde_json::from_str(&content2).unwrap();
    assert_eq!(parsed2["belief"].as_f64().unwrap(), 1.0);
}

// ============================================================================
// node.mutate tests
// ============================================================================

#[test]
fn test_node_mutate_basic() {
    let runtime = create_test_runtime();

    // Create a node
    let create_resp = execute_tool(&runtime, "node.create", json!({
        "belief": 0.5,
        "energy": 100.0
    }));
    let node: serde_json::Value = serde_json::from_str(&create_resp.get_content_text().unwrap()).unwrap();
    let node_id = node["id"].as_str().unwrap();

    // Mutate the node
    let mutate_resp = execute_tool(&runtime, "node.mutate", json!({
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
    let runtime = create_test_runtime();

    // Create a node at belief 0.9
    let create_resp = execute_tool(&runtime, "node.create", json!({
        "belief": 0.9,
        "energy": 100.0
    }));
    let node: serde_json::Value = serde_json::from_str(&create_resp.get_content_text().unwrap()).unwrap();
    let node_id = node["id"].as_str().unwrap();

    // Mutate with delta that would exceed 1.0
    let mutate_resp = execute_tool(&runtime, "node.mutate", json!({
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
    let runtime = create_test_runtime();

    let response = execute_tool(&runtime, "node.mutate", json!({
        "node_id": "00000000-0000-0000-0000-000000000000",
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
    let runtime = create_test_runtime();

    // Create a node
    let create_resp = execute_tool(&runtime, "node.create", json!({
        "belief": 0.5,
        "energy": 100.0
    }));
    let node: serde_json::Value = serde_json::from_str(&create_resp.get_content_text().unwrap()).unwrap();
    let node_id = node["id"].as_str().unwrap();

    // Query the node
    let query_resp = execute_tool(&runtime, "node.query", json!({
        "node_id": node_id
    }));

    assert!(query_resp.is_success());
    query_resp.assert_no_forbidden_fields();

    let queried: serde_json::Value = serde_json::from_str(&query_resp.get_content_text().unwrap()).unwrap();
    assert_eq!(queried["id"].as_str().unwrap(), node_id);
    assert_eq!(queried["belief"].as_f64().unwrap(), 0.5);
}

#[test]
fn test_node_query_invalid_node() {
    let runtime = create_test_runtime();

    let response = execute_tool(&runtime, "node.query", json!({
        "node_id": "00000000-0000-0000-0000-000000000000"
    }));

    assert!(response.is_error());
    response.assert_no_forbidden_fields();
}

// ============================================================================
// edge.bind tests
// ============================================================================

#[test]
fn test_edge_bind_basic() {
    let runtime = create_test_runtime();

    // Create two nodes
    let node1_resp = execute_tool(&runtime, "node.create", json!({"belief": 0.5, "energy": 100.0}));
    let node1: serde_json::Value = serde_json::from_str(&node1_resp.get_content_text().unwrap()).unwrap();
    let node1_id = node1["id"].as_str().unwrap();

    let node2_resp = execute_tool(&runtime, "node.create", json!({"belief": 0.3, "energy": 50.0}));
    let node2: serde_json::Value = serde_json::from_str(&node2_resp.get_content_text().unwrap()).unwrap();
    let node2_id = node2["id"].as_str().unwrap();

    // Bind edge
    let bind_resp = execute_tool(&runtime, "edge.bind", json!({
        "src": node1_id,
        "dst": node2_id,
        "weight": 0.5
    }));

    assert!(bind_resp.is_success());
    bind_resp.assert_no_forbidden_fields();
}

#[test]
fn test_edge_bind_invalid_nodes() {
    let runtime = create_test_runtime();

    let response = execute_tool(&runtime, "edge.bind", json!({
        "src": "00000000-0000-0000-0000-000000000000",
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
    let runtime = create_test_runtime();

    // Create two nodes
    let node1_resp = execute_tool(&runtime, "node.create", json!({"belief": 0.8, "energy": 100.0}));
    let node1: serde_json::Value = serde_json::from_str(&node1_resp.get_content_text().unwrap()).unwrap();
    let node1_id = node1["id"].as_str().unwrap();

    let node2_resp = execute_tool(&runtime, "node.create", json!({"belief": 0.2, "energy": 50.0}));
    let node2: serde_json::Value = serde_json::from_str(&node2_resp.get_content_text().unwrap()).unwrap();
    let node2_id = node2["id"].as_str().unwrap();

    // Bind edge
    let bind_resp = execute_tool(&runtime, "edge.bind", json!({
        "src": node1_id,
        "dst": node2_id,
        "weight": 0.5
    }));
    let edge: serde_json::Value = serde_json::from_str(&bind_resp.get_content_text().unwrap()).unwrap();
    let edge_id = edge["id"].as_str().unwrap();

    // Propagate
    let prop_resp = execute_tool(&runtime, "edge.propagate", json!({
        "edge_id": edge_id
    }));

    assert!(prop_resp.is_success());
    prop_resp.assert_no_forbidden_fields();
}

#[test]
fn test_edge_propagate_invalid_edge() {
    let runtime = create_test_runtime();

    let response = execute_tool(&runtime, "edge.propagate", json!({
        "edge_id": "00000000-0000-0000-0000-000000000000"
    }));

    assert!(response.is_error());
    response.assert_no_forbidden_fields();
}

// ============================================================================
// governor.status tests
// ============================================================================

#[test]
fn test_governor_status_empty_graph() {
    let runtime = create_test_runtime();

    let response = execute_tool(&runtime, "governor.status", json!({}));

    assert!(response.is_success());
    response.assert_no_forbidden_fields();

    let content = response.get_content_text().unwrap();
    let status: serde_json::Value = serde_json::from_str(&content).unwrap();
    
    assert_eq!(status["node_count"].as_u64().unwrap(), 0);
    assert_eq!(status["edge_count"].as_u64().unwrap(), 0);
}

#[test]
fn test_governor_status_with_nodes() {
    let runtime = create_test_runtime();

    // Create nodes
    execute_tool(&runtime, "node.create", json!({"belief": 0.5, "energy": 100.0}));
    execute_tool(&runtime, "node.create", json!({"belief": 0.3, "energy": 50.0}));

    let response = execute_tool(&runtime, "governor.status", json!({}));

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
    let runtime = create_test_runtime();

    let create_resp = execute_tool(&runtime, "node.create", json!({
        "belief": 0.5,
        "energy": 100.0
    }));
    let node: serde_json::Value = serde_json::from_str(&create_resp.get_content_text().unwrap()).unwrap();
    let node_id = node["id"].as_str().unwrap();

    let audit_resp = execute_tool(&runtime, "esv.audit", json!({
        "node_id": node_id
    }));

    assert!(audit_resp.is_success());
    audit_resp.assert_no_forbidden_fields();
    audit_resp.assert_content_contains("VALID");
}

#[test]
fn test_esv_audit_invalid_node() {
    let runtime = create_test_runtime();

    let response = execute_tool(&runtime, "esv.audit", json!({
        "node_id": "00000000-0000-0000-0000-000000000000"
    }));

    assert!(response.is_error());
    response.assert_no_forbidden_fields();
}

// ============================================================================
// lineage tests
// ============================================================================

#[test]
fn test_lineage_replay_basic() {
    let runtime = create_test_runtime();

    // Create operations
    execute_tool(&runtime, "node.create", json!({"belief": 0.5, "energy": 100.0}));

    let response = execute_tool(&runtime, "lineage.replay", json!({}));

    assert!(response.is_success());
    response.assert_no_forbidden_fields();

    let content = response.get_content_text().unwrap();
    let entry: serde_json::Value = serde_json::from_str(&content).unwrap();
    
    // Should have checksum (64 hex chars)
    let checksum = entry["checksum"].as_str().unwrap();
    assert_eq!(checksum.len(), 64);
}

#[test]
fn test_lineage_replay_empty() {
    let runtime = create_test_runtime();

    let response = execute_tool(&runtime, "lineage.replay", json!({}));

    assert!(response.is_success());
    response.assert_no_forbidden_fields();
}
