//! INTERNAL — NOT FOR PUBLIC DISTRIBUTION
//! This file directly invokes Iter server internals for testing,
//! validation, and development purposes. Do not expose publicly.
//!
//! Determinism Demo
//!
//! Demonstrates tool calls and repeatable outcomes via the Iter Server MCP surface.
//!
//! Run with:
//! ```bash
//! cargo run --release --example determinism_demo
//! ```

use scg_mcp_server::mcp_handler::handle_rpc;
use scg_mcp_server::substrate_runtime::{SubstrateRuntime, SubstrateRuntimeConfig};
use scg_mcp_server::types::RpcRequest;
use serde_json::json;

fn main() {
    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║             ITER DETERMINISM DEMONSTRATION v0.3.0                ║");
    println!("║     Deterministic decision paths & audit — not a CRUD API        ║");
    println!("╚══════════════════════════════════════════════════════════════════╝\n");

    // Initialize runtime with default config
    let config = SubstrateRuntimeConfig::default();
    let mut runtime = SubstrateRuntime::new(config).expect("Failed to create runtime");

    // ========================================================================
// PHASE 1: Runtime Initialization
    // ========================================================================
    println!("┌─────────────────────────────────────────────────────────────────┐");
println!("│ PHASE 1: Runtime Initialization                                │");
    println!("└─────────────────────────────────────────────────────────────────┘");
    println!("  Initializing iter-server runtime...\n");

    let init_req = RpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "initialize".to_string(),
        params: json!({}),
        id: Some(json!(1)),
    };
    let _init_resp = handle_rpc(&mut runtime, init_req);
println!("  ✓ Runtime initialized\n");

    // ========================================================================
// PHASE 2: Instantiate Nodes
    // ========================================================================
    println!("┌─────────────────────────────────────────────────────────────────┐");
println!("│ PHASE 2: Instantiate Nodes                                     │");
    println!("└─────────────────────────────────────────────────────────────────┘");
    println!("  Creating two nodes...\n");

    // Create Node A: High mass (high energy = stable beliefs)
    let create_a = RpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "tools/call".to_string(),
        params: json!({
            "name": "node.create",
            "arguments": { "belief": 0.7, "energy": 100.0 }
        }),
        id: Some(json!(2)),
    };
    let resp_a = handle_rpc(&mut runtime, create_a);
    let node_a = extract_content(&resp_a);
    println!("  Node A:");
    println!("  ├── Belief: 0.7");
    println!("  └── Energy: 100.0");
    println!("      Response: {}\n", node_a);

    // Create Node B: Lower mass (lower energy = more malleable)
    let create_b = RpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "tools/call".to_string(),
        params: json!({
            "name": "node.create",
            "arguments": { "belief": 0.3, "energy": 50.0 }
        }),
        id: Some(json!(3)),
    };
    let resp_b = handle_rpc(&mut runtime, create_b);
    let node_b = extract_content(&resp_b);
    println!("  Node B:");
    println!("  ├── Belief: 0.3");
    println!("  └── Energy: 50.0");
    println!("      Response: {}\n", node_b);

    // ========================================================================
    // PHASE 3: Bind Conductive Pathway
    // ========================================================================
    println!("┌─────────────────────────────────────────────────────────────────┐");
    println!("│ PHASE 3: Bind Conductive Pathway                               │");
    println!("└─────────────────────────────────────────────────────────────────┘");
    println!("  Binding an edge from Node A → Node B...\n");

    let bind = RpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "tools/call".to_string(),
        params: json!({
            "name": "edge.bind",
            "arguments": { "src": "0", "dst": "1", "weight": 0.7 }
        }),
        id: Some(json!(4)),
    };
    let resp_bind = handle_rpc(&mut runtime, bind);
    let edge = extract_content(&resp_bind);
    println!("  Edge:");
    println!("  └── Weight: 0.7");
    println!("      Response: {}\n", edge);

    // ========================================================================
    // PHASE 4: The Impossible Perturbation — Energy Cost Demo
    // ========================================================================
    println!("┌─────────────────────────────────────────────────────────────────┐");
    println!("│ PHASE 4: Node Mutation                                         │");
    println!("└─────────────────────────────────────────────────────────────────┘");
    println!("  Mutating Node A belief by +0.1\n");

    // Query before mutation
    let query_before = RpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "tools/call".to_string(),
        params: json!({
            "name": "node.query",
            "arguments": { "node_id": "0" }
        }),
        id: Some(json!(5)),
    };
    let resp_before = handle_rpc(&mut runtime, query_before);
    let before = extract_content(&resp_before);
    println!("  BEFORE mutation: {}", before);

    // Mutate
    let mutate = RpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "tools/call".to_string(),
        params: json!({
            "name": "node.mutate",
            "arguments": { "node_id": "0", "delta": 0.1 }
        }),
        id: Some(json!(6)),
    };
    let resp_mutate = handle_rpc(&mut runtime, mutate);
    let after = extract_content(&resp_mutate);
    println!("  AFTER mutation:  {}\n", after);

    println!("  Result captured from tool response.\n");

    // ========================================================================
    // PHASE 5: Temporal Dynamics — Propagation
    // ========================================================================
    println!("┌─────────────────────────────────────────────────────────────────┐");
    println!("│ PHASE 5: Temporal Dynamics — Propagation                       │");
    println!("└─────────────────────────────────────────────────────────────────┘");
    println!("  Running a step...\n");

    // Query Node B before propagation
    let query_b_before = RpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "tools/call".to_string(),
        params: json!({
            "name": "node.query",
            "arguments": { "node_id": "1" }
        }),
        id: Some(json!(7)),
    };
    let resp_b_before = handle_rpc(&mut runtime, query_b_before);
    let b_before = extract_content(&resp_b_before);
    println!("  Node B BEFORE propagation: {}", b_before);

    // Run propagation step
    let propagate = RpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "tools/call".to_string(),
        params: json!({
            "name": "edge.propagate",
            "arguments": { "edge_id": "0" }
        }),
        id: Some(json!(8)),
    };
    let _prop_resp = handle_rpc(&mut runtime, propagate);
    println!("  → Propagation tick executed");

    // Query Node B after propagation
    let query_b_after = RpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "tools/call".to_string(),
        params: json!({
            "name": "node.query",
            "arguments": { "node_id": "1" }
        }),
        id: Some(json!(9)),
    };
    let resp_b_after = handle_rpc(&mut runtime, query_b_after);
    let b_after = extract_content(&resp_b_after);
    println!("  Node B AFTER propagation:  {}\n", b_after);

    println!("  Result captured from tool response.\n");

    // ========================================================================
    // PHASE 6: Governance Status
    // ========================================================================
    println!("┌─────────────────────────────────────────────────────────────────┐");
    println!("│ PHASE 6: Governance Status                                     │");
    println!("└─────────────────────────────────────────────────────────────────┘");
