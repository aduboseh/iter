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

use serde::{Deserialize, Deserializer, Serialize};
use sha2::{Digest, Sha256};

use crate::contracts::numeric;
use crate::contracts::{
    ContractError, EnergyEnvelope, LearningEnvelope, PolicyDecision, PolicyEnvelope,
    ReasoningEnvelope, SystemState,
};
use crate::provenance;

/// Compile-time SCG contract provenance exported by build.rs.
#[cfg_attr(feature = "schema-gen", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContractProvenance {
    /// SCG governance bridge contract version.
    pub contract_version: String,
    /// Exact SCG source commit used as the vendored bridge origin.
    #[cfg_attr(feature = "schema-gen", schemars(regex(pattern = "^[0-9a-f]{40}$")))]
    pub scg_source_commit: String,
    /// SCG master head that had accepted the vendored bridge at vendor time.
    #[cfg_attr(feature = "schema-gen", schemars(regex(pattern = "^[0-9a-f]{40}$")))]
    pub scg_vendor_master_head: String,
    /// SHA-256 of vendored contract.rs.
    #[cfg_attr(feature = "schema-gen", schemars(regex(pattern = "^[0-9a-f]{64}$")))]
    pub bridge_contract_rs_sha256: String,
    /// SHA-256 of vendored trace.rs.
    #[cfg_attr(feature = "schema-gen", schemars(regex(pattern = "^[0-9a-f]{64}$")))]
    pub bridge_trace_rs_sha256: String,
    /// SHA-256 of vendored errors.rs.
    #[cfg_attr(feature = "schema-gen", schemars(regex(pattern = "^[0-9a-f]{64}$")))]
    pub bridge_errors_rs_sha256: String,
    /// SHA-256 of vendored lib.rs.
    #[cfg_attr(feature = "schema-gen", schemars(regex(pattern = "^[0-9a-f]{64}$")))]
    pub bridge_lib_rs_sha256: String,
    /// Raw-byte SHA-256 of CANONICAL_VECTORS.json.
    #[cfg_attr(feature = "schema-gen", schemars(regex(pattern = "^[0-9a-f]{64}$")))]
    pub canonical_vectors_sha256: String,
    /// Canonicalization rule bound to the vendored contract.
    pub canonicalization_rule: String,
}

impl ContractProvenance {
    /// Read the build-time contract facts emitted by build.rs.
    pub fn compile_time() -> Self {
        Self {
            contract_version: provenance::CONTRACT_VERSION.to_string(),
            scg_source_commit: provenance::SCG_SOURCE_COMMIT.to_string(),
            scg_vendor_master_head: provenance::SCG_VENDOR_MASTER_HEAD.to_string(),
            bridge_contract_rs_sha256: provenance::BRIDGE_CONTRACT_RS_SHA256.to_string(),
            bridge_trace_rs_sha256: provenance::BRIDGE_TRACE_RS_SHA256.to_string(),
            bridge_errors_rs_sha256: provenance::BRIDGE_ERRORS_RS_SHA256.to_string(),
            bridge_lib_rs_sha256: provenance::BRIDGE_LIB_RS_SHA256.to_string(),
            canonical_vectors_sha256: provenance::CANONICAL_VECTORS_SHA256.to_string(),
            canonicalization_rule: provenance::CANONICALIZATION_RULE.to_string(),
        }
    }
}

/// Provenance source declarations for static and dynamic proof-packet values.
#[cfg_attr(feature = "schema-gen", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProvenanceSource {
    /// Source of static contract and bridge values.
    pub contract_values: String,
    /// Source of decision-specific values.
    pub decision_values: String,
    /// Exact encoding used for proof-critical numeric packet fields.
    pub numeric_encoding: String,
    /// Integrity validation method for the canonical vector file.
    pub canonical_vector_integrity: String,
    /// Validation method for canonical vector digest casing.
    pub vector_digest_casing: String,
}

impl ProvenanceSource {
    /// Return the fixed source declarations for Iter decision packets.
    pub fn compile_time_and_runtime() -> Self {
        Self {
            contract_values: "compile_time_build_rs_rustc_env".to_string(),
            decision_values: "runtime_execution".to_string(),
            numeric_encoding: numeric::F64_HEX_ENCODING.to_string(),
            canonical_vector_integrity: "raw_byte_sha256".to_string(),
            vector_digest_casing: "raw_text_validation".to_string(),
        }
    }
}

