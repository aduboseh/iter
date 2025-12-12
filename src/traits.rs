//! Engine Boundary Traits
//!
//! These traits define the **minimal, stable interface** between the MCP layer
//! and the internal engine. They serve as an abstraction boundary that:
//!
//! 1. Prevents substrate type leakage into public API
//! 2. Enables version-stable interaction with substrate internals
//! 3. Documents the exact contract MCP depends on
//!
//! # Boundary Invariant
//!
//! External consumers see ONLY these traits, never the underlying substrate types.
//! Implementations are `pub(crate)` and connect to real substrate types internally.

use crate::types::{SubstrateNodeState, SubstrateEdge};

// ============================================================================
// Node Boundary Trait
// ============================================================================

/// View trait for substrate node state.
///
/// Provides a stable, minimal interface for reading node properties
/// without exposing internal substrate representation.
pub trait SubstrateNodeView {
    /// Node identifier type
    type Id: Copy;

    /// Get the node's unique identifier
    fn id(&self) -> Self::Id;

    /// Get the node's confidence level (substrate: belief) [0.0, 1.0]
    fn confidence(&self) -> f64;

    /// Get the node's resource level (substrate: energy)
    fn resource_level(&self) -> f64;

    /// Get the node's coherence indicator (substrate: stability) [0.0, 1.0]
    fn coherence(&self) -> f64;

    /// Check if the node is in a valid ESV state
    fn esv_valid(&self) -> bool;
}

// ============================================================================
// Edge Boundary Trait
// ============================================================================

/// View trait for substrate edge state.
///
/// Provides a stable, minimal interface for reading edge properties
/// without exposing internal substrate representation.
pub trait SubstrateEdgeView {
    /// Edge identifier type
    type Id: Copy;
    /// Node identifier type
    type NodeId: Copy;

    /// Get the edge's unique identifier
    fn id(&self) -> Self::Id;

    /// Get the source node ID
    fn source(&self) -> Self::NodeId;

    /// Get the target node ID
    fn target(&self) -> Self::NodeId;

    /// Get the edge weight [0.0, 1.0]
    fn weight(&self) -> f64;
}

// ============================================================================
// Governor Boundary Trait
// ============================================================================

/// View trait for substrate governor/governance state.
///
/// Provides high-level health indicators without exposing
/// internal governance machinery.
pub trait SubstrateGovernorView {
    /// Check if energy drift is within acceptable bounds
    fn drift_ok(&self) -> bool;

    /// Get the current drift value
    fn energy_drift(&self) -> f64;

    /// Get the coherence index [0.0, 1.0]
    fn coherence(&self) -> f64;

    /// Check overall system health
    fn healthy(&self) -> bool;
}

// ============================================================================
// Implementations (Internal - connect to real substrate types)
// ============================================================================

impl SubstrateNodeView for SubstrateNodeState {
    type Id = u64;

    fn id(&self) -> Self::Id {
        self.id.0
    }

    fn confidence(&self) -> f64 {
        self.belief
    }

    fn resource_level(&self) -> f64 {
        self.mirror_energy()
    }

    fn coherence(&self) -> f64 {
        self.stability
    }

    fn esv_valid(&self) -> bool {
        // ESV validation is done at operation time, not state inspection
        true
    }
}

impl SubstrateEdgeView for SubstrateEdge {
    type Id = u64;
    type NodeId = u64;

    fn id(&self) -> Self::Id {
        self.id.0
    }

    fn source(&self) -> Self::NodeId {
        self.source.0
    }

    fn target(&self) -> Self::NodeId {
        self.target.0
    }

    fn weight(&self) -> f64 {
        self.weight
    }
}
