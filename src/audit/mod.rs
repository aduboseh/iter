//! ITER-PAR-01: Decision Packet Export (Audit and Replay)
//!
//! Iter exports decision packets for governed actions. A hostile reviewer
//! can inspect a single packet and answer: what the system knew, whether it
//! was allowed to learn, why it did or did not learn, and what policy decided.
//!
//! # Invariants
//!
//! - INV-ITER-05: DecisionPacket contains everything needed to explain and
//!   reproduce the decision path without re-running learning.
//! - Packet checksum mismatch is a hard error.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::contracts::{
    ContractError, EnergyEnvelope, LearningEnvelope, PolicyDecision, PolicyEnvelope,
    ReasoningEnvelope, SystemState,
};

/// Decision packet - complete audit record for a governed action.
///
/// # Contents
/// - System identifiers (build hashes)
/// - Tick
/// - Energy snapshot
/// - Reasoning signature
/// - Learning audit (status, hashes, scarcity streak)
/// - Policy snapshot (hash, evaluated rules, decision, reason codes)
/// - Permits/budgets if used
/// - Checksum over the packet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionPacket {
    /// Iter build hash (compile-time)
    pub iter_build_hash: String,
    /// SCG build hash (from contract)
    pub scg_build_hash: String,
    /// Decision tick
    pub tick: u64,
    /// Energy snapshot
    pub energy: EnergyEnvelope,
    /// Reasoning signature
    pub reasoning: ReasoningEnvelope,
    /// Learning audit
    pub learning: LearningEnvelope,
    /// Policy snapshot
    pub policy: PolicyEnvelope,
    /// Active permit hash (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permit_hash: Option<String>,
    /// Economics config hash
    pub economics_hash: String,
    /// Evaluated rule IDs (ordered)
    pub evaluated_rules: Vec<String>,
    /// SHA-256 checksum of packet (excluding this field)
    pub checksum: String,
}

