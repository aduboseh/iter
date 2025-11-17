// SCG Connectome v2.0.0-alpha: Neuroanatomical Region Definitions
//
// ZERO SUBSTRATE COUPLING GUARANTEE:
// This module interacts with substrate ONLY via MCP protocol (no direct imports from src/scg_core.rs, src/types.rs, etc.)
//
// Canonical Reference: SCG Neuro Mapping v1, CONNECTOME_V2_SCAFFOLD.md §3.2

use std::collections::HashMap;

/// Region identification (neuroanatomical mapping)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RegionId {
    /// Anterior Cingulate Cortex (conflict monitoring, error detection)
    ACC,
    /// Dorsolateral Prefrontal Cortex (executive function, working memory)
    DLPFC,
    /// Orbitofrontal Cortex (valuation, reward processing)
    OFC,
    /// Hippocampus (episodic memory, spatial navigation)
    Hippocampus,
    /// Amygdala (emotional salience, threat detection)
    Amygdala,
}

/// Region state (cognitive activity level)
#[derive(Debug, Clone)]
pub struct RegionState {
    pub region_id: RegionId,
    pub activation_level: f64,  // [0.0, 1.0] cognitive activation
    pub node_count: usize,       // Number of substrate nodes mapped to this region
    pub last_update_ms: u64,     // Timestamp of last activity
}

/// Region interface (interacts with substrate via MCP protocol)
pub struct Region {
    pub id: RegionId,
    pub state: RegionState,
    pub function: RegionFunction,
    // MCP client will be injected here (not imported from substrate)
}

/// Region-specific cognitive function
#[derive(Debug, Clone)]
pub enum RegionFunction {
    ConflictMonitoring {
        conflict_threshold: f64,
        resolution_strategy: ConflictResolution,
    },
    Executive {
        task_queue_capacity: usize,
        planning_horizon_ms: u64,
    },
    Valuation {
        reward_discount_factor: f64,
        exploration_epsilon: f64,
    },
    EpisodicMemory {
        consolidation_threshold: f64,
        recall_latency_ms: u64,
    },
    EmotionalSalience {
        threat_sensitivity: f64,
        habituation_rate: f64,
    },
}

#[derive(Debug, Clone)]
pub enum ConflictResolution {
    Inhibition,
    TaskSwitching,
    CognitiveControl,
}

impl Region {
    /// Create a new region with default configuration
    pub fn new(id: RegionId) -> Self {
        let function = match id {
            RegionId::ACC => RegionFunction::ConflictMonitoring {
                conflict_threshold: 0.7,
                resolution_strategy: ConflictResolution::CognitiveControl,
            },
            RegionId::DLPFC => RegionFunction::Executive {
                task_queue_capacity: 10,
                planning_horizon_ms: 5000,
            },
            RegionId::OFC => RegionFunction::Valuation {
                reward_discount_factor: 0.95,
                exploration_epsilon: 0.1,
            },
            RegionId::Hippocampus => RegionFunction::EpisodicMemory {
                consolidation_threshold: 0.85,
                recall_latency_ms: 100,
            },
            RegionId::Amygdala => RegionFunction::EmotionalSalience {
                threat_sensitivity: 0.8,
                habituation_rate: 0.05,
            },
        };

        Region {
            id,
            state: RegionState {
                region_id: id,
                activation_level: 0.0,
                node_count: 0,
                last_update_ms: 0,
            },
            function,
        }
    }

    /// Update region activation (called by connectome orchestrator)
    pub fn update_activation(&mut self, new_activation: f64, timestamp_ms: u64) {
        self.state.activation_level = new_activation.clamp(0.0, 1.0);
        self.state.last_update_ms = timestamp_ms;
    }

