use crate::types::{EdgeState, GovernorStatus, LineageEntry, NodeState};
use crate::telemetry::TelemetryEmitter;
use crate::fault::{QuarantineController, QuarantineReason, error_codes};
use crate::lineage::LineageBuilder;
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
    pub initial_energy: f64,
    pub esv_threshold: f64,
    pub operation_count: usize,
}

impl Default for ScgRuntimeInner {
    fn default() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            lineage: Vec::new(),
            total_energy: 0.0,
            initial_energy: 0.0,
            esv_threshold: 1.0, // default ESV threshold
            operation_count: 0,
        }
    }
}

#[derive(Clone)]
pub struct ScgRuntime {
    inner: Arc<RwLock<ScgRuntimeInner>>,
    telemetry: Arc<TelemetryEmitter>,
    quarantine: Arc<QuarantineController>,
}

impl Default for ScgRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl ScgRuntime {
    pub const DRIFT_THRESHOLD: f64 = 1e-10;
    pub const COHERENCE_THRESHOLD: f64 = 0.97;

    pub fn new() -> Self {
        let cluster_id = std::env::var("SCG_CLUSTER_ID").unwrap_or_else(|_| "SCG-RUNTIME-01".to_string());
        
        Self {
            inner: Arc::new(RwLock::new(ScgRuntimeInner::default())),
            telemetry: Arc::new(TelemetryEmitter::new(cluster_id)),
            quarantine: Arc::new(QuarantineController::new()),
        }
    }
    
    /// Checks if system is quarantined
    pub fn is_quarantined(&self) -> bool {
        self.quarantine.is_quarantined()
    }
    
    /// Computes current energy drift
    fn compute_energy_drift(&self, inner: &ScgRuntimeInner) -> f64 {
        if inner.initial_energy == 0.0 {
            return 0.0;
        }
        (inner.total_energy - inner.initial_energy).abs()
    }
    
    /// Computes coherence index
    fn compute_coherence(&self, inner: &ScgRuntimeInner) -> f64 {
        if inner.nodes.is_empty() {
            return 1.0;
        }
        
        // Coherence = ratio of ESV-valid nodes
        let valid_count = inner.nodes.values().filter(|n| n.esv_valid).count();
        valid_count as f64 / inner.nodes.len() as f64
    }
    
    /// Emits telemetry and checks for violations
    fn emit_telemetry_and_check(&self, inner: &ScgRuntimeInner) {
        let drift = self.compute_energy_drift(inner);
        let coherence = self.compute_coherence(inner);
        let esv_valid_ratio = coherence; // Same calculation for now
        let entropy_index = drift; // Simplified: use drift as entropy proxy
        
        self.telemetry.emit(drift, coherence, esv_valid_ratio, entropy_index);
        
        // Check for violations and trigger quarantine
        if drift > Self::DRIFT_THRESHOLD {
            eprintln!("[SCG] CRITICAL: Energy drift exceeded: {} > {}", drift, Self::DRIFT_THRESHOLD);
            self.quarantine.enter_quarantine(
                QuarantineReason::EnergyDriftExceeded {
                    drift,
                    threshold: Self::DRIFT_THRESHOLD,
                },
                None,
            );
        }
        
        if coherence < Self::COHERENCE_THRESHOLD {
            eprintln!("[SCG] CRITICAL: Coherence below threshold: {} < {}", coherence, Self::COHERENCE_THRESHOLD);
            self.quarantine.enter_quarantine(
                QuarantineReason::EsvViolation {
                    node_id: Uuid::nil(),
                    checksum_mismatch: format!("Coherence: {}", coherence),
                },
                None,
            );
        }
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
        // Check quarantine
        if self.is_quarantined() {
            eprintln!("[SCG] Operation blocked: System is quarantined");
            return NodeState {
                id: Uuid::nil(),
                belief: 0.0,
                energy: 0.0,
                esv_valid: false,
            };
        }
        
        let mut inner = self.inner.write();
        
        // Set initial energy on first node
        if inner.nodes.is_empty() {
            inner.initial_energy = energy;
        }
        
        let id = Uuid::new_v4();
        let node = NodeState {
            id,
            belief,
            energy,
            esv_valid: true,
        };
        inner.total_energy += energy;
        inner.nodes.insert(id, node.clone());
        inner.operation_count += 1;
        Self::append_lineage(&mut inner, &format!("node.create:{}", id));
        
        // Emit telemetry and check invariants
        self.emit_telemetry_and_check(&inner);
        
        node
    }

    pub fn node_mutate(&self, id: Uuid, delta: f64) -> Result<NodeState, String> {
        // Check quarantine
        if self.is_quarantined() {
            return Err("System is quarantined".to_string());
        }
        
        let mut inner = self.inner.write();

        // First mutable borrow
        let entry = inner.nodes.get_mut(&id).ok_or("Node not found".to_owned())?;
        entry.belief += delta;

        // Clone BEFORE append_lineage to release the borrow
        let out = entry.clone();

        // Now safe to borrow `inner` mutably again
        inner.operation_count += 1;
        Self::append_lineage(&mut inner, &format!("node.mutate:{}", id));
        
        // Emit telemetry and check invariants
        self.emit_telemetry_and_check(&inner);

        Ok(out)
    }

    pub fn node_query(&self, id: Uuid) -> Option<NodeState> {
        let inner = self.inner.read();
        inner.nodes.get(&id).cloned()
    }

    pub fn edge_bind(&self, src: Uuid, dst: Uuid, weight: f64) -> Result<EdgeState, String> {
        // Check quarantine
        if self.is_quarantined() {
            return Err("System is quarantined".to_string());
        }
        
        let mut inner = self.inner.write();
        if !inner.nodes.contains_key(&src) || !inner.nodes.contains_key(&dst) {
            return Err("Source or destination not found".into());
        }
        let id = Uuid::new_v4();
        let edge = EdgeState { id, src, dst, weight };
        inner.edges.insert(id, edge.clone());
        inner.operation_count += 1;
        Self::append_lineage(&mut inner, &format!("edge.bind:{}", id));
        
        // Emit telemetry
        self.emit_telemetry_and_check(&inner);
        
        Ok(edge)
    }

    pub fn edge_propagate(&self, edge_id: Uuid) -> Result<(), String> {
        // Check quarantine
        if self.is_quarantined() {
            return Err("System is quarantined".to_string());
        }
        
        let mut inner = self.inner.write();

        // First borrow
        let edge = inner.edges.get(&edge_id).ok_or("Edge not found".to_owned())?.clone();

        let src = inner.nodes.get(&edge.src).ok_or("Source missing".to_owned())?.clone();
        {
            let dst = inner.nodes.get_mut(&edge.dst).ok_or("Dest missing".to_owned())?;
            dst.belief += src.belief * edge.weight;
        }

        // Safe to mutate lineage now
        inner.operation_count += 1;
        Self::append_lineage(&mut inner, &format!("edge.propagate:{}", edge_id));
        
        // Emit telemetry and check invariants
        self.emit_telemetry_and_check(&inner);
        
        Ok(())
    }

    pub fn governor_status(&self) -> GovernorStatus {
        let inner = self.inner.read();
        let drift = self.compute_energy_drift(&inner);
        let coherence = self.compute_coherence(&inner);
        
        GovernorStatus {
            energy_drift: drift,
            coherence,
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
