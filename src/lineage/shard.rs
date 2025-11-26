use crate::lineage::LineageEntry;
/// SCG Substrate: Lineage Shard Boundary Manager
///
/// Formalizes shard rotation semantics:
/// - Shard rotates at completion of operation N (no partial operations)
/// - Entries always belong to shard in which they began
/// - Global hash construction uses ascending shard order
///
/// Shard rotation interval: N = 250 operations
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[allow(dead_code)]
pub const SHARD_ROTATION_INTERVAL: usize = 250;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct LineageShard {
    pub shard_id: u64,
    pub operations_start: u64,
    pub operations_end: u64,
    pub entries: Vec<LineageEntry>,
    pub shard_hash: String,
    pub created_at: String,
    pub finalized_at: Option<String>,
}

#[allow(dead_code)]
impl LineageShard {
    /// Creates a new shard starting at operation number
    pub fn new(shard_id: u64, operations_start: u64) -> Self {
        Self {
            shard_id,
            operations_start,
            operations_end: operations_start,
            entries: Vec::new(),
            shard_hash: String::new(),
            created_at: chrono::Utc::now().to_rfc3339(),
            finalized_at: None,
        }
    }

    /// Adds an entry to the shard (must be within operation bounds)
    pub fn add_entry(&mut self, entry: LineageEntry, operation_number: u64) -> Result<(), String> {
        if operation_number < self.operations_start {
            return Err(format!(
                "Operation {} is before shard start {}",
                operation_number, self.operations_start
            ));
        }

        if self.is_finalized() {
            return Err("Cannot add entries to finalized shard".into());
        }

        self.entries.push(entry);
        self.operations_end = operation_number;

        Ok(())
    }

    /// Finalizes the shard and computes its hash
    pub fn finalize(&mut self) {
        if self.is_finalized() {
            return;
        }

        // Compute shard hash: SHA256 of all entry hashes
        let mut hasher = Sha256::new();

        for entry in &self.entries {
            hasher.update(entry.operation_hash.as_bytes());
        }

        self.shard_hash = format!("{:x}", hasher.finalize());
        self.finalized_at = Some(chrono::Utc::now().to_rfc3339());

        eprintln!(
            "[SHARD_FINALIZED] Shard {} (ops {}-{}) hash: {}",
            self.shard_id, self.operations_start, self.operations_end, self.shard_hash
        );
    }

    /// Checks if shard is finalized
    pub fn is_finalized(&self) -> bool {
        self.finalized_at.is_some()
    }

    /// Returns the number of operations in this shard
    pub fn operation_count(&self) -> usize {
        self.entries.len()
    }
}

/// Manages lineage shard rotation and global hash construction
#[allow(dead_code)]
pub struct ShardManager {
    shards: parking_lot::Mutex<Vec<LineageShard>>,
    current_shard: parking_lot::Mutex<Option<LineageShard>>,
    operation_counter: parking_lot::Mutex<u64>,
}

#[allow(dead_code)]
impl ShardManager {
    pub fn new() -> Self {
        Self {
            shards: parking_lot::Mutex::new(Vec::new()),
            current_shard: parking_lot::Mutex::new(Some(LineageShard::new(0, 0))),
            operation_counter: parking_lot::Mutex::new(0),
        }
    }

    /// Adds an entry and handles shard rotation if needed
    pub fn add_entry(&self, entry: LineageEntry) -> Result<(), String> {
        let mut counter = self.operation_counter.lock();
        let operation_number = *counter;
        *counter += 1;

        let mut current_shard = self.current_shard.lock();
        let mut shard = current_shard.take().ok_or("No current shard")?;

        // Add entry to current shard
        shard.add_entry(entry, operation_number)?;

        // Check if shard should rotate (at completion of operation N)
        if shard.operation_count() >= SHARD_ROTATION_INTERVAL {
            // Finalize current shard
            shard.finalize();

            // Archive finalized shard
            let shard_id = shard.shard_id;
            self.shards.lock().push(shard);

            // Create new shard
            let new_shard = LineageShard::new(shard_id + 1, operation_number + 1);
            *current_shard = Some(new_shard);
        } else {
            *current_shard = Some(shard);
        }

        Ok(())
    }

    /// Computes global hash from all shards (oldest to newest)
    pub fn compute_global_hash(&self) -> String {
        let shards = self.shards.lock();
        let current = self.current_shard.lock();

        let mut hasher = Sha256::new();

        // Hash finalized shards in order
        for shard in shards.iter() {
            hasher.update(shard.shard_hash.as_bytes());
        }

        // Hash current shard entries
        if let Some(ref shard) = *current {
            for entry in &shard.entries {
                hasher.update(entry.operation_hash.as_bytes());
            }
        }

        format!("{:x}", hasher.finalize())
    }

    /// Returns all finalized shards
    pub fn get_finalized_shards(&self) -> Vec<LineageShard> {
        self.shards.lock().clone()
    }

    /// Returns current shard (not yet finalized)
    pub fn get_current_shard(&self) -> Option<LineageShard> {
        self.current_shard.lock().clone()
    }

    /// Exports shard metadata for audit
    pub fn export_shard_metadata(&self) -> Result<String, serde_json::Error> {
        #[derive(Serialize)]
        struct ShardMetadata {
            total_shards: usize,
            finalized_shards: usize,
            current_shard_operations: usize,
            total_operations: u64,
            global_hash: String,
        }

        let shards = self.shards.lock();
        let current = self.current_shard.lock();
        let counter = self.operation_counter.lock();

        let metadata = ShardMetadata {
            total_shards: shards.len() + 1,
            finalized_shards: shards.len(),
            current_shard_operations: current.as_ref().map(|s| s.operation_count()).unwrap_or(0),
            total_operations: *counter,
            global_hash: self.compute_global_hash(),
        };

        serde_json::to_string_pretty(&metadata)
    }
}

impl Default for ShardManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shard_rotation() {
        let manager = ShardManager::new();

        // Add entries up to rotation threshold
        for i in 0..SHARD_ROTATION_INTERVAL {
            let entry = LineageEntry {
                id: uuid::Uuid::new_v4(),
                timestamp: chrono::Utc::now().to_rfc3339(),
                operation: format!("op_{}", i),
                operation_hash: format!("hash_{}", i),
                params: serde_json::json!({}),
            };

            manager.add_entry(entry).unwrap();
        }

        // Should have finalized one shard
        assert_eq!(manager.get_finalized_shards().len(), 1);

        // Current shard should be new
        let current = manager.get_current_shard().unwrap();
        assert_eq!(current.shard_id, 1);
        assert_eq!(current.operations_start, SHARD_ROTATION_INTERVAL as u64);
    }

    #[test]
    fn test_global_hash_construction() {
        let manager = ShardManager::new();

        // Add some entries
        for i in 0..10 {
            let entry = LineageEntry {
                id: uuid::Uuid::new_v4(),
                timestamp: chrono::Utc::now().to_rfc3339(),
                operation: format!("op_{}", i),
                operation_hash: format!("hash_{}", i),
                params: serde_json::json!({}),
            };

            manager.add_entry(entry).unwrap();
        }

        // Global hash should be deterministic
        let hash1 = manager.compute_global_hash();
        let hash2 = manager.compute_global_hash();

        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64); // SHA256 hex length
    }

    #[test]
    fn test_shard_finalization() {
        let mut shard = LineageShard::new(0, 0);

        assert!(!shard.is_finalized());

        shard.finalize();

        assert!(shard.is_finalized());
        assert!(!shard.shard_hash.is_empty());
        assert!(shard.finalized_at.is_some());
    }
}
