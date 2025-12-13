//! Runtime facade.
//!
//! Thin wrapper around the underlying engine runtime used by the MCP boundary.
//! This module focuses on type translation and error mapping.

#![allow(dead_code)]

use parking_lot::RwLock;
use std::sync::atomic::{AtomicU64, Ordering};

use scg_sim::{
    Edge, EdgeId, IntegratedConfig, IntegratedSimulation, NodeId, NodeState as SubstrateNodeState,
    SimConfig,
};
use scg_energy::EnergyConfig;
use scg_consensus::ConsensusConfig;
use scg_ethics::EthicsConfig;
use scg_governance::{GovernanceValidator, DRIFT_EPSILON};
// CausalEvent used via types.rs re-export

use crate::types::{
    McpError, McpNodeState, McpEdgeState, McpGovernorStatus, McpLineageEntry,
};

/// Configuration for SubstrateRuntime initialization
#[derive(Debug, Clone, Default)]
pub struct SubstrateRuntimeConfig {
    pub sim: SimConfig,
    pub energy: EnergyConfig,
    pub consensus: ConsensusConfig,
    pub ethics: EthicsConfig,
    /// Optional seed override for deterministic testing
    pub seed: Option<u64>,
}


/// Facade wrapping the real IntegratedSimulation.
/// 
/// All MCP operations route through this facade.
pub struct SubstrateRuntime {
    /// The real integrated simulation
    sim: IntegratedSimulation,
    /// Governance validator for drift/ESV checks
    governance: GovernanceValidator,
    /// Node ID counter for sequential allocation
    next_node_id: AtomicU64,
    /// Edge ID counter for sequential allocation
    next_edge_id: AtomicU64,
}

impl SubstrateRuntime {
    /// Create a new SubstrateRuntime with the given configuration.
    pub fn new(config: SubstrateRuntimeConfig) -> Result<Self, McpError> {
        let mut sim_config = config.sim;
        if let Some(seed) = config.seed {
            sim_config.seed = seed;
        }

        let integrated_config = IntegratedConfig {
            sim: sim_config,
            energy: config.energy,
            consensus: config.consensus,
            ethics: config.ethics,
        };

        let sim = IntegratedSimulation::new(integrated_config)
            .map_err(|e| McpError::SubstrateError { message: format!("Failed to initialize simulation: {}", e) })?;

        Ok(Self {
            sim,
            governance: GovernanceValidator::new(),
            next_node_id: AtomicU64::new(0),
            next_edge_id: AtomicU64::new(0),
        })
    }

    /// Create a SubstrateRuntime with default configuration.
    pub fn with_defaults() -> Result<Self, McpError> {
        Self::new(SubstrateRuntimeConfig::default())
    }

    // ========================================================================
    // Node Operations
    // ========================================================================

    /// Create a new node in the substrate.
    /// 
    /// Delegates to `IntegratedSimulation::add_node` which:
    /// - Registers the node in the energy ledger
    /// - Adds the node to the cognitive graph
    pub fn create_node(&mut self, belief: f64, energy: f64) -> Result<McpNodeState, McpError> {
        let node_id = NodeId(self.next_node_id.fetch_add(1, Ordering::SeqCst));
        let node = SubstrateNodeState::new(node_id, belief, energy);

        self.sim.add_node(node.clone())
            .map_err(|e| McpError::SubstrateError { message: format!("Failed to add node: {}", e) })?;

        Ok(McpNodeState::from(&node))
    }

    /// Query a node's state by ID.
    /// 
    /// Returns the sanitized MCP view of the node state.
    /// 
    /// Note: This requires `&mut self` because `IntegratedSimulation` only exposes
    /// `graph_mut()` for graph access. This is a limitation of the current substrate API.
    pub fn query_node(&mut self, node_id: u64) -> Result<McpNodeState, McpError> {
        let id = NodeId(node_id);
        
        // Access graph through the simulation
        // IntegratedSimulation exposes graph_mut for testing/internal access
        let graph = self.sim.graph_mut();
        let node = graph.get_node(id)
            .map_err(|_| McpError::NodeNotFound { id: node_id })?;

        Ok(McpNodeState::from(node))
    }

    /// Get the total number of nodes in the substrate.
    pub fn node_count(&self) -> usize {
        self.sim.node_count()
    }

