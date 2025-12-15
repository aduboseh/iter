//! Public stub substrate for demonstration and protocol validation.
//!
//! This module provides deterministic placeholder responses that demonstrate
//! the MCP interface contract without executing real cognitive operations.
//!
//! # Design
//!
//! - Energy and belief are fixed placeholder values
//! - Lineage hashes are derived deterministically from inputs
//! - No internal topology or substrate mechanics are exposed
//! - Responses are MCP schema-compliant (no extra fields)

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

/// Counter for generating sequential IDs
static NODE_COUNTER: AtomicU64 = AtomicU64::new(0);
static EDGE_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Stub node state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StubNode {
    pub id: u64,
    pub belief: f64,
    pub energy: f64,
    pub esv_valid: bool,
}

/// Stub edge state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StubEdge {
    pub id: u64,
    pub src: u64,
    pub dst: u64,
    pub weight: f64,
}

/// Stub runtime for public demonstration
pub struct StubRuntime {
    nodes: HashMap<u64, StubNode>,
    edges: HashMap<u64, StubEdge>,
    lineage: Vec<LineageEntry>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LineageEntry {
    pub sequence: u64,
    pub operation: String,
    pub checksum: String,
}

impl Default for StubRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl StubRuntime {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            lineage: Vec::new(),
        }
    }

    /// Create a node with placeholder values
    pub fn create_node(&mut self, belief: f64, energy: f64) -> StubNode {
        let id = NODE_COUNTER.fetch_add(1, Ordering::SeqCst);
        let node = StubNode {
            id,
            belief: belief.clamp(0.0, 1.0),
            energy: energy.max(0.0),
            esv_valid: true, // Stub always reports valid
        };
        self.nodes.insert(id, node.clone());
        self.record_lineage("node.create", &format!("id:{}", id));
        node
    }

    /// Query a node by ID
    pub fn query_node(&self, id: u64) -> Option<&StubNode> {
        self.nodes.get(&id)
    }

    /// Mutate a node's belief
    pub fn mutate_node(&mut self, id: u64, delta: f64) -> Option<StubNode> {
        // Check if node exists first
        if !self.nodes.contains_key(&id) {
            return None;
        }

        // Update node
        let node = self.nodes.get_mut(&id).unwrap();
        node.belief = (node.belief + delta).clamp(0.0, 1.0);
        let result = node.clone();

        // Record lineage after mutation is complete
        self.record_lineage("node.mutate", &format!("id:{},delta:{}", id, delta));
        Some(result)
    }

    /// Bind an edge between nodes
    pub fn bind_edge(&mut self, src: u64, dst: u64, weight: f64) -> Option<StubEdge> {
        if !self.nodes.contains_key(&src) || !self.nodes.contains_key(&dst) {
            return None;
        }
        let id = EDGE_COUNTER.fetch_add(1, Ordering::SeqCst);
        let edge = StubEdge {
            id,
            src,
            dst,
            weight: weight.clamp(0.0, 1.0),
        };
        self.edges.insert(id, edge.clone());
        self.record_lineage("edge.bind", &format!("{}→{}", src, dst));
        Some(edge)
    }

    /// Run a propagation step (stub: no-op with deterministic response)
    pub fn propagate(&mut self) -> String {
        self.record_lineage("edge.propagate", "step");
        "Propagation step executed (stub mode)".to_string()
    }

    /// Get governor status (stub: always healthy)
    pub fn governor_status(&self) -> GovernorStatus {
        GovernorStatus {
            drift_ok: true,
            energy_drift: 0.0,
            coherence: 1.0,
            node_count: self.nodes.len(),
            edge_count: self.edges.len(),
            healthy: true,
        }
    }

    /// ESV audit (stub: always valid)
    pub fn esv_audit(&self, node_id: u64) -> Option<EsvAudit> {
        self.nodes.get(&node_id).map(|_| EsvAudit {
            node_id,
            valid: true,
            compliance_status: "compliant".to_string(),
        })
    }

    /// Get lineage entries
    pub fn lineage_replay(&self) -> &[LineageEntry] {
        &self.lineage
    }

    fn record_lineage(&mut self, operation: &str, data: &str) {
        let sequence = self.lineage.len() as u64;
        let checksum = compute_stable_hash(&format!("{}:{}:{}", sequence, operation, data));
        self.lineage.push(LineageEntry {
            sequence,
            operation: operation.to_string(),
            checksum,
        });
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct GovernorStatus {
    pub drift_ok: bool,
    pub energy_drift: f64,
    pub coherence: f64,
    pub node_count: usize,
    pub edge_count: usize,
    pub healthy: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct EsvAudit {
    pub node_id: u64,
    pub valid: bool,
    pub compliance_status: String,
}

fn compute_stable_hash(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_is_deterministic() {
        let mut rt1 = StubRuntime::new();
        let mut rt2 = StubRuntime::new();

        let n1 = rt1.create_node(0.5, 100.0);
        let n2 = rt2.create_node(0.5, 100.0);

        // IDs may differ due to global counter, but behavior is deterministic
        assert_eq!(n1.belief, n2.belief);
        assert_eq!(n1.energy, n2.energy);
    }

    #[test]
    fn stub_clamps_belief() {
        let mut rt = StubRuntime::new();
        let node = rt.create_node(1.5, 100.0);
        assert_eq!(node.belief, 1.0);

        let node2 = rt.create_node(-0.5, 100.0);
        assert_eq!(node2.belief, 0.0);
    }

    #[test]
    fn stub_response_schema_compliant() {
        let rt = StubRuntime::new();
        let status = rt.governor_status();

        // Ensure no extra fields via serialization
        let json = serde_json::to_value(&status).unwrap();
        let obj = json.as_object().unwrap();

        // Only expected fields
        assert!(obj.contains_key("drift_ok"));
        assert!(obj.contains_key("healthy"));
        assert!(!obj.contains_key("_mode")); // No mode field
    }
}
