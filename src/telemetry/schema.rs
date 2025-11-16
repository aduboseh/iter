/// SCG Substrate: Telemetry Schema and Emission System
/// 
/// Real-time observability for:
/// - Energy drift monitoring
/// - ESV validation ratios
/// - Lineage events
/// - Coherence and entropy indices
/// 
/// Invariant: Telemetry is read-only and does not perturb cognitive state.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Core telemetry record emitted on each propagation cycle.
/// 
/// Format compatible with OpenTelemetry, Prometheus, Azure Monitor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryRecord {
    /// ISO 8601 timestamp
    pub timestamp: String,
    
    /// Unique cluster identifier
    pub cluster_id: String,
    
    /// Current energy drift: |E_total(t) - E_total(0)|
    /// Invariant: Must remain ≤ 1e-10
    pub energy_drift: f64,
    
    /// Synchronization coherence index C(t)
    /// Invariant: Must remain ≥ 0.97 under load
    pub coherence: f64,
    
    /// ESV validation success ratio (0.0 to 1.0)
    /// Invariant: Must be 1.0 (100% pass rate)
    pub esv_valid_ratio: f64,
    
    /// Entropy index S_c(t) - measures system disorder
    /// Lower is better; high entropy indicates instability
    pub entropy_index: f64,
    
    /// Optional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_count: Option<usize>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edge_count: Option<usize>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lineage_depth: Option<usize>,
}

impl TelemetryRecord {
    /// Creates a new telemetry record with current timestamp.
    pub fn new(
        cluster_id: impl Into<String>,
        energy_drift: f64,
        coherence: f64,
        esv_valid_ratio: f64,
        entropy_index: f64,
    ) -> Self {
        Self {
            timestamp: Utc::now().to_rfc3339(),
            cluster_id: cluster_id.into(),
            energy_drift,
            coherence,
            esv_valid_ratio,
            entropy_index,
            node_count: None,
            edge_count: None,
            lineage_depth: None,
        }
    }
    
    /// Adds graph size metadata.
    pub fn with_graph_size(mut self, nodes: usize, edges: usize) -> Self {
        self.node_count = Some(nodes);
        self.edge_count = Some(edges);
        self
    }
    
    /// Adds lineage depth metadata.
    pub fn with_lineage_depth(mut self, depth: usize) -> Self {
        self.lineage_depth = Some(depth);
        self
    }
    