    /// Get the total number of edges in the substrate.
    pub fn edge_count(&mut self) -> usize {
        self.sim.graph_mut().edge_count()
    }

    /// Mutate a node's belief by a delta amount.
    ///
    /// Note: this endpoint is intended for testing and demonstration.
    pub fn mutate_node(&mut self, node_id: u64, delta: f64) -> Result<McpNodeState, McpError> {
        let id = NodeId(node_id);
        
        // Access through graph_mut (testing API)
        let graph = self.sim.graph_mut();
        
        // Get current state to compute new values
        let node = graph.get_node(id)
            .map_err(|_| McpError::NodeNotFound { id: node_id })?;
        
        let old_belief = node.belief;
        #[allow(deprecated)] // Direct energy access for debug mutation
        let old_energy = node.energy;
        let new_belief = (old_belief + delta).clamp(0.0, 1.0);
        
        // Compute energy cost for this mutation (using simple model)
        let belief_change = (new_belief - old_belief).abs();
        let energy_cost = 0.1 + 0.05 * belief_change; // Base + proportional to change
        let new_energy = (old_energy - energy_cost).max(0.0);
        
        // Apply mutation via get_node_mut
        let node = graph.get_node_mut(id)
            .map_err(|_| McpError::NodeNotFound { id: node_id })?;
        node.belief = new_belief;
        #[allow(deprecated)] // Direct energy access for debug mutation
        {
            node.energy = new_energy;
        }
        
        // Return updated state
        let graph = self.sim.graph_mut();
        let node = graph.get_node(id)
            .map_err(|_| McpError::NodeNotFound { id: node_id })?;
        
        Ok(McpNodeState::from(node))
    }

    // ========================================================================
    // Edge Operations
    // ========================================================================

    /// Create a new edge between two nodes.
    /// 
    /// Delegates to `IntegratedSimulation::add_edge`.
    pub fn create_edge(&mut self, src: u64, dst: u64, weight: f64) -> Result<McpEdgeState, McpError> {
        let edge_id = EdgeId(self.next_edge_id.fetch_add(1, Ordering::SeqCst));
        let edge = Edge::new(edge_id, NodeId(src), NodeId(dst), weight, true);

        self.sim.add_edge(edge.clone())
            .map_err(|e| {
                // Check if it's a node not found error
                let msg = e.to_string();
                if msg.contains("not found") {
                    if msg.contains(&format!("N{}", src)) {
                        McpError::NodeNotFound { id: src }
                    } else if msg.contains(&format!("N{}", dst)) {
                        McpError::NodeNotFound { id: dst }
                    } else {
                        McpError::SubstrateError { message: msg }
                    }
                } else if msg.contains("cycle") {
                    McpError::SubstrateError { message: format!("Edge would create cycle: {} -> {}", src, dst) }
                } else {
                    McpError::SubstrateError { message: msg }
                }
            })?;

        Ok(McpEdgeState::from(&edge))
    }

    // ========================================================================
    // Simulation Operations
    // ========================================================================

    /// Execute a single simulation step.
    pub fn step(&mut self) -> Result<(), McpError> {
        self.sim.step()
            .map_err(|e| McpError::SubstrateError { message: format!("Simulation step failed: {}", e) })?;
        Ok(())
    }

    /// Get the current simulation tick.
    pub fn current_tick(&self) -> u64 {
        self.sim.current_tick()
    }

    // ========================================================================
    // Governance Operations
    // ========================================================================

    /// Get the current governance status.
    ///
    /// Returns a sanitized status summary for MCP.
    pub fn governance_status(&mut self) -> Result<McpGovernorStatus, McpError> {
        // Get energy metrics from the substrate
        let ledger = self.sim.ledger();
        let initial = ledger.initial_total();
        let current = ledger.total_energy();
        let dissipated = ledger.total_dissipated();
        
        // Compute drift summary
        let drift = if initial > 0.0 {
            (current + dissipated - initial).abs()
        } else {
            0.0
        };

        // Update governance validator with current drift
        self.governance.set_drift(drift);

        // Check against configured threshold
        let drift_ok = drift <= DRIFT_EPSILON;

        // Compute coherence summary
        let mean_belief = self.sim.mean_belief();
        let coherence = 1.0 - (mean_belief - 0.5).abs() * 2.0;

        // Get edge count from substrate graph
        let edge_count = self.sim.graph_mut().edge_count();

        Ok(McpGovernorStatus {
            drift_ok,
            energy_drift: drift,
            coherence: coherence.clamp(0.0, 1.0),
            node_count: self.sim.node_count(),
            edge_count,
            healthy: drift_ok,
        })
    }

