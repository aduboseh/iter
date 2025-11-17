// SCG Connectome v2.0.0-alpha: Tract (Connection) Definitions
//
// ZERO SUBSTRATE COUPLING GUARANTEE:
// This module interacts with substrate ONLY via MCP protocol (no direct imports from src/scg_core.rs, src/types.rs, etc.)
//
// Canonical Reference: SCG Neuro Mapping v1, CONNECTOME_V2_SCAFFOLD.md §3.2

use super::regions::RegionId;
use std::collections::HashMap;

/// Tract identification (anatomical connection between regions)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TractId {
    pub source: RegionId,
    pub target: RegionId,
}

/// Tract properties (anatomical and functional characteristics)
#[derive(Debug, Clone)]
pub struct TractProperties {
    pub tract_id: TractId,
    pub conduction_velocity_ms: f64,  // Signal propagation delay
    pub strength: f64,                // Connection weight [0.0, 1.0]
    pub bidirectional: bool,          // Whether signal flows both ways
    pub plasticity: f64,              // Synaptic plasticity coefficient
}

/// Tract manager (coordinates all inter-region connections)
pub struct TractManager {
    tracts: HashMap<TractId, TractProperties>,
}

impl TractManager {
    /// Create manager with canonical neuroanatomical tracts
    pub fn new() -> Self {
        let mut tracts = HashMap::new();

        // ACC ↔ DLPFC (executive control and conflict monitoring)
        tracts.insert(
            TractId {
                source: RegionId::ACC,
                target: RegionId::DLPFC,
            },
            TractProperties {
                tract_id: TractId {
                    source: RegionId::ACC,
                    target: RegionId::DLPFC,
                },
                conduction_velocity_ms: 50.0,
                strength: 0.8,
                bidirectional: true,
                plasticity: 0.05,
            },
        );

        // DLPFC ↔ OFC (executive function and valuation)
        tracts.insert(
            TractId {
                source: RegionId::DLPFC,
                target: RegionId::OFC,
            },
            TractProperties {
                tract_id: TractId {
                    source: RegionId::DLPFC,
                    target: RegionId::OFC,
                },
                conduction_velocity_ms: 60.0,
                strength: 0.75,
                bidirectional: true,
                plasticity: 0.03,
            },
        );

        // Hippocampus → DLPFC (memory-guided executive control)
        tracts.insert(
            TractId {
                source: RegionId::Hippocampus,
                target: RegionId::DLPFC,
            },
            TractProperties {
                tract_id: TractId {
                    source: RegionId::Hippocampus,
                    target: RegionId::DLPFC,
                },
                conduction_velocity_ms: 80.0,
                strength: 0.7,
                bidirectional: false,
                plasticity: 0.08,
            },
        );

        // Amygdala → ACC (emotional salience to conflict monitoring)
        tracts.insert(
            TractId {
                source: RegionId::Amygdala,
                target: RegionId::ACC,
            },
            TractProperties {
                tract_id: TractId {
                    source: RegionId::Amygdala,
                    target: RegionId::ACC,
                },
                conduction_velocity_ms: 40.0,
                strength: 0.9,
                bidirectional: false,
                plasticity: 0.02,
            },
        );

        // Amygdala ↔ Hippocampus (emotional memory encoding)
        tracts.insert(
            TractId {
                source: RegionId::Amygdala,
                target: RegionId::Hippocampus,
            },
            TractProperties {
                tract_id: TractId {
                    source: RegionId::Amygdala,
                    target: RegionId::Hippocampus,
                },
                conduction_velocity_ms: 70.0,
                strength: 0.85,
                bidirectional: true,
                plasticity: 0.06,
            },
        );

        // OFC → Amygdala (valuation modulates emotional response)
        tracts.insert(
            TractId {
                source: RegionId::OFC,
                target: RegionId::Amygdala,
            },
            TractProperties {
                tract_id: TractId {
                    source: RegionId::OFC,
                    target: RegionId::Amygdala,
                },
                conduction_velocity_ms: 55.0,
                strength: 0.65,
                bidirectional: false,
                plasticity: 0.04,
            },
        );

        TractManager { tracts }
    }

    /// Get tract properties
    pub fn get_tract(&self, source: RegionId, target: RegionId) -> Option<&TractProperties> {
        let tract_id = TractId { source, target };
        self.tracts.get(&tract_id)
    }

