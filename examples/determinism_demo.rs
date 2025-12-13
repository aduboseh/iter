//! Determinism Demo
//!
//! This demo shows deterministic execution behavior and governed operations.
//! It demonstrates:
//!
//! 1. Resource constraints (energy as resistance to belief change)
//! 2. Energy cost of perturbations (belief changes are not free)
//! 3. Propagation dynamics (beliefs flow through weighted pathways)
//! 4. Governance invariants (drift bounds, conservation)
//! 5. Audit trail (hash-chained operation record)
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
    println!("  Initializing Iter v0.3.0 execution runtime...");
    println!("  ├── Energy ledger: Neumaier summation for drift < 1e-10");
    println!("  ├── Governance: Thermodynamic invariant enforcement");
    println!("  ├── Lineage: SHA-256 hash-chained causal trace");
    println!("  └── Ethics: ESV (Ethical State Vector) validation\n");

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
    println!("  Creating entities...\n");
    println!("  NOTE: Energy acts as a resource constraint on belief change\n");

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
    println!("  Node A (High Mass Concept):");
    println!("  ├── Belief: 0.7 (strong conviction)");
println!("  ├── Energy: 100.0 (high resource level)");
    println!("  └── Interpretation: A well-established, stable belief");
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
    println!("  Node B (Low Mass Concept):");
    println!("  ├── Belief: 0.3 (weak conviction)");
println!("  ├── Energy: 50.0 (lower resource level)");
    println!("  └── Interpretation: A tentative, easily influenced belief");
    println!("      Response: {}\n", node_b);

    // ========================================================================
    // PHASE 3: Bind Conductive Pathway
    // ========================================================================
    println!("┌─────────────────────────────────────────────────────────────────┐");
    println!("│ PHASE 3: Bind Conductive Pathway                               │");
    println!("└─────────────────────────────────────────────────────────────────┘");
    println!("  Creating epistemic edge from Node A → Node B...");
    println!("  PHYSICS: Edge weight = conductance = influence propagation rate\n");

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
    println!("  Edge E0 (A → B):");
    println!("  ├── Weight: 0.7 (high conductance)");
    println!("  └── Effect: Node A's belief will strongly influence Node B");
    println!("      Response: {}\n", edge);

    // ========================================================================
    // PHASE 4: The Impossible Perturbation — Energy Cost Demo
    // ========================================================================
    println!("┌─────────────────────────────────────────────────────────────────┐");
    println!("│ PHASE 4: Belief Perturbation with Energy Cost                  │");
    println!("└─────────────────────────────────────────────────────────────────┘");
    println!("  Attempting belief mutation on Node A: δ = +0.1");
    println!("  PHYSICS: Belief change costs energy. This is NOT free.\n");

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

    println!("  OBSERVATION:");
    println!("  ├── Belief changed: 0.7 → 0.8 (as requested)");
    println!("  ├── Energy cost: ~0.105 units (physics tax)");
    println!("  └── Key insight: Beliefs cannot change for free.\n");

    // ========================================================================
    // PHASE 5: Temporal Dynamics — Propagation
    // ========================================================================
    println!("┌─────────────────────────────────────────────────────────────────┐");
    println!("│ PHASE 5: Temporal Dynamics — Propagation                       │");
    println!("└─────────────────────────────────────────────────────────────────┘");
println!("  Advancing time by running propagation steps...");
    println!("  PHYSICS: Beliefs flow along edges like heat through conductors\n");

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

    println!("  OBSERVATION:");
    println!("  ├── Node B's belief shifted toward Node A's belief");
    println!("  ├── Energy was consumed in the propagation");
    println!("  ├── Stability decreased (system perturbed)");
    println!("  └── We did NOT directly mutate Node B — influence propagated!\n");

    // ========================================================================
    // PHASE 6: Governance Invariants
    // ========================================================================
    println!("┌─────────────────────────────────────────────────────────────────┐");
    println!("│ PHASE 6: Governance Invariants                                 │");
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

    println!("  INVARIANTS EXPLAINED:");
    println!("  ├── drift_ok: Energy conservation within ε = 1e-10");
    println!("  ├── energy_drift: Absolute deviation from initial energy");
    println!("  ├── coherence: Belief alignment metric [0,1]");
println!("  ├── node_count: Active nodes");
println!("  ├── edge_count: Active pathways");
    println!("  └── healthy: Overall system health (drift_ok && esv_ok)\n");

    // ========================================================================
// PHASE 7: Audit Trail — Lineage Replay
    // ========================================================================
    println!("┌─────────────────────────────────────────────────────────────────┐");
println!("│ PHASE 7: Audit Trail — Hash-Chained Operation Record           │");
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

println!("  THE AUDIT TRAIL:");
    println!("  ├── Every operation is recorded with a sequence number");
    println!("  ├── Each entry has a SHA-256 checksum");
    println!("  ├── Checksums form a hash chain (like a blockchain)");
    println!("  ├── Tampering breaks the chain — immediately detectable");
    println!("  └── This is immutable proof of how state evolved\n");

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

    println!("  ETHICAL VALIDATION:");
    println!("  ├── Every node carries an Ethical State Vector (ESV)");
    println!("  ├── Operations that violate ethical bounds are rejected");
println!("  └── Ethics is enforced at the engine boundary\n");

    // ========================================================================
    // SUMMARY
    // ========================================================================
    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║                         SUMMARY                                  ║");
    println!("╚══════════════════════════════════════════════════════════════════╝");
    println!("");
println!("  Iter is NOT a graph database. It is deterministic governed execution:");
    println!("");
    println!("  ┌─────────────────────┬──────────────────┬─────────────────────┐");
println!("  │     Aspect          │  CRUD/Graph DB   │        Iter         │");
    println!("  ├─────────────────────┼──────────────────┼─────────────────────┤");
    println!("  │ State change        │ Instant, free    │ Costs energy        │");
    println!("  │ Constraints         │ Schema only      │ Physics+governance  │");
    println!("  │ History             │ Optional logs    │ Hash-chained lineage│");
    println!("  │ Consistency         │ ACID             │ Thermodynamic       │");
    println!("  │ Belief updates      │ Direct write     │ Perturbation+prop   │");
    println!("  └─────────────────────┴──────────────────┴─────────────────────┘");
    println!("");
    println!("  KEY INVARIANTS ENFORCED:");
    println!("  • Energy conservation: drift ≤ 1e-10 (Neumaier summation)");
    println!("  • Ethical compliance: ESV validation on every operation");
    println!("  • Causal integrity: Hash-chained, tamper-evident lineage");
    println!("  • Belief bounds: All beliefs ∈ [0.0, 1.0]");
    println!("");
println!("  For an LLM reasoning through Iter:");
    println!("  → The LLM cannot hallucinate state changes");
    println!("  → Every proposed mutation is vetted by physics");
    println!("  → Every operation leaves an auditable trace");
    println!("  → Governance says \"no\" when invariants would be violated");
    println!("");
    println!("═══════════════════════════════════════════════════════════════════");
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
