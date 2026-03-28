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
/// - Governance artifact binding
/// - Ordered execution trace
/// - Policy snapshot (hash, evaluated rules, decision, reason codes)
/// - Permits/budgets if used
/// - Checksum over the packet
#[cfg_attr(feature = "schema-gen", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionPacket {
    /// Iter build hash (compile-time)
    pub iter_build_hash: String,
    /// SCG build hash (from contract)
    pub substrate_build_hash: String,
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
    /// Canonical governance hash bound to this packet, if available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub governance_hash: Option<String>,
    /// Ordered evaluation trace. Empty when no trace has been attached.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub execution_trace: Vec<String>,
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
        substrate_build_hash: String,
        state: &SystemState,
        permit_hash: Option<String>,
        economics_hash: String,
        evaluated_rules: Vec<String>,
    ) -> Result<Self, ContractError> {
        let mut packet = Self {
            iter_build_hash,
            substrate_build_hash,
            tick: state.tick,
            energy: state.energy.clone(),
            reasoning: state.reasoning.clone(),
            learning: state.learning.clone(),
            policy: state.policy.clone(),
            permit_hash,
            economics_hash,
            governance_hash: None,
            execution_trace: Vec::new(),
            evaluated_rules,
            checksum: String::new(),
        };
        packet.checksum = packet.compute_checksum();
        Ok(packet)
    }

    /// Compute SHA-256 checksum of packet (excluding checksum field).
    ///
    /// Uses RFC 8785 JCS for deterministic canonicalization.
    /// Clones the packet with checksum zeroed before serialization.
    fn compute_checksum(&self) -> String {
        let mut input = self.clone();
        input.checksum = String::new();
        let canonical = serde_json_canonicalizer::to_string(&input).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(canonical.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Verify packet checksum.
    pub fn verify_checksum(&self) -> Result<(), AuditError> {
        let computed = self.compute_checksum();
        if computed != self.checksum {
            return Err(AuditError::ChecksumMismatch {
                expected: self.checksum.clone(),
                actual: computed,
            });
        }
        Ok(())
    }

    /// Export packet as canonical JSON string.
    pub fn export(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }

    /// Get the final policy decision.
    pub fn decision(&self) -> PolicyDecision {
        self.policy.decision
    }

    /// Attach deterministic governance metadata and refresh the packet checksum.
    pub fn bind_governance_context(
        &mut self,
        governance_hash: String,
        execution_trace: Vec<String>,
    ) {
        self.governance_hash = Some(governance_hash);
        self.execution_trace = execution_trace;
        self.checksum = self.compute_checksum();
    }

    /// Check if learning was committed.
    pub fn learning_committed(&self) -> bool {
        self.learning.status == crate::contracts::LearningStatus::Committed
    }

    /// Governance hash attached to this packet, if available.
    pub fn governance_hash(&self) -> Option<&str> {
        self.governance_hash.as_deref()
    }

    /// Ordered execution trace carried by this packet.
    pub fn execution_trace(&self) -> &[String] {
        &self.execution_trace
    }

    /// Get reason codes for decision.
    pub fn reason_codes(&self) -> &[String] {
        &self.policy.reason_codes
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
    /// RFC 3339 timestamp recorded at event creation.
    /// Not included in checksum — used for search result ordering.
    #[serde(default)]
    pub created_at: String,
}

impl AuditEvent {
    /// Create event from packet.
    pub fn from_packet(sequence: u64, packet: &DecisionPacket) -> Self {
        let created_at = chrono::Utc::now()
            .format("%Y-%m-%dT%H:%M:%S%.3fZ")
            .to_string();
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
            created_at,
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
        hasher.update(
            serde_json::to_string(&payload)
                .unwrap_or_default()
                .as_bytes(),
        );
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
