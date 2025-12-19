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

/// Derived state for reference propagation artifact.
///
/// This is a deterministic summary of substrate state computed during propagation.
/// It demonstrates conservation and replay integrity without exposing kernel logic.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DerivedState {
    /// Total number of nodes
    pub node_count: usize,
    /// Total number of edges
    pub edge_count: usize,
    /// Sum of all node energy values
    pub total_energy: f64,
    /// Arithmetic mean of all node belief values
    pub mean_belief: f64,
}

/// Reference propagation artifact returned by edge.propagate in stub mode.
///
/// This artifact enables deterministic replay verification without exposing
/// any proprietary kernel logic or weighted dynamics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropagationArtifact {
    /// Stable decision identifier (derived from lineage sequence)
    pub decision_id: String,
    /// Deterministic derived state summary
    pub derived_state: DerivedState,
    /// SHA-256 checksum of serialized derived_state
    pub propagation_checksum: String,
    /// Mode label - always "reference-stub" for this artifact
    pub mode: String,
}

/// Stub runtime for public demonstration
pub struct StubRuntime {
    nodes: HashMap<u64, StubNode>,
    edges: HashMap<u64, StubEdge>,
    lineage: Vec<LineageEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageEntry {
    pub sequence: u64,
    pub operation: String,
    pub checksum: String,
    /// Optional propagation artifact attached for edge.propagate operations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub propagation_artifact: Option<PropagationArtifact>,
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

    /// Run a propagation step (stub: deterministic reference artifact)
    ///
    /// Returns a PropagationArtifact containing:
    /// - Deterministic derived state (node_count, edge_count, total_energy, mean_belief)
    /// - SHA-256 checksum for replay verification
    /// - Mode label "reference-stub"
    pub fn propagate(&mut self) -> PropagationArtifact {
        let derived_state = self.compute_derived_state();
        let propagation_checksum = Self::compute_checksum(&derived_state);
        let sequence = self.lineage.len() as u64;
        let decision_id = format!("prop-{}", sequence);

        let artifact = PropagationArtifact {
            decision_id: decision_id.clone(),
            derived_state,
            propagation_checksum,
            mode: "reference-stub".to_string(),
        };

        // Record lineage with attached artifact
        self.record_lineage_with_artifact("edge.propagate", "step", Some(artifact.clone()));

        artifact
    }

    /// Compute deterministic derived state from current substrate.
    ///
    /// Rules (per RPSU-01):
    /// - node_count = total nodes
    /// - edge_count = total edges
    /// - total_energy = sum of node.energy
    /// - mean_belief = arithmetic mean of node.belief
    /// - Fixed ordering, no randomness, no weights
    fn compute_derived_state(&self) -> DerivedState {
        let node_count = self.nodes.len();
        let edge_count = self.edges.len();

        // Sum energy and belief from nodes in deterministic order (sorted by ID)
        let mut node_ids: Vec<u64> = self.nodes.keys().copied().collect();
        node_ids.sort();

        let (total_energy, total_belief) = node_ids.iter().fold((0.0, 0.0), |(e, b), id| {
            let node = &self.nodes[id];
            (e + node.energy, b + node.belief)
        });

        let mean_belief = if node_count > 0 {
            total_belief / node_count as f64
        } else {
            0.0
        };

        DerivedState {
            node_count,
            edge_count,
            total_energy,
            mean_belief,
        }
    }

    /// Compute deterministic SHA-256 checksum of derived state.
    ///
    /// Rules (per RPSU-01):
    /// - Stable field order (guaranteed by struct definition)
    /// - Stable numeric formatting (serde_json default)
    /// - No timestamps
    /// - No environment dependencies
    fn compute_checksum(derived_state: &DerivedState) -> String {
        let bytes =
            serde_json::to_vec(derived_state).expect("DerivedState serialization is infallible");
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        format!("{:x}", hasher.finalize())
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
    pub fn lineage_entries(&self) -> &[LineageEntry] {
        &self.lineage
    }

    /// Replay lineage with verification.
    ///
    /// For edge.propagate entries with attached artifacts:
    /// - Recomputes derived_state from current substrate state
    /// - Recomputes checksum
    /// - Returns match/blocked status
    ///
    /// Per RPSU-01: Silent divergence is forbidden.
    pub fn lineage_replay(&self) -> Vec<ReplayResult> {
        self.lineage
            .iter()
            .map(|entry| {
                if entry.operation == "edge.propagate" {
                    if let Some(ref artifact) = entry.propagation_artifact {
                        // Recompute and verify
                        let current_derived = self.compute_derived_state();
                        let current_checksum = Self::compute_checksum(&current_derived);

                        if current_checksum == artifact.propagation_checksum {
                            ReplayResult {
                                decision_id: artifact.decision_id.clone(),
                                replay_status: ReplayStatus::Match,
                                propagation_checksum: Some(current_checksum),
                                reason: None,
                            }
                        } else {
                            ReplayResult {
                                decision_id: artifact.decision_id.clone(),
                                replay_status: ReplayStatus::Blocked,
                                propagation_checksum: Some(current_checksum),
                                reason: Some("derived_state_mismatch".to_string()),
                            }
                        }
                    } else {
                        // Legacy entry without artifact - treat as match for backwards compat
                        ReplayResult {
                            decision_id: format!("legacy-{}", entry.sequence),
                            replay_status: ReplayStatus::Match,
                            propagation_checksum: None,
                            reason: None,
                        }
                    }
                } else {
                    // Non-propagate entries always match
                    ReplayResult {
                        decision_id: format!("op-{}", entry.sequence),
                        replay_status: ReplayStatus::Match,
                        propagation_checksum: None,
                        reason: None,
                    }
                }
            })
            .collect()
    }

    fn record_lineage(&mut self, operation: &str, data: &str) {
        self.record_lineage_with_artifact(operation, data, None);
    }

    fn record_lineage_with_artifact(
        &mut self,
        operation: &str,
        data: &str,
        propagation_artifact: Option<PropagationArtifact>,
    ) {
        let sequence = self.lineage.len() as u64;
        let checksum = compute_stable_hash(&format!("{}:{}:{}", sequence, operation, data));
        self.lineage.push(LineageEntry {
            sequence,
            operation: operation.to_string(),
            checksum,
            propagation_artifact,
        });
    }
}

/// Result of replaying a single lineage entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayResult {
    /// Decision identifier from the original artifact
    pub decision_id: String,
    /// Replay verification status
    pub replay_status: ReplayStatus,
    /// Recomputed checksum (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub propagation_checksum: Option<String>,
    /// Reason for blocked status (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// Status of replay verification.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ReplayStatus {
    /// Checksum matches - state unchanged since recording
    Match,
    /// Checksum mismatch - structural mutation detected
    Blocked,
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

    // ========================================================================
    // RPSU-01 Tests: Reference Propagation Artifact
    // ========================================================================

    #[test]
    fn propagation_artifact_has_reference_stub_mode() {
        let mut rt = StubRuntime::new();
        rt.create_node(0.5, 100.0);
        rt.create_node(0.3, 50.0);

        let artifact = rt.propagate();

        // Per RPSU-01: Output must be labeled "reference-stub"
        assert_eq!(artifact.mode, "reference-stub");
    }

    #[test]
    fn propagation_artifact_contains_derived_state() {
        let mut rt = StubRuntime::new();
        rt.create_node(0.6, 100.0);
        rt.create_node(0.4, 50.0);
        rt.bind_edge(
            rt.nodes.keys().min().copied().unwrap(),
            rt.nodes.keys().max().copied().unwrap(),
            0.5,
        );

        let artifact = rt.propagate();

        // Verify derived state values
        assert_eq!(artifact.derived_state.node_count, 2);
        assert_eq!(artifact.derived_state.edge_count, 1);
        assert!((artifact.derived_state.total_energy - 150.0).abs() < 1e-10);
        assert!((artifact.derived_state.mean_belief - 0.5).abs() < 1e-10);
    }

    #[test]
    fn propagation_checksum_is_deterministic() {
        // Per RPSU-01: Checksums must be identical across multiple runs

        // Run 1
        let mut rt1 = StubRuntime::new();
        let n1_id = rt1.create_node(0.7, 100.0).id;
        let n2_id = rt1.create_node(0.3, 50.0).id;
        rt1.bind_edge(n1_id, n2_id, 0.8);
        let artifact1 = rt1.propagate();

        // Run 2 - fresh runtime, same operations
        let mut rt2 = StubRuntime::new();
        let n3_id = rt2.create_node(0.7, 100.0).id;
        let n4_id = rt2.create_node(0.3, 50.0).id;
        rt2.bind_edge(n3_id, n4_id, 0.8);
        let artifact2 = rt2.propagate();

        // Derived states must match
        assert_eq!(artifact1.derived_state, artifact2.derived_state);

        // Checksums must be identical
        assert_eq!(
            artifact1.propagation_checksum,
            artifact2.propagation_checksum
        );
    }

    #[test]
    fn replay_succeeds_without_state_change() {
        // Per RPSU-01: Replay must succeed when state unchanged
        let mut rt = StubRuntime::new();
        rt.create_node(0.5, 100.0);
        rt.create_node(0.5, 100.0);
        rt.propagate();

        // Replay immediately after - no state change
        let results = rt.lineage_replay();

        // Find the propagation entry
        let prop_result = results
            .iter()
            .find(|r| r.decision_id.starts_with("prop-"))
            .expect("Should have propagation replay result");

        assert_eq!(prop_result.replay_status, ReplayStatus::Match);
        assert!(prop_result.reason.is_none());
    }

    #[test]
    fn replay_blocks_on_structural_mutation() {
        // Per RPSU-01: Replay must block on structural mutation
        let mut rt = StubRuntime::new();
        let _n1 = rt.create_node(0.5, 100.0);
        let _n2 = rt.create_node(0.5, 100.0);
        rt.propagate();

        // Mutate structure - add a new node
        rt.create_node(0.8, 200.0);

        // Replay after structural change
        let results = rt.lineage_replay();

        // Find the propagation entry
        let prop_result = results
            .iter()
            .find(|r| r.decision_id.starts_with("prop-"))
            .expect("Should have propagation replay result");

        assert_eq!(prop_result.replay_status, ReplayStatus::Blocked);
        assert_eq!(
            prop_result.reason.as_deref(),
            Some("derived_state_mismatch")
        );
    }

    #[test]
    fn replay_blocks_on_belief_mutation() {
        // Per RPSU-01: Replay must block on any state mutation
        let mut rt = StubRuntime::new();
        let n1 = rt.create_node(0.5, 100.0);
        rt.propagate();

        // Mutate belief
        rt.mutate_node(n1.id, 0.1);

        // Replay after belief change
        let results = rt.lineage_replay();

        let prop_result = results
            .iter()
            .find(|r| r.decision_id.starts_with("prop-"))
            .expect("Should have propagation replay result");

        assert_eq!(prop_result.replay_status, ReplayStatus::Blocked);
    }

    #[test]
    fn lineage_entry_contains_artifact() {
        // Per RPSU-01: Lineage must attach derived_state, checksum, mode
        let mut rt = StubRuntime::new();
        rt.create_node(0.5, 100.0);
        rt.propagate();

        let entries = rt.lineage_entries();
        let prop_entry = entries
            .iter()
            .find(|e| e.operation == "edge.propagate")
            .expect("Should have propagate entry");

        let artifact = prop_entry
            .propagation_artifact
            .as_ref()
            .expect("Propagate entry must have artifact");

        assert_eq!(artifact.mode, "reference-stub");
        assert!(!artifact.propagation_checksum.is_empty());
        assert_eq!(artifact.derived_state.node_count, 1);
    }

    #[test]
    fn propagation_artifact_json_schema() {
        // Verify JSON output matches RPSU-01 specification
        let mut rt = StubRuntime::new();
        rt.create_node(0.6, 1.0);
        rt.create_node(0.5, 1.0);
        rt.bind_edge(
            rt.nodes.keys().min().copied().unwrap(),
            rt.nodes.keys().max().copied().unwrap(),
            0.5,
        );

        let artifact = rt.propagate();
        let json = serde_json::to_value(&artifact).unwrap();
        let obj = json.as_object().unwrap();

        // Required fields per RPSU-01
        assert!(obj.contains_key("decision_id"));
        assert!(obj.contains_key("derived_state"));
        assert!(obj.contains_key("propagation_checksum"));
        assert!(obj.contains_key("mode"));

        // Derived state fields
        let ds = obj.get("derived_state").unwrap().as_object().unwrap();
        assert!(ds.contains_key("node_count"));
        assert!(ds.contains_key("edge_count"));
        assert!(ds.contains_key("total_energy"));
        assert!(ds.contains_key("mean_belief"));

        // Verify values match spec example
        assert_eq!(ds.get("node_count").unwrap(), 2);
        assert_eq!(ds.get("edge_count").unwrap(), 1);
        assert_eq!(ds.get("total_energy").unwrap(), 2.0);
        assert_eq!(ds.get("mean_belief").unwrap(), 0.55);
    }

    #[test]
    fn empty_substrate_propagation() {
        // Edge case: propagation on empty substrate
        let mut rt = StubRuntime::new();
        let artifact = rt.propagate();

        assert_eq!(artifact.derived_state.node_count, 0);
        assert_eq!(artifact.derived_state.edge_count, 0);
        assert_eq!(artifact.derived_state.total_energy, 0.0);
        assert_eq!(artifact.derived_state.mean_belief, 0.0);
        assert_eq!(artifact.mode, "reference-stub");
    }

    #[test]
    fn multiple_propagations_have_unique_decision_ids() {
        let mut rt = StubRuntime::new();
        rt.create_node(0.5, 100.0);

        let a1 = rt.propagate();
        let a2 = rt.propagate();
        let a3 = rt.propagate();

        // Decision IDs must be unique
        assert_ne!(a1.decision_id, a2.decision_id);
        assert_ne!(a2.decision_id, a3.decision_id);
        assert_ne!(a1.decision_id, a3.decision_id);
    }
}
