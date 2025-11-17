// SCG Connectome v2.0.0-alpha: Temporal Dynamics (Timestep Management)
//
// ZERO SUBSTRATE COUPLING GUARANTEE:
// This module interacts with substrate ONLY via MCP protocol (no direct imports from src/scg_core.rs, src/types.rs, etc.)
//
// Canonical Reference: SCG Neuro Mapping v1, CONNECTOME_V2_SCAFFOLD.md §3.2

use super::regions::{Region, RegionId, RegionManager};
use super::tracts::TractManager;
use std::collections::HashMap;

/// Timestep configuration (temporal resolution)
#[derive(Debug, Clone)]
pub struct TimestepConfig {
    pub timestep_duration_ms: u64,  // Duration of each timestep (default: 10ms)
    pub max_propagation_delay_ms: u64,  // Maximum tract delay
}

impl Default for TimestepConfig {
    fn default() -> Self {
        TimestepConfig {
            timestep_duration_ms: 10,
            max_propagation_delay_ms: 200,
        }
    }
}

/// Signal in transit (propagating through tract)
#[derive(Debug, Clone)]
pub struct PropagatingSignal {
    pub source: RegionId,
    pub target: RegionId,
    pub signal_strength: f64,
    pub arrival_time_ms: u64,
}

/// Timestep manager (coordinates temporal dynamics)
pub struct TimestepManager {
    config: TimestepConfig,
    current_time_ms: u64,
    propagating_signals: Vec<PropagatingSignal>,
}

