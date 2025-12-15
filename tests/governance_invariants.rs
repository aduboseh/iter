//! Governance Invariant Test Harness
//!
//! This test file runs all governance invariant checks.
//! It compiles in public_stub mode and blocks CI on failure.
//!
//! # What This Tests
//!
//! - Schema stability: Protocol types maintain stable shapes
//! - Error taxonomy: Error codes are exhaustive and documented
//!
//! # Adding New Invariants
//!
//! Add new test modules under tests/governance/ and include them here.

mod governance;
