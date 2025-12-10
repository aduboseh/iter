//! Metrics Abstraction for MCP Server
//!
//! Provides a thin abstraction layer for metrics that can be wired to
//! Prometheus, OpenTelemetry, or other backends later.
//!
//! Currently implemented as no-op stubs with tracing fallback.

use std::sync::atomic::{AtomicU64, Ordering};
use tracing::{debug, warn};

use crate::types::McpError;

// ============================================================================
// Global Counters (Atomic for thread-safety)
// ============================================================================

static REQUESTS_TOTAL: AtomicU64 = AtomicU64::new(0);
static REQUESTS_SUCCESS: AtomicU64 = AtomicU64::new(0);
static REQUESTS_FAILED: AtomicU64 = AtomicU64::new(0);
static INVARIANT_FAILURES: AtomicU64 = AtomicU64::new(0);

// ============================================================================
// Metrics API
// ============================================================================

/// Observe a completed request.
/// 
/// Called by handlers after processing completes.
/// Logs to tracing and updates internal counters.
pub fn observe_request<T>(tool: &str, duration_ms: u64, result: &Result<T, McpError>) {
    REQUESTS_TOTAL.fetch_add(1, Ordering::Relaxed);
    
    match result {
        Ok(_) => {
            REQUESTS_SUCCESS.fetch_add(1, Ordering::Relaxed);
            debug!(
                tool = tool,
                duration_ms = duration_ms,
                outcome = "success",
                "MCP request completed"
            );
        }
        Err(err) => {
            REQUESTS_FAILED.fetch_add(1, Ordering::Relaxed);
            debug!(
                tool = tool,
                duration_ms = duration_ms,
                outcome = "error",
                error_code = err.code(),
                error_type = err.code_string(),
                "MCP request failed"
            );
        }
    }
}

/// Increment invariant failure counter.
/// 
/// Called when a governance or substrate invariant is violated.
pub fn incr_invariant_failure(kind: &str) {
    INVARIANT_FAILURES.fetch_add(1, Ordering::Relaxed);
    warn!(kind = kind, "Invariant failure detected");
}

/// Get current metrics snapshot.
/// 
/// Returns a snapshot of all tracked metrics.
pub fn snapshot() -> MetricsSnapshot {
    MetricsSnapshot {
        requests_total: REQUESTS_TOTAL.load(Ordering::Relaxed),
        requests_success: REQUESTS_SUCCESS.load(Ordering::Relaxed),
        requests_failed: REQUESTS_FAILED.load(Ordering::Relaxed),
        invariant_failures: INVARIANT_FAILURES.load(Ordering::Relaxed),
    }
}

/// Reset all metrics (for testing).
#[cfg(test)]
pub fn reset() {
    REQUESTS_TOTAL.store(0, Ordering::Relaxed);
    REQUESTS_SUCCESS.store(0, Ordering::Relaxed);
    REQUESTS_FAILED.store(0, Ordering::Relaxed);
    INVARIANT_FAILURES.store(0, Ordering::Relaxed);
}

// ============================================================================
// Metrics Types
// ============================================================================

/// Snapshot of current metrics values.
#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub requests_total: u64,
    pub requests_success: u64,
    pub requests_failed: u64,
    pub invariant_failures: u64,
}

impl MetricsSnapshot {
    /// Calculate success rate as percentage (0.0 - 100.0).
    pub fn success_rate(&self) -> f64 {
        if self.requests_total == 0 {
            100.0
        } else {
            (self.requests_success as f64 / self.requests_total as f64) * 100.0
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_observe_success() {
        reset();
        let result: Result<(), McpError> = Ok(());
        observe_request("test.tool", 10, &result);
        
        let snap = snapshot();
        assert_eq!(snap.requests_total, 1);
        assert_eq!(snap.requests_success, 1);
        assert_eq!(snap.requests_failed, 0);
    }

    #[test]
    fn test_observe_failure() {
        reset();
        let result: Result<(), McpError> = Err(McpError::BadRequest { message: "test".into() });
        observe_request("test.tool", 10, &result);
        
        let snap = snapshot();
        assert_eq!(snap.requests_total, 1);
        assert_eq!(snap.requests_success, 0);
        assert_eq!(snap.requests_failed, 1);
    }

    #[test]
    fn test_invariant_failure() {
        reset();
        incr_invariant_failure("drift_exceeded");
        
        let snap = snapshot();
        assert_eq!(snap.invariant_failures, 1);
    }

    #[test]
    fn test_success_rate() {
        reset();
        observe_request("test", 1, &Ok::<(), McpError>(()));
        observe_request("test", 1, &Ok::<(), McpError>(()));
        observe_request("test", 1, &Err::<(), McpError>(McpError::BadRequest { message: "".into() }));
        
        let snap = snapshot();
        assert!((snap.success_rate() - 66.666).abs() < 0.01);
    }
}
