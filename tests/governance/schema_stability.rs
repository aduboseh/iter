//! Schema Stability Invariant Tests
//!
//! These tests enforce that public protocol types maintain stable shapes.
//! Any breaking change to field names, types, or required fields will fail CI.
//!
//! # Governance Contract
//!
//! - RpcRequest/RpcResponse shapes are frozen at v1
//! - Field additions require minor version bump
//! - Field removals/renames require major version bump
//! - These tests compile in public_stub mode (no substrate deps)

use iter_mcp_server::types::mcp::*;
use iter_mcp_server::types::protocol::*;
use serde_json::json;

// ============================================================================
// RpcRequest Schema Invariants
// ============================================================================

#[test]
fn rpc_request_has_required_fields() {
    // Minimal valid request must parse
    let minimal = json!({
        "jsonrpc": "2.0",
        "method": "test"
    });

    let req: RpcRequest = serde_json::from_value(minimal).expect("minimal request should parse");
    assert_eq!(req.jsonrpc, "2.0");
    assert_eq!(req.method, "test");
    assert!(req.params.is_null(), "params should default to null");
    assert!(req.id.is_none(), "id should default to None");
}

#[test]
fn rpc_request_accepts_optional_fields() {
    let full = json!({
        "jsonrpc": "2.0",
        "method": "node.create",
        "params": {"belief": 0.5, "energy": 1.0},
        "id": 42
    });

    let req: RpcRequest = serde_json::from_value(full).expect("full request should parse");
    assert_eq!(req.method, "node.create");
    assert!(!req.params.is_null());
    assert_eq!(req.id, Some(json!(42)));
}

#[test]
fn rpc_request_rejects_missing_required() {
    // Missing jsonrpc
    let bad1 = json!({"method": "test"});
    assert!(serde_json::from_value::<RpcRequest>(bad1).is_err());

    // Missing method
    let bad2 = json!({"jsonrpc": "2.0"});
    assert!(serde_json::from_value::<RpcRequest>(bad2).is_err());
}

// ============================================================================
// RpcResponse Schema Invariants
// ============================================================================

#[test]
fn rpc_response_success_shape() {
    let resp = RpcResponse::success(json!(1), json!({"result": "ok"}));

    let serialized = serde_json::to_value(&resp).expect("should serialize");

    // Required fields
    assert_eq!(serialized["jsonrpc"], "2.0");
    assert_eq!(serialized["id"], 1);
    assert!(serialized.get("result").is_some());

    // Error must be absent on success
    assert!(serialized.get("error").is_none());
}

#[test]
fn rpc_response_error_shape() {
    let resp = RpcResponse::error(json!(2), -32600, "Invalid Request");

    let serialized = serde_json::to_value(&resp).expect("should serialize");

    // Required fields
    assert_eq!(serialized["jsonrpc"], "2.0");
    assert_eq!(serialized["id"], 2);
    assert!(serialized.get("error").is_some());

    // Error structure
    assert_eq!(serialized["error"]["code"], -32600);
    assert_eq!(serialized["error"]["message"], "Invalid Request");

    // Result must be absent on error
    assert!(serialized.get("result").is_none());
}

#[test]
fn rpc_response_roundtrip() {
    let original = RpcResponse::success(json!("req-123"), json!({"node_id": 1}));
    let json_str = serde_json::to_string(&original).expect("serialize");
    let parsed: RpcResponse = serde_json::from_str(&json_str).expect("deserialize");

    assert_eq!(parsed.jsonrpc, "2.0");
    assert_eq!(parsed.id, json!("req-123"));
}

// ============================================================================
// MCP Type Schema Invariants
// ============================================================================

#[test]
fn mcp_node_state_has_all_fields() {
    let node = McpNodeState {
        id: 1,
        belief: 0.5,
        energy: 1.0,
        esv_valid: true,
        stability: 0.9,
    };

    let serialized = serde_json::to_value(&node).expect("should serialize");

    // All fields must be present
    assert!(serialized.get("id").is_some());
    assert!(serialized.get("belief").is_some());
    assert!(serialized.get("energy").is_some());
    assert!(serialized.get("esv_valid").is_some());
    assert!(serialized.get("stability").is_some());
}

#[test]
fn mcp_edge_state_has_all_fields() {
    let edge = McpEdgeState {
        id: 1,
        src: 2,
        dst: 3,
        weight: 0.7,
    };

    let serialized = serde_json::to_value(&edge).expect("should serialize");

    assert!(serialized.get("id").is_some());
    assert!(serialized.get("src").is_some());
    assert!(serialized.get("dst").is_some());
    assert!(serialized.get("weight").is_some());
}

#[test]
fn mcp_governor_status_has_all_fields() {
    let status = McpGovernorStatus {
        drift_ok: true,
        energy_drift: 0.0,
        coherence: 1.0,
        node_count: 5,
        edge_count: 4,
        healthy: true,
    };

    let serialized = serde_json::to_value(&status).expect("should serialize");

    assert!(serialized.get("drift_ok").is_some());
    assert!(serialized.get("energy_drift").is_some());
    assert!(serialized.get("coherence").is_some());
    assert!(serialized.get("node_count").is_some());
    assert!(serialized.get("edge_count").is_some());
    assert!(serialized.get("healthy").is_some());
}

#[test]
fn mcp_lineage_entry_has_all_fields() {
    let entry = McpLineageEntry {
        sequence: 1,
        operation: "tick".to_string(),
        checksum: "abc123".to_string(),
        tick: 100,
    };

    let serialized = serde_json::to_value(&entry).expect("should serialize");

    assert!(serialized.get("sequence").is_some());
    assert!(serialized.get("operation").is_some());
    assert!(serialized.get("checksum").is_some());
    assert!(serialized.get("tick").is_some());
}

// ============================================================================
// Parameter Types Schema Invariants
// ============================================================================

#[test]
fn create_node_params_schema() {
    let valid = json!({"belief": 0.5, "energy": 1.0});
    let params: CreateNodeParams = serde_json::from_value(valid).expect("should parse");
    assert_eq!(params.belief, 0.5);
    assert_eq!(params.energy, 1.0);
}

#[test]
fn mutate_node_params_schema() {
    let valid = json!({"node_id": 42, "delta": -0.1});
    let params: MutateNodeParams = serde_json::from_value(valid).expect("should parse");
    assert_eq!(params.node_id, 42);
    assert_eq!(params.delta, -0.1);
}

#[test]
fn query_node_params_schema() {
    let valid = json!({"node_id": 99});
    let params: QueryNodeParams = serde_json::from_value(valid).expect("should parse");
    assert_eq!(params.node_id, 99);
}

#[test]
fn bind_edge_params_schema() {
    let valid = json!({"src": 1, "dst": 2, "weight": 0.8});
    let params: BindEdgeParams = serde_json::from_value(valid).expect("should parse");
    assert_eq!(params.src, 1);
    assert_eq!(params.dst, 2);
    assert_eq!(params.weight, 0.8);
}

#[test]
fn propagate_edge_params_schema() {
    let valid = json!({"edge_id": 7});
    let params: PropagateEdgeParams = serde_json::from_value(valid).expect("should parse");
    assert_eq!(params.edge_id, 7);
}

#[test]
fn export_lineage_params_schema() {
    let valid = json!({"path": "/tmp/lineage.json"});
    let params: ExportLineageParams = serde_json::from_value(valid).expect("should parse");
    assert_eq!(params.path, "/tmp/lineage.json");
}
