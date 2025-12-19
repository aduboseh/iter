//! MCP Response Types (Sanitized)
//!
//! These types are always available regardless of build mode.
//! They define the sanitized MCP response shapes without substrate dependencies.
//!
//! # Security
//!
//! These types expose ONLY sanitized views of substrate state.
//! Internal fields (topology, raw ESV, energy matrices) are never exposed.

use serde::{Deserialize, Serialize};
use std::fmt;

// ============================================================================
// MCP Error Types
// ============================================================================

/// MCP Error codes aligned with JSON-RPC 2.0 and the engine boundary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum McpError {
    /// Node not found by ID
    NodeNotFound { id: u64 },
    /// Edge not found by ID
    EdgeNotFound { id: u64 },
    /// ESV validation failed
    EsvValidationFailed { reason: String },
    /// Drift exceeded threshold
    DriftExceeded { drift: f64, threshold: f64 },
    /// Lineage integrity violation
    LineageCorruption { details: String },
    /// Generic substrate error
    SubstrateError { message: String },
    /// Invalid request parameters
    BadRequest { message: String },
}

impl fmt::Display for McpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            McpError::NodeNotFound { id } => write!(f, "Node not found: N{}", id),
            McpError::EdgeNotFound { id } => write!(f, "Edge not found: E{}", id),
            McpError::EsvValidationFailed { reason } => {
                write!(f, "ESV validation failed: {}", reason)
            }
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
    /// Stable numeric error code
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

    /// Stable string error code
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

    /// Convert to JSON-RPC error code
    pub fn error_code(&self) -> i32 {
        self.code() as i32
    }
}

// ============================================================================
// MCP Response Types (Sanitized for External Exposure)
// ============================================================================

/// Sanitized node state for MCP responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpNodeState {
    /// Node identifier
    pub id: u64,
    /// Current belief value [0.0, 1.0]
    pub belief: f64,
    /// Energy level (summarized)
    pub energy: f64,
    /// ESV compliance status
    pub esv_valid: bool,
    /// Stability indicator [0.0, 1.0]
    pub stability: f64,
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

/// Sanitized governor status for MCP responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpGovernorStatus {
    /// Whether energy drift is within bounds
    pub drift_ok: bool,
    /// Current drift value
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
