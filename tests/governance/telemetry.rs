//! Telemetry & Audit Governance Tests
//!
//! These tests enforce audit event invariants and redaction guarantees.
//!
//! # Governance Contract
//!
//! - Audit events contain only allowlisted fields
//! - Denied fields never appear in serialized events
//! - Every event carries trace context and protocol version
//! - Events serialize to valid JSON Lines format

use iter_mcp_server::{
    TraceContext, AuditPhase, AuditOutcome, AuditEvent,
    AUDIT_ALLOWLIST, AUDIT_DENYLIST, is_field_allowed, is_field_denied,
    PROTOCOL_VERSION,
};
use serde_json::Value;

// ============================================================================
// Allowlist/Denylist Invariants
// ============================================================================

#[test]
fn allowlist_is_non_empty() {
    assert!(!AUDIT_ALLOWLIST.is_empty(), "Audit allowlist must not be empty");
}

#[test]
fn denylist_is_non_empty() {
    assert!(!AUDIT_DENYLIST.is_empty(), "Audit denylist must not be empty");
}

#[test]
fn allowlist_and_denylist_are_disjoint() {
    for allowed in AUDIT_ALLOWLIST {
        assert!(
            !AUDIT_DENYLIST.contains(allowed),
            "Field '{}' appears in both allowlist and denylist",
            allowed
        );
    }
}

#[test]
fn substrate_internals_are_denied() {
    let substrate_fields = ["dag_topology", "esv_raw", "energy_matrix", "lineage_hash_chain"];
    for field in substrate_fields {
        assert!(
            is_field_denied(field),
            "Substrate internal field '{}' must be in denylist",
            field
        );
    }
}

#[test]
fn payload_fields_are_denied() {
    let payload_fields = ["params", "result", "payload", "body"];
    for field in payload_fields {
        assert!(
            is_field_denied(field),
            "Payload field '{}' must be in denylist",
            field
        );
    }
}

#[test]
fn pii_fields_are_denied() {
    let pii_fields = ["user_id", "session_id", "ip_address", "credentials"];
    for field in pii_fields {
        assert!(
            is_field_denied(field),
            "PII field '{}' must be in denylist",
            field
        );
    }
}

#[test]
fn trace_fields_are_allowed() {
    let trace_fields = ["trace_id", "span_id", "parent_span_id"];
    for field in trace_fields {
        assert!(
            is_field_allowed(field),
            "Trace field '{}' must be in allowlist",
            field
        );
    }
}

#[test]
fn metadata_fields_are_allowed() {
    let metadata_fields = ["timestamp", "phase", "outcome", "protocol_version", "method"];
    for field in metadata_fields {
        assert!(
            is_field_allowed(field),
            "Metadata field '{}' must be in allowlist",
            field
        );
    }
}

// ============================================================================
// Audit Event Structure Invariants
// ============================================================================

#[test]
fn audit_event_has_required_fields() {
    let trace = TraceContext::new_root("test-trace");
    let event = AuditEvent::new(trace, AuditPhase::Received, "node.query");
    
    let json: Value = serde_json::to_value(&event).unwrap();
    
    // Required fields must be present
    assert!(json.get("timestamp").is_some(), "timestamp is required");
    assert!(json.get("trace").is_some(), "trace is required");
    assert!(json.get("phase").is_some(), "phase is required");
    assert!(json.get("protocol_version").is_some(), "protocol_version is required");
    assert!(json.get("method").is_some(), "method is required");
}

#[test]
fn audit_event_trace_has_required_fields() {
    let trace = TraceContext::new_root("test-trace");
    let event = AuditEvent::new(trace, AuditPhase::Received, "test");
    
    let json: Value = serde_json::to_value(&event).unwrap();
    let trace_json = &json["trace"];
    
    assert!(trace_json.get("trace_id").is_some(), "trace.trace_id is required");
    assert!(trace_json.get("span_id").is_some(), "trace.span_id is required");
}

#[test]
fn audit_event_includes_protocol_version() {
    let trace = TraceContext::new_root("t1");
    let event = AuditEvent::new(trace, AuditPhase::Received, "test");
    
    assert_eq!(event.protocol_version, PROTOCOL_VERSION);
}

#[test]
fn audit_event_optional_fields_are_skipped_when_none() {
    let trace = TraceContext::new_root("t1");
    let event = AuditEvent::new(trace, AuditPhase::Received, "test");
    
    let json: Value = serde_json::to_value(&event).unwrap();
    
    // Optional fields should not appear when None
    assert!(json.get("outcome").is_none(), "outcome should be skipped when None");
    assert!(json.get("request_id").is_none(), "request_id should be skipped when None");
    assert!(json.get("duration_us").is_none(), "duration_us should be skipped when None");
    assert!(json.get("error_code").is_none(), "error_code should be skipped when None");
}

#[test]
fn audit_event_optional_fields_appear_when_set() {
    let trace = TraceContext::new_root("t1");
    let event = AuditEvent::new(trace, AuditPhase::Responded, "test")
        .with_outcome(AuditOutcome::Success)
        .with_request_id("req-123")
        .with_duration_us(5000);
    
    let json: Value = serde_json::to_value(&event).unwrap();
    
    assert!(json.get("outcome").is_some());
    assert!(json.get("request_id").is_some());
    assert!(json.get("duration_us").is_some());
}

