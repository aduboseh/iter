//! Iter Reference Client
//!
//! Demonstrates all MCP tools available via the server.
//!
//! # Usage
//!
//! ```bash
//! cargo run --example mcp_client
//! ```
//!
//! # Tools Demonstrated
//!
//! - `node.create` - Create node with belief and energy
//! - `node.query` - Query node state by ID
//! - `node.mutate` - Mutate node belief (DEBUG operation)
//! - `edge.bind` - Bind edge between nodes
//! - `edge.propagate` - Run simulation step
//! - `governor.status` - Query governor status
//! - `governance.status` - Query full governance health
//! - `esv.audit` - Audit node ethical state vector
//! - `lineage.replay` - Replay lineage history
//! - `lineage.export` - Export lineage to file

use scg_mcp_server::mcp_handler::handle_rpc;
use scg_mcp_server::substrate_runtime::{SubstrateRuntime, SubstrateRuntimeConfig};
use scg_mcp_server::types::RpcRequest;
use serde_json::json;

fn main() {
    println!("=== Iter Reference Client ===\n");

    // Initialize runtime with default config
    let config = SubstrateRuntimeConfig::default();
    let mut runtime = SubstrateRuntime::new(config).expect("Failed to create runtime");

    // ========================================================================
    // 1. Protocol Initialization
    // ========================================================================
    println!("--- Protocol Initialization ---");

    let init_req = RpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "initialize".to_string(),
        params: json!({}),
        id: Some(json!(1)),
    };
    let init_resp = handle_rpc(&mut runtime, init_req);
    println!("initialize: {}\n", serde_json::to_string_pretty(&init_resp).unwrap());

    // ========================================================================
    // 2. List Available Tools
    // ========================================================================
    println!("--- List Tools ---");

    let list_req = RpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "tools/list".to_string(),
        params: json!({}),
        id: Some(json!(2)),
    };
    let list_resp = handle_rpc(&mut runtime, list_req);
    
    // Extract tool names for display
    if let Some(result) = list_resp.result.as_ref() {
        if let Some(tools) = result.get("tools").and_then(|t| t.as_array()) {
            println!("Available tools ({}):", tools.len());
            for tool in tools {
                if let Some(name) = tool.get("name").and_then(|n| n.as_str()) {
                    let desc = tool.get("description").and_then(|d| d.as_str()).unwrap_or("");
                    println!("  - {}: {}", name, desc);
                }
            }
        }
    }
    println!();

    // ========================================================================
    // 3. Node Operations
    // ========================================================================
    println!("--- Node Operations ---");

    // Create first node
    let create_req = RpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "tools/call".to_string(),
        params: json!({
            "name": "node.create",
            "arguments": {
                "belief": 0.7,
                "energy": 100.0
            }
        }),
        id: Some(json!(3)),
    };
    let create_resp = handle_rpc(&mut runtime, create_req);
    println!("node.create (belief=0.7, energy=100): {}", 
        extract_content(&create_resp));

    // Extract node ID from response
    let node1_id = extract_node_id(&create_resp).expect("Failed to get node ID");
    println!("  -> Created node ID: {}", node1_id);

    // Create second node
    let create_req2 = RpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "tools/call".to_string(),
        params: json!({
            "name": "node.create",
            "arguments": {
                "belief": 0.3,
                "energy": 50.0
            }
        }),
        id: Some(json!(4)),
    };
    let create_resp2 = handle_rpc(&mut runtime, create_req2);
    let node2_id = extract_node_id(&create_resp2).expect("Failed to get node ID");
    println!("node.create (belief=0.3, energy=50): created node {}", node2_id);

    // Query node
    let query_req = RpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "tools/call".to_string(),
        params: json!({
            "name": "node.query",
            "arguments": {
                "node_id": node1_id.to_string()
            }
        }),
        id: Some(json!(5)),
    };
    let query_resp = handle_rpc(&mut runtime, query_req);
    println!("node.query (node {}): {}", node1_id, extract_content(&query_resp));

    // Mutate node (DEBUG operation)
    let mutate_req = RpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "tools/call".to_string(),
        params: json!({
            "name": "node.mutate",
            "arguments": {
                "node_id": node1_id.to_string(),
                "delta": 0.1
            }
        }),
        id: Some(json!(6)),
    };
    let mutate_resp = handle_rpc(&mut runtime, mutate_req);
    println!("node.mutate (node {}, delta=+0.1): {}", node1_id, extract_content(&mutate_resp));
    println!();

    // ========================================================================
    // 4. Edge Operations
    // ========================================================================
    println!("--- Edge Operations ---");

    // Bind edge
    let bind_req = RpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "tools/call".to_string(),
        params: json!({
            "name": "edge.bind",
            "arguments": {
                "src": node1_id.to_string(),
                "dst": node2_id.to_string(),
                "weight": 0.5
            }
        }),
        id: Some(json!(7)),
    };
    let bind_resp = handle_rpc(&mut runtime, bind_req);
    println!("edge.bind ({}→{}, weight=0.5): {}", node1_id, node2_id, extract_content(&bind_resp));

    // Propagate (run simulation step)
    let prop_req = RpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "tools/call".to_string(),
        params: json!({
            "name": "edge.propagate",
            "arguments": {
                "edge_id": "0"
            }
        }),
        id: Some(json!(8)),
    };
    let prop_resp = handle_rpc(&mut runtime, prop_req);
    println!("edge.propagate (step): {}", extract_content(&prop_resp));

    // Query node after propagation
    let query_req2 = RpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "tools/call".to_string(),
        params: json!({
            "name": "node.query",
            "arguments": {
                "node_id": node2_id.to_string()
            }
        }),
        id: Some(json!(9)),
    };
    let query_resp2 = handle_rpc(&mut runtime, query_req2);
    println!("node.query (node {} after propagation): {}", node2_id, extract_content(&query_resp2));
    println!();

    // ========================================================================
    // 5. Governance Operations
    // ========================================================================
    println!("--- Governance Operations ---");

    // Governor status
    let gov_req = RpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "tools/call".to_string(),
        params: json!({
            "name": "governor.status",
            "arguments": {}
        }),
        id: Some(json!(10)),
    };
    let gov_resp = handle_rpc(&mut runtime, gov_req);
    println!("governor.status: {}", extract_content(&gov_resp));

    // Full governance status
    let governance_req = RpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "tools/call".to_string(),
        params: json!({
            "name": "governance.status",
            "arguments": {}
        }),
        id: Some(json!(11)),
    };
    let governance_resp = handle_rpc(&mut runtime, governance_req);
    println!("governance.status: {}", extract_content(&governance_resp));

    // ESV audit
    let esv_req = RpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "tools/call".to_string(),
        params: json!({
            "name": "esv.audit",
            "arguments": {
                "node_id": node1_id.to_string()
            }
        }),
        id: Some(json!(12)),
    };
    let esv_resp = handle_rpc(&mut runtime, esv_req);
    println!("esv.audit (node {}): {}", node1_id, extract_content(&esv_resp));
    println!();

    // ========================================================================
    // 6. Lineage Operations
    // ========================================================================
    println!("--- Lineage Operations ---");

    // Lineage replay
    let replay_req = RpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "tools/call".to_string(),
        params: json!({
            "name": "lineage.replay",
            "arguments": {}
        }),
        id: Some(json!(13)),
    };
    let replay_resp = handle_rpc(&mut runtime, replay_req);
    println!("lineage.replay: {}", extract_content(&replay_resp));

    // Lineage export
    let export_path = std::env::temp_dir().join("iter_lineage_demo.json");
    let export_req = RpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "tools/call".to_string(),
        params: json!({
            "name": "lineage.export",
            "arguments": {
                "path": export_path.to_string_lossy()
            }
        }),
        id: Some(json!(14)),
    };
    let export_resp = handle_rpc(&mut runtime, export_req);
    println!("lineage.export (path={}): {}", 
        export_path.display(), 
        extract_content(&export_resp));
    println!();

    // ========================================================================
    // 7. Error Handling Demo
    // ========================================================================
    println!("--- Error Handling Demo ---");

    // Query non-existent node
    let bad_query_req = RpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "tools/call".to_string(),
        params: json!({
            "name": "node.query",
            "arguments": {
                "node_id": "999999"
            }
        }),
        id: Some(json!(15)),
    };
    let bad_query_resp = handle_rpc(&mut runtime, bad_query_req);
    println!("node.query (non-existent node 999999):");
    if let Some(err) = &bad_query_resp.error {
        println!("  Error code: {}", err.code);
        println!("  Error message: {}", err.message);
    }

    // Invalid tool name
    let bad_tool_req = RpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "tools/call".to_string(),
        params: json!({
            "name": "invalid.tool",
            "arguments": {}
        }),
        id: Some(json!(16)),
    };
    let bad_tool_resp = handle_rpc(&mut runtime, bad_tool_req);
    println!("tools/call (invalid.tool):");
    if let Some(err) = &bad_tool_resp.error {
        println!("  Error code: {}", err.code);
        println!("  Error message: {}", err.message);
    }
    println!();

    println!("=== Reference Client Complete ===");
}

/// Extract content text from MCP tool response.
fn extract_content(resp: &scg_mcp_server::types::RpcResponse) -> String {
    resp.result
        .as_ref()
        .and_then(|r| r.get("content"))
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|item| item.get("text"))
        .and_then(|t| t.as_str())
        .unwrap_or_else(|| {
            // Fallback to error message
            resp.error
                .as_ref()
                .map(|e| e.message.as_str())
                .unwrap_or("(no content)")
        })
        .to_string()
}

/// Extract node ID from create response.
fn extract_node_id(resp: &scg_mcp_server::types::RpcResponse) -> Option<u64> {
    let text = resp
        .result
        .as_ref()?
        .get("content")?
        .as_array()?
        .first()?
        .get("text")?
        .as_str()?;
    
    // Parse JSON from text content
    let json: serde_json::Value = serde_json::from_str(text).ok()?;
    json.get("id")?.as_u64()
}
