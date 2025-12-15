//! MCP Type Definitions
//!
//! This module provides the public MCP type interface.
//!
//! # Architecture
//!
//! - `protocol`: JSON-RPC 2.0 wire types (always available)
//! - `mcp`: Sanitized MCP response types (always available)
//! - `version`: Protocol versioning and compatibility (always available)
//! - `substrate`: Substrate adapters (full_substrate only)
//!
//! # Security
//!
//! MCP types expose ONLY sanitized views of substrate state.
//! Internal fields (topology, raw ESV, energy matrices) are never exposed.

// Always-available modules
pub mod mcp;
pub mod protocol;
pub mod version;

// Substrate adapters (full_substrate only)
#[cfg(feature = "full_substrate")]
pub mod substrate;

// Re-export protocol types
pub use protocol::{
    RpcRequest, RpcResponse, RpcError,
    ToolInfo, ToolList,
    CreateNodeParams, MutateNodeParams, QueryNodeParams,
    BindEdgeParams, PropagateEdgeParams, ExportLineageParams,
};

// Re-export MCP types
pub use mcp::{
    McpError, McpNodeState, McpEdgeState, McpGovernorStatus, McpLineageEntry,
};

// Re-export version types
pub use version::{
    PROTOCOL_VERSION, PROTOCOL_MAJOR, PROTOCOL_MINOR, PROTOCOL_PATCH,
    MIN_SUPPORTED_MAJOR, ProtocolVersion, CompatibilityStatus, Deprecation,
};
