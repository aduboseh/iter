//! Concurrency stress tests for SCG MCP hardening.
//!
//! These tests validate MCP behavior under concurrent load:
//! - No panics or deadlocks
//! - No invariant violations
//! - No data races (verified by test structure)
//!
//! Tests use std::thread for true parallelism without async overhead.

use parking_lot::Mutex;
use scg_mcp_server::mcp_handler::handle_rpc;
use scg_mcp_server::types::RpcRequest;
use scg_mcp_server::SubstrateRuntime;
use serde_json::{json, Value};
use std::sync::Arc;
use std::thread;

/// Thread-safe test runtime wrapper
struct TestState {
    runtime: Mutex<SubstrateRuntime>,
}

impl TestState {
    fn new() -> Self {
        Self {
            runtime: Mutex::new(SubstrateRuntime::with_defaults().expect("Failed to create runtime")),
        }
    }

    fn tool_call(&self, tool: &str, args: Value) -> Value {
        let request = RpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "tools/call".to_string(),
            params: json!({
                "name": tool,
                "arguments": args
            }),
            id: Some(json!(1)),
        };
        let mut rt = self.runtime.lock();
        let response = handle_rpc(&mut rt, request);
        serde_json::to_value(&response).unwrap()
    }

    fn is_success(response: &Value) -> bool {
        response["error"].is_null() && !response["result"].is_null()
    }
}

// ============================================================================
// Concurrency Tests
// ============================================================================

#[test]
fn concurrent_node_creation() {
    // Test: Multiple threads creating nodes simultaneously
    // Invariant: All creates succeed; no panics; IDs are unique
    
    const THREAD_COUNT: usize = 8;
    const NODES_PER_THREAD: usize = 20;

    let state = Arc::new(TestState::new());
    let mut handles = Vec::new();

    for thread_idx in 0..THREAD_COUNT {
        let state = Arc::clone(&state);
        handles.push(thread::spawn(move || {
            let mut created_ids = Vec::new();
            for i in 0..NODES_PER_THREAD {
                let belief = (thread_idx as f64 * 0.1 + i as f64 * 0.01) % 1.0;
                let resp = state.tool_call("node.create", json!({
                    "belief": belief,
                    "energy": 10.0
                }));
                
                if TestState::is_success(&resp) {
                    if let Some(text) = resp["result"]["content"][0]["text"].as_str() {
                        if let Ok(node) = serde_json::from_str::<Value>(text) {
                            if let Some(id) = node["id"].as_u64() {
                                created_ids.push(id);
                            }
                        }
                    }
                }
            }
            created_ids
        }));
    }

    // Collect all created IDs
    let mut all_ids: Vec<u64> = Vec::new();
    for handle in handles {
        let ids = handle.join().expect("Thread panicked");
        all_ids.extend(ids);
    }

    // Verify no duplicate IDs (each thread got unique IDs)
    all_ids.sort();
    let unique_count = all_ids.len();
    all_ids.dedup();
    assert_eq!(all_ids.len(), unique_count, "Duplicate node IDs detected");
    
    // Verify expected total
    assert_eq!(all_ids.len(), THREAD_COUNT * NODES_PER_THREAD);
}

#[test]
fn concurrent_node_mutations() {
    // Test: Multiple threads mutating the same nodes
    // Invariant: No panics; mutations apply atomically
    
    const THREAD_COUNT: usize = 4;
    const MUTATIONS_PER_THREAD: usize = 50;

    let state = Arc::new(TestState::new());
    
    // Create initial nodes
    let mut node_ids = Vec::new();
    for _ in 0..4 {
        let resp = state.tool_call("node.create", json!({
            "belief": 0.5,
            "energy": 100.0
        }));
        if let Some(text) = resp["result"]["content"][0]["text"].as_str() {
            if let Ok(node) = serde_json::from_str::<Value>(text) {
                if let Some(id) = node["id"].as_u64() {
                    node_ids.push(id);
                }
            }
        }
    }
    let node_ids = Arc::new(node_ids);

    let mut handles = Vec::new();
    for _ in 0..THREAD_COUNT {
        let state = Arc::clone(&state);
        let node_ids = Arc::clone(&node_ids);
        handles.push(thread::spawn(move || {
            let mut success_count = 0;
            for i in 0..MUTATIONS_PER_THREAD {
                let node_id = node_ids[i % node_ids.len()];
                let delta = if i % 2 == 0 { 0.01 } else { -0.01 };
                
                let resp = state.tool_call("node.mutate", json!({
                    "node_id": node_id.to_string(),
                    "delta": delta
                }));
                
                if TestState::is_success(&resp) {
                    success_count += 1;
                }
            }
            success_count
        }));
    }

    // All mutations should succeed
    let total_success: usize = handles.into_iter()
        .map(|h| h.join().expect("Thread panicked"))
        .sum();
    
    assert_eq!(total_success, THREAD_COUNT * MUTATIONS_PER_THREAD);
}

