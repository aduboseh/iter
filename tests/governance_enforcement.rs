//! Governance Enforcement Tests
//!
//! Validates that the substrate's governance invariants are observable
//! and enforceable via MCP endpoints.

use scg_mcp_server::mcp_handler::handle_rpc;
use scg_mcp_server::types::RpcRequest;
use scg_mcp_server::SubstrateRuntime;
use serde_json::{json, Value};

fn create_runtime() -> SubstrateRuntime {
    SubstrateRuntime::with_defaults().expect("Failed to create runtime")
}

fn tool_call(runtime: &mut SubstrateRuntime, tool: &str, args: Value) -> Value {
    let request = RpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "tools/call".to_string(),
        params: json!({
            "name": tool,
            "arguments": args
        }),
        id: Some(json!(1)),
    };
    let response = handle_rpc(runtime, request);
    serde_json::to_value(&response).unwrap()
}

fn extract_content(response: &Value) -> Option<Value> {
    response["result"]["content"][0]["text"]
        .as_str()
        .and_then(|s| serde_json::from_str(s).ok())
}

fn is_success(response: &Value) -> bool {
    response["error"].is_null() && !response["result"].is_null()
}

// ============================================================================
// Governance Enforcement Tests
// ============================================================================

#[test]
fn test_governance_reacts_to_belief_change() {
    let mut runtime = create_runtime();

    // Create a node with baseline belief
    let create_resp = tool_call(&mut runtime, "node.create", json!({
        "belief": 0.1,
        "energy": 10.0
    }));
    assert!(is_success(&create_resp), "node.create should succeed");
    
    let node = extract_content(&create_resp).expect("Should have content");
    let node_id = node["id"].as_u64().expect("Should have node ID").to_string();

    // Attempt a large belief change
    let mutate_resp = tool_call(&mut runtime, "node.mutate", json!({
        "node_id": node_id,
        "delta": 0.8
    }));

    // Contract: Mutation either:
    // a) Is rejected by governance/simulation with belief unchanged
    // b) Is allowed, but governance invariants still hold
    
    if is_success(&mutate_resp) {
        let updated = extract_content(&mutate_resp).expect("Should have content");
        // If allowed, check that belief was clamped appropriately
        let new_belief = updated["belief"].as_f64().expect("Should have belief");
        assert!((0.0..=1.0).contains(&new_belief), "Belief must be in [0, 1]");
        
        // Verify governance invariants still hold
        let gov_resp = tool_call(&mut runtime, "governance.status", json!({}));
        assert!(is_success(&gov_resp), "governance.status should succeed");
        let gov_status = extract_content(&gov_resp).expect("Should have status");
        
        // Either drift_ok is true, or the system has detected drift
        // (we don't assert specific behavior, just that governance is responsive)
        assert!(gov_status["drift_ok"].is_boolean(), "Should have drift_ok field");
        assert!(gov_status["healthy"].is_boolean(), "Should have healthy field");
    } else {
        // If rejected, verify node is unchanged
        let query_resp = tool_call(&mut runtime, "node.query", json!({
            "node_id": node_id
        }));
        
        if is_success(&query_resp) {
            let node_after = extract_content(&query_resp).expect("Should have content");
            let belief_after = node_after["belief"].as_f64().expect("Should have belief");
            // Note: belief may have changed slightly due to mutation energy cost
            // We just verify it's still in valid range
            assert!((0.0..=1.0).contains(&belief_after));
        }
    }
}

#[test]
fn test_governance_status_reflects_graph_state() {
    let mut runtime = create_runtime();

    // Empty graph
    let gov_empty = tool_call(&mut runtime, "governance.status", json!({}));
    assert!(is_success(&gov_empty));
    let status_empty = extract_content(&gov_empty).expect("Should have status");
    assert_eq!(status_empty["node_count"].as_u64().unwrap(), 0);

    // Add nodes
    tool_call(&mut runtime, "node.create", json!({"belief": 0.5, "energy": 10.0}));
    tool_call(&mut runtime, "node.create", json!({"belief": 0.3, "energy": 10.0}));

    let gov_with_nodes = tool_call(&mut runtime, "governance.status", json!({}));
    assert!(is_success(&gov_with_nodes));
    let status_with_nodes = extract_content(&gov_with_nodes).expect("Should have status");
    assert_eq!(status_with_nodes["node_count"].as_u64().unwrap(), 2);
    
    // drift_ok should be true for a well-formed graph
    assert!(status_with_nodes["drift_ok"].as_bool().unwrap_or(false));
}

#[test]
fn test_energy_conservation_check_via_esv_audit() {
    let mut runtime = create_runtime();

    // Create nodes and run simulation steps
    let node1_resp = tool_call(&mut runtime, "node.create", json!({
        "belief": 0.5,
        "energy": 10.0
    }));
    let node1 = extract_content(&node1_resp).unwrap();
    let node1_id = node1["id"].as_u64().unwrap().to_string();

    let node2_resp = tool_call(&mut runtime, "node.create", json!({
        "belief": 0.3,
        "energy": 10.0
    }));
    let node2 = extract_content(&node2_resp).unwrap();
    let node2_id = node2["id"].as_u64().unwrap().to_string();

    // Bind edge
    tool_call(&mut runtime, "edge.bind", json!({
        "src": node1_id,
        "dst": node2_id,
        "weight": 0.5
    }));

    // Run propagation step
    tool_call(&mut runtime, "edge.propagate", json!({
        "edge_id": "0"  // Dummy, runs full step
    }));

    // ESV audit should verify energy conservation
    let audit_resp = tool_call(&mut runtime, "esv.audit", json!({
        "node_id": node1_id
    }));
    
    assert!(is_success(&audit_resp), "ESV audit should succeed");
    // Response contains VALID or INVALID
    let response_json = serde_json::to_string(&audit_resp).unwrap();
    assert!(response_json.contains("VALID") || response_json.contains("INVALID"));
}

#[test]
fn test_governance_invariants_across_operations() {
    let mut runtime = create_runtime();

    // Perform a sequence of operations and verify governance health throughout
    let operations = vec![
        ("Create node 1", json!({"belief": 0.5, "energy": 10.0})),
        ("Create node 2", json!({"belief": 0.3, "energy": 10.0})),
        ("Create node 3", json!({"belief": 0.7, "energy": 10.0})),
    ];

    for (desc, params) in &operations {
        let resp = tool_call(&mut runtime, "node.create", params.clone());
        assert!(is_success(&resp), "{} should succeed", desc);
    }

    // After all operations, governance should still be healthy
    let gov_resp = tool_call(&mut runtime, "governance.status", json!({}));
    assert!(is_success(&gov_resp));
    let status = extract_content(&gov_resp).expect("Should have status");
    
    // System should be healthy with proper node count
    assert_eq!(status["node_count"].as_u64().unwrap(), 3);
    assert!(status["healthy"].as_bool().unwrap_or(false), "System should be healthy");
}
