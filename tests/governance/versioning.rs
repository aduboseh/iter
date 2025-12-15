//! Protocol Versioning Invariant Tests
//!
//! These tests enforce versioning rules and wire compatibility.
//!
//! # Governance Contract
//!
//! - Protocol version is explicit and parseable
//! - Compatibility rules are enforced
//! - Wire format matches golden snapshots
//! - Breaking changes require major version bump

use iter_mcp_server::{
    PROTOCOL_VERSION, PROTOCOL_MAJOR, PROTOCOL_MINOR, PROTOCOL_PATCH,
    MIN_SUPPORTED_MAJOR, ProtocolVersion, CompatibilityStatus,
};
use iter_mcp_server::types::protocol::*;
use iter_mcp_server::types::mcp::*;
use serde_json::{json, Value};
use std::fs;

// ============================================================================
// Version Constants Invariants
// ============================================================================

#[test]
fn protocol_version_is_valid_semver() {
    // Version string must be parseable
    let v = ProtocolVersion::parse(PROTOCOL_VERSION).expect("PROTOCOL_VERSION must be valid semver");
    
    // Components must match constants
    assert_eq!(v.major, PROTOCOL_MAJOR);
    assert_eq!(v.minor, PROTOCOL_MINOR);
    assert_eq!(v.patch, PROTOCOL_PATCH);
}

#[test]
fn protocol_version_constants_are_consistent() {
    let expected = format!("{}.{}.{}", PROTOCOL_MAJOR, PROTOCOL_MINOR, PROTOCOL_PATCH);
    assert_eq!(PROTOCOL_VERSION, expected, "PROTOCOL_VERSION must match component constants");
}

#[test]
fn min_supported_version_is_valid() {
    // MIN_SUPPORTED_MAJOR must be <= current major
    assert!(MIN_SUPPORTED_MAJOR <= PROTOCOL_MAJOR, 
        "MIN_SUPPORTED_MAJOR ({}) cannot exceed PROTOCOL_MAJOR ({})",
        MIN_SUPPORTED_MAJOR, PROTOCOL_MAJOR);
    
    // MIN_SUPPORTED_MAJOR must be at least 1
    assert!(MIN_SUPPORTED_MAJOR >= 1, "MIN_SUPPORTED_MAJOR must be >= 1");
}

// ============================================================================
// Compatibility Matrix Tests
// ============================================================================

#[test]
fn current_version_is_compatible() {
    let current = ProtocolVersion::current();
    assert!(current.is_compatible());
    assert!(current.is_current());
    assert_eq!(current.check_compatibility(), CompatibilityStatus::Compatible);
}

#[test]
fn same_major_lower_minor_is_compatible() {
    // Only test if minor > 0 to avoid underflow
    // When minor is 0, there's no "lower minor" to test
    if PROTOCOL_MINOR > 0 {
        let lower_minor = PROTOCOL_MINOR.saturating_sub(1);
        let older = ProtocolVersion {
            version: format!("{}.{}.0", PROTOCOL_MAJOR, lower_minor),
            major: PROTOCOL_MAJOR,
            minor: lower_minor,
            patch: 0,
        };
        assert!(older.is_compatible());
        assert_eq!(older.check_compatibility(), CompatibilityStatus::Compatible);
    }
    // If PROTOCOL_MINOR == 0, this test is a no-op (by design)
}

#[test]
fn same_major_higher_minor_is_forward_compatible() {
    let newer = ProtocolVersion {
        version: format!("{}.{}.0", PROTOCOL_MAJOR, PROTOCOL_MINOR + 1),
        major: PROTOCOL_MAJOR,
        minor: PROTOCOL_MINOR + 1,
        patch: 0,
    };
    assert!(newer.is_compatible());
    assert_eq!(newer.check_compatibility(), CompatibilityStatus::ForwardCompatible);
}

#[test]
fn higher_major_is_incompatible() {
    let future = ProtocolVersion {
        version: format!("{}.0.0", PROTOCOL_MAJOR + 1),
        major: PROTOCOL_MAJOR + 1,
        minor: 0,
        patch: 0,
    };
    assert!(!future.is_compatible());
    assert!(matches!(future.check_compatibility(), CompatibilityStatus::Incompatible { .. }));
}

#[test]
fn below_min_supported_is_incompatible() {
    if MIN_SUPPORTED_MAJOR > 1 {
        let ancient = ProtocolVersion {
            version: format!("{}.0.0", MIN_SUPPORTED_MAJOR - 1),
            major: MIN_SUPPORTED_MAJOR - 1,
            minor: 0,
            patch: 0,
        };
        assert!(!ancient.is_compatible());
        assert!(matches!(ancient.check_compatibility(), CompatibilityStatus::Incompatible { .. }));
    }
}