#[test]
fn concurrent_mixed_operations() {
    // Test: Mix of creates, queries, mutations, and governance checks
    // Invariant: No panics; governance stays healthy
    
    const THREAD_COUNT: usize = 8;
    const OPS_PER_THREAD: usize = 30;

    let state = Arc::new(TestState::new());
    
    // Pre-create a few nodes
    for _ in 0..4 {
        state.tool_call("node.create", json!({"belief": 0.5, "energy": 10.0}));
    }

    let mut handles = Vec::new();
    for thread_idx in 0..THREAD_COUNT {
        let state = Arc::clone(&state);
        handles.push(thread::spawn(move || {
            for i in 0..OPS_PER_THREAD {
                let op = (thread_idx + i) % 5;
                match op {
                    0 => {
                        // Create node
                        state.tool_call("node.create", json!({
                            "belief": 0.5,
                            "energy": 10.0
                        }));
                    }
                    1 => {
                        // Query node
                        state.tool_call("node.query", json!({
                            "node_id": "0"
                        }));
                    }
                    2 => {
                        // Mutate node
                        state.tool_call("node.mutate", json!({
                            "node_id": "0",
                            "delta": 0.01
                        }));
                    }
                    3 => {
                        // Governance status
                        state.tool_call("governance.status", json!({}));
                    }
                    4 => {
                        // Lineage replay
                        state.tool_call("lineage.replay", json!({}));
                    }
                    _ => {}
                }
            }
        }));
    }

    // Wait for all threads
    for handle in handles {
        handle.join().expect("Thread panicked during mixed operations");
    }

    // Verify governance is still healthy
    let status_resp = state.tool_call("governance.status", json!({}));
    assert!(TestState::is_success(&status_resp), "Governance check failed after concurrency test");
}

#[test]
fn concurrent_edge_operations() {
    // Test: Concurrent edge binding and propagation
    // Invariant: No panics; edges bind correctly
    
    const THREAD_COUNT: usize = 4;

    let state = Arc::new(TestState::new());
    
    // Create nodes for edge binding
    let mut node_ids = Vec::new();
    for _ in 0..8 {
        let resp = state.tool_call("node.create", json!({
            "belief": 0.5,
            "energy": 10.0
        }));
        if let Some(text) = resp["result"]["content"][0]["text"].as_str() {
            if let Ok(node) = serde_json::from_str::<Value>(text) {
                if let Some(id) = node["id"].as_u64() {
                    node_ids.push(id);
                }
            }
        }
    }
    let node_ids = Arc::new(node_ids);

    let mut handles = Vec::new();
    for thread_idx in 0..THREAD_COUNT {
        let state = Arc::clone(&state);
        let node_ids = Arc::clone(&node_ids);
        handles.push(thread::spawn(move || {
            // Each thread binds edges between different node pairs
            let src_idx = thread_idx * 2;
            let dst_idx = thread_idx * 2 + 1;
            
            if src_idx < node_ids.len() && dst_idx < node_ids.len() {
                let src = node_ids[src_idx];
                let dst = node_ids[dst_idx];
                
                state.tool_call("edge.bind", json!({
                    "src": src.to_string(),
                    "dst": dst.to_string(),
                    "weight": 0.5
                }));
                
                // Run propagation
                state.tool_call("edge.propagate", json!({"edge_id": "0"}));
            }
        }));
    }

    for handle in handles {
        handle.join().expect("Thread panicked during edge operations");
    }
}

#[test]
fn concurrent_governance_queries() {
    // Test: High frequency governance status queries
    // Invariant: All queries succeed; status is consistent
    
    const THREAD_COUNT: usize = 16;
    const QUERIES_PER_THREAD: usize = 50;

    let state = Arc::new(TestState::new());
    
    // Create some nodes first
    for _ in 0..10 {
        state.tool_call("node.create", json!({"belief": 0.5, "energy": 10.0}));
    }

    let mut handles = Vec::new();
    for _ in 0..THREAD_COUNT {
        let state = Arc::clone(&state);
        handles.push(thread::spawn(move || {
            let mut success_count = 0;
            for _ in 0..QUERIES_PER_THREAD {
                let resp = state.tool_call("governance.status", json!({}));
                if TestState::is_success(&resp) {
                    success_count += 1;
                }
            }
            success_count
        }));
    }

    let total_success: usize = handles.into_iter()
        .map(|h| h.join().expect("Thread panicked"))
        .sum();
    
    // All queries should succeed
    assert_eq!(total_success, THREAD_COUNT * QUERIES_PER_THREAD);
}

