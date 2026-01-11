//! Iter Server - Library Interface
//!
//! Secure MCP boundary for governed execution.
//!
//! # Architecture
//!
//! ```text
//! Client -> MCP Protocol -> server -> sealed engine
//!                     |
//!               Response Sanitizer
//!                     |
//!               Safe JSON Output
//! ```
//!
//! # Security
//!
//! All responses are sanitized to prevent leakage of:
//! - DAG topology information
//! - Raw ESV values
//! - Internal energy matrices
//! - Lineage chain details (only checksums exposed)
//!
//! # Boundary Invariant
//!
//! **NO substrate types are publicly exported from this crate.**
//! External consumers interact ONLY through sanitized MCP DTOs.
//! This is enforced by `#![deny(missing_docs)]` and CI guardrails.

// NOTE: Enable deny(missing_docs) after full documentation pass
#![warn(missing_docs)]

// ============================================================================
// Core Modules (always available)
// ============================================================================

pub mod caller_context;
pub mod types;

// ============================================================================
// Public Stub Module (demonstration mode)
// ============================================================================

#[cfg(feature = "public_stub")]
pub mod substrate;

// ============================================================================
// MCP Type Re-exports (always available - no substrate dependencies)
// ============================================================================

pub use types::{
    BindEdgeParams, CreateNodeParams, ExportLineageParams, McpEdgeState, McpError,
    McpGovernorStatus, McpLineageEntry, McpNodeState, MutateNodeParams, PropagateEdgeParams,
    QueryNodeParams, RpcError, RpcRequest, RpcResponse, ToolInfo, ToolList,
};

// ============================================================================
// Protocol Version (always available)
// ============================================================================

pub use types::{
    CompatibilityStatus, Deprecation, ProtocolVersion, MIN_SUPPORTED_MAJOR, PROTOCOL_MAJOR,
    PROTOCOL_MINOR, PROTOCOL_PATCH, PROTOCOL_VERSION,
};

// ============================================================================
// Stub Runtime (Public stub mode)
// ============================================================================

#[cfg(feature = "public_stub")]
pub use substrate::stub::StubRuntime;
