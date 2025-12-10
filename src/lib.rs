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

// ============================================================================
// Core Modules
// ============================================================================

pub mod caller_context;
pub mod governance;
pub mod mcp_handler;
pub mod metrics;
pub mod services;
pub mod substrate_runtime;
pub mod types;
pub mod validation;

// ============================================================================
// SCG Substrate Re-exports
// ============================================================================

/// SCG Simulation core - IntegratedSimulation with full substrate composition
pub use scg_sim::{IntegratedSimulation, IntegratedConfig, SimError};
pub use scg_sim::{NodeId, EdgeId, NodeState as SubstrateNodeState, Edge as SubstrateEdge};

/// SCG Governance - drift validation, ESV params, rule hash verification
pub use scg_governance::{GovernanceValidator, DRIFT_EPSILON};

/// SCG Trace - deterministic causal event logging with hash chaining
pub use scg_trace::{CausalTrace, CausalEvent};

/// SCG Energy - thermodynamic conservation tracking
pub use scg_energy::{EnergyLedger, EnergyConfig};

/// SCG Ethics - moral reasoning and harm evaluation
pub use scg_ethics::{EthicsKernel, EthicsDecision};

// ============================================================================
// MCP Type Re-exports (Sanitized for external use)
// ============================================================================

pub use types::{
    McpNodeState, McpEdgeState, McpGovernorStatus, McpLineageEntry,
    McpError, RpcRequest, RpcResponse, RpcError,
};

// ============================================================================
// Substrate Runtime
// ============================================================================

pub use substrate_runtime::{SubstrateRuntime, SubstrateRuntimeConfig, SharedSubstrateRuntime};
