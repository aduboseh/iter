//! MCP Type Definitions and Engine Adapters
//!
//! This module defines:
//! 1. JSON-RPC 2.0 request/response types for MCP protocol
//! 2. Sanitized MCP response types (safe for external exposure)
//! 3. Adapters from internal engine types to MCP types
//!
//! # Security
//!
//! MCP types expose ONLY sanitized views of substrate state.
//! Internal fields (topology, raw ESV, energy matrices) are never exposed.
//!
//! # Boundary Invariant
//!
//! All substrate types are imported as `pub(crate)` - they are NOT
//! re-exported to external consumers. Only MCP DTOs cross the boundary.

#![allow(dead_code)]
#![allow(unused_imports)] // Substrate types used conditionally across modules

use serde::{Deserialize, Serialize};
use std::fmt;

// ============================================================================
// Engine Types (INTERNAL USE ONLY - not re-exported)
// ============================================================================

pub(crate) use scg_sim::{NodeId, EdgeId, NodeState as SubstrateNodeState, Edge as SubstrateEdge};
pub(crate) use scg_sim::{IntegratedSimulation, IntegratedConfig, SimError};
pub(crate) use scg_governance::{GovernanceValidator, GovernanceStatus as SubstrateGovernanceStatus, DRIFT_EPSILON};
pub(crate) use scg_trace::{CausalTrace, CausalEvent};
pub(crate) use scg_energy::{EnergyLedger, EnergyConfig};
pub(crate) use scg_ethics::{EthicsKernel, EthicsDecision};

// ============================================================================
// MCP Error Types
// ============================================================================

/// MCP Error codes aligned with JSON-RPC 2.0 and the engine boundary
/// 
/// Each variant has:
/// - A stable numeric code (for machines)
/// - A stable string code (for humans/docs)
/// - Optional details field for diagnostic info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum McpError {
    /// Node not found by ID (code 4004, "node_not_found")
    NodeNotFound { id: u64 },
    /// Edge not found by ID (code 4004, "edge_not_found")
    EdgeNotFound { id: u64 },
    /// ESV validation failed (code 1000, "esv_validation_failed")
    EsvValidationFailed { reason: String },
    /// Thermodynamic drift exceeded (code 2000, "drift_exceeded")
    DriftExceeded { drift: f64, threshold: f64 },
    /// Lineage integrity violation (code 3000, "lineage_corruption")
    LineageCorruption { details: String },
    /// Generic substrate error (code 5000, "substrate_error")
    SubstrateError { message: String },
    /// Invalid request parameters (code 4000, "bad_request")
    BadRequest { message: String },
}

impl fmt::Display for McpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            McpError::NodeNotFound { id } => write!(f, "Node not found: N{}", id),
            McpError::EdgeNotFound { id } => write!(f, "Edge not found: E{}", id),
            McpError::EsvValidationFailed { reason } => write!(f, "ESV validation failed: {}", reason),
            McpError::DriftExceeded { drift, threshold } => {
                write!(f, "Drift exceeded: {} > {}", drift, threshold)
            }
            McpError::LineageCorruption { details } => write!(f, "Lineage corruption: {}", details),
            McpError::SubstrateError { message } => write!(f, "Substrate error: {}", message),
            McpError::BadRequest { message } => write!(f, "Bad request: {}", message),
        }
    }
}

impl std::error::Error for McpError {}

impl McpError {
    /// Stable numeric error code (for machines)
    pub fn code(&self) -> u32 {
        match self {
            McpError::NodeNotFound { .. } => 4004,
            McpError::EdgeNotFound { .. } => 4004,
            McpError::EsvValidationFailed { .. } => 1000,
            McpError::DriftExceeded { .. } => 2000,
            McpError::LineageCorruption { .. } => 3000,
            McpError::SubstrateError { .. } => 5000,
            McpError::BadRequest { .. } => 4000,
        }
    }

    /// Stable string error code (for humans/docs)
    pub fn code_string(&self) -> &'static str {
        match self {
            McpError::NodeNotFound { .. } => "node_not_found",
            McpError::EdgeNotFound { .. } => "edge_not_found",
            McpError::EsvValidationFailed { .. } => "esv_validation_failed",
            McpError::DriftExceeded { .. } => "drift_exceeded",
            McpError::LineageCorruption { .. } => "lineage_corruption",
            McpError::SubstrateError { .. } => "substrate_error",
            McpError::BadRequest { .. } => "bad_request",
        }
    }

    /// Convert to JSON-RPC error code (alias for code() as i32)
    pub fn error_code(&self) -> i32 {
        self.code() as i32
    }
}

impl From<SimError> for McpError {
    fn from(err: SimError) -> Self {
        McpError::SubstrateError { message: err.to_string() }
    }
}