impl TimestepManager {
    /// Create new timestep manager with default config
    pub fn new() -> Self {
        TimestepManager {
            config: TimestepConfig::default(),
            current_time_ms: 0,
            propagating_signals: Vec::new(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: TimestepConfig) -> Self {
        TimestepManager {
            config,
            current_time_ms: 0,
            propagating_signals: Vec::new(),
        }
    }

    /// Advance time by one timestep
    pub fn step(&mut self) -> u64 {
        self.current_time_ms += self.config.timestep_duration_ms;
        self.current_time_ms
    }

    /// Get current simulation time
    pub fn current_time(&self) -> u64 {
        self.current_time_ms
    }

    /// Send signal from source to target (with tract delay)
    pub fn send_signal(
        &mut self,
        source: RegionId,
        target: RegionId,
        signal_strength: f64,
        conduction_velocity_ms: f64,
    ) {
        let arrival_time = self.current_time_ms + conduction_velocity_ms as u64;
        
        self.propagating_signals.push(PropagatingSignal {
            source,
            target,
            signal_strength,
            arrival_time_ms: arrival_time,
        });
    }

    /// Get signals that have arrived at their targets
    pub fn collect_arrived_signals(&mut self) -> Vec<PropagatingSignal> {
        let current_time = self.current_time_ms;
        
        let (arrived, still_propagating): (Vec<_>, Vec<_>) = self
            .propagating_signals
            .drain(..)
            .partition(|signal| signal.arrival_time_ms <= current_time);
        
        self.propagating_signals = still_propagating;
        arrived
    }

    /// Get count of signals currently in transit
    pub fn in_transit_count(&self) -> usize {
        self.propagating_signals.len()
    }

    /// Reset simulation time
    pub fn reset(&mut self) {
        self.current_time_ms = 0;
        self.propagating_signals.clear();
    }
}

impl Default for TimestepManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Connectome temporal dynamics coordinator
pub struct TemporalDynamics {
    timestep_manager: TimestepManager,
    region_manager: RegionManager,
    tract_manager: TractManager,
}

impl TemporalDynamics {
    /// Create new temporal dynamics coordinator
    pub fn new() -> Self {
        TemporalDynamics {
            timestep_manager: TimestepManager::new(),
            region_manager: RegionManager::new(),
            tract_manager: TractManager::new(),
        }
    }

    /// Advance simulation by one timestep
    pub fn step(&mut self) -> u64 {
        let current_time = self.timestep_manager.step();
        
        // Collect signals that have arrived
        let arrived_signals = self.timestep_manager.collect_arrived_signals();
        
        // Update region activations based on arrived signals
        let mut activation_updates: HashMap<RegionId, Vec<f64>> = HashMap::new();
        
        for signal in arrived_signals {
            activation_updates
                .entry(signal.target)
                .or_insert_with(Vec::new)
                .push(signal.signal_strength);
        }
        
        // Apply activation updates
        for (region_id, signals) in activation_updates {
            if let Some(region) = self.region_manager.get_region_mut(region_id) {
                // Sum incoming signals (simplified integration)
                let total_input: f64 = signals.iter().sum();
                let current_activation = region.state.activation_level;
                
                // Update activation with decay and input
                let decay_factor = 0.9;  // Activation decay per timestep
                let new_activation = (current_activation * decay_factor + total_input * 0.1).clamp(0.0, 1.0);
                
                region.update_activation(new_activation, current_time);
            }
        }
        
        current_time
    }

    /// Propagate signal from source region to all connected targets
    pub fn propagate_from_region(&mut self, source: RegionId, signal_strength: f64) {
        let outgoing_tracts = self.tract_manager.outgoing_tracts(source);
        
        for tract in outgoing_tracts {
            let modulated_strength = signal_strength * tract.strength;
            self.timestep_manager.send_signal(
                source,
                tract.tract_id.target,
                modulated_strength,
                tract.conduction_velocity_ms,
            );
        }
    }

    /// Get current simulation time
    pub fn current_time(&self) -> u64 {
        self.timestep_manager.current_time()
    }

    /// Get current connectivity state (for telemetry)
    pub fn connectivity_snapshot(&self) -> HashMap<(RegionId, RegionId), f64> {
        self.tract_manager.connectivity_matrix()
    }

    /// Get current activation snapshot
    pub fn activation_snapshot(&self) -> HashMap<RegionId, f64> {
        self.region_manager.activation_snapshot()
    }
}

impl Default for TemporalDynamics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timestep_advancement() {
        let mut manager = TimestepManager::new();
        assert_eq!(manager.current_time(), 0);
        
        manager.step();
        assert_eq!(manager.current_time(), 10);  // Default 10ms timestep
        
        manager.step();
        assert_eq!(manager.current_time(), 20);
    }

    #[test]
    fn test_signal_propagation() {
        let mut manager = TimestepManager::new();
        
        manager.send_signal(RegionId::ACC, RegionId::DLPFC, 0.5, 50.0);
        assert_eq!(manager.in_transit_count(), 1);
        
        // Signal should not have arrived yet
        let arrived = manager.collect_arrived_signals();
        assert_eq!(arrived.len(), 0);
        
        // Advance time past signal arrival
        for _ in 0..6 {
            manager.step();  // 60ms total
        }
        
        let arrived = manager.collect_arrived_signals();
        assert_eq!(arrived.len(), 1);
        assert_eq!(arrived[0].source, RegionId::ACC);
        assert_eq!(arrived[0].target, RegionId::DLPFC);
    }

    #[test]
    fn test_temporal_dynamics_step() {
        let mut dynamics = TemporalDynamics::new();
        
        let t0 = dynamics.current_time();
        let t1 = dynamics.step();
        
        assert_eq!(t1, t0 + 10);  // Default 10ms timestep
    }

    #[test]
    fn test_region_propagation() {
        let mut dynamics = TemporalDynamics::new();
        
        // Propagate signal from ACC
        dynamics.propagate_from_region(RegionId::ACC, 0.8);
        
        // Signal should be in transit
        assert!(dynamics.timestep_manager.in_transit_count() > 0);
    }

    #[test]
    fn test_activation_decay() {
        let mut dynamics = TemporalDynamics::new();
        
        // Set initial activation
        if let Some(region) = dynamics.region_manager.get_region_mut(RegionId::ACC) {
            region.update_activation(1.0, 0);
        }
        
        // Step without input (activation should decay)
        for _ in 0..10 {
            dynamics.step();
        }
        
        let activation = dynamics.activation_snapshot()[&RegionId::ACC];
        assert!(activation < 1.0);  // Should have decayed
    }

    #[test]
    fn test_reset() {
        let mut manager = TimestepManager::new();
        manager.step();
        manager.send_signal(RegionId::ACC, RegionId::DLPFC, 0.5, 50.0);
        
        manager.reset();
        
        assert_eq!(manager.current_time(), 0);
        assert_eq!(manager.in_transit_count(), 0);
    }
}