impl DecisionPacket {
    /// Create packet from system state and additional context.
    ///
    /// Checksum is computed automatically.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        iter_build_hash: String,
        scg_build_hash: String,
        state: &SystemState,
        permit_hash: Option<String>,
        economics_hash: String,
        evaluated_rules: Vec<String>,
    ) -> Result<Self, ContractError> {
        let mut packet = Self {
            iter_build_hash,
            scg_build_hash,
            tick: state.tick,
            energy: state.energy.clone(),
            reasoning: state.reasoning.clone(),
            learning: state.learning.clone(),
            policy: state.policy.clone(),
            permit_hash,
            economics_hash,
            evaluated_rules,
            checksum: String::new(),
        };
        packet.checksum = packet.compute_checksum();
        Ok(packet)
    }

    /// Compute SHA-256 checksum of packet (excluding checksum field).
    fn compute_checksum(&self) -> String {
        // Canonical JSON with sorted keys
        let canonical = self.to_canonical_json();
        let mut hasher = Sha256::new();
        hasher.update(canonical.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Verify packet checksum.
    pub fn verify_checksum(&self) -> Result<(), AuditError> {
        let computed = self.compute_checksum_for_verify();
        if computed != self.checksum {
            return Err(AuditError::ChecksumMismatch {
                expected: self.checksum.clone(),
                actual: computed,
            });
        }
        Ok(())
    }

    /// Compute checksum for verification (same as compute_checksum but for existing packet).
    fn compute_checksum_for_verify(&self) -> String {
        // Create copy without checksum
        let mut copy = self.clone();
        copy.checksum = String::new();
        copy.compute_checksum()
    }

    /// Serialize to canonical JSON (sorted keys, stable floats).
    fn to_canonical_json(&self) -> String {
        // Use serde_json with sorted keys
        // Note: For production, use a proper canonical JSON library (RFC 8785)
        serde_json::to_string(&CanonicalPacket::from(self)).unwrap_or_default()
    }

    /// Export packet as canonical JSON string.
    pub fn export(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }

    /// Get the final policy decision.
    pub fn decision(&self) -> PolicyDecision {
        self.policy.decision
    }

    /// Check if learning was committed.
    pub fn learning_committed(&self) -> bool {
        self.learning.status == crate::contracts::LearningStatus::Committed
    }

    /// Get reason codes for decision.
    pub fn reason_codes(&self) -> &[String] {
        &self.policy.reason_codes
    }
}

/// Canonical representation for hashing (sorted keys).
#[derive(Serialize)]
struct CanonicalPacket {
    economics_hash: String,
    energy: CanonicalEnergy,
    evaluated_rules: Vec<String>,
    iter_build_hash: String,
    learning: CanonicalLearning,
    permit_hash: Option<String>,
    policy: CanonicalPolicy,
    reasoning: CanonicalReasoning,
    scg_build_hash: String,
    tick: u64,
}

#[derive(Serialize)]
struct CanonicalEnergy {
    integrity: f64,
    nodes: f64,
    reservoir: f64,
}

#[derive(Serialize)]
struct CanonicalReasoning {
    conflict_signal: f64,
    control_signal: f64,
    quality: f64,
    value_signal: f64,
}

#[derive(Serialize)]
struct CanonicalLearning {
    capsule_id: String,
    epoch: u64,
    scarcity_streak: u64,
    status: String,
    update_cost: f64,
    update_paid: f64,
    update_quality: f64,
    version_hash: String,
}

#[derive(Serialize)]
struct CanonicalPolicy {
    decision: String,
    policy_hash: String,
    reason_codes: Vec<String>,
}

impl From<&DecisionPacket> for CanonicalPacket {
    fn from(p: &DecisionPacket) -> Self {
        Self {
            economics_hash: p.economics_hash.clone(),
            energy: CanonicalEnergy {
                integrity: p.energy.integrity,
                nodes: p.energy.nodes,
                reservoir: p.energy.reservoir,
            },
            evaluated_rules: p.evaluated_rules.clone(),
            iter_build_hash: p.iter_build_hash.clone(),
            learning: CanonicalLearning {
                capsule_id: p.learning.capsule_id.clone(),
                epoch: p.learning.epoch,
                scarcity_streak: p.learning.scarcity_streak,
                status: format!("{:?}", p.learning.status).to_uppercase(),
                update_cost: p.learning.update_cost,
                update_paid: p.learning.update_paid,
                update_quality: p.learning.update_quality,
                version_hash: p.learning.version_hash.clone(),
            },
            permit_hash: p.permit_hash.clone(),
            policy: CanonicalPolicy {
                decision: format!("{:?}", p.policy.decision).to_uppercase(),
                policy_hash: p.policy.policy_hash.clone(),
                reason_codes: p.policy.reason_codes.clone(),
            },
            reasoning: CanonicalReasoning {
                conflict_signal: p.reasoning.conflict_signal,
                control_signal: p.reasoning.control_signal,
                quality: p.reasoning.quality,
                value_signal: p.reasoning.value_signal,
            },
            scg_build_hash: p.scg_build_hash.clone(),
            tick: p.tick,
        }
    }
}

/// Audit errors.
#[derive(Debug, Clone, thiserror::Error)]
pub enum AuditError {
    /// Checksum mismatch
    #[error("checksum mismatch: expected {expected}, got {actual}")]
    ChecksumMismatch {
        /// Expected checksum value
        expected: String,
        /// Actual computed checksum
        actual: String,
    },

    /// Contract error
    #[error("contract error: {0}")]
    ContractError(#[from] ContractError),
}

// ============================================================================
// Audit Log
// ============================================================================

/// Append-only audit event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Event sequence number
    pub sequence: u64,
    /// Tick when event occurred
    pub tick: u64,
    /// Decision ID (packet checksum)
    pub decision_id: String,
    /// Capsule hash (if learning involved)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capsule_hash: Option<String>,
    /// Learning status
    pub learning_status: String,
    /// Reason codes
    pub reason_codes: Vec<String>,
    /// Event checksum
    pub checksum: String,
}

impl AuditEvent {
    /// Create event from packet.
    pub fn from_packet(sequence: u64, packet: &DecisionPacket) -> Self {
        let mut event = Self {
            sequence,
            tick: packet.tick,
            decision_id: packet.checksum.clone(),
            capsule_hash: if packet.learning.capsule_id.is_empty() {
                None
            } else {
                Some(packet.learning.version_hash.clone())
            },
            learning_status: packet.learning.status.reason_code().to_string(),
            reason_codes: packet.policy.reason_codes.clone(),
            checksum: String::new(),
        };
        event.checksum = event.compute_checksum();
        event
    }

    fn compute_checksum(&self) -> String {
        let payload = serde_json::json!({
            "sequence": self.sequence,
            "tick": self.tick,
            "decision_id": self.decision_id,
            "capsule_hash": self.capsule_hash,
            "learning_status": self.learning_status,
            "reason_codes": self.reason_codes,
        });
        let mut hasher = Sha256::new();
        hasher.update(serde_json::to_string(&payload).unwrap_or_default().as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

/// Append-only audit log.
pub struct AuditLog {
    events: Vec<AuditEvent>,
}

impl Default for AuditLog {
    fn default() -> Self {
        Self::new()
    }
}

impl AuditLog {
    /// Create empty log.
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    /// Append event from packet.
    pub fn append(&mut self, packet: &DecisionPacket) -> &AuditEvent {
        let sequence = self.events.len() as u64;
        let event = AuditEvent::from_packet(sequence, packet);
        self.events.push(event);
        self.events.last().expect("just pushed")
    }

    /// Get event by sequence.
    pub fn get(&self, sequence: u64) -> Option<&AuditEvent> {
        self.events.get(sequence as usize)
    }

    /// Get all events.
    pub fn events(&self) -> &[AuditEvent] {
        &self.events
    }

    /// Get event count.
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Export log as JSON.
    pub fn export(&self) -> String {
        serde_json::to_string_pretty(&self.events).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::{LearningStatus, PolicyDecision};

    fn make_test_state() -> SystemState {
        let energy = EnergyEnvelope::new(100.0, 10.0, 0.95).unwrap();
        let reasoning = ReasoningEnvelope::new(0.9, 0.5, 0.1, 0.5).unwrap();
        let learning = LearningEnvelope::new(
            "cap1".to_string(),
            1,
            "a".repeat(64),
            0.5,
            0.5,
            1.0,
            LearningStatus::Committed,
            0,
        )
        .unwrap();
        let policy = PolicyEnvelope::new("b".repeat(64), PolicyDecision::Allow, vec![]).unwrap();

        SystemState::new(100, energy, reasoning, learning, policy)
    }

    #[test]
    fn packet_checksum_is_deterministic() {
        let state = make_test_state();
        let p1 = DecisionPacket::new(
            "iter-hash".to_string(),
            "scg-hash".to_string(),
            &state,
            None,
            "econ-hash".to_string(),
            vec!["RULE1".to_string()],
        )
        .unwrap();

        let p2 = DecisionPacket::new(
            "iter-hash".to_string(),
            "scg-hash".to_string(),
            &state,
            None,
            "econ-hash".to_string(),
            vec!["RULE1".to_string()],
        )
        .unwrap();

        assert_eq!(p1.checksum, p2.checksum);
    }

    #[test]
    fn packet_checksum_changes_with_input() {
        let state = make_test_state();
        let p1 = DecisionPacket::new(
            "iter-hash".to_string(),
            "scg-hash".to_string(),
            &state,
            None,
            "econ-hash".to_string(),
            vec!["RULE1".to_string()],
        )
        .unwrap();

        let p2 = DecisionPacket::new(
            "iter-hash-different".to_string(),
            "scg-hash".to_string(),
            &state,
            None,
            "econ-hash".to_string(),
            vec!["RULE1".to_string()],
        )
        .unwrap();

        assert_ne!(p1.checksum, p2.checksum);
    }

    #[test]
    fn packet_verify_succeeds_for_valid() {
        let state = make_test_state();
        let packet = DecisionPacket::new(
            "iter-hash".to_string(),
            "scg-hash".to_string(),
            &state,
            None,
            "econ-hash".to_string(),
            vec![],
        )
        .unwrap();

        assert!(packet.verify_checksum().is_ok());
    }

    #[test]
    fn packet_verify_fails_for_tampered() {
        let state = make_test_state();
        let mut packet = DecisionPacket::new(
            "iter-hash".to_string(),
            "scg-hash".to_string(),
            &state,
            None,
            "econ-hash".to_string(),
            vec![],
        )
        .unwrap();

        // Tamper with packet
        packet.tick = 999;

        assert!(packet.verify_checksum().is_err());
    }

    #[test]
    fn audit_log_append_and_retrieve() {
        let state = make_test_state();
        let packet = DecisionPacket::new(
            "iter-hash".to_string(),
            "scg-hash".to_string(),
            &state,
            None,
            "econ-hash".to_string(),
            vec![],
        )
        .unwrap();

        let mut log = AuditLog::new();
        log.append(&packet);

        assert_eq!(log.len(), 1);
        let event = log.get(0).unwrap();
        assert_eq!(event.tick, 100);
        assert_eq!(event.decision_id, packet.checksum);
    }

    #[test]
    fn audit_event_checksum_is_deterministic() {
        let state = make_test_state();
        let packet = DecisionPacket::new(
            "iter-hash".to_string(),
            "scg-hash".to_string(),
            &state,
            None,
            "econ-hash".to_string(),
            vec![],
        )
        .unwrap();

        let e1 = AuditEvent::from_packet(0, &packet);
        let e2 = AuditEvent::from_packet(0, &packet);

        assert_eq!(e1.checksum, e2.checksum);
    }
}
