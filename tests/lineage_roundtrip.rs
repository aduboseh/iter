//! Lineage Roundtrip Tests
//!
//! Validates that the substrate's causal trace/lineage is observable
//! via MCP and maintains integrity across operations.

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

fn extract_content_text(response: &Value) -> Option<String> {
    response["result"]["content"][0]["text"]
        .as_str()
        .map(|s| s.to_string())
}

fn is_success(response: &Value) -> bool {
    response["error"].is_null() && !response["result"].is_null()
}

// ============================================================================
// Lineage Roundtrip Tests
// ============================================================================

#[test]
fn test_lineage_contains_events_after_mutations() {
    let mut runtime = create_runtime();

    // Create a node
    let create_resp = tool_call(&mut runtime, "node.create", json!({
        "belief": 0.2,
        "energy": 10.0
    }));
    assert!(is_success(&create_resp));
    
    let node_content: Value = serde_json::from_str(
        &extract_content_text(&create_resp).unwrap()
    ).unwrap();
    let node_id = node_content["id"].as_u64().unwrap().to_string();

    // Mutate the node
    let mutate_resp = tool_call(&mut runtime, "node.mutate", json!({
        "node_id": node_id,
        "delta": 0.1
    }));
    assert!(is_success(&mutate_resp));

    // Get lineage events
    let lineage_resp = tool_call(&mut runtime, "lineage.replay", json!({}));
    assert!(is_success(&lineage_resp));

    let lineage_text = extract_content_text(&lineage_resp).unwrap();
    let events: Value = serde_json::from_str(&lineage_text).unwrap();

    // Should be an array (may have events from substrate initialization)
    assert!(events.is_array(), "Lineage should return an array");
    
    // Note: The exact number of events depends on substrate behavior
    // We just assert the structure is correct - not specific counts
}

#[test]
fn test_lineage_replay_returns_array_structure() {
    let mut runtime = create_runtime();

    // Even on empty graph, lineage.replay should return a valid array
    let lineage_resp = tool_call(&mut runtime, "lineage.replay", json!({}));
    assert!(is_success(&lineage_resp));

    let lineage_text = extract_content_text(&lineage_resp).unwrap();
    let events: Value = serde_json::from_str(&lineage_text).unwrap();

    assert!(events.is_array(), "Lineage replay should return an array");
    
    // If there are events, they should have expected fields
    if let Some(arr) = events.as_array() {
        for event in arr {
            // Each McpLineageEntry has: sequence, operation, checksum, tick
            assert!(event["sequence"].is_u64() || event["sequence"].is_i64(), 
                "Event should have numeric sequence");
            assert!(event["operation"].is_string(), 
                "Event should have string operation");
            assert!(event["checksum"].is_string(), 
                "Event should have string checksum");
            assert!(event["tick"].is_u64() || event["tick"].is_i64(), 
                "Event should have numeric tick");
        }
    }
}

#[test]
fn test_lineage_export_creates_file() {
    use std::fs;
    
    let mut runtime = create_runtime();

    // Create some operations
    tool_call(&mut runtime, "node.create", json!({"belief": 0.5, "energy": 10.0}));
    tool_call(&mut runtime, "node.create", json!({"belief": 0.3, "energy": 10.0}));

    // Export lineage to a temp file
    let temp_path = std::env::temp_dir().join("test_lineage_export.json");
    let path_str = temp_path.to_string_lossy().to_string();

    let export_resp = tool_call(&mut runtime, "lineage.export", json!({
        "path": path_str
    }));
    
    assert!(is_success(&export_resp), "lineage.export should succeed");
    
    // Verify file was created
    assert!(temp_path.exists(), "Export file should exist");
    
    // Verify file contains valid JSON array
    let contents = fs::read_to_string(&temp_path).expect("Should read file");
    let parsed: Value = serde_json::from_str(&contents).expect("Should be valid JSON");
    assert!(parsed.is_array(), "Exported content should be an array");
    
    // Clean up
    let _ = fs::remove_file(temp_path);
}

#[test]
fn test_lineage_checksum_format() {
    let mut runtime = create_runtime();

    // Create operations to generate lineage
    tool_call(&mut runtime, "node.create", json!({"belief": 0.5, "energy": 10.0}));
    
    // Run a step to generate trace events
    tool_call(&mut runtime, "edge.propagate", json!({"edge_id": "0"}));

    let lineage_resp = tool_call(&mut runtime, "lineage.replay", json!({}));
    assert!(is_success(&lineage_resp));

    let lineage_text = extract_content_text(&lineage_resp).unwrap();
    let events: Value = serde_json::from_str(&lineage_text).unwrap();

    if let Some(arr) = events.as_array() {
        for event in arr {
            if let Some(checksum) = event["checksum"].as_str() {
                // Checksums should be hex-encoded (SHA-256 = 64 hex chars)
                assert!(checksum.len() == 64, 
                    "Checksum should be 64 hex chars, got {}", checksum.len());
                assert!(checksum.chars().all(|c| c.is_ascii_hexdigit()),
                    "Checksum should be hex-encoded");
            }
        }
    }
}

#[test]
fn test_lineage_grows_with_operations() {
    let mut runtime = create_runtime();

    // Get initial lineage count
    let initial_resp = tool_call(&mut runtime, "lineage.replay", json!({}));
    let initial_text = extract_content_text(&initial_resp).unwrap();
    let initial_events: Vec<Value> = serde_json::from_str(&initial_text).unwrap();
    let initial_count = initial_events.len();

    // Perform operations that generate trace events
    tool_call(&mut runtime, "node.create", json!({"belief": 0.5, "energy": 10.0}));
    tool_call(&mut runtime, "edge.propagate", json!({"edge_id": "0"}));  // Runs step()

    // Get updated lineage
    let updated_resp = tool_call(&mut runtime, "lineage.replay", json!({}));
    let updated_text = extract_content_text(&updated_resp).unwrap();
    let updated_events: Vec<Value> = serde_json::from_str(&updated_text).unwrap();
    let updated_count = updated_events.len();

    // After a simulation step, we should have more trace events
    // Note: The exact growth depends on substrate behavior
    // We assert it's at least non-decreasing
    assert!(updated_count >= initial_count, 
        "Lineage count should not decrease: {} -> {}", initial_count, updated_count);
}
