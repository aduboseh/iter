//! Compile-time Iter/SCG provenance exported by build.rs.
//!
//! Downstream crates and tools should read these constants instead of
//! duplicating contract-critical literals.

/// SCG governance bridge contract version.
pub const CONTRACT_VERSION: &str = env!("ITER_SCG_CONTRACT_VERSION");
/// Exact SCG source commit used as the vendored bridge origin.
pub const SCG_SOURCE_COMMIT: &str = env!("ITER_SCG_SOURCE_COMMIT");
/// Governed SCG master head at vendor acceptance time.
pub const SCG_VENDOR_MASTER_HEAD: &str = env!("ITER_SCG_VENDOR_MASTER_HEAD");
/// SHA-256 of vendored contract.rs.
pub const BRIDGE_CONTRACT_RS_SHA256: &str = env!("ITER_BRIDGE_CONTRACT_RS_SHA256");
/// SHA-256 of vendored trace.rs.
pub const BRIDGE_TRACE_RS_SHA256: &str = env!("ITER_BRIDGE_TRACE_RS_SHA256");
/// SHA-256 of vendored errors.rs.
pub const BRIDGE_ERRORS_RS_SHA256: &str = env!("ITER_BRIDGE_ERRORS_RS_SHA256");
/// SHA-256 of vendored lib.rs.
pub const BRIDGE_LIB_RS_SHA256: &str = env!("ITER_BRIDGE_LIB_RS_SHA256");
/// Raw-byte SHA-256 of CANONICAL_VECTORS.json.
pub const CANONICAL_VECTORS_SHA256: &str = env!("ITER_CANONICAL_VECTORS_SHA256");
/// Canonicalization rule bound to the vendored bridge.
pub const CANONICALIZATION_RULE: &str = env!("ITER_CANONICALIZATION_RULE");
/// Target triple used to build this crate.
pub const TARGET_TRIPLE: &str = env!("ITER_TARGET_TRIPLE");
/// rustc version used to build this crate.
pub const RUSTC_VERSION: &str = env!("ITER_RUSTC_VERSION");
/// Replay scope claimed by current proof packets.
pub const REPLAY_SCOPE: &str = "same_binary_only";
/// Cross-platform replay is explicitly not claimed.
pub const CROSS_PLATFORM_REPLAY_CLAIMED: bool = false;
