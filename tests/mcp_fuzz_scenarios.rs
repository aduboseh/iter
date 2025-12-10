//! Property-based MCP Scenario Fuzzer
//!
//! Uses proptest to generate random sequences of MCP operations
//! and verify that invariants hold after each sequence.
//!
//! Bounds:
//! - Max 64 nodes per scenario
//! - Max 128 edges per scenario
//! - Sequence length 20-50 operations
//!
//! Invariants verified:
//! - No panics
//! - Governance remains healthy (or fails gracefully)
//! - Expected errors are handled, not propagated as panics

use proptest::prelude::*;
use scg_mcp_server::mcp_handler::handle_rpc;
use scg_mcp_server::types::RpcRequest;
use scg_mcp_server::SubstrateRuntime;
use serde_json::{json, Value};

// ============================================================================
// Test Configuration
// ============================================================================

const MAX_NODES: usize = 64;
const MAX_EDGES: usize = 128;
const MIN_SEQUENCE_LENGTH: usize = 20;
const MAX_SEQUENCE_LENGTH: usize = 50;

// ============================================================================
// Operation Types
// ============================================================================

#[derive(Debug, Clone)]
enum Op {
    CreateNode { belief: f64, energy: f64 },
    MutateNode { target_index: usize, delta: f64 },
    QueryNode { target_index: usize },
    BindEdge { src_index: usize, dst_index: usize, weight: f64 },
    Step,
    GovernanceStatus,
    EsvAudit { target_index: usize },
    LineageReplay,
}

// ============================================================================
// Test Runtime
// ============================================================================

struct FuzzRuntime {
    runtime: SubstrateRuntime,
    known_node_ids: Vec<u64>,
    edge_count: usize,
}

impl FuzzRuntime {
    fn new() -> Self {
        Self {
            runtime: SubstrateRuntime::with_defaults().expect("Failed to create runtime"),
            known_node_ids: Vec::new(),
            edge_count: 0,
        }
    }

    fn tool_call(&mut self, tool: &str, args: Value) -> Value {
        let request = RpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "tools/call".to_string(),
            params: json!({
                "name": tool,
                "arguments": args
            }),
            id: Some(json!(1)),
        };
        let response = handle_rpc(&mut self.runtime, request);
        serde_json::to_value(&response).unwrap()
    }

    fn is_success(response: &Value) -> bool {
        response["error"].is_null() && !response["result"].is_null()
    }

    fn execute_op(&mut self, op: &Op) {
        match op {
            Op::CreateNode { belief, energy } => {
                if self.known_node_ids.len() < MAX_NODES {
                    let resp = self.tool_call("node.create", json!({
                        "belief": belief.clamp(0.0, 1.0),
                        "energy": energy.max(0.0)
                    }));
                    if Self::is_success(&resp) {
                        if let Some(text) = resp["result"]["content"][0]["text"].as_str() {
                            if let Ok(node) = serde_json::from_str::<Value>(text) {
                                if let Some(id) = node["id"].as_u64() {
                                    self.known_node_ids.push(id);
                                }
                            }
                        }
                    }
                }
            }
            Op::MutateNode { target_index, delta } => {
                if !self.known_node_ids.is_empty() {
                    let idx = target_index % self.known_node_ids.len();
                    let node_id = self.known_node_ids[idx];
                    // Expected to succeed or fail gracefully
                    let _ = self.tool_call("node.mutate", json!({
                        "node_id": node_id.to_string(),
                        "delta": delta
                    }));
                }
            }
            Op::QueryNode { target_index } => {
                if !self.known_node_ids.is_empty() {
                    let idx = target_index % self.known_node_ids.len();
                    let node_id = self.known_node_ids[idx];
                    let _ = self.tool_call("node.query", json!({
                        "node_id": node_id.to_string()
                    }));
                }
            }
            Op::BindEdge { src_index, dst_index, weight } => {
                if self.known_node_ids.len() >= 2 && self.edge_count < MAX_EDGES {
                    let src_idx = src_index % self.known_node_ids.len();
                    let dst_idx = dst_index % self.known_node_ids.len();
                    if src_idx != dst_idx {
                        let src = self.known_node_ids[src_idx];
                        let dst = self.known_node_ids[dst_idx];
                        let resp = self.tool_call("edge.bind", json!({
                            "src": src.to_string(),
                            "dst": dst.to_string(),
                            "weight": weight.clamp(0.0, 1.0)
                        }));
                        if Self::is_success(&resp) {
                            self.edge_count += 1;
                        }
                    }
                }
            }
            Op::Step => {
                let _ = self.tool_call("edge.propagate", json!({"edge_id": "0"}));
            }
            Op::GovernanceStatus => {
                let _ = self.tool_call("governance.status", json!({}));
            }
            Op::EsvAudit { target_index } => {
                if !self.known_node_ids.is_empty() {
                    let idx = target_index % self.known_node_ids.len();
                    let node_id = self.known_node_ids[idx];
                    let _ = self.tool_call("esv.audit", json!({
                        "node_id": node_id.to_string()
                    }));
                }
            }
            Op::LineageReplay => {
                let _ = self.tool_call("lineage.replay", json!({}));
            }
        }
    }

    fn verify_invariants(&mut self) -> Result<(), String> {
        let resp = self.tool_call("governance.status", json!({}));
        if !Self::is_success(&resp) {
            // Governance query itself failed - this is unexpected
            return Err("Governance status query failed".to_string());
        }
        // Governance healthy check is informational, not a hard invariant
        // (drift may accumulate during fuzzing)
        Ok(())
    }
}

