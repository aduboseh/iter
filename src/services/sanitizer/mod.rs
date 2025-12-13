// Governance: Deterministic | ESV-Compliant | Drift ≤1e-10
// Lineage: MCP_BOUNDARY_V2.0

//! Sanitizer Module - MCP Boundary Protection
//!
//! Ensures no internal engine internals leak through the MCP boundary.
//!
//! Many items are public API for test code and external crates.

#![allow(dead_code)]

pub mod forbidden;
pub mod response;

// Re-export commonly used items
pub use forbidden::{
    contains_forbidden, is_forbidden, normalize_for_matching, FORBIDDEN_PATTERNS,
    SENSITIVE_PATTERNS,
};
pub use response::{
    ResponseSanitizer, SanitizationResult, SanitizedGovernorStatus, SanitizedNodeState,
    SanitizedTraceSummary,
};
