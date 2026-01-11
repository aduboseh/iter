//! ITER-PAR-01: Contract and Schema Module
//!
//! Typed contracts for SCG-CTX-03 + SCG-INT-04 governance.
//! Iter consumes only typed contracts (floats, enums, hashes).
//! Iter does NOT compute SCG signals.
//!
//! # Invariants
//!
//! - INV-ITER-01: Contract purity — no signal computation
//! - INV-ITER-02: Deterministic evaluation — identical input => identical output
//! - INV-ITER-03: Causal completeness — explicit cause for every rejection

pub mod envelopes;
pub mod validation;

pub use envelopes::{
    EnergyEnvelope, LearningEnvelope, LearningStatus, PolicyDecision, PolicyEnvelope,
    ReasoningEnvelope, SystemState,
};
pub use validation::{validate_bounded_float, validate_hash, ContractError};
