//! MCP Type Definitions
//!
//! This module provides the public MCP type interface.
//!
//! # Architecture
//!
//! - `protocol`: JSON-RPC 2.0 wire types (always available)
//! - `mcp`: Sanitized MCP response types (always available)
//! - `version`: Protocol versioning and compatibility (always available)
//!
//! # Security
//!
//! MCP types expose ONLY sanitized views of substrate state.
//! Internal fields (topology, raw ESV, energy matrices) are never exposed.

pub mod mcp;
pub mod protocol;
pub mod version;

// Re-export protocol types
pub use protocol::{
    BindEdgeParams, CreateNodeParams, ExportLineageParams, MutateNodeParams, PropagateEdgeParams,
    QueryNodeParams, RpcError, RpcRequest, RpcResponse, ToolInfo, ToolList,
};

// Re-export MCP types
pub use mcp::{McpEdgeState, McpError, McpGovernorStatus, McpLineageEntry, McpNodeState};

// Re-export version types
pub use version::{
    CompatibilityStatus, Deprecation, ProtocolVersion, MIN_SUPPORTED_MAJOR, PROTOCOL_MAJOR,
    PROTOCOL_MINOR, PROTOCOL_PATCH, PROTOCOL_VERSION,
};