// ============================================================================
// Serialization Format Invariants
// ============================================================================

#[test]
fn audit_event_serializes_to_single_line() {
    let trace = TraceContext::new_root("t1");
    let event = AuditEvent::new(trace, AuditPhase::Responded, "tools/list")
        .with_outcome(AuditOutcome::Success);
    
    let jsonl = event.to_jsonl().unwrap();
    
    assert!(!jsonl.contains('\n'), "JSON Lines format must be single line");
    assert!(!jsonl.contains('\r'), "JSON Lines format must not contain CR");
}

#[test]
fn audit_event_roundtrip_is_stable() {
    let trace = TraceContext::new_root("trace-abc");
    let event = AuditEvent::new(trace, AuditPhase::Executed, "node.create")
        .with_request_id("req-1")
        .with_outcome(AuditOutcome::Success)
        .with_duration_us(1234);
    
    let json1 = event.to_jsonl().unwrap();
    let parsed: AuditEvent = serde_json::from_str(&json1).unwrap();
    let json2 = parsed.to_jsonl().unwrap();
    
    // Parse and compare as Value to handle field ordering
    let v1: Value = serde_json::from_str(&json1).unwrap();
    let v2: Value = serde_json::from_str(&json2).unwrap();
    
    assert_eq!(v1["trace"]["trace_id"], v2["trace"]["trace_id"]);
    assert_eq!(v1["method"], v2["method"]);
    assert_eq!(v1["phase"], v2["phase"]);
}

#[test]
fn audit_phase_serializes_to_snake_case() {
    let phases = [
        (AuditPhase::Received, "received"),
        (AuditPhase::Validated, "validated"),
        (AuditPhase::Executed, "executed"),
        (AuditPhase::Responded, "responded"),
        (AuditPhase::Error, "error"),
    ];
    
    for (phase, expected) in phases {
        let json = serde_json::to_string(&phase).unwrap();
        assert_eq!(json, format!("\"{}\"", expected));
    }
}

#[test]
fn audit_outcome_serializes_to_snake_case() {
    let outcomes = [
        (AuditOutcome::Success, "success"),
        (AuditOutcome::Failure, "failure"),
        (AuditOutcome::Rejected, "rejected"),
        (AuditOutcome::Timeout, "timeout"),
    ];
    
    for (outcome, expected) in outcomes {
        let json = serde_json::to_string(&outcome).unwrap();
        assert_eq!(json, format!("\"{}\"", expected));
    }
}

// ============================================================================
// Redaction Guarantee Tests
// ============================================================================

#[test]
fn serialized_event_contains_no_denied_fields() {
    let trace = TraceContext::new_root("t1");
    let event = AuditEvent::new(trace, AuditPhase::Responded, "test")
        .with_outcome(AuditOutcome::Success)
        .with_duration_us(100)
        .with_error(4004, "node_not_found");
    
    let json = event.to_jsonl().unwrap();
    
    for denied in AUDIT_DENYLIST {
        assert!(
            !json.contains(&format!("\"{}\"", denied)),
            "Denied field '{}' found in serialized event",
            denied
        );
    }
}

#[test]
fn error_events_contain_category_not_message() {
    let trace = TraceContext::new_root("t1");
    let event = AuditEvent::new(trace, AuditPhase::Error, "node.query")
        .with_outcome(AuditOutcome::Failure)
        .with_error(4004, "node_not_found");
    
    let json: Value = serde_json::to_value(&event).unwrap();
    
    // Should have error_code and error_category
    assert!(json.get("error_code").is_some());
    assert!(json.get("error_category").is_some());
    
    // Should NOT have error message or details
    assert!(json.get("error_message").is_none());
    assert!(json.get("error_details").is_none());
}

// ============================================================================
// Trace Context Invariants
// ============================================================================

#[test]
fn trace_context_root_has_matching_ids() {
    let trace = TraceContext::new_root("abc-123");
    
    assert_eq!(trace.trace_id, "abc-123");
    assert_eq!(trace.span_id, "abc-123"); // Root span_id equals trace_id
    assert!(trace.parent_span_id.is_none());
}

#[test]
fn trace_context_child_preserves_trace_id() {
    let root = TraceContext::new_root("trace-main");
    let child = root.child("span-child");
    
    assert_eq!(child.trace_id, "trace-main"); // Same trace
    assert_eq!(child.span_id, "span-child"); // New span
    assert_eq!(child.parent_span_id, Some("trace-main".to_string()));
}

#[test]
fn trace_context_grandchild_chain() {
    let root = TraceContext::new_root("t");
    let child = root.child("s1");
    let grandchild = child.child("s2");
    
    assert_eq!(grandchild.trace_id, "t");
    assert_eq!(grandchild.span_id, "s2");
    assert_eq!(grandchild.parent_span_id, Some("s1".to_string()));
}