    /// Get mutable tract properties
    pub fn get_tract_mut(&mut self, source: RegionId, target: RegionId) -> Option<&mut TractProperties> {
        let tract_id = TractId { source, target };
        self.tracts.get_mut(&tract_id)
    }

    /// Check if tract exists
    pub fn tract_exists(&self, source: RegionId, target: RegionId) -> bool {
        let tract_id = TractId { source, target };
        self.tracts.contains_key(&tract_id)
    }

    /// Get all outgoing tracts from a region
    pub fn outgoing_tracts(&self, source: RegionId) -> Vec<&TractProperties> {
        self.tracts
            .values()
            .filter(|tract| tract.tract_id.source == source)
            .collect()
    }

    /// Get all incoming tracts to a region
    pub fn incoming_tracts(&self, target: RegionId) -> Vec<&TractProperties> {
        self.tracts
            .values()
            .filter(|tract| tract.tract_id.target == target)
            .collect()
    }

    /// Update tract strength (synaptic plasticity)
    pub fn update_strength(&mut self, source: RegionId, target: RegionId, delta: f64) {
        if let Some(tract) = self.get_tract_mut(source, target) {
            let new_strength = (tract.strength + delta * tract.plasticity).clamp(0.0, 1.0);
            tract.strength = new_strength;
        }
    }

    /// Get total number of tracts
    pub fn tract_count(&self) -> usize {
        self.tracts.len()
    }

    /// Get connectivity matrix (for visualization)
    pub fn connectivity_matrix(&self) -> HashMap<(RegionId, RegionId), f64> {
        self.tracts
            .iter()
            .map(|(tract_id, props)| ((*tract_id).into(), props.strength))
            .collect()
    }
}

impl Default for TractManager {
    fn default() -> Self {
        Self::new()
    }
}

impl From<TractId> for (RegionId, RegionId) {
    fn from(tract_id: TractId) -> Self {
        (tract_id.source, tract_id.target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tract_manager_initialization() {
        let manager = TractManager::new();
        assert!(manager.tract_count() >= 6);  // At least 6 canonical tracts
    }

    #[test]
    fn test_tract_existence() {
        let manager = TractManager::new();
        assert!(manager.tract_exists(RegionId::ACC, RegionId::DLPFC));
        assert!(manager.tract_exists(RegionId::Amygdala, RegionId::ACC));
        assert!(!manager.tract_exists(RegionId::OFC, RegionId::DLPFC));  // No direct connection
    }

    #[test]
    fn test_tract_properties() {
        let manager = TractManager::new();
        let tract = manager.get_tract(RegionId::ACC, RegionId::DLPFC).unwrap();
        assert_eq!(tract.tract_id.source, RegionId::ACC);
        assert_eq!(tract.tract_id.target, RegionId::DLPFC);
        assert!(tract.strength > 0.0 && tract.strength <= 1.0);
    }

    #[test]
    fn test_outgoing_tracts() {
        let manager = TractManager::new();
        let outgoing = manager.outgoing_tracts(RegionId::ACC);
        assert!(!outgoing.is_empty());
        assert!(outgoing.iter().all(|t| t.tract_id.source == RegionId::ACC));
    }

    #[test]
    fn test_incoming_tracts() {
        let manager = TractManager::new();
        let incoming = manager.incoming_tracts(RegionId::ACC);
        assert!(!incoming.is_empty());
        assert!(incoming.iter().all(|t| t.tract_id.target == RegionId::ACC));
    }

    #[test]
    fn test_strength_update() {
        let mut manager = TractManager::new();
        let initial_strength = manager.get_tract(RegionId::ACC, RegionId::DLPFC).unwrap().strength;
        
        manager.update_strength(RegionId::ACC, RegionId::DLPFC, 0.1);
        
        let updated_strength = manager.get_tract(RegionId::ACC, RegionId::DLPFC).unwrap().strength;
        assert!(updated_strength > initial_strength);
    }

    #[test]
    fn test_strength_clamping() {
        let mut manager = TractManager::new();
        
        // Try to push strength above 1.0
        for _ in 0..100 {
            manager.update_strength(RegionId::ACC, RegionId::DLPFC, 1.0);
        }
        
        let strength = manager.get_tract(RegionId::ACC, RegionId::DLPFC).unwrap().strength;
        assert_eq!(strength, 1.0);  // Should be clamped at 1.0
    }
}