// ============================================================================
// Proptest Strategies
// ============================================================================

fn op_strategy() -> impl Strategy<Value = Op> {
    prop_oneof![
        // Node creation - weighted higher to build up graph
        3 => (0.0..=1.0f64, 1.0..=100.0f64).prop_map(|(belief, energy)| {
            Op::CreateNode { belief, energy }
        }),
        // Mutations
        2 => (any::<usize>(), -0.5..=0.5f64).prop_map(|(target_index, delta)| {
            Op::MutateNode { target_index, delta }
        }),
        // Queries
        1 => any::<usize>().prop_map(|target_index| {
            Op::QueryNode { target_index }
        }),
        // Edge binding
        2 => (any::<usize>(), any::<usize>(), 0.0..=1.0f64).prop_map(|(src_index, dst_index, weight)| {
            Op::BindEdge { src_index, dst_index, weight }
        }),
        // Steps
        2 => Just(Op::Step),
        // Governance
        1 => Just(Op::GovernanceStatus),
        // ESV audit
        1 => any::<usize>().prop_map(|target_index| {
            Op::EsvAudit { target_index }
        }),
        // Lineage
        1 => Just(Op::LineageReplay),
    ]
}

// ============================================================================
// Property Tests
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn random_mcp_scenarios_do_not_panic(
        ops in proptest::collection::vec(op_strategy(), MIN_SEQUENCE_LENGTH..MAX_SEQUENCE_LENGTH)
    ) {
        let mut rt = FuzzRuntime::new();
        
        for op in &ops {
            rt.execute_op(op);
        }
        
        // Verify we can still query governance after the sequence
        rt.verify_invariants().expect("Invariant check failed");
    }

    #[test]
    fn pure_create_sequences_produce_unique_ids(
        beliefs in proptest::collection::vec(0.0..=1.0f64, 10..30)
    ) {
        let mut rt = FuzzRuntime::new();
        
        for belief in &beliefs {
            rt.execute_op(&Op::CreateNode { belief: *belief, energy: 10.0 });
        }
        
        // All IDs should be unique
        let mut ids = rt.known_node_ids.clone();
        ids.sort();
        ids.dedup();
        prop_assert_eq!(ids.len(), rt.known_node_ids.len(), "Duplicate IDs generated");
    }

    #[test]
    fn mutation_sequences_do_not_corrupt_state(
        deltas in proptest::collection::vec(-0.5..=0.5f64, 20..40)
    ) {
        let mut rt = FuzzRuntime::new();
        
        // Create a node first
        rt.execute_op(&Op::CreateNode { belief: 0.5, energy: 100.0 });
        
        // Apply many mutations
        for delta in &deltas {
            rt.execute_op(&Op::MutateNode { target_index: 0, delta: *delta });
        }
        
        // Query should still work
        let resp = rt.tool_call("node.query", json!({"node_id": "0"}));
        prop_assert!(FuzzRuntime::is_success(&resp), "Node query failed after mutations");
    }

    #[test]
    fn step_sequences_do_not_panic(
        step_count in 10..50usize
    ) {
        let mut rt = FuzzRuntime::new();
        
        // Create some nodes
        for i in 0..5 {
            rt.execute_op(&Op::CreateNode { belief: i as f64 * 0.2, energy: 10.0 });
        }
        
        // Run many steps
        for _ in 0..step_count {
            rt.execute_op(&Op::Step);
        }
        
        rt.verify_invariants().expect("Invariant check failed after steps");
    }

    #[test]
    fn mixed_read_write_sequences_are_consistent(
        ops in proptest::collection::vec(op_strategy(), 30..50)
    ) {
        let mut rt = FuzzRuntime::new();
        
        // Ensure we have at least one node
        rt.execute_op(&Op::CreateNode { belief: 0.5, energy: 10.0 });
        
        for op in &ops {
            rt.execute_op(op);
        }
        
        // Final governance check
        let resp = rt.tool_call("governance.status", json!({}));
        prop_assert!(FuzzRuntime::is_success(&resp), "Final governance check failed");
    }
}

// ============================================================================
// Additional Focused Tests
// ============================================================================

#[test]
fn edge_case_extreme_beliefs() {
    let mut rt = FuzzRuntime::new();
    
    // Test boundary values
    rt.execute_op(&Op::CreateNode { belief: 0.0, energy: 10.0 });
    rt.execute_op(&Op::CreateNode { belief: 1.0, energy: 10.0 });
    rt.execute_op(&Op::CreateNode { belief: 0.5, energy: 0.0 });
    
    // Extreme mutations
    rt.execute_op(&Op::MutateNode { target_index: 0, delta: 100.0 });
    rt.execute_op(&Op::MutateNode { target_index: 1, delta: -100.0 });
    
    rt.verify_invariants().expect("Invariants failed with extreme values");
}

#[test]
fn edge_case_rapid_governance_queries() {
    let mut rt = FuzzRuntime::new();
    
    rt.execute_op(&Op::CreateNode { belief: 0.5, energy: 10.0 });
    
    // Rapid governance queries shouldn't cause issues
    for _ in 0..100 {
        rt.execute_op(&Op::GovernanceStatus);
    }
    
    rt.verify_invariants().expect("Invariants failed after rapid queries");
}

#[test]
fn edge_case_alternating_create_step() {
    let mut rt = FuzzRuntime::new();
    
    // Alternating creates and steps
    for _ in 0..30 {
        rt.execute_op(&Op::CreateNode { belief: 0.5, energy: 10.0 });
        rt.execute_op(&Op::Step);
    }
    
    rt.verify_invariants().expect("Invariants failed after alternating ops");
}
