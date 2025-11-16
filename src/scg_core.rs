use crate::types::{EdgeState, GovernorStatus, LineageEntry, NodeState};
use parking_lot::RwLock;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

pub struct ScgRuntimeInner {
    pub nodes: HashMap<Uuid, NodeState>,
    pub edges: HashMap<Uuid, EdgeState>,
    pub lineage: Vec<LineageEntry>,
    pub total_energy: f64,
    pub esv_threshold: f64,
}

impl Default for ScgRuntimeInner {
    fn default() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            lineage: Vec::new(),
            total_energy: 0.0,
            esv_threshold: 1.0, // default ESV threshold
        }
    }
}

#[derive(Clone, Default)]
pub struct ScgRuntime {
    inner: Arc<RwLock<ScgRuntimeInner>>,
}

impl ScgRuntime {
    pub const DRIFT_THRESHOLD: f64 = 1e-10;

    pub fn new() -> Self {
        Self::default()
    }

    // ESV threshold API
    pub fn set_esv_threshold(&self, value: f64) {
        self.inner.write().esv_threshold = value;
    }

    pub fn get_esv_threshold(&self) -> f64 {
        self.inner.read().esv_threshold
    }

    // Lineage export
    pub fn export_lineage_to_file(&self, path: &str) -> Result<String, String> {
        let inner = self.inner.read();
        let serialized =
            serde_json::to_string(&inner.lineage).map_err(|e| e.to_string())?;
        std::fs::write(path, &serialized).map_err(|e| e.to_string())?;
        let mut hasher = Sha256::new();
        hasher.update(serialized.as_bytes());
        Ok(format!("{:x}", hasher.finalize()))
    }

    fn append_lineage(inner: &mut ScgRuntimeInner, op: &str) {
        let mut hasher = Sha256::new();
        hasher.update(op.as_bytes());
        if let Some(last) = inner.lineage.last() {
            hasher.update(last.checksum.as_bytes());
        }
        let checksum = format!("{:x}", hasher.finalize());
        inner.lineage.push(LineageEntry {
            id: Uuid::new_v4(),
            op: op.to_owned(),
            checksum,
        });
    }

    pub fn node_create(&self, belief: f64, energy: f64) -> NodeState {
        let mut inner = self.inner.write();
        let id = Uuid::new_v4();
        let node = NodeState {
            id,
            belief,
            energy,
            esv_valid: true,
        };
        inner.total_energy += energy;
        inner.nodes.insert(id, node.clone());
        Self::append_lineage(&mut inner, &format!("node.create:{}", id));
        node
    }

    pub fn node_mutate(&self, id: Uuid, delta: f64) -> Result<NodeState, String> {
        let mut inner = self.inner.write();

        // First mutable borrow
        let entry = inner.nodes.get_mut(&id).ok_or("Node not found".to_owned())?;
        entry.belief += delta;

        // Clone BEFORE append_lineage to release the borrow
        let out = entry.clone();

        // Now safe to borrow `inner` mutably again
        Self::append_lineage(&mut inner, &format!("node.mutate:{}", id));

        Ok(out)
    }

    pub fn node_query(&self, id: Uuid) -> Option<NodeState> {
        let inner = self.inner.read();
        inner.nodes.get(&id).cloned()
    }

    pub fn edge_bind(&self, src: Uuid, dst: Uuid, weight: f64) -> Result<EdgeState, String> {
        let mut inner = self.inner.write();
        if !inner.nodes.contains_key(&src) || !inner.nodes.contains_key(&dst) {
            return Err("Source or destination not found".into());
        }
        let id = Uuid::new_v4();
        let edge = EdgeState { id, src, dst, weight };
        inner.edges.insert(id, edge.clone());
        Self::append_lineage(&mut inner, &format!("edge.bind:{}", id));
        Ok(edge)
    }

    pub fn edge_propagate(&self, edge_id: Uuid) -> Result<(), String> {
        let mut inner = self.inner.write();

        // First borrow
        let edge = inner.edges.get(&edge_id).ok_or("Edge not found".to_owned())?.clone();

        let src = inner.nodes.get(&edge.src).ok_or("Source missing".to_owned())?.clone();
        {
            let dst = inner.nodes.get_mut(&edge.dst).ok_or("Dest missing".to_owned())?;
            dst.belief += src.belief * edge.weight;
        }

        // Safe to mutate lineage now
        Self::append_lineage(&mut inner, &format!("edge.propagate:{}", edge_id));
        Ok(())
    }

    pub fn governor_status(&self) -> GovernorStatus {
        let inner = self.inner.read();
        GovernorStatus {
            energy_drift: 0.0,
            coherence: 1.0,
            node_count: inner.nodes.len(),
            edge_count: inner.edges.len(),
        }
    }

    pub fn esv_audit(&self, id: Uuid) -> Result<bool,String> {
        let inner = self.inner.read();
        Ok(inner.nodes.get(&id).ok_or("Node not found".to_owned())?.esv_valid)
    }

    pub fn replay_lineage(&self) -> LineageEntry {
        let inner = self.inner.read();
        inner.lineage.last().cloned().unwrap_or(
            LineageEntry {
                id: Uuid::nil(),
                op: "empty".into(),
                checksum: "0".repeat(64),
            }
        )
    }

    pub fn energy_drift_ok(&self) -> bool {
        self.governor_status().energy_drift.abs() <= Self::DRIFT_THRESHOLD
    }
}