println!("  Querying governance health...\n");

    let gov = RpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "tools/call".to_string(),
        params: json!({
            "name": "governance.status",
            "arguments": {}
        }),
        id: Some(json!(10)),
    };
    let gov_resp = handle_rpc(&mut runtime, gov);
    let gov_status = extract_content(&gov_resp);
    println!("  Governance Status: {}\n", gov_status);

    println!("  Status captured from tool response.\n");

    // ========================================================================
// PHASE 7: Audit Trail — Lineage Replay
    // ========================================================================
    println!("┌─────────────────────────────────────────────────────────────────┐");
    println!("│ PHASE 7: Audit Summary                                         │");
    println!("└─────────────────────────────────────────────────────────────────┘");
println!("  Replaying lineage...\n");

    let lineage = RpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "tools/call".to_string(),
        params: json!({
            "name": "lineage.replay",
            "arguments": {}
        }),
        id: Some(json!(11)),
    };
    let lineage_resp = handle_rpc(&mut runtime, lineage);
    let lineage_data = extract_content(&lineage_resp);
    println!("  Lineage: {}\n", lineage_data);

    println!("  Audit summary captured from tool response.\n");

    // ========================================================================
    // PHASE 8: ESV Audit — Ethical State Vector
    // ========================================================================
    println!("┌─────────────────────────────────────────────────────────────────┐");
    println!("│ PHASE 8: ESV Audit — Ethical State Vector Validation           │");
    println!("└─────────────────────────────────────────────────────────────────┘");
println!("  Auditing Node A's compliance...\n");

    let esv = RpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "tools/call".to_string(),
        params: json!({
            "name": "esv.audit",
            "arguments": { "node_id": "0" }
        }),
        id: Some(json!(12)),
    };
    let esv_resp = handle_rpc(&mut runtime, esv);
    let esv_status = extract_content(&esv_resp);
    println!("  ESV Status: {}\n", esv_status);

    println!("  Status captured from tool response.\n");

    // ========================================================================
    // SUMMARY
    // ========================================================================
    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║                         SUMMARY                                  ║");
    println!("╚══════════════════════════════════════════════════════════════════╝");
    println!("");
    println!("  Summary:");
    println!("  - tool calls executed: initialize, create/query/mutate, bind/step, status, audit");
    println!("  - outputs captured from MCP responses");
    println!("\n═══════════════════════════════════════════════════════════════════");
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
            resp.error
                .as_ref()
                .map(|e| e.message.as_str())
                .unwrap_or("(no content)")
        })
        .to_string()
}