    /// Validate current energy conservation.
    /// 
    /// Delegates to substrate's conservation check.
    pub fn check_energy_conservation(&self, tolerance: f64) -> Result<(), McpError> {
        self.sim.check_energy_conservation(tolerance)
            .map_err(|_e| McpError::DriftExceeded {
                drift: 0.0, // Substrate error doesn't expose the actual drift
                threshold: tolerance,
            })
    }

    // ========================================================================
    // Lineage Operations
    // ========================================================================

    /// Get recent lineage entries as sanitized MCP types.
    /// 
    /// Returns only checksums and operation types, not internal state.
    pub fn lineage_recent(&self, limit: usize) -> Vec<McpLineageEntry> {
        let trace = self.sim.trace();
        let events = trace.events();
        
        events.iter()
            .rev()
            .take(limit)
            .map(McpLineageEntry::from)
            .collect()
    }

    /// Verify the integrity of lineage data.
    pub fn verify_lineage(&self) -> Result<(), McpError> {
        self.sim.verify_trace()
            .map_err(|e| McpError::LineageCorruption {
                details: format!("Trace verification failed: {}", e),
            })
    }

    /// Get all lineage events (for export).
    pub fn lineage_all(&self) -> Vec<McpLineageEntry> {
        let trace = self.sim.trace();
        trace.events().iter().map(McpLineageEntry::from).collect()
    }
}

// ============================================================================
// Thread-safe wrapper for use in handlers
// ============================================================================

/// Thread-safe wrapper around SubstrateRuntime.
pub struct SharedSubstrateRuntime {
    inner: RwLock<SubstrateRuntime>,
}

impl SharedSubstrateRuntime {
    pub fn new(runtime: SubstrateRuntime) -> Self {
        Self {
            inner: RwLock::new(runtime),
        }
    }

    /// Get read access to the runtime.
    pub fn read(&self) -> parking_lot::RwLockReadGuard<'_, SubstrateRuntime> {
        self.inner.read()
    }

    /// Get write access to the runtime.
    pub fn write(&self) -> parking_lot::RwLockWriteGuard<'_, SubstrateRuntime> {
        self.inner.write()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_substrate_runtime_creation() {
        let runtime = SubstrateRuntime::with_defaults();
        assert!(runtime.is_ok());
    }

    #[test]
    fn test_create_node() {
        let mut runtime = SubstrateRuntime::with_defaults().unwrap();
        let node = runtime.create_node(0.5, 10.0);
        assert!(node.is_ok());
        let node = node.unwrap();
        assert_eq!(node.id, 0);
        assert!((node.belief - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_create_multiple_nodes() {
        let mut runtime = SubstrateRuntime::with_defaults().unwrap();
        
        let n1 = runtime.create_node(0.5, 10.0).unwrap();
        let n2 = runtime.create_node(0.7, 15.0).unwrap();
        let n3 = runtime.create_node(0.3, 5.0).unwrap();

        assert_eq!(n1.id, 0);
        assert_eq!(n2.id, 1);
        assert_eq!(n3.id, 2);
        assert_eq!(runtime.node_count(), 3);
    }

    #[test]
    fn test_create_edge() {
        let mut runtime = SubstrateRuntime::with_defaults().unwrap();
        
        runtime.create_node(0.5, 10.0).unwrap();
        runtime.create_node(0.7, 10.0).unwrap();

        let edge = runtime.create_edge(0, 1, 0.8);
        assert!(edge.is_ok());
        let edge = edge.unwrap();
        assert_eq!(edge.src, 0);
        assert_eq!(edge.dst, 1);
    }

    #[test]
    fn test_governance_status() {
        let mut runtime = SubstrateRuntime::with_defaults().unwrap();
        runtime.create_node(0.5, 10.0).unwrap();

        let status = runtime.governance_status();
        assert!(status.is_ok());
        let status = status.unwrap();
        assert!(status.drift_ok);
        assert!(status.healthy);
    }
}
