// SCG MCP Integration Test Suite - Boundary Tests
// Core sanitization validation for all MCP endpoints

use super::common::*;
use serde_json::json;

#[test]
fn test_node_create_response_sanitization() {
    let mut runtime = create_test_runtime();

    let response = execute_tool(&mut runtime, "node.create", json!({
        "belief": 0.5,
        "energy": 100.0
    }));

    assert!(response.is_success(), "node.create should succeed");
    response.assert_no_forbidden_fields();
    
    // Should have content with node data
    let content = response.get_content_text();
    assert!(content.is_some(), "Should have content text");
    
    // Content should contain node ID but no internal state
    let text = content.unwrap();
    assert!(text.contains("id"), "Should contain node id");
    assert!(!text.contains("internal_state"), "Should not contain internal_state");
    assert!(!text.contains("adjacency"), "Should not contain adjacency");
}

#[test]
fn test_node_query_response_sanitization() {
    let mut runtime = create_test_runtime();

    // First create a node
    let create_resp = execute_tool(&mut runtime, "node.create", json!({
        "belief": 0.5,
        "energy": 100.0
    }));
    assert!(create_resp.is_success());

    // Extract the node ID from the response
    let node_id = extract_node_id(&create_resp);

    // Query the node
    let response = execute_tool(&mut runtime, "node.query", json!({
        "node_id": node_id
    }));

    assert!(response.is_success(), "node.query should succeed");
    response.assert_no_forbidden_fields();
}

#[test]
fn test_governor_status_response_sanitization() {
    let mut runtime = create_test_runtime();

    // Create some nodes to have governor state
    execute_tool(&mut runtime, "node.create", json!({
        "belief": 0.5,
        "energy": 100.0
    }));

    let response = execute_tool(&mut runtime, "governor.status", json!({}));

    assert!(response.is_success(), "governor.status should succeed");
    response.assert_no_forbidden_fields();

    // Should contain status info but no internal details
    let content = response.get_content_text().unwrap();
    
    // Should NOT contain forbidden internal patterns
    assert!(!content.contains("governor_quorum_state"));
    assert!(!content.contains("drift_correction_vector"));
    assert!(!content.contains("node_energy_deltas"));
    assert!(!content.contains("quorum_members"));
}

#[test]
fn test_esv_audit_response_sanitization() {
    let mut runtime = create_test_runtime();

    // Create a node
    let create_resp = execute_tool(&mut runtime, "node.create", json!({
        "belief": 0.5,
        "energy": 100.0
    }));
    let node_id = extract_node_id(&create_resp);

    // Audit ESV
    let response = execute_tool(&mut runtime, "esv.audit", json!({
        "node_id": node_id
    }));

    assert!(response.is_success(), "esv.audit should succeed");
    response.assert_no_forbidden_fields();

    // Should show validity status but no raw ESV values
    let text = response.get_content_text().unwrap();
    assert!(!text.contains("esv_raw"));
    assert!(!text.contains("ethical_gradient"));
    assert!(!text.contains("harm_potential_raw"));
}

#[test]
fn test_lineage_replay_response_sanitization() {
    let mut runtime = create_test_runtime();

    // Create operations to generate lineage
    execute_tool(&mut runtime, "node.create", json!({
        "belief": 0.5,
        "energy": 100.0
    }));

    let response = execute_tool(&mut runtime, "lineage.replay", json!({}));

    assert!(response.is_success(), "lineage.replay should succeed");
    response.assert_no_forbidden_fields();

    // Should have checksum but no internal lineage chain
    let text = response.get_content_text().unwrap();
    assert!(!text.contains("lineage_hash_chain"));
    assert!(!text.contains("cascade_hash_internal"));
    assert!(!text.contains("parent_hash"));
}

#[test]
fn test_edge_bind_response_sanitization() {
    let mut runtime = create_test_runtime();

    // Create two nodes
    let node1_resp = execute_tool(&mut runtime, "node.create", json!({
        "belief": 0.5,
        "energy": 100.0
    }));
    let node1_id = extract_node_id(&node1_resp);

    let node2_resp = execute_tool(&mut runtime, "node.create", json!({
        "belief": 0.3,
        "energy": 50.0
    }));
    let node2_id = extract_node_id(&node2_resp);

    // Bind edge
    let response = execute_tool(&mut runtime, "edge.bind", json!({
        "src": node1_id,
        "dst": node2_id,
        "weight": 0.5
    }));

    assert!(response.is_success(), "edge.bind should succeed");
    response.assert_no_forbidden_fields();

    // Should NOT expose topology internals
    let text = response.get_content_text().unwrap();
    assert!(!text.contains("adjacency"));
    assert!(!text.contains("topology"));
    assert!(!text.contains("dag_structure"));
}

#[test]
fn test_all_forbidden_patterns_blocked_complex_operation() {
    let mut runtime = create_test_runtime();

    // Create a complex graph to maximize internal state generation
    let mut node_ids = Vec::new();
    for i in 0..5 {
        let resp = execute_tool(&mut runtime, "node.create", json!({
            "belief": 0.1 * (i as f64),
            "energy": 100.0
        }));
        assert!(resp.is_success());
        node_ids.push(extract_node_id(&resp));
        
        // Each response must be sanitized
        resp.assert_no_forbidden_fields();
    }

    // Create edges between nodes
    for i in 0..4 {
        let resp = execute_tool(&mut runtime, "edge.bind", json!({
            "src": node_ids[i],
            "dst": node_ids[i + 1],
            "weight": 0.5
        }));
        assert!(resp.is_success());
        resp.assert_no_forbidden_fields();
    }

    // Query governor status after complex operations
    let gov_resp = execute_tool(&mut runtime, "governor.status", json!({}));
    gov_resp.assert_no_forbidden_fields();

    // Replay lineage
    let lineage_resp = execute_tool(&mut runtime, "lineage.replay", json!({}));
    lineage_resp.assert_no_forbidden_fields();
}

#[test]
fn test_tools_list_sanitization() {
    let mut runtime = create_test_runtime();
    let request = build_rpc_request("tools/list", json!({}));
    let response = execute_rpc(&mut runtime, request);

    assert!(response.is_success(), "tools/list should succeed");
    response.assert_no_forbidden_fields();

    // Should list tools but not expose internal implementation
    response.assert_result_field_exists("tools");
}

#[test]
fn test_initialize_response_sanitization() {
    let mut runtime = create_test_runtime();
    let request = build_rpc_request("initialize", json!({}));
    let response = execute_rpc(&mut runtime, request);

    assert!(response.is_success(), "initialize should succeed");
    response.assert_no_forbidden_fields();

    // Should have protocol info
    response.assert_result_field_exists("protocolVersion");
    response.assert_result_field_exists("capabilities");
    response.assert_result_field_exists("serverInfo");
}

#[test]
fn test_governance_status_sanitization() {
    let mut runtime = create_test_runtime();
    let response = execute_tool(&mut runtime, "governance.status", json!({}));

    assert!(response.is_success(), "governance.status should succeed");
    response.assert_no_forbidden_fields();

    // Should have governance info but no internal state
    let text = response.get_content_text().unwrap();
    assert!(!text.contains("internal_state"));
    assert!(!text.contains("substrate_state"));
}
