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

use crate::audit::DecisionPacket;
use crate::contracts::{EnergyEnvelope, LearningEnvelope, ReasoningEnvelope, SystemState};
use crate::economics::EconomicsConfig;
use crate::policy::{PolicyConfig, PolicyEvaluator};

/// Counter for generating sequential IDs
static NODE_COUNTER: AtomicU64 = AtomicU64::new(0);
static EDGE_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Stub node state for protocol validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StubNode {
    /// Stable node identifier
    pub id: u64,
    /// Belief scalar used for deterministic evaluation
    pub belief: f64,
    /// Energy value used for bounded state transitions
    pub energy: f64,
    /// Whether the node satisfies ESV constraints
    pub esv_valid: bool,
}

/// Stub edge state for protocol validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StubEdge {
    /// Stable edge identifier
    pub id: u64,
    /// Source node identifier
    pub src: u64,
    /// Destination node identifier
    pub dst: u64,
    /// Edge weight used during traversal or aggregation
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
    /// Current tick (ITER-PAR-01)
    tick: u64,
    /// Policy evaluator (ITER-PAR-01)
    policy_evaluator: PolicyEvaluator,
    /// Economics config (ITER-PAR-01)
    economics_config: EconomicsConfig,
    /// Last decision packet (ITER-PAR-01)
    last_decision_packet: Option<DecisionPacket>,
}

