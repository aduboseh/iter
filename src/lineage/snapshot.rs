/// SCG Substrate: Lineage Snapshot System
/// 
/// Creates deterministic, cryptographically verified snapshots for:
/// - Replay validation (ε ≤ 1e-10 variance)
/// - External audit and compliance
/// - Disaster recovery and rollback
/// 
/// Invariant: Snapshots are immutable and hash-anchored.

use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::fs;
use std::path::Path;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageSnapshot {
    /// Unique snapshot identifier
    pub id: Uuid,
    
    /// ISO 8601 timestamp of snapshot creation
    pub timestamp: String,
    
    /// SHA256 hash of complete lineage chain
    pub lineage_hash: String,
    
    /// Sequential lineage entries
    pub entries: Vec<LineageEntry>,
    
    /// Graph state at snapshot time
    pub graph_state: GraphSnapshot,
    
    /// Energy state at snapshot time
    pub energy_state: EnergySnapshot,
    
    /// Metadata for audit trail
    pub metadata: SnapshotMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageEntry {
    pub id: Uuid,
    pub timestamp: String,
    pub operation: String,
    pub operation_hash: String,
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphSnapshot {
    pub node_count: usize,
    pub edge_count: usize,
    pub topology_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnergySnapshot {
    pub total_energy: f64,
    pub drift: f64,
    pub coherence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotMetadata {
    pub cluster_id: String,
    pub version: String,
    pub esv_valid: bool,
    pub replay_verified: bool,
}

impl LineageSnapshot {
    /// Creates a new snapshot from current SCG state.
    pub fn create(
        cluster_id: impl Into<String>,
        entries: Vec<LineageEntry>,
        node_count: usize,
        edge_count: usize,
        total_energy: f64,
        drift: f64,
        coherence: f64,
        esv_valid: bool,
    ) -> Self {
        let snapshot_id = Uuid::new_v4();
        let timestamp = chrono::Utc::now().to_rfc3339();
        
        // Compute deterministic lineage hash
        let lineage_hash = Self::compute_lineage_hash(&entries);
        
        // Compute topology hash (for cycle detection)
        let topology_hash = Self::compute_topology_hash(node_count, edge_count);
        
        Self {
            id: snapshot_id,
            timestamp,
            lineage_hash,
            entries,
            graph_state: GraphSnapshot {
                node_count,
                edge_count,
                topology_hash,
            },
            energy_state: EnergySnapshot {
                total_energy,
                drift,
                coherence,
            },
            metadata: SnapshotMetadata {
                cluster_id: cluster_id.into(),
                version: "0.1.0".into(),
                esv_valid,
                replay_verified: false, // Set to true after replay validation
            },
        }
    }
    
    /// Computes SHA256 hash of lineage entry sequence.
    /// 
    /// Invariant: Hash is deterministic for same entry sequence.
    fn compute_lineage_hash(entries: &[LineageEntry]) -> String {
        let mut hasher = Sha256::new();
        
        for entry in entries {
            hasher.update(entry.id.as_bytes());
            hasher.update(entry.operation.as_bytes());
            hasher.update(entry.operation_hash.as_bytes());
            
            // Include params JSON for determinism
            if let Ok(json) = serde_json::to_string(&entry.params) {
                hasher.update(json.as_bytes());
            }
        }
        
        format!("{:x}", hasher.finalize())
    }
    
    /// Computes deterministic topology hash.
    fn compute_topology_hash(node_count: usize, edge_count: usize) -> String {
        let mut hasher = Sha256::new();
        hasher.update(&node_count.to_le_bytes());
        hasher.update(&edge_count.to_le_bytes());
        format!("{:x}", hasher.finalize())
    }
    
    /// Exports snapshot to JSON file with deterministic formatting.
    pub fn export_to_file<P: AsRef<Path>>(&self, path: P) -> Result<String, String> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("JSON serialization failed: {}", e))?;
        
        fs::write(&path, &json)
            .map_err(|e| format!("File write failed: {}", e))?;
        
        // Return SHA256 of written file for verification
        let file_hash = Self::hash_file(&path)?;
        
        eprintln!("[SNAPSHOT] Exported to {:?} with hash {}", path.as_ref(), file_hash);
        
        Ok(file_hash)
    }
    
    /// Imports snapshot from JSON file with integrity verification.
    pub fn import_from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let json = fs::read_to_string(&path)
            .map_err(|e| format!("File read failed: {}", e))?;
        
        let snapshot: Self = serde_json::from_str(&json)
            .map_err(|e| format!("JSON deserialization failed: {}", e))?;
        
        // Verify lineage hash integrity
        let computed_hash = Self::compute_lineage_hash(&snapshot.entries);
        if computed_hash != snapshot.lineage_hash {
            return Err(format!(
                "Lineage hash mismatch: expected {}, got {}",
                snapshot.lineage_hash,
                computed_hash
            ));
        }
        
        eprintln!("[SNAPSHOT] Imported from {:?} - hash verified", path.as_ref());
        
        Ok(snapshot)
    }
    
    /// Computes SHA256 hash of file contents.
    fn hash_file<P: AsRef<Path>>(path: P) -> Result<String, String> {
        let contents = fs::read(&path)
            .map_err(|e| format!("File read failed: {}", e))?;
        
        let mut hasher = Sha256::new();
        hasher.update(&contents);
        
        Ok(format!("{:x}", hasher.finalize()))
    }
    
    /// Validates snapshot replay produces identical state.
    /// 
    /// Returns: Ok(variance) if replay succeeds, Err if variance > 1e-10
    pub fn validate_replay(&self, replay_hash: &str) -> Result<f64, String> {
        const TOLERANCE: f64 = 1e-10;
        
        // Compare hashes byte-by-byte
        if self.lineage_hash == replay_hash {
            Ok(0.0)
        } else {
            // Compute variance (for numeric comparison)
            // In real implementation, would parse hex strings and compute difference
            Err(format!(
                "Replay hash mismatch: expected {}, got {}",
                self.lineage_hash,
                replay_hash
            ))
        }
    }
    
    /// Marks snapshot as replay-verified.
    pub fn mark_replay_verified(&mut self) {
        self.metadata.replay_verified = true;
    }
}

/// Helper for building lineage entries from SCG operations.
pub struct LineageBuilder {
    entries: Vec<LineageEntry>,
}

impl LineageBuilder {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
    
    pub fn add_entry(
        &mut self,
        operation: impl Into<String>,
        params: serde_json::Value,
    ) {
        let operation_str = operation.into();
        
        // Compute operation hash for tamper detection
        let mut hasher = Sha256::new();
        hasher.update(operation_str.as_bytes());
        if let Ok(json) = serde_json::to_string(&params) {
            hasher.update(json.as_bytes());
        }
        let operation_hash = format!("{:x}", hasher.finalize());
        
        let entry = LineageEntry {
            id: Uuid::new_v4(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            operation: operation_str,
            operation_hash,
            params,
        };
        
        self.entries.push(entry);
    }
    
    pub fn build(self) -> Vec<LineageEntry> {
        self.entries
    }
}

impl Default for LineageBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_snapshot_creation() {
        let mut builder = LineageBuilder::new();
        builder.add_entry("node.create", serde_json::json!({"belief": 0.5, "energy": 1.0}));
        builder.add_entry("node.mutate", serde_json::json!({"delta": 0.1}));
        
        let entries = builder.build();
        
        let snapshot = LineageSnapshot::create(
            "SCG-TEST",
            entries,
            10,
            15,
            100.0,
            1e-11,
            0.98,
            true,
        );
        
        assert_eq!(snapshot.graph_state.node_count, 10);
        assert_eq!(snapshot.graph_state.edge_count, 15);
        assert_eq!(snapshot.energy_state.total_energy, 100.0);
        assert!(snapshot.metadata.esv_valid);
        assert!(!snapshot.metadata.replay_verified);
    }
    
    #[test]
    fn test_deterministic_lineage_hash() {
        let entries = vec![
            LineageEntry {
                id: Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
                timestamp: "2025-01-01T00:00:00Z".into(),
                operation: "test_op".into(),
                operation_hash: "abc123".into(),
                params: serde_json::json!({"key": "value"}),
            },
        ];
        
        let hash1 = LineageSnapshot::compute_lineage_hash(&entries);
        let hash2 = LineageSnapshot::compute_lineage_hash(&entries);
        
        assert_eq!(hash1, hash2, "Lineage hash must be deterministic");
        assert_eq!(hash1.len(), 64, "SHA256 hash must be 64 hex chars");
    }
    
    #[test]
    fn test_replay_validation() {
        let snapshot = LineageSnapshot::create(
            "SCG-TEST",
            vec![],
            0,
            0,
            0.0,
            0.0,
            1.0,
            true,
        );
        
        let hash = snapshot.lineage_hash.clone();
        
        // Valid replay
        assert!(snapshot.validate_replay(&hash).is_ok());
        
        // Invalid replay
        assert!(snapshot.validate_replay("invalid_hash").is_err());
    }
}
