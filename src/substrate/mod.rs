//! Substrate interface layer.
//!
//! Conditionally imports either stub or full substrate based on feature flags.
//!
//! # Build Modes
//!
//! - `public_stub` feature: Uses stub substrate for demonstration
//! - `full_substrate` feature: Uses real proprietary substrate (requires workspace access)

#[cfg(feature = "public_stub")]
pub mod stub;

#[cfg(feature = "public_stub")]
pub use stub::*;

// Full substrate is handled by existing substrate_runtime module
// which imports from scg-* crates when full_substrate feature is enabled