    /// Checks if record indicates violation of operational invariants.
    pub fn has_violation(&self) -> Option<TelemetryViolation> {
        const ENERGY_THRESHOLD: f64 = 1e-10;
        const COHERENCE_THRESHOLD: f64 = 0.97;
        const ESV_THRESHOLD: f64 = 1.0;
        
        if self.energy_drift.abs() > ENERGY_THRESHOLD {
            return Some(TelemetryViolation::EnergyDrift {
                current: self.energy_drift,
                threshold: ENERGY_THRESHOLD,
            });
        }
        
        if self.coherence < COHERENCE_THRESHOLD {
            return Some(TelemetryViolation::CoherenceLow {
                current: self.coherence,
                threshold: COHERENCE_THRESHOLD,
            });
        }
        
        if self.esv_valid_ratio < ESV_THRESHOLD {
            return Some(TelemetryViolation::EsvFailure {
                ratio: self.esv_valid_ratio,
            });
        }
        
        None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TelemetryViolation {
    EnergyDrift { current: f64, threshold: f64 },
    CoherenceLow { current: f64, threshold: f64 },
    EsvFailure { ratio: f64 },
}

/// Thread-safe telemetry emitter with buffering.
pub struct TelemetryEmitter {
    cluster_id: String,
    buffer: parking_lot::Mutex<Vec<TelemetryRecord>>,
    emit_callback: Option<Box<dyn Fn(&TelemetryRecord) + Send + Sync>>,
}

impl TelemetryEmitter {
    /// Creates a new telemetry emitter for the given cluster.
    pub fn new(cluster_id: impl Into<String>) -> Self {
        Self {
            cluster_id: cluster_id.into(),
            buffer: parking_lot::Mutex::new(Vec::new()),
            emit_callback: None,
        }
    }
    
    /// Registers a callback to be invoked on each emission.
    /// Useful for integrating with external systems (Prometheus, Azure Monitor).
    pub fn with_callback<F>(mut self, callback: F) -> Self 
    where
        F: Fn(&TelemetryRecord) + Send + Sync + 'static,
    {
        self.emit_callback = Some(Box::new(callback));
        self
    }
    
    /// Emits a telemetry record.
    /// 
    /// Invariant: This operation must not perturb SCG state or introduce entropy.
    pub fn emit(
        &self,
        energy_drift: f64,
        coherence: f64,
        esv_valid_ratio: f64,
        entropy_index: f64,
    ) {
        let record = TelemetryRecord::new(
            &self.cluster_id,
            energy_drift,
            coherence,
            esv_valid_ratio,
            entropy_index,
        );
        
        // Check for violations
        if let Some(violation) = record.has_violation() {
            eprintln!("[TELEMETRY] VIOLATION DETECTED: {:?}", violation);
        }
        
        // Buffer record
        self.buffer.lock().push(record.clone());
        
        // Invoke callback if registered
        if let Some(ref callback) = self.emit_callback {
            callback(&record);
        }
        
        // Emit to stderr for local debugging
        if let Ok(json) = serde_json::to_string(&record) {
            eprintln!("[TELEMETRY] {}", json);
        }
    }
    
    /// Retrieves buffered telemetry records (for batch export).
    pub fn drain_buffer(&self) -> Vec<TelemetryRecord> {
        let mut buffer = self.buffer.lock();
        std::mem::take(&mut *buffer)
    }
    
    /// Exports telemetry buffer as JSON array.
    pub fn export_json(&self) -> Result<String, serde_json::Error> {
        let buffer = self.buffer.lock();
        serde_json::to_string_pretty(&*buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_telemetry_record_creation() {
        let record = TelemetryRecord::new("SCG-01", 5e-11, 0.98, 1.0, 0.0001);
        
        assert_eq!(record.cluster_id, "SCG-01");
        assert_eq!(record.energy_drift, 5e-11);
        assert_eq!(record.coherence, 0.98);
        assert_eq!(record.esv_valid_ratio, 1.0);
        assert_eq!(record.entropy_index, 0.0001);
        assert!(record.has_violation().is_none());
    }
    
    #[test]
    fn test_energy_drift_violation_detection() {
        let record = TelemetryRecord::new("SCG-01", 5e-9, 0.98, 1.0, 0.0001);
        
        assert!(matches!(
            record.has_violation(),
            Some(TelemetryViolation::EnergyDrift { .. })
        ));
    }
    
    #[test]
    fn test_coherence_violation_detection() {
        let record = TelemetryRecord::new("SCG-01", 1e-11, 0.95, 1.0, 0.0001);
        
        assert!(matches!(
            record.has_violation(),
            Some(TelemetryViolation::CoherenceLow { .. })
        ));
    }
    
    #[test]
    fn test_esv_violation_detection() {
        let record = TelemetryRecord::new("SCG-01", 1e-11, 0.98, 0.99, 0.0001);
        
        assert!(matches!(
            record.has_violation(),
            Some(TelemetryViolation::EsvFailure { .. })
        ));
    }
    
    #[test]
    fn test_emitter_buffering() {
        let emitter = TelemetryEmitter::new("SCG-TEST");
        
        emitter.emit(1e-11, 0.98, 1.0, 0.0001);
        emitter.emit(2e-11, 0.99, 1.0, 0.0002);
        
        let buffer = emitter.drain_buffer();
        assert_eq!(buffer.len(), 2);
        
        // Buffer should be empty after drain
        let buffer2 = emitter.drain_buffer();
        assert_eq!(buffer2.len(), 0);
    }
}