// ============================================================================
// MCP Response Types (Sanitized for External Exposure)
// ============================================================================

/// Sanitized node state for MCP responses
/// SECURITY: Exposes only belief and validity flag, not internal state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpNodeState {
    /// Node identifier (numeric, not internal NodeId)
    pub id: u64,
    /// Current belief value [0.0, 1.0]
    pub belief: f64,
    /// Energy level (summarized, not raw)
    pub energy: f64,
    /// ESV compliance status (boolean, not raw ESV vector)
    pub esv_valid: bool,
    /// Stability indicator [0.0, 1.0]
    pub stability: f64,
}

impl From<&SubstrateNodeState> for McpNodeState {
    fn from(node: &SubstrateNodeState) -> Self {
        Self {
            id: node.id.0,
            belief: node.belief,
            energy: node.mirror_energy(),
            esv_valid: true, // ESV validation happens at operation time
            stability: node.stability,
        }
    }
}

/// Sanitized edge state for MCP responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpEdgeState {
    /// Edge identifier
    pub id: u64,
    /// Source node ID
    pub src: u64,
    /// Destination node ID
    pub dst: u64,
    /// Edge weight [0.0, 1.0]
    pub weight: f64,
}

impl From<&SubstrateEdge> for McpEdgeState {
    fn from(edge: &SubstrateEdge) -> Self {
        Self {
            id: edge.id.0,
            src: edge.source.0,
            dst: edge.target.0,
            weight: edge.weight,
        }
    }
}

/// Sanitized governor status for MCP responses
/// SECURITY: Exposes only high-level health indicators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpGovernorStatus {
    /// Whether energy drift is within bounds (ε ≤ 1e-10)
    pub drift_ok: bool,
    /// Current drift value (for diagnostics)
    pub energy_drift: f64,
    /// Coherence index [0.0, 1.0]
    pub coherence: f64,
    /// Total node count
    pub node_count: usize,
    /// Total edge count  
    pub edge_count: usize,
    /// Overall health status
    pub healthy: bool,
}

/// Sanitized lineage entry for MCP responses
/// SECURITY: Exposes only checksum, not full hash chain or internal state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpLineageEntry {
    /// Entry sequence number
    pub sequence: u64,
    /// Operation type (sanitized)
    pub operation: String,
    /// SHA-256 checksum (hex encoded)
    pub checksum: String,
    /// Tick when recorded
    pub tick: u64,
}

impl From<&CausalEvent> for McpLineageEntry {
    fn from(event: &CausalEvent) -> Self {
        let operation = match &event.event_type {
            scg_trace::EventType::Tick(_) => "tick".to_string(),
            scg_trace::EventType::Decision(_) => "decision".to_string(),
            scg_trace::EventType::Ethics(_) => "ethics".to_string(),
            scg_trace::EventType::Energy(_) => "energy".to_string(),
        };
        Self {
            sequence: event.event_id,
            operation,
            checksum: hex::encode(event.hash),
            tick: event.tick,
        }
    }
}

// ============================================================================
// MCP Request Parameter Types
// ============================================================================

/// Parameters for node.create
#[derive(Debug, Clone, Deserialize)]
pub struct CreateNodeParams {
    pub belief: f64,
    pub energy: f64,
}

/// Parameters for node.mutate
#[derive(Debug, Clone, Deserialize)]
pub struct MutateNodeParams {
    pub node_id: u64,
    pub delta: f64,
}

/// Parameters for node.query
#[derive(Debug, Clone, Deserialize)]
pub struct QueryNodeParams {
    pub node_id: u64,
}

/// Parameters for edge.bind
#[derive(Debug, Clone, Deserialize)]
pub struct BindEdgeParams {
    pub src: u64,
    pub dst: u64,
    pub weight: f64,
}

/// Parameters for edge.propagate
#[derive(Debug, Clone, Deserialize)]
pub struct PropagateEdgeParams {
    pub edge_id: u64,
}

/// Parameters for lineage.export
#[derive(Debug, Clone, Deserialize)]
pub struct ExportLineageParams {
    pub path: String,
}

// ============================================================================
// JSON-RPC 2.0 Protocol Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcRequest {
    pub jsonrpc: String,
    pub method: String,
    #[serde(default)]
    pub params: serde_json::Value,
    #[serde(default)]
    pub id: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcError>,
    pub id: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
}

impl RpcResponse {
    pub fn success(id: serde_json::Value, value: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            result: Some(value),
            error: None,
            id,
        }
    }

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

    pub fn from_mcp_error(id: serde_json::Value, err: McpError) -> Self {
        Self::error(id, err.error_code(), err.to_string())
    }
}

// ============================================================================
// MCP Tool Metadata Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
    pub version: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolList {
    pub tools: Vec<ToolInfo>,
}

