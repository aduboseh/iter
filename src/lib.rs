//! SCG MCP Server - Library Interface
//!
//! Secure MCP boundary between AI assistants and the SCG cognitive substrate.
//!
//! # Architecture
//!
//! ```text
//! AI Assistant -> MCP Protocol -> scg_mcp_server -> SCG Substrate
//!                                    |
//!                             Response Sanitizer
//!                                    |
//!                             Safe JSON Output
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
// Core Modules
// ============================================================================

pub mod caller_context;
pub mod governance;
pub mod mcp_handler;
pub mod metrics;
pub mod services;
pub mod substrate_runtime;
pub mod traits;
pub mod types;
pub mod validation;

// ============================================================================
// MCP Type Re-exports (Sanitized for external use)
// ============================================================================

pub use types::{
    McpNodeState, McpEdgeState, McpGovernorStatus, McpLineageEntry,
    McpError, RpcRequest, RpcResponse, RpcError,
    CreateNodeParams, MutateNodeParams, QueryNodeParams,
    BindEdgeParams, PropagateEdgeParams, ExportLineageParams,
    ToolInfo, ToolList,
};

// ============================================================================
// Boundary Traits (Public API for substrate interaction)
// ============================================================================

pub use traits::{SubstrateNodeView, SubstrateEdgeView, SubstrateGovernorView};

// ============================================================================
// Substrate Runtime (Public facade only)
// ============================================================================

pub use substrate_runtime::{SubstrateRuntime, SubstrateRuntimeConfig, SharedSubstrateRuntime};
