//! Audit Event Types for Telemetry
//!
//! Defines structured audit events for request lifecycle tracking.
//! All events are deterministic, redacted-by-default, and substrate-agnostic.
//!
//! # Design Principles
//!
//! - **Deterministic**: Same input produces same event structure
//! - **Redacted-by-default**: Only allowlisted fields are emitted
//! - **Traceable**: Every event carries trace_id and span_id
//! - **Auditable**: Events form an immutable chain for compliance
//!
//! # Event Lifecycle
//!
//! ```text
//! Request → Received → Validated → Executed → Responded
//!              ↓           ↓           ↓           ↓
//!           AuditEvent  AuditEvent  AuditEvent  AuditEvent
//! ```

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::types::version::PROTOCOL_VERSION;

// ============================================================================
// Trace Context
// ============================================================================

/// Trace context for correlating events across systems
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TraceContext {
    /// Unique trace identifier (UUID format recommended)
    pub trace_id: String,
    /// Span identifier within the trace
    pub span_id: String,
    /// Parent span identifier (None for root spans)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_span_id: Option<String>,
}

impl TraceContext {
    /// Create a new root trace context
    pub fn new_root(trace_id: impl Into<String>) -> Self {
        let trace = trace_id.into();
        Self {
            trace_id: trace.clone(),
            span_id: trace,
            parent_span_id: None,
        }
    }

    /// Create a child span
    pub fn child(&self, span_id: impl Into<String>) -> Self {
        Self {
            trace_id: self.trace_id.clone(),
            span_id: span_id.into(),
            parent_span_id: Some(self.span_id.clone()),
        }
    }
}

// ============================================================================
// Audit Event Types
// ============================================================================

/// Audit event lifecycle phase
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuditPhase {
    /// Request received (before validation)
    Received,
    /// Request validated (schema/auth checks passed)
    Validated,
    /// Request executed (handler invoked)
    Executed,
    /// Response sent
    Responded,
    /// Error occurred
    Error,
}

/// Audit event outcome
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuditOutcome {
    /// Operation succeeded
    Success,
    /// Operation failed (expected error)
    Failure,
    /// Operation rejected (validation/auth)
    Rejected,
    /// Operation timed out
    Timeout,
}

/// Structured audit event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Event timestamp (UTC)
    pub timestamp: DateTime<Utc>,
    /// Trace context for correlation
    pub trace: TraceContext,
    /// Lifecycle phase
    pub phase: AuditPhase,
    /// Event outcome (set on terminal phases)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outcome: Option<AuditOutcome>,
    /// Protocol version
    pub protocol_version: String,
    /// Method name (redacted-safe)
    pub method: String,
    /// Request ID (from JSON-RPC)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    /// Duration in microseconds (set on terminal phases)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_us: Option<u64>,
    /// Error code (set on error phases)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<i32>,
    /// Error category (set on error phases, no message details)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_category: Option<String>,
}

impl AuditEvent {
    /// Create a new audit event
    pub fn new(trace: TraceContext, phase: AuditPhase, method: impl Into<String>) -> Self {
        Self {
            timestamp: Utc::now(),
            trace,
            phase,
            outcome: None,
            protocol_version: PROTOCOL_VERSION.to_string(),
            method: method.into(),
            request_id: None,
            duration_us: None,
            error_code: None,
            error_category: None,
        }
    }

    /// Set the request ID
    pub fn with_request_id(mut self, id: impl Into<String>) -> Self {
        self.request_id = Some(id.into());
        self
    }

    /// Set the outcome
    pub fn with_outcome(mut self, outcome: AuditOutcome) -> Self {
        self.outcome = Some(outcome);
        self
    }

    /// Set the duration
    pub fn with_duration_us(mut self, duration: u64) -> Self {
        self.duration_us = Some(duration);
        self
    }

    /// Set error information
    pub fn with_error(mut self, code: i32, category: impl Into<String>) -> Self {
        self.error_code = Some(code);
        self.error_category = Some(category.into());
        self
    }
}

// ============================================================================
// Allowlisted Fields (Redaction Policy)
// ============================================================================

/// Fields that are safe to emit in audit events
/// 
/// This is an explicit allowlist. Any field not listed here
/// MUST NOT appear in audit events.
pub const AUDIT_ALLOWLIST: &[&str] = &[
    // Trace context
    "trace_id",
    "span_id",
    "parent_span_id",
    // Event metadata
    "timestamp",
    "phase",
    "outcome",
    "protocol_version",
    // Request identification (no payload)
    "method",
    "request_id",
    // Performance
    "duration_us",
    // Error classification (no details)
    "error_code",
    "error_category",
];

/// Fields that are explicitly forbidden in audit events
pub const AUDIT_DENYLIST: &[&str] = &[
    // Request/response payloads
    "params",
    "result",
    "payload",
    "body",
    // Internal state
    "belief",
    "energy",
    "esv",
    "drift",
    "topology",
    "adjacency",
    // Substrate internals
    "dag_topology",
    "esv_raw",
    "energy_matrix",
    "lineage_hash_chain",
    "internal_state",
    // User data
    "user_id",
    "session_id",
    "ip_address",
    "credentials",
];

/// Check if a field is allowed in audit events
pub fn is_field_allowed(field: &str) -> bool {
    AUDIT_ALLOWLIST.contains(&field)
}

/// Check if a field is explicitly denied in audit events
pub fn is_field_denied(field: &str) -> bool {
    AUDIT_DENYLIST.contains(&field)
}

// ============================================================================
// Audit Event Serialization (JSON Lines compatible)
// ============================================================================

impl AuditEvent {
    /// Serialize to JSON Lines format (single line, no trailing newline)
    pub fn to_jsonl(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audit_event_creation() {
        let trace = TraceContext::new_root("trace-123");
        let event = AuditEvent::new(trace, AuditPhase::Received, "node.create")
            .with_request_id("req-1");
        
        assert_eq!(event.phase, AuditPhase::Received);
        assert_eq!(event.method, "node.create");
        assert_eq!(event.request_id, Some("req-1".to_string()));
    }

    #[test]
    fn trace_context_child() {
        let root = TraceContext::new_root("trace-abc");
        let child = root.child("span-xyz");
        
        assert_eq!(child.trace_id, "trace-abc");
        assert_eq!(child.span_id, "span-xyz");
        assert_eq!(child.parent_span_id, Some("trace-abc".to_string()));
    }

    #[test]
    fn audit_event_serializes_to_jsonl() {
        let trace = TraceContext::new_root("t1");
        let event = AuditEvent::new(trace, AuditPhase::Responded, "tools/list")
            .with_outcome(AuditOutcome::Success)
            .with_duration_us(1500);
        
        let json = event.to_jsonl().unwrap();
        
        // Should be single line
        assert!(!json.contains('\n'));
        // Should contain expected fields
        assert!(json.contains("\"phase\":\"responded\""));
        assert!(json.contains("\"outcome\":\"success\""));
    }

    #[test]
    fn allowlist_contains_expected_fields() {
        assert!(is_field_allowed("trace_id"));
        assert!(is_field_allowed("method"));
        assert!(is_field_allowed("duration_us"));
    }

    #[test]
    fn denylist_blocks_sensitive_fields() {
        assert!(is_field_denied("params"));
        assert!(is_field_denied("esv_raw"));
        assert!(is_field_denied("credentials"));
    }

    #[test]
    fn protocol_version_included() {
        let trace = TraceContext::new_root("t1");
        let event = AuditEvent::new(trace, AuditPhase::Received, "test");
        
        assert_eq!(event.protocol_version, PROTOCOL_VERSION);
    }
}
