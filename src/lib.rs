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
// ITER-PAR-01: Contract and Governance Modules
// ============================================================================

/// Contract envelopes for SCG-CTX-03 + SCG-INT-04 governance.
pub mod contracts;
/// Policy primitives for deterministic rule evaluation.
pub mod policy;
/// Economic control plane for learning costs and permits.
pub mod economics;
/// Decision packet export for audit and replay.
pub mod audit;

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

// ============================================================================
// ITER-PAR-01 Re-exports (Contract/Policy/Audit types)
// ============================================================================

pub use contracts::{
    ContractError, EnergyEnvelope, LearningEnvelope, LearningStatus, PolicyDecision,
    PolicyEnvelope, ReasoningEnvelope, SystemState,
};
pub use policy::{PolicyConfig, PolicyEvaluator, PolicyResult};
pub use economics::{EconomicsConfig, EconomicsController, LearningPermit};
pub use audit::{AuditError, AuditEvent, AuditLog, DecisionPacket};