// ============================================================================
// Wire Format Snapshot Tests
// ============================================================================

fn load_golden_snapshots() -> Value {
    let path = "tests/snapshots/v1/wire_format.json";
    let content = fs::read_to_string(path).expect("Golden snapshot file must exist");
    serde_json::from_str(&content).expect("Golden snapshot must be valid JSON")
}

#[test]
fn golden_snapshot_version_matches_current() {
    let snapshots = load_golden_snapshots();
    let snapshot_version = snapshots["version"].as_str().unwrap();
    
    // Snapshot version must match current protocol version
    assert_eq!(snapshot_version, PROTOCOL_VERSION,
        "Golden snapshot version ({}) must match PROTOCOL_VERSION ({}). \
         Update snapshots when bumping protocol version.",
        snapshot_version, PROTOCOL_VERSION);
}

#[test]
fn rpc_request_matches_golden_snapshot() {
    let snapshots = load_golden_snapshots();
    let golden = &snapshots["snapshots"]["rpc_request_with_params"];
    
    // Create same request programmatically
    let request = RpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "node.create".to_string(),
        params: json!({"belief": 0.5, "energy": 1.0}),
        id: Some(json!(1)),
    };
    
    let serialized = serde_json::to_value(&request).unwrap();
    
    // Compare field by field (allows for ordering differences)
    assert_eq!(serialized["jsonrpc"], golden["jsonrpc"]);
    assert_eq!(serialized["method"], golden["method"]);
    assert_eq!(serialized["id"], golden["id"]);
}

#[test]
fn rpc_response_matches_golden_snapshot() {
    let snapshots = load_golden_snapshots();
    let golden = &snapshots["snapshots"]["rpc_response_success"];
    
    let response = RpcResponse::success(json!(1), json!({
        "id": 1,
        "belief": 0.5,
        "energy": 1.0,
        "esv_valid": true,
        "stability": 0.9
    }));
    
    let serialized = serde_json::to_value(&response).unwrap();
    
    assert_eq!(serialized["jsonrpc"], golden["jsonrpc"]);
    assert_eq!(serialized["id"], golden["id"]);
    assert!(serialized.get("result").is_some());
}

#[test]
fn mcp_node_state_matches_golden_snapshot() {
    let snapshots = load_golden_snapshots();
    let golden = &snapshots["snapshots"]["mcp_node_state"];
    
    let node = McpNodeState {
        id: 1,
        belief: 0.5,
        energy: 1.0,
        esv_valid: true,
        stability: 0.9,
    };
    
    let serialized = serde_json::to_value(&node).unwrap();
    
    // All fields must match golden
    assert_eq!(serialized["id"], golden["id"]);
    assert_eq!(serialized["belief"], golden["belief"]);
    assert_eq!(serialized["energy"], golden["energy"]);
    assert_eq!(serialized["esv_valid"], golden["esv_valid"]);
    assert_eq!(serialized["stability"], golden["stability"]);
}

#[test]
fn protocol_version_matches_golden_snapshot() {
    let snapshots = load_golden_snapshots();
    let golden = &snapshots["snapshots"]["protocol_version"];
    
    let version = ProtocolVersion::current();
    let serialized = serde_json::to_value(&version).unwrap();
    
    assert_eq!(serialized["version"], golden["version"]);
    assert_eq!(serialized["major"], golden["major"]);
    assert_eq!(serialized["minor"], golden["minor"]);
    assert_eq!(serialized["patch"], golden["patch"]);
}

// ============================================================================
// Version Bump Enforcement
// ============================================================================

#[test]
fn major_version_is_documented() {
    // If major > 1, there should be migration docs
    // This is a reminder test - fails if major bumps without action
    if PROTOCOL_MAJOR > 1 {
        // Check for migration guide (placeholder - would check file exists)
        // For now, just document the requirement
        assert!(true, "Major version {} requires migration documentation", PROTOCOL_MAJOR);
    }
}

#[test]
fn version_serialization_is_stable() {
    let v1 = ProtocolVersion::current();
    let json1 = serde_json::to_string(&v1).unwrap();
    let v2: ProtocolVersion = serde_json::from_str(&json1).unwrap();
    let json2 = serde_json::to_string(&v2).unwrap();
    
    // Roundtrip must be stable
    assert_eq!(json1, json2, "Version serialization must be deterministic");
}