/// Entry in the immutable lineage log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageEntry {
    /// Monotonic sequence number
    pub sequence: u64,
    /// Operation type identifier
    pub operation: String,
    /// SHA-256 checksum of this entry
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
    /// Creates a new empty stub runtime.
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            lineage: Vec::new(),
            tick: 0,
            policy_evaluator: PolicyEvaluator::new(PolicyConfig::default()),
            economics_config: EconomicsConfig::default(),
            last_decision_packet: None,
        }
    }

    /// Creates a new stub runtime with custom policy and economics config.
    pub fn with_config(policy_config: PolicyConfig, economics_config: EconomicsConfig) -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            lineage: Vec::new(),
            tick: 0,
            policy_evaluator: PolicyEvaluator::new(policy_config),
            economics_config,
            last_decision_packet: None,
        }
    }

    /// Get current tick.
    pub fn current_tick(&self) -> u64 {
        self.tick
    }

    /// Advance tick.
    pub fn advance_tick(&mut self) {
        self.tick += 1;
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
    #[allow(dead_code)]
    pub fn lineage_entries(&self) -> &[LineageEntry] {
        &self.lineage
    }

    /// Evaluate a governance proposal and return authoritative verdict.
    ///
    /// This is the PHASE 0+ entry point for Haltra consumption.
    /// Iter evaluates deterministic admissibility only.
    ///
    /// Verdict mapping (canonical, single source of truth):
    /// - healthy=true AND drift_ok=true AND coherence>=0.7 → ALLOW
    /// - healthy=false OR drift_ok=false → BLOCK  
    /// - coherence<0.7 AND healthy=true → REVIEW
    ///
    /// JCS Canonicalization (Patch A):
    /// - If proposal_c14n and proposal_hash are provided, Iter verifies the hash
    /// - Receipt binds to proposal_hash for deterministic proof chain
    pub fn evaluate_governance(
        &mut self,
        proposal: &GovernanceProposal,
    ) -> Result<GovernanceEvaluation, GovernanceError> {
        // Patch A: Verify JCS canonical hash if provided
        let verified_proposal_hash = self.verify_proposal_hash(proposal)?;

        // Get current governor status for verdict determination
        let status = self.governor_status();

        // Canonical verdict mapping (GAP 5 resolution)
        let verdict = if !status.healthy || !status.drift_ok {
            GovernanceVerdict::Block
        } else if status.coherence >= 0.7 {
            GovernanceVerdict::Allow
        } else {
            GovernanceVerdict::Review
        };

        // Build determinism proof
        let determinism = DeterminismProof {
            drift_ok: status.drift_ok,
            energy_drift: status.energy_drift,
            coherence: status.coherence,
        };

        // Compute CIH for this evaluation (now includes proposal_hash)
        let eval_payload = serde_json::json!({
            "proposal_id": proposal.proposal_id,
            "proposal_hash": verified_proposal_hash,
            "state_snapshot_hash": proposal.state_snapshot_hash,
            "verdict": verdict,
            "determinism": determinism,
            "lineage_len": self.lineage.len(),
        });
        let cih = compute_stable_hash(&serde_json::to_string(&eval_payload).unwrap_or_default());

        // Compute artifact hash
        let artifact_hash = compute_stable_hash(&format!(
            "{}:{}:{:?}",
            proposal.proposal_id, verified_proposal_hash, verdict
        ));

        // Record in lineage
        self.record_lineage(
            "governance.evaluate",
            &format!(
                "proposal:{},hash:{},verdict:{:?}",
                proposal.proposal_id, verified_proposal_hash, verdict
            ),
        );

        // Build receipt with proposal_hash binding (Patch A) + verdict/version (Hardening)
        let receipt = GovernanceReceipt {
            cih,
            artifact_hash,
            replay_ref: format!("lineage:{}", self.lineage.len() - 1),
            proposal_hash: verified_proposal_hash,
            verdict,
            iter_version: env!("CARGO_PKG_VERSION").to_string(),
        };

        Ok(GovernanceEvaluation {
            verdict,
            determinism,
            receipt,
        })
    }

    /// Verify proposal hash matches canonical bytes (Patch A: JCS verification).
    ///
    /// If proposal_c14n and proposal_hash are provided:
    /// - Decode base64 proposal_c14n
    /// - Compute SHA-256 of decoded bytes
    /// - Verify computed hash matches proposal_hash
    ///
    /// If not provided, falls back to legacy hash computation.
    fn verify_proposal_hash(
        &self,
        proposal: &GovernanceProposal,
    ) -> Result<String, GovernanceError> {
        use base64::{engine::general_purpose::STANDARD, Engine as _};

        match (&proposal.proposal_c14n, &proposal.proposal_hash) {
            (Some(c14n_b64), Some(claimed_hash)) => {
                // Decode base64 canonical bytes
                let c14n_bytes = STANDARD.decode(c14n_b64).map_err(|e| {
                    GovernanceError::InvalidCanonicalization {
                        reason: format!("base64 decode failed: {}", e),
                    }
                })?;

                // Compute SHA-256 of canonical bytes
                let computed_hash = compute_stable_hash(&String::from_utf8_lossy(&c14n_bytes));

                // Verify hash match
                if computed_hash != *claimed_hash {
                    return Err(GovernanceError::InvalidCanonicalization {
                        reason: format!(
                            "proposal_hash mismatch: computed={}, claimed={}",
                            computed_hash, claimed_hash
                        ),
                    });
                }

                Ok(claimed_hash.clone())
            }
            (None, Some(hash)) => {
                // Hash provided without canonical bytes - trust but warn
                // (backwards compatibility for Phase 0 clients)
                Ok(hash.clone())
            }
            _ => {
                // Legacy mode: compute hash from proposal fields
                let legacy_hash = compute_stable_hash(&format!(
                    "{}:{}:{}",
                    proposal.proposal_id, proposal.state_snapshot_hash, proposal.requested_action
                ));
                Ok(legacy_hash)
            }
        }
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

    // ========================================================================
    // ITER-PAR-01: System State and Decision Packet Methods
    // ========================================================================

    /// Get current system state (ITER-PAR-01: Deliverable A).
    ///
    /// Returns a complete snapshot for governance evaluation including:
    /// - Energy envelope
    /// - Reasoning envelope (stub: simulated)
    /// - Learning envelope (stub: no learning)
    /// - Policy envelope
    pub fn system_state(&self) -> Result<SystemState, GovernanceError> {
        let derived = self.compute_derived_state();
        let status = self.governor_status();

        // Build energy envelope from derived state
        let energy = EnergyEnvelope::new(
            derived.total_energy,
            0.0, // Stub: no reservoir
            if status.healthy { 1.0 } else { 0.5 },
        )
        .map_err(|e: crate::contracts::ContractError| GovernanceError::SubstrateError {
            reason: e.to_string(),
        })?;

        // Build reasoning envelope (stub: simulated based on coherence)
        let reasoning = ReasoningEnvelope::new(
            status.coherence, // quality = coherence in stub
            derived.mean_belief, // value_signal from belief
            0.0, // conflict_signal (stub: no conflict)
            status.coherence, // control_signal = coherence
        )
        .map_err(|e: crate::contracts::ContractError| GovernanceError::SubstrateError {
            reason: e.to_string(),
        })?;

        // Build learning envelope (stub: no learning)
        let learning = LearningEnvelope::default_no_learning();

        // Evaluate policy
        let policy_result = self.policy_evaluator.evaluate(&reasoning, &learning, &energy);
        let policy = self
            .policy_evaluator
            .build_envelope(&policy_result)
            .map_err(|e: crate::contracts::ContractError| GovernanceError::SubstrateError {
                reason: e.to_string(),
            })?;

        Ok(SystemState::new(self.tick, energy, reasoning, learning, policy))
    }

    /// Build decision packet for current state (ITER-PAR-01: Deliverable D).
    ///
    /// Creates a complete audit record with checksum for replay verification.
    pub fn build_decision_packet(
        &mut self,
        scg_build_hash: &str,
    ) -> Result<DecisionPacket, GovernanceError> {
        let state = self.system_state()?;

        // Evaluate policy to get evaluated rules
        let policy_result = self.policy_evaluator.evaluate(
            &state.reasoning,
            &state.learning,
            &state.energy,
        );

        let packet = DecisionPacket::new(
            env!("CARGO_PKG_VERSION").to_string(),
            scg_build_hash.to_string(),
            &state,
            None, // No permit in stub
            self.economics_config.compute_hash(),
            policy_result.evaluated_rules.iter().map(|s: &&str| s.to_string()).collect(),
        )
        .map_err(|e: crate::contracts::ContractError| GovernanceError::SubstrateError {
            reason: e.to_string(),
        })?;

        self.last_decision_packet = Some(packet.clone());

        // Record in lineage
        self.record_lineage(
            "decision.packet",
            &format!("tick:{},checksum:{}", self.tick, packet.checksum),
        );

        Ok(packet)
    }

    /// Get last decision packet (if any).
    pub fn last_decision_packet(&self) -> Option<&DecisionPacket> {
        self.last_decision_packet.as_ref()
    }

    /// Update economics configuration.
    pub fn set_economics_config(&mut self, config: EconomicsConfig) {
        self.economics_config = config;
    }

    /// Get economics configuration.
    pub fn economics_config(&self) -> &EconomicsConfig {
        &self.economics_config
    }

    /// Update policy configuration.
    pub fn set_policy_config(&mut self, config: PolicyConfig) {
        self.policy_evaluator = PolicyEvaluator::new(config);
    }

    /// Get policy configuration.
    pub fn policy_config(&self) -> &PolicyConfig {
        self.policy_evaluator.config()
    }

    /// Evaluate and produce decision packet for SCG state input.
    ///
    /// This is the main ITER-PAR-01 entry point for SCG integration.
    /// Accepts typed contract inputs and produces auditable decision output.
    pub fn evaluate_scg_state(
        &mut self,
        energy: &EnergyEnvelope,
        reasoning: &ReasoningEnvelope,
        learning: &LearningEnvelope,
        scg_build_hash: &str,
    ) -> Result<DecisionPacket, GovernanceError> {
        // Evaluate policy
        let policy_result = self.policy_evaluator.evaluate(reasoning, learning, energy);
        let policy = self
            .policy_evaluator
            .build_envelope(&policy_result)
            .map_err(|e: crate::contracts::ContractError| GovernanceError::SubstrateError {
                reason: e.to_string(),
            })?;

        // Build system state
        let state = SystemState::new(
            self.tick,
            energy.clone(),
            reasoning.clone(),
            learning.clone(),
            policy,
        );

        // Build decision packet
        let packet = DecisionPacket::new(
            env!("CARGO_PKG_VERSION").to_string(),
            scg_build_hash.to_string(),
            &state,
            None,
            self.economics_config.compute_hash(),
            policy_result.evaluated_rules.iter().map(|s: &&str| s.to_string()).collect(),
        )
        .map_err(|e: crate::contracts::ContractError| GovernanceError::SubstrateError {
            reason: e.to_string(),
        })?;

        self.last_decision_packet = Some(packet.clone());
        self.advance_tick();

        // Record in lineage
        self.record_lineage(
            "scg.evaluate",
            &format!("tick:{},decision:{:?}", self.tick - 1, state.policy.decision),
        );

        Ok(packet)
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

/// Governor status metrics for substrate health monitoring.
#[derive(Debug, Clone, Serialize)]
pub struct GovernorStatus {
    /// Whether drift constraints are satisfied
    pub drift_ok: bool,
    /// Net energy drift measured during evaluation
    pub energy_drift: f64,
    /// Coherence score of the current substrate state
    pub coherence: f64,
    /// Total number of nodes in the substrate
    pub node_count: usize,
    /// Total number of edges in the substrate
    pub edge_count: usize,
    /// Overall health indicator derived from governance checks
    pub healthy: bool,
}

/// ESV constraint audit result for a single node.
#[derive(Debug, Clone, Serialize)]
pub struct EsvAudit {
    /// Identifier of the audited node
    pub node_id: u64,
    /// Whether the node satisfies ESV constraints
    pub valid: bool,
    /// Human-readable compliance classification
    pub compliance_status: String,
}

/// Verdict for governance evaluation.
///
/// This is the authoritative decision boundary. Haltra proposes, Iter decides.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GovernanceVerdict {
    /// Action is admissible under current governance constraints
    Allow,
    /// Action is blocked due to governance violation
    Block,
    /// Action requires human review (coherence below threshold)
    Review,
}

/// Errors that can occur during governance evaluation (Patch A/B).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "error_code", content = "reason")]
pub enum GovernanceError {
    /// RFC 8785 canonical bytes failed verification.
    #[serde(rename = "INVALID_CANONICALIZATION")]
    InvalidCanonicalization {
        /// Human-readable error description
        reason: String,
    },
    /// Receipt integrity check failed.
    #[serde(rename = "INVALID_RECEIPT")]
    InvalidReceipt {
        /// Human-readable error description
        reason: String,
    },
    /// Substrate error during evaluation.
    #[serde(rename = "SUBSTRATE_ERROR")]
    SubstrateError {
        /// Human-readable error description
        reason: String,
    },
}

impl std::fmt::Display for GovernanceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GovernanceError::InvalidCanonicalization { reason } => {
                write!(f, "Invalid canonicalization: {}", reason)
            }
            GovernanceError::InvalidReceipt { reason } => {
                write!(f, "Invalid receipt: {}", reason)
            }
            GovernanceError::SubstrateError { reason } => {
                write!(f, "Substrate error: {}", reason)
            }
        }
    }
}

