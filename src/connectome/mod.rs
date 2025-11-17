// SCG Connectome v2.0.0-alpha: Module Root
//
// ZERO SUBSTRATE COUPLING GUARANTEE:
// This entire connectome layer is ISOLATED from substrate internals.
// All interaction with substrate occurs ONLY via MCP protocol.
//
// Forbidden Imports (CI-enforced):
// - use crate::scg_core::*
// - use crate::types::*
// - use crate::fault::*
// - use crate::telemetry::*
// - use crate::lineage::*
// - use crate::mcp_handler::* (except for MCP client types when implemented)
//
// Canonical Reference: CONNECTOME_V2_SCAFFOLD.md, LTS_STRATEGY.md

pub mod regions;
pub mod tracts;
pub mod timestep;

// Re-export key types for convenience
pub use regions::{Region, RegionId, RegionManager, RegionState, RegionFunction};
pub use tracts::{TractId, TractProperties, TractManager};
pub use timestep::{TimestepConfig, TimestepManager, TemporalDynamics, PropagatingSignal};

/// Connectome version information
pub const CONNECTOME_VERSION: &str = "v2.0.0-alpha";
pub const SUBSTRATE_COMPATIBILITY: &str = "v1.0.x-substrate";

/// Verify zero coupling at compile time
///
/// This function is intentionally left empty and serves as a documentation
/// marker for the coupling audit. The CI system (`connectome_audit` workflow)
/// will verify that no connectome module imports substrate internals.
#[allow(dead_code)]
fn verify_zero_coupling() {
    // Compile-time assertion: this module MUST NOT import substrate internals
    // CI enforcement: .github/workflows/connectome_audit.yml
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connectome_initialization() {
        let regions = RegionManager::new();
        let tracts = TractManager::new();
        let timestep = TimestepManager::new();

        assert_eq!(regions.all_region_ids().len(), 5);
        assert!(tracts.tract_count() >= 6);
        assert_eq!(timestep.current_time(), 0);
    }

    #[test]
    fn test_temporal_dynamics() {
        let mut dynamics = TemporalDynamics::new();
        
        let t0 = dynamics.current_time();
        dynamics.step();
        let t1 = dynamics.current_time();
        
        assert!(t1 > t0);
    }

    #[test]
    fn test_version_info() {
        assert_eq!(CONNECTOME_VERSION, "v2.0.0-alpha");
        assert_eq!(SUBSTRATE_COMPATIBILITY, "v1.0.x-substrate");
    }
}