/// Replay scope bound into each proof packet.
#[cfg_attr(feature = "schema-gen", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReplayScope {
    /// Replay guarantee scope for this packet.
    pub replay_scope: String,
    /// Target triple used to build the producing Iter binary.
    pub platform: String,
    /// rustc version used to build the producing Iter binary.
    pub rustc_version: String,
    /// Whether this packet claims cross-platform replay equivalence.
    pub cross_platform_replay_claimed: bool,
}

impl ReplayScope {
    /// Read the build environment exported by build.rs.
    pub fn compile_time() -> Self {
        Self {
            replay_scope: provenance::REPLAY_SCOPE.to_string(),
            platform: provenance::TARGET_TRIPLE.to_string(),
            rustc_version: provenance::RUSTC_VERSION.to_string(),
            cross_platform_replay_claimed: provenance::CROSS_PLATFORM_REPLAY_CLAIMED,
        }
    }
}

/// Decision packet - complete audit record for a governed action.
///
/// # Contents
/// - System identifiers (build hashes)
/// - Replay scope and build environment
/// - Compile-time SCG contract provenance
/// - Provenance source declarations
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
#[derive(Debug, Clone, Serialize)]
pub struct DecisionPacket {
    /// Iter build hash (compile-time)
    pub iter_build_hash: String,
    /// SCG build hash (from contract)
    pub substrate_build_hash: String,
    /// Replay scope and build environment for this packet.
    pub replay_scope: ReplayScope,
    /// Contract-critical bridge provenance exported by build.rs.
    pub contract_provenance: ContractProvenance,
    /// Source declarations for static and dynamic packet values.
    pub provenance_source: ProvenanceSource,
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

#[derive(Deserialize)]
struct DecisionPacketWire {
    iter_build_hash: String,
    substrate_build_hash: String,
    #[serde(default = "ReplayScope::compile_time")]
    replay_scope: ReplayScope,
    #[serde(default = "ContractProvenance::compile_time")]
    contract_provenance: ContractProvenance,
    #[serde(default = "ProvenanceSource::compile_time_and_runtime")]
    provenance_source: ProvenanceSource,
    tick: u64,
    energy: EnergyEnvelope,
    reasoning: ReasoningEnvelope,
    learning: LearningEnvelope,
    policy: PolicyEnvelope,
    permit_hash: Option<String>,
    economics_hash: String,
    governance_hash: Option<String>,
    #[serde(default)]
    execution_trace: Vec<String>,
    evaluated_rules: Vec<String>,
    checksum: String,
}

impl<'de> Deserialize<'de> for DecisionPacket {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = DecisionPacketWire::deserialize(deserializer)?;
        Ok(Self {
            iter_build_hash: wire.iter_build_hash,
            substrate_build_hash: wire.substrate_build_hash,
            replay_scope: wire.replay_scope,
            contract_provenance: wire.contract_provenance,
            provenance_source: wire.provenance_source,
            tick: wire.tick,
            energy: wire.energy,
            reasoning: wire.reasoning,
            learning: wire.learning,
            policy: wire.policy,
            permit_hash: wire.permit_hash,
            economics_hash: wire.economics_hash,
            governance_hash: wire.governance_hash,
            execution_trace: wire.execution_trace,
            evaluated_rules: wire.evaluated_rules,
            checksum: wire.checksum,
        })
    }
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
            replay_scope: ReplayScope::compile_time(),
            contract_provenance: ContractProvenance::compile_time(),
            provenance_source: ProvenanceSource::compile_time_and_runtime(),
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
        self.validate_contract_fields()?;
        let computed = self.compute_checksum();
        if computed != self.checksum {
            return Err(AuditError::ChecksumMismatch {
                expected: self.checksum.clone(),
                actual: computed,
            });
        }
        Ok(())
    }

    fn validate_contract_fields(&self) -> Result<(), AuditError> {
        self.energy
            .validate()
            .map_err(|err| AuditError::InvalidPacketContract {
                reason: err.to_string(),
            })?;
        self.reasoning
            .validate()
            .map_err(|err| AuditError::InvalidPacketContract {
                reason: err.to_string(),
            })?;
        self.learning
            .validate()
            .map_err(|err| AuditError::InvalidPacketContract {
                reason: err.to_string(),
            })?;
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
    pub(crate) fn bind_governance_context(
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
    /// Packet contract field failed validation
    #[error("invalid packet contract field: {reason}")]
    InvalidPacketContract {
        /// Validation failure reason
        reason: String,
    },

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
    /// Policy decision outcome.
    pub decision: String,
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
            decision: match packet.policy.decision {
                PolicyDecision::Allow => "ALLOW".to_string(),
                PolicyDecision::Deny => "DENY".to_string(),
                PolicyDecision::FreezeLearning => "FREEZE_LEARNING".to_string(),
                PolicyDecision::DegradedMode => "DEGRADED_MODE".to_string(),
                PolicyDecision::RequireReview => "REQUIRE_REVIEW".to_string(),
            },
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
            "decision": self.decision,
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
    fn packet_provenance_uses_compile_time_exports() {
        let state = make_test_state();
        let packet = DecisionPacket::new(
            "iter-hash".to_string(),
            "scg-hash".to_string(),
            &state,
            None,
            "economics-hash".to_string(),
            vec!["rule-1".to_string()],
        )
        .unwrap();

        assert_eq!(packet.contract_provenance.contract_version, "scg.v1");
        assert_eq!(packet.replay_scope.replay_scope, "same_binary_only");
        assert!(!packet.replay_scope.platform.is_empty());
        assert!(packet.replay_scope.rustc_version.starts_with("rustc "));
        assert!(!packet.replay_scope.cross_platform_replay_claimed);
        assert_eq!(
            packet.contract_provenance.scg_source_commit,
            env!("ITER_SCG_SOURCE_COMMIT")
        );
        assert_eq!(
            packet.contract_provenance.scg_vendor_master_head,
            env!("ITER_SCG_VENDOR_MASTER_HEAD")
        );
        assert_eq!(
            packet.provenance_source.contract_values,
            "compile_time_build_rs_rustc_env"
        );
        assert_eq!(
            packet.provenance_source.decision_values,
            "runtime_execution"
        );
        assert_eq!(
            packet.provenance_source.numeric_encoding,
            crate::contracts::numeric::F64_HEX_ENCODING
        );
        assert_eq!(
            packet.provenance_source.canonical_vector_integrity,
            "raw_byte_sha256"
        );
        assert_eq!(
            packet.provenance_source.vector_digest_casing,
            "raw_text_validation"
        );
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
    fn packet_exports_proof_critical_numbers_as_hex_strings() {
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
        let value = serde_json::to_value(&packet).expect("packet serializes");
        let energy_nodes = crate::contracts::numeric::f64_to_hex(packet.energy.nodes);
        let reasoning_quality = crate::contracts::numeric::f64_to_hex(packet.reasoning.quality);
        let learning_quality =
            crate::contracts::numeric::f64_to_hex(packet.learning.update_quality);

        assert_eq!(
            value["energy"]["nodes"].as_str(),
            Some(energy_nodes.as_str())
        );
        assert_eq!(
            value["reasoning"]["quality"].as_str(),
            Some(reasoning_quality.as_str())
        );
        assert_eq!(
            value["learning"]["update_quality"].as_str(),
            Some(learning_quality.as_str())
        );
        assert!(value["energy"]["nodes"].as_str().is_some());
    }

    #[test]
    fn packet_deserializes_legacy_v1_numbers_and_defaulted_metadata() {
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
        let mut value = serde_json::to_value(&packet).expect("packet serializes");
        let obj = value.as_object_mut().expect("packet value is an object");
        obj.remove("replay_scope");
        obj.remove("contract_provenance");
        obj.remove("provenance_source");
        value["energy"]["nodes"] = serde_json::json!(100.0);
        value["reasoning"]["quality"] = serde_json::json!(0.9);
        value["learning"]["update_quality"] = serde_json::json!(1.0);

        let legacy: DecisionPacket =
            serde_json::from_value(value).expect("legacy v1 packet shape deserializes");

        assert_eq!(legacy.energy.nodes.to_bits(), 100.0f64.to_bits());
        assert_eq!(legacy.reasoning.quality.to_bits(), 0.9f64.to_bits());
        assert_eq!(legacy.learning.update_quality.to_bits(), 1.0f64.to_bits());
        assert_eq!(legacy.replay_scope.replay_scope, "same_binary_only");
        assert_eq!(legacy.contract_provenance.contract_version, "scg.v1");
        assert_eq!(
            legacy.provenance_source.numeric_encoding,
            crate::contracts::numeric::F64_HEX_ENCODING
        );
    }

    #[test]
    fn packet_verify_rejects_out_of_bounds_deserialized_numeric_values() {
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
        let mut value = serde_json::to_value(&packet).expect("packet serializes");
        value["energy"]["integrity"] =
            serde_json::Value::String(crate::contracts::numeric::f64_to_hex(1.5));
        let packet: DecisionPacket = serde_json::from_value(value).expect("packet deserializes");

        let err = packet
            .verify_checksum()
            .expect_err("out-of-bounds packet rejected");
        assert!(
            err.to_string().contains("energy.integrity"),
            "unexpected error: {err}"
        );
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