impl std::error::Error for GovernanceError {}

/// Determinism proof returned with governance evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeterminismProof {
    /// Whether energy drift is within bounds (≤1×10⁻¹⁰)
    pub drift_ok: bool,
    /// Current energy drift value
    pub energy_drift: f64,
    /// Coherence index [0.0, 1.0]
    pub coherence: f64,
}

/// Receipt for governance evaluation (cryptographic audit trail).
///
/// Self-contained for auditor verification — includes verdict and version.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceReceipt {
    /// Composite Integrity Hash for this decision
    pub cih: String,
    /// SHA-256 hash of the evaluation artifact
    pub artifact_hash: String,
    /// Reference for replay verification
    pub replay_ref: String,
    /// SHA-256 hash of the canonical proposal (RFC 8785 binding)
    pub proposal_hash: String,
    /// Verdict included for self-contained audit verification
    pub verdict: GovernanceVerdict,
    /// Iter version that produced this receipt (provenance)
    pub iter_version: String,
}

/// Domain-agnostic governance proposal input.
///
/// Iter does NOT interpret domain semantics. It evaluates deterministic
/// admissibility only. Haltra owns domain ethics interpretation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceProposal {
    /// Unique proposal identifier
    pub proposal_id: String,
    /// SHA-256 hash of the state snapshot being evaluated
    pub state_snapshot_hash: String,
    /// Constraints to evaluate (opaque to Iter)
    #[serde(default)]
    pub constraints: serde_json::Value,
    /// Requested action (opaque to Iter)
    pub requested_action: String,
    /// RFC 8785 (JCS) canonical JSON bytes, base64-encoded
    #[serde(default)]
    pub proposal_c14n: Option<String>,
    /// SHA-256 hash of proposal_c14n bytes (Haltra computes, Iter verifies)
    #[serde(default)]
    pub proposal_hash: Option<String>,
}

/// Result of governance evaluation.
///
/// This is the authoritative response from Iter. Haltra must not claim
/// determinism, ethical validity, or audit integrity independently.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceEvaluation {
    /// Authoritative verdict: ALLOW, BLOCK, or REVIEW
    pub verdict: GovernanceVerdict,
    /// Determinism proof metrics
    pub determinism: DeterminismProof,
    /// Cryptographic receipt for audit trail
    pub receipt: GovernanceReceipt,
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
