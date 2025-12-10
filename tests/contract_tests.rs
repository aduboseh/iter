//! Contract Tests - MCP API Schema Validation
//!
//! These tests validate that MCP types serialize/deserialize correctly
//! and that error responses match the documented contract.
//!
//! Contract tests are non-optional: if a diff changes JSON shape,
//! these tests must fail and force an explicit decision.

use scg_mcp_server::types::{
    McpError, McpNodeState, McpEdgeState, McpGovernorStatus, McpLineageEntry,
    RpcRequest, RpcResponse, RpcError,
};
use serde_json::{json, Value};

// ============================================================================
// McpError Contract Tests
// ============================================================================

#[test]
fn test_mcp_error_code_stability() {
    // These codes are part of the contract and must not change
    assert_eq!(McpError::NodeNotFound { id: 0 }.code(), 4004);
    assert_eq!(McpError::EdgeNotFound { id: 0 }.code(), 4004);
    assert_eq!(McpError::BadRequest { message: String::new() }.code(), 4000);
    assert_eq!(McpError::EsvValidationFailed { reason: String::new() }.code(), 1000);
    assert_eq!(McpError::DriftExceeded { drift: 0.0, threshold: 0.0 }.code(), 2000);
    assert_eq!(McpError::LineageCorruption { details: String::new() }.code(), 3000);
    assert_eq!(McpError::SubstrateError { message: String::new() }.code(), 5000);
}

#[test]
fn test_mcp_error_code_string_stability() {
    // These strings are part of the contract and must not change
    assert_eq!(McpError::NodeNotFound { id: 0 }.code_string(), "node_not_found");
    assert_eq!(McpError::EdgeNotFound { id: 0 }.code_string(), "edge_not_found");
    assert_eq!(McpError::BadRequest { message: String::new() }.code_string(), "bad_request");
    assert_eq!(McpError::EsvValidationFailed { reason: String::new() }.code_string(), "esv_validation_failed");
    assert_eq!(McpError::DriftExceeded { drift: 0.0, threshold: 0.0 }.code_string(), "drift_exceeded");
    assert_eq!(McpError::LineageCorruption { details: String::new() }.code_string(), "lineage_corruption");
    assert_eq!(McpError::SubstrateError { message: String::new() }.code_string(), "substrate_error");
}

#[test]
fn test_mcp_error_display_format() {
    // Display format should be human-readable
    let err = McpError::NodeNotFound { id: 42 };
    assert!(err.to_string().contains("42"));
    assert!(err.to_string().contains("Node not found"));

    let err = McpError::BadRequest { message: "test error".to_string() };
    assert!(err.to_string().contains("test error"));
}

// ============================================================================
// McpNodeState Contract Tests
// ============================================================================

#[test]
fn test_mcp_node_state_roundtrip() {
    let node = McpNodeState {
        id: 123,
        belief: 0.75,
        energy: 10.5,
        esv_valid: true,
        stability: 0.95,
    };

    // Serialize to JSON
    let json_str = serde_json::to_string(&node).expect("Should serialize");
    
    // Deserialize back
    let parsed: McpNodeState = serde_json::from_str(&json_str).expect("Should deserialize");
    
    assert_eq!(parsed.id, 123);
    assert!((parsed.belief - 0.75).abs() < 1e-10);
    assert!((parsed.energy - 10.5).abs() < 1e-10);
    assert!(parsed.esv_valid);
    assert!((parsed.stability - 0.95).abs() < 1e-10);
}

#[test]
fn test_mcp_node_state_schema_shape() {
    let node = McpNodeState {
        id: 0,
        belief: 0.5,
        energy: 10.0,
        esv_valid: true,
        stability: 1.0,
    };

    let json: Value = serde_json::to_value(&node).expect("Should convert to Value");
    
    // Validate required fields exist with correct types
    assert!(json["id"].is_u64());
    assert!(json["belief"].is_f64());
    assert!(json["energy"].is_f64());
    assert!(json["esv_valid"].is_boolean());
    assert!(json["stability"].is_f64());
}

// ============================================================================
// McpEdgeState Contract Tests
// ============================================================================

#[test]
fn test_mcp_edge_state_roundtrip() {
    let edge = McpEdgeState {
        id: 456,
        src: 1,
        dst: 2,
        weight: 0.8,
    };

    let json_str = serde_json::to_string(&edge).expect("Should serialize");
    let parsed: McpEdgeState = serde_json::from_str(&json_str).expect("Should deserialize");
    
    assert_eq!(parsed.id, 456);
    assert_eq!(parsed.src, 1);
    assert_eq!(parsed.dst, 2);
    assert!((parsed.weight - 0.8).abs() < 1e-10);
}

#[test]
fn test_mcp_edge_state_schema_shape() {
    let edge = McpEdgeState {
        id: 0,
        src: 1,
        dst: 2,
        weight: 0.5,
    };

    let json: Value = serde_json::to_value(&edge).expect("Should convert to Value");
    
    assert!(json["id"].is_u64());
    assert!(json["src"].is_u64());
    assert!(json["dst"].is_u64());
    assert!(json["weight"].is_f64());
}

// ============================================================================
// McpGovernorStatus Contract Tests
// ============================================================================