    /// Get region name (for logging and telemetry)
    pub fn name(&self) -> &'static str {
        match self.id {
            RegionId::ACC => "Anterior Cingulate Cortex",
            RegionId::DLPFC => "Dorsolateral Prefrontal Cortex",
            RegionId::OFC => "Orbitofrontal Cortex",
            RegionId::Hippocampus => "Hippocampus",
            RegionId::Amygdala => "Amygdala",
        }
    }

    /// Get region abbreviation
    pub fn abbreviation(&self) -> &'static str {
        match self.id {
            RegionId::ACC => "ACC",
            RegionId::DLPFC => "DLPFC",
            RegionId::OFC => "OFC",
            RegionId::Hippocampus => "HIPP",
            RegionId::Amygdala => "AMYG",
        }
    }
}

/// Connectome region manager (coordinates all regions)
pub struct RegionManager {
    regions: HashMap<RegionId, Region>,
}

impl RegionManager {
    /// Create manager with all 5 canonical regions
    pub fn new() -> Self {
        let mut regions = HashMap::new();
        
        regions.insert(RegionId::ACC, Region::new(RegionId::ACC));
        regions.insert(RegionId::DLPFC, Region::new(RegionId::DLPFC));
        regions.insert(RegionId::OFC, Region::new(RegionId::OFC));
        regions.insert(RegionId::Hippocampus, Region::new(RegionId::Hippocampus));
        regions.insert(RegionId::Amygdala, Region::new(RegionId::Amygdala));

        RegionManager { regions }
    }

    /// Get region by ID
    pub fn get_region(&self, id: RegionId) -> Option<&Region> {
        self.regions.get(&id)
    }

    /// Get mutable region by ID
    pub fn get_region_mut(&mut self, id: RegionId) -> Option<&mut Region> {
        self.regions.get_mut(&id)
    }

    /// Get all region IDs
    pub fn all_region_ids(&self) -> Vec<RegionId> {
        vec![
            RegionId::ACC,
            RegionId::DLPFC,
            RegionId::OFC,
            RegionId::Hippocampus,
            RegionId::Amygdala,
        ]
    }

    /// Get activation snapshot (for telemetry)
    pub fn activation_snapshot(&self) -> HashMap<RegionId, f64> {
        self.regions
            .iter()
            .map(|(id, region)| (*id, region.state.activation_level))
            .collect()
    }
}

impl Default for RegionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_region_creation() {
        let acc = Region::new(RegionId::ACC);
        assert_eq!(acc.id, RegionId::ACC);
        assert_eq!(acc.name(), "Anterior Cingulate Cortex");
        assert_eq!(acc.abbreviation(), "ACC");
    }

    #[test]
    fn test_region_activation_update() {
        let mut dlpfc = Region::new(RegionId::DLPFC);
        dlpfc.update_activation(0.75, 1000);
        assert_eq!(dlpfc.state.activation_level, 0.75);
        assert_eq!(dlpfc.state.last_update_ms, 1000);
    }

    #[test]
    fn test_activation_clamping() {
        let mut ofc = Region::new(RegionId::OFC);
        ofc.update_activation(1.5, 2000);  // Above max
        assert_eq!(ofc.state.activation_level, 1.0);
        
        ofc.update_activation(-0.5, 3000);  // Below min
        assert_eq!(ofc.state.activation_level, 0.0);
    }

    #[test]
    fn test_region_manager_initialization() {
        let manager = RegionManager::new();
        assert_eq!(manager.regions.len(), 5);
        assert!(manager.get_region(RegionId::ACC).is_some());
        assert!(manager.get_region(RegionId::Hippocampus).is_some());
    }

    #[test]
    fn test_activation_snapshot() {
        let mut manager = RegionManager::new();
        manager.get_region_mut(RegionId::ACC).unwrap().update_activation(0.5, 1000);
        manager.get_region_mut(RegionId::DLPFC).unwrap().update_activation(0.8, 1000);
        
        let snapshot = manager.activation_snapshot();
        assert_eq!(snapshot.len(), 5);
        assert_eq!(snapshot[&RegionId::ACC], 0.5);
        assert_eq!(snapshot[&RegionId::DLPFC], 0.8);
    }
}
