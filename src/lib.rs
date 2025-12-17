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
// Full Substrate Modules (require proprietary substrate)
// ============================================================================

#[cfg(feature = "full_substrate")]
pub mod governance;
#[cfg(feature = "full_substrate")]
pub mod metrics;
#[cfg(feature = "full_substrate")]
pub mod mcp_handler;
#[cfg(feature = "full_substrate")]
pub mod services;
#[cfg(feature = "full_substrate")]
pub mod substrate_runtime;
#[cfg(feature = "full_substrate")]
pub mod traits;
#[cfg(feature = "full_substrate")]
pub mod validation;

// ============================================================================
// Public Stub Module (demonstration mode)
// ============================================================================

#[cfg(feature = "public_stub")]
pub mod substrate;

// ============================================================================
// MCP Type Re-exports (always available - no substrate dependencies)
// ============================================================================

pub use types::{
    McpNodeState, McpEdgeState, McpGovernorStatus, McpLineageEntry,
    McpError, RpcRequest, RpcResponse, RpcError,
    CreateNodeParams, MutateNodeParams, QueryNodeParams,
    BindEdgeParams, PropagateEdgeParams, ExportLineageParams,
    ToolInfo, ToolList,
};

// ============================================================================
// Protocol Version (always available)
// ============================================================================

pub use types::{
    PROTOCOL_VERSION, PROTOCOL_MAJOR, PROTOCOL_MINOR, PROTOCOL_PATCH,
    MIN_SUPPORTED_MAJOR, ProtocolVersion, CompatibilityStatus, Deprecation,
};

// ============================================================================
// Audit & Telemetry (always available)
// ============================================================================

pub use types::{
    TraceContext, AuditPhase, AuditOutcome, AuditEvent,
    AUDIT_ALLOWLIST, AUDIT_DENYLIST, is_field_allowed, is_field_denied,
};

// ============================================================================
// Boundary Traits (Public API for substrate interaction - full substrate only)
// ============================================================================

#[cfg(feature = "full_substrate")]
pub use traits::{SubstrateNodeView, SubstrateEdgeView, SubstrateGovernorView};

// ============================================================================
// Substrate Runtime (Public facade only - full substrate mode)
// ============================================================================

#[cfg(feature = "full_substrate")]
pub use substrate_runtime::{SubstrateRuntime, SubstrateRuntimeConfig, SharedSubstrateRuntime};

// ============================================================================
// Stub Runtime (Public stub mode)
// ============================================================================

#[cfg(feature = "public_stub")]
pub use substrate::stub::StubRuntime;