#[test]
fn test_mcp_governor_status_roundtrip() {
    let status = McpGovernorStatus {
        drift_ok: true,
        energy_drift: 1e-12,
        coherence: 0.98,
        node_count: 10,
        edge_count: 15,
        healthy: true,
    };

    let json_str = serde_json::to_string(&status).expect("Should serialize");
    let parsed: McpGovernorStatus = serde_json::from_str(&json_str).expect("Should deserialize");
    
    assert!(parsed.drift_ok);
    assert!(parsed.energy_drift < 1e-10);
    assert!((parsed.coherence - 0.98).abs() < 1e-10);
    assert_eq!(parsed.node_count, 10);
    assert_eq!(parsed.edge_count, 15);
    assert!(parsed.healthy);
}

#[test]
fn test_mcp_governor_status_schema_shape() {
    let status = McpGovernorStatus {
        drift_ok: true,
        energy_drift: 0.0,
        coherence: 1.0,
        node_count: 0,
        edge_count: 0,
        healthy: true,
    };

    let json: Value = serde_json::to_value(&status).expect("Should convert to Value");
    
    assert!(json["drift_ok"].is_boolean());
    assert!(json["energy_drift"].is_f64());
    assert!(json["coherence"].is_f64());
    assert!(json["node_count"].is_u64());
    assert!(json["edge_count"].is_u64());
    assert!(json["healthy"].is_boolean());
}

// ============================================================================
// McpLineageEntry Contract Tests
// ============================================================================

#[test]
fn test_mcp_lineage_entry_roundtrip() {
    let entry = McpLineageEntry {
        sequence: 42,
        operation: "tick".to_string(),
        checksum: "abcd1234".repeat(8), // 64 chars
        tick: 100,
    };

    let json_str = serde_json::to_string(&entry).expect("Should serialize");
    let parsed: McpLineageEntry = serde_json::from_str(&json_str).expect("Should deserialize");
    
    assert_eq!(parsed.sequence, 42);
    assert_eq!(parsed.operation, "tick");
    assert_eq!(parsed.checksum.len(), 64);
    assert_eq!(parsed.tick, 100);
}

#[test]
fn test_mcp_lineage_entry_schema_shape() {
    let entry = McpLineageEntry {
        sequence: 0,
        operation: "decision".to_string(),
        checksum: "0".repeat(64),
        tick: 0,
    };

    let json: Value = serde_json::to_value(&entry).expect("Should convert to Value");
    
    assert!(json["sequence"].is_u64());
    assert!(json["operation"].is_string());
    assert!(json["checksum"].is_string());
    assert!(json["tick"].is_u64());
}

// ============================================================================
// RPC Protocol Contract Tests
// ============================================================================

#[test]
fn test_rpc_request_roundtrip() {
    let request = RpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "node.create".to_string(),
        params: json!({"belief": 0.5, "energy": 10.0}),
        id: Some(json!(1)),
    };

    let json_str = serde_json::to_string(&request).expect("Should serialize");
    let parsed: RpcRequest = serde_json::from_str(&json_str).expect("Should deserialize");
    
    assert_eq!(parsed.jsonrpc, "2.0");
    assert_eq!(parsed.method, "node.create");
    assert!(parsed.id.is_some());
}

#[test]
fn test_rpc_response_success_shape() {
    let response = RpcResponse::success(json!(1), json!({"id": 0, "belief": 0.5}));
    
    let json: Value = serde_json::to_value(&response).expect("Should convert");
    
    assert_eq!(json["jsonrpc"], "2.0");
    assert!(json["result"].is_object());
    assert!(json["error"].is_null());
    assert_eq!(json["id"], 1);
}

#[test]
fn test_rpc_response_error_shape() {
    let response = RpcResponse::error(json!(1), 4004, "Node not found");
    
    let json: Value = serde_json::to_value(&response).expect("Should convert");
    
    assert_eq!(json["jsonrpc"], "2.0");
    assert!(json["result"].is_null());
    assert!(json["error"].is_object());
    assert_eq!(json["error"]["code"], 4004);
    assert_eq!(json["error"]["message"], "Node not found");
    assert_eq!(json["id"], 1);
}

#[test]
fn test_rpc_error_structure() {
    let error = RpcError {
        code: 4000,
        message: "Bad request".to_string(),
    };

    let json: Value = serde_json::to_value(&error).expect("Should convert");
    
    assert!(json["code"].is_i64());
    assert!(json["message"].is_string());
}

// ============================================================================
// Integration: Error -> RPC Response Contract
// ============================================================================

#[test]
fn test_mcp_error_to_rpc_response() {
    let err = McpError::NodeNotFound { id: 42 };
    let response = RpcResponse::from_mcp_error(json!(1), err);
    
    let json: Value = serde_json::to_value(&response).expect("Should convert");
    
    assert_eq!(json["error"]["code"], 4004);
    assert!(json["error"]["message"].as_str().unwrap().contains("42"));
}

#[test]
fn test_all_error_variants_produce_valid_rpc() {
    let errors = vec![
        McpError::NodeNotFound { id: 1 },
        McpError::EdgeNotFound { id: 2 },
        McpError::BadRequest { message: "test".to_string() },
        McpError::EsvValidationFailed { reason: "test".to_string() },
        McpError::DriftExceeded { drift: 0.1, threshold: 0.01 },
        McpError::LineageCorruption { details: "test".to_string() },
        McpError::SubstrateError { message: "test".to_string() },
    ];

    for err in errors {
        let response = RpcResponse::from_mcp_error(json!(1), err);
        let json: Value = serde_json::to_value(&response).expect("Should convert");
        
        // All error responses must have error.code and error.message
        assert!(json["error"]["code"].is_i64(), "Missing error code");
        assert!(json["error"]["message"].is_string(), "Missing error message");
        assert!(json["result"].is_null(), "Error response should not have result");
    }
}
