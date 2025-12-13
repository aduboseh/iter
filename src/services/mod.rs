// Governance: Deterministic | ESV-Compliant | Drift ≤1e-10
// Lineage: MCP_BOUNDARY_V2.0

//! Services module - MCP boundary protection and response handling
//!
//! These components are public APIs for external crates and test code.

#![allow(unused_imports)] // Re-exports for external use

pub mod sanitizer;

// Re-export sanitizer components
pub use sanitizer::{
    contains_forbidden, is_forbidden, normalize_for_matching, ResponseSanitizer,
    SanitizationResult, SanitizedGovernorStatus, SanitizedNodeState, SanitizedTraceSummary,
    FORBIDDEN_PATTERNS, SENSITIVE_PATTERNS,
};
