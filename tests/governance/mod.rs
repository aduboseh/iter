//! Governance Invariant Tests
//!
//! These tests enforce protocol stability and policy compliance.
//! All tests compile in public_stub mode (no substrate dependencies).
//!
//! # Test Categories
//!
//! - `schema_stability`: Protocol type shape invariants
//! - `error_taxonomy`: Error code completeness and stability
//! - `versioning`: Protocol version and compatibility rules
//! - `telemetry`: Audit event invariants and redaction guarantees
//!
//! # Governance Contract
//!
//! Failures in this module block merge. These are not unit tests—they are
//! invariant checks that enforce the public API contract.

pub mod schema_stability;
pub mod error_taxonomy;
pub mod telemetry;
pub mod versioning;
