//! Substrate interface layer.
//!
//! Provides the stub substrate for public_stub mode (demonstration).
//! The `full_substrate` feature name is reserved and intentionally unsupported here.

#[cfg(feature = "public_stub")]
pub mod stub;
