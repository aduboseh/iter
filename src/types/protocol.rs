//! JSON-RPC 2.0 Protocol Types
//!
//! These types are always available regardless of build mode.
//! They define the MCP wire protocol without any substrate dependencies.

use serde::{Deserialize, Serialize};

// ============================================================================
// JSON-RPC 2.0 Protocol Types
// ============================================================================

/// JSON-RPC 2.0 Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcRequest {
    /// Protocol version (always "2.0")
    pub jsonrpc: String,
    /// Method name
    pub method: String,
    /// Method parameters
    #[serde(default)]
    pub params: serde_json::Value,
    /// Request ID
    #[serde(default)]
    pub id: Option<serde_json::Value>,
}

/// JSON-RPC 2.0 Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcResponse {
    /// Protocol version (always "2.0")
    pub jsonrpc: String,
    /// Success result
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    /// Error result
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcError>,
    /// Request ID
    pub id: serde_json::Value,
}

/// JSON-RPC 2.0 Error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcError {
    /// Error code
    pub code: i32,
    /// Error message
    pub message: String,
}

impl RpcResponse {
    /// Create a success response
    pub fn success(id: serde_json::Value, value: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            result: Some(value),
            error: None,
            id,
        }
    }

    /// Create an error response
    pub fn error(id: serde_json::Value, code: i32, msg: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            result: None,
            error: Some(RpcError {
                code,
                message: msg.into(),
            }),
            id,
        }
    }
}

// ============================================================================
// MCP Tool Metadata Types
// ============================================================================

/// MCP Tool information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    /// Tool name
    pub name: String,
    /// Tool description
    pub description: String,
    /// Tool version
    pub version: String,
    /// JSON Schema for input
    #[serde(rename = "inputSchema")]
    pub input_schema: serde_json::Value,
}

/// List of available MCP tools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolList {
    /// Available tools
    pub tools: Vec<ToolInfo>,
}

// ============================================================================
// MCP Request Parameter Types
// ============================================================================

/// Parameters for node.create
#[derive(Debug, Clone, Deserialize)]
pub struct CreateNodeParams {
    /// Initial belief value
    pub belief: f64,
    /// Initial energy value
    pub energy: f64,
}

/// Parameters for node.mutate
#[derive(Debug, Clone, Deserialize)]
pub struct MutateNodeParams {
    /// Target node ID
    pub node_id: u64,
    /// Belief delta
    pub delta: f64,
}

/// Parameters for node.query
#[derive(Debug, Clone, Deserialize)]
pub struct QueryNodeParams {
    /// Target node ID
    pub node_id: u64,
}

/// Parameters for edge.bind
#[derive(Debug, Clone, Deserialize)]
pub struct BindEdgeParams {
    /// Source node ID
    pub src: u64,
    /// Destination node ID
    pub dst: u64,
    /// Edge weight
    pub weight: f64,
}

/// Parameters for edge.propagate
#[derive(Debug, Clone, Deserialize)]
pub struct PropagateEdgeParams {
    /// Target edge ID
    pub edge_id: u64,
}

/// Parameters for lineage.export
#[derive(Debug, Clone, Deserialize)]
pub struct ExportLineageParams {
    /// Export file path
    pub path: String,
}