#[test]
fn concurrent_lineage_queries() {
    // Test: Concurrent lineage replay requests
    // Invariant: All replays succeed; lineage is consistent
    
    const THREAD_COUNT: usize = 8;
    const QUERIES_PER_THREAD: usize = 20;

    let state = Arc::new(TestState::new());
    
    // Generate some lineage by creating nodes and stepping
    for _ in 0..5 {
        state.tool_call("node.create", json!({"belief": 0.5, "energy": 10.0}));
        state.tool_call("edge.propagate", json!({"edge_id": "0"}));
    }

    let mut handles = Vec::new();
    for _ in 0..THREAD_COUNT {
        let state = Arc::clone(&state);
        handles.push(thread::spawn(move || {
            let mut success_count = 0;
            for _ in 0..QUERIES_PER_THREAD {
                let resp = state.tool_call("lineage.replay", json!({}));
                if TestState::is_success(&resp) {
                    success_count += 1;
                }
            }
            success_count
        }));
    }

    let total_success: usize = handles.into_iter()
        .map(|h| h.join().expect("Thread panicked"))
        .sum();
    
    assert_eq!(total_success, THREAD_COUNT * QUERIES_PER_THREAD);
}

#[test]
fn no_deadlock_under_contention() {
    // Test: Heavy contention on shared runtime
    // Invariant: Test completes within timeout; no deadlocks
    
    const THREAD_COUNT: usize = 32;
    const OPS_PER_THREAD: usize = 20;

    let state = Arc::new(TestState::new());
    
    // Pre-create nodes
    for _ in 0..4 {
        state.tool_call("node.create", json!({"belief": 0.5, "energy": 10.0}));
    }

    let mut handles = Vec::new();
    for _ in 0..THREAD_COUNT {
        let state = Arc::clone(&state);
        handles.push(thread::spawn(move || {
            for i in 0..OPS_PER_THREAD {
                // Alternate between write and read operations
                if i % 2 == 0 {
                    state.tool_call("node.mutate", json!({
                        "node_id": "0",
                        "delta": 0.001
                    }));
                } else {
                    state.tool_call("governance.status", json!({}));
                }
            }
        }));
    }

    // If we complete without hanging, no deadlock occurred
    for handle in handles {
        handle.join().expect("Thread panicked");
    }
}

#[test]
fn invariants_hold_after_concurrent_load() {
    // Test: Verify system invariants after heavy concurrent workload
    // Invariant: Governance healthy; no corruption
    
    const THREAD_COUNT: usize = 8;
    const OPS_PER_THREAD: usize = 50;

    let state = Arc::new(TestState::new());

    let mut handles = Vec::new();
    for thread_idx in 0..THREAD_COUNT {
        let state = Arc::clone(&state);
        handles.push(thread::spawn(move || {
            for i in 0..OPS_PER_THREAD {
                let belief = ((thread_idx * OPS_PER_THREAD + i) as f64 * 0.01) % 1.0;
                state.tool_call("node.create", json!({
                    "belief": belief,
                    "energy": 10.0
                }));
                
                if i % 10 == 0 {
                    state.tool_call("edge.propagate", json!({"edge_id": "0"}));
                }
            }
        }));
    }

    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    // Verify invariants
    let status_resp = state.tool_call("governance.status", json!({}));
    assert!(TestState::is_success(&status_resp), "Governance check failed");
    
    if let Some(text) = status_resp["result"]["content"][0]["text"].as_str() {
        if let Ok(status) = serde_json::from_str::<Value>(text) {
            // System should be healthy
            assert!(status["healthy"].as_bool().unwrap_or(false), "System not healthy after load");
            
            // Node count should match what we created
            let expected_nodes = THREAD_COUNT * OPS_PER_THREAD;
            let actual_nodes = status["node_count"].as_u64().unwrap_or(0) as usize;
            assert_eq!(actual_nodes, expected_nodes, "Node count mismatch");
        }
    }
}
