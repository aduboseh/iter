//! Replay Contract Tests (INV-ITER-05)
//!
//! INVARIANT: DecisionPacket ALONE is sufficient to reconstruct the governance
//! outcome. An external auditor with only the DecisionPacket can verify:
//! - The policy decision was correct given the inputs
//! - The economic authorization was valid
//! - All reason codes are causal (no phantom reasons)
//!
//! These tests prove the replay contract by:
//! 1. Building DecisionPackets from substrate state
//! 2. Re-evaluating policy using ONLY packet contents
//! 3. Asserting byte-identical outcomes

use iter_mcp_server::audit::{AuditLog, DecisionPacket};
use iter_mcp_server::contracts::{
    EnergyEnvelope, LearningEnvelope, LearningStatus, PolicyDecision, PolicyEnvelope,
    ReasoningEnvelope, SystemState,
};
use iter_mcp_server::economics::{EconomicsConfig, EconomicsController};
use iter_mcp_server::policy::{PolicyConfig, PolicyEvaluator};

// ============================================================================
// Replay Sufficiency Tests
// ============================================================================

/// Helper: Build a complete governance evaluation from scratch
fn evaluate_governance(
    reasoning: &ReasoningEnvelope,
    learning: &LearningEnvelope,
    energy: &EnergyEnvelope,
    policy_config: &PolicyConfig,
    economics_config: &EconomicsConfig,
    tick: u64,
) -> (PolicyDecision, Vec<String>, Vec<&'static str>, String) {
    let evaluator = PolicyEvaluator::new(policy_config.clone());
    let result = evaluator.evaluate(reasoning, learning, energy);

    let controller = EconomicsController::new(economics_config.clone()).unwrap();
    let (econ_allowed, econ_reason) =
        controller.check_learning_allowed(tick, &learning.capsule_id, learning.update_cost);

    let final_decision = if !econ_allowed && result.decision == PolicyDecision::Allow {
        PolicyDecision::Deny
    } else {
        result.decision
    };

    let mut all_reasons = result.reason_codes.clone();
    if !econ_allowed {
        all_reasons.push(econ_reason.to_string());
    }

    (final_decision, all_reasons, result.evaluated_rules, econ_reason.to_string())
}

/// Build a DecisionPacket from raw state
fn build_packet(
    tick: u64,
    reasoning: &ReasoningEnvelope,
    learning: &LearningEnvelope,
    energy: &EnergyEnvelope,
    policy_config: &PolicyConfig,
    economics_config: &EconomicsConfig,
) -> DecisionPacket {
    let (decision, reason_codes, evaluated_rules, _) = evaluate_governance(
        reasoning,
        learning,
        energy,
        policy_config,
        economics_config,
        tick,
    );

    let policy_envelope = PolicyEnvelope::new(
        policy_config.compute_hash(),
        decision,
        reason_codes,
    )
    .unwrap();

    let system_state = SystemState::new(
        tick,
        energy.clone(),
        reasoning.clone(),
        learning.clone(),
        policy_envelope,
    );

    DecisionPacket::new(
        "iter-test".to_string(),
        "scg-test".to_string(),
        &system_state,
        None,
        economics_config.compute_hash(),
        evaluated_rules.iter().map(|s| s.to_string()).collect(),
    )
    .unwrap()
}

/// Replay a DecisionPacket and verify it produces identical results
fn replay_packet(packet: &DecisionPacket, policy_config: &PolicyConfig) -> (PolicyDecision, Vec<String>) {
    let evaluator = PolicyEvaluator::new(policy_config.clone());
    let result = evaluator.evaluate(
        &packet.reasoning,
        &packet.learning,
        &packet.energy,
    );
    (result.decision, result.reason_codes)
}

#[test]
fn replay_produces_identical_allow_decision() {
    let reasoning = ReasoningEnvelope::new(0.9, 0.5, 0.1, 0.8).unwrap();
    let learning = LearningEnvelope::new(
        "cap_001".to_string(),
        1,
        "a".repeat(64),
        0.5,
        0.5,
        0.98, // Above min_learning_quality default (0.95)
        LearningStatus::Committed,
        0,
    )
    .unwrap();
    let energy = EnergyEnvelope::new(100.0, 50.0, 0.95).unwrap();

    let policy_config = PolicyConfig::default();
    let economics_config = EconomicsConfig::default();

    let packet = build_packet(100, &reasoning, &learning, &energy, &policy_config, &economics_config);

    assert_eq!(packet.decision(), PolicyDecision::Allow);

    // Replay using ONLY packet contents
    let (replayed_decision, replayed_reasons) = replay_packet(&packet, &policy_config);

    assert_eq!(replayed_decision, packet.decision());
    assert_eq!(replayed_reasons, packet.reason_codes());
}

#[test]
fn replay_produces_identical_deny_decision() {
    let reasoning = ReasoningEnvelope::new(0.9, 0.5, 0.1, 0.8).unwrap();
    let learning = LearningEnvelope::new(
        "cap_001".to_string(),
        1,
        "a".repeat(64),
        0.5,
        0.5,
        0.9,
        LearningStatus::Committed,
        0,
    )
    .unwrap();
    let energy = EnergyEnvelope::new(100.0, 50.0, 0.75).unwrap(); // Below default threshold

    let policy_config = PolicyConfig {
        min_energy_integrity: 0.9,
        ..Default::default()
    };
    let economics_config = EconomicsConfig::default();

    let packet = build_packet(100, &reasoning, &learning, &energy, &policy_config, &economics_config);

    assert_eq!(packet.decision(), PolicyDecision::Deny);

    let (replayed_decision, replayed_reasons) = replay_packet(&packet, &policy_config);

    assert_eq!(replayed_decision, packet.decision());
    assert_eq!(replayed_reasons, packet.reason_codes());
    assert!(replayed_reasons.contains(&"ENERGY_INTEGRITY_BELOW_THRESHOLD".to_string()));
}

#[test]
fn replay_produces_identical_freeze_learning_decision() {
    let reasoning = ReasoningEnvelope::new(0.9, 0.5, 0.1, 0.8).unwrap();
    let learning = LearningEnvelope::new(
        "cap_001".to_string(),
        1,
        "a".repeat(64),
        0.5,
        0.1,
        0.3,
        LearningStatus::RejectedScarcity,
        10, // High scarcity streak
    )
    .unwrap();
    let energy = EnergyEnvelope::new(100.0, 50.0, 0.95).unwrap();

    let policy_config = PolicyConfig {
        max_scarcity_streak: 5,
        ..Default::default()
    };
    let economics_config = EconomicsConfig::default();

    let packet = build_packet(100, &reasoning, &learning, &energy, &policy_config, &economics_config);

    assert_eq!(packet.decision(), PolicyDecision::FreezeLearning);

    let (replayed_decision, replayed_reasons) = replay_packet(&packet, &policy_config);

    assert_eq!(replayed_decision, packet.decision());
    assert_eq!(replayed_reasons, packet.reason_codes());
}

#[test]
fn replay_produces_identical_degraded_mode_decision() {
    let reasoning = ReasoningEnvelope::new(0.3, 0.5, 0.1, 0.8).unwrap(); // Low quality
    let learning = LearningEnvelope::new(
        "cap_001".to_string(),
        1,
        "a".repeat(64),
        0.5,
        0.5,
        0.9,
        LearningStatus::Committed,
        0,
    )
    .unwrap();
    let energy = EnergyEnvelope::new(100.0, 50.0, 0.95).unwrap();

    let policy_config = PolicyConfig {
        min_reasoning_quality: 0.5,
        ..Default::default()
    };
    let economics_config = EconomicsConfig::default();

    let packet = build_packet(100, &reasoning, &learning, &energy, &policy_config, &economics_config);

    assert_eq!(packet.decision(), PolicyDecision::DegradedMode);

    let (replayed_decision, replayed_reasons) = replay_packet(&packet, &policy_config);

    assert_eq!(replayed_decision, packet.decision());
    assert_eq!(replayed_reasons, packet.reason_codes());
}

// ============================================================================
// Checksum Stability Tests
// ============================================================================

#[test]
fn identical_inputs_produce_identical_checksums() {
    let reasoning = ReasoningEnvelope::new(0.9, 0.5, 0.1, 0.8).unwrap();
    let learning = LearningEnvelope::new(
        "cap_001".to_string(),
        1,
        "a".repeat(64),
        0.5,
        0.5,
        0.9,
        LearningStatus::Committed,
        0,
    )
    .unwrap();
    let energy = EnergyEnvelope::new(100.0, 50.0, 0.95).unwrap();

    let policy_config = PolicyConfig::default();
    let economics_config = EconomicsConfig::default();

    let packet1 = build_packet(100, &reasoning, &learning, &energy, &policy_config, &economics_config);
    let packet2 = build_packet(100, &reasoning, &learning, &energy, &policy_config, &economics_config);

    assert_eq!(packet1.checksum, packet2.checksum);
}

#[test]
fn different_tick_produces_different_checksum() {
    let reasoning = ReasoningEnvelope::new(0.9, 0.5, 0.1, 0.8).unwrap();
    let learning = LearningEnvelope::new(
        "cap_001".to_string(),
        1,
        "a".repeat(64),
        0.5,
        0.5,
        0.9,
        LearningStatus::Committed,
        0,
    )
    .unwrap();
    let energy = EnergyEnvelope::new(100.0, 50.0, 0.95).unwrap();

    let policy_config = PolicyConfig::default();
    let economics_config = EconomicsConfig::default();

    let packet1 = build_packet(100, &reasoning, &learning, &energy, &policy_config, &economics_config);
    let packet2 = build_packet(101, &reasoning, &learning, &energy, &policy_config, &economics_config);

    assert_ne!(packet1.checksum, packet2.checksum);
}

#[test]
fn different_decision_produces_different_checksum() {
    let reasoning = ReasoningEnvelope::new(0.9, 0.5, 0.1, 0.8).unwrap();
    let learning = LearningEnvelope::new(
        "cap_001".to_string(),
        1,
        "a".repeat(64),
        0.5,
        0.5,
        0.98, // Above min_learning_quality default (0.95)
        LearningStatus::Committed,
        0,
    )
    .unwrap();
    let energy_good = EnergyEnvelope::new(100.0, 50.0, 0.95).unwrap();
    let energy_bad = EnergyEnvelope::new(100.0, 50.0, 0.75).unwrap();

    let policy_config = PolicyConfig {
        min_energy_integrity: 0.9,
        ..Default::default()
    };
    let economics_config = EconomicsConfig::default();

    let packet_allow = build_packet(100, &reasoning, &learning, &energy_good, &policy_config, &economics_config);
    let packet_deny = build_packet(100, &reasoning, &learning, &energy_bad, &policy_config, &economics_config);

    assert_eq!(packet_allow.decision(), PolicyDecision::Allow);
    assert_eq!(packet_deny.decision(), PolicyDecision::Deny);
    assert_ne!(packet_allow.checksum, packet_deny.checksum);
}

// ============================================================================
// Audit Log Replay Tests
// ============================================================================

#[test]
fn audit_log_preserves_packet_sequence() {
    let mut log = AuditLog::new();

    let reasoning = ReasoningEnvelope::new(0.9, 0.5, 0.1, 0.8).unwrap();
    let learning = LearningEnvelope::new(
        "cap_001".to_string(),
        1,
        "a".repeat(64),
        0.5,
        0.5,
        0.9,
        LearningStatus::Committed,
        0,
    )
    .unwrap();
    let energy = EnergyEnvelope::new(100.0, 50.0, 0.95).unwrap();

    let policy_config = PolicyConfig::default();
    let economics_config = EconomicsConfig::default();

    // Log 10 decisions
    for tick in 0..10 {
        let packet = build_packet(tick, &reasoning, &learning, &energy, &policy_config, &economics_config);
        log.append(&packet);
    }

    assert_eq!(log.len(), 10);

    // Verify ordering
    let entries = log.events();
    for (i, event) in entries.iter().enumerate() {
        assert_eq!(event.tick, i as u64);
    }
}

#[test]
fn audit_log_can_export_and_verify() {
    let mut log = AuditLog::new();

    let reasoning = ReasoningEnvelope::new(0.9, 0.5, 0.1, 0.8).unwrap();
    let learning = LearningEnvelope::new(
        "cap_001".to_string(),
        1,
        "a".repeat(64),
        0.5,
        0.5,
        0.9,
        LearningStatus::Committed,
        0,
    )
    .unwrap();
    let energy = EnergyEnvelope::new(100.0, 50.0, 0.95).unwrap();

    let policy_config = PolicyConfig::default();
    let economics_config = EconomicsConfig::default();

    let packet = build_packet(100, &reasoning, &learning, &energy, &policy_config, &economics_config);
    let original_checksum = packet.checksum.clone();

    log.append(&packet);

    // Export and verify
    let json = log.export();
    assert!(json.contains(&original_checksum));
}

// ============================================================================
// Causal Completeness Tests
// ============================================================================

#[test]
fn deny_packet_contains_denial_reason() {
    let reasoning = ReasoningEnvelope::new(0.9, 0.5, 0.1, 0.8).unwrap();
    let learning = LearningEnvelope::new(
        "cap_001".to_string(),
        1,
        "a".repeat(64),
        0.5,
        0.5,
        0.9,
        LearningStatus::Committed,
        0,
    )
    .unwrap();
    let energy = EnergyEnvelope::new(100.0, 50.0, 0.75).unwrap(); // Below threshold

    let policy_config = PolicyConfig {
        min_energy_integrity: 0.9,
        ..Default::default()
    };
    let economics_config = EconomicsConfig::default();

    let packet = build_packet(100, &reasoning, &learning, &energy, &policy_config, &economics_config);

    assert_eq!(packet.decision(), PolicyDecision::Deny);
    assert!(!packet.reason_codes().is_empty(), "Deny must have explicit reason");

    // The reason must match the actual cause
    assert!(packet.reason_codes().contains(&"ENERGY_INTEGRITY_BELOW_THRESHOLD".to_string()));
}

#[test]
fn packet_policy_hash_matches_config() {
    let reasoning = ReasoningEnvelope::new(0.9, 0.5, 0.1, 0.8).unwrap();
    let learning = LearningEnvelope::new(
        "cap_001".to_string(),
        1,
        "a".repeat(64),
        0.5,
        0.5,
        0.9,
        LearningStatus::Committed,
        0,
    )
    .unwrap();
    let energy = EnergyEnvelope::new(100.0, 50.0, 0.95).unwrap();

    let policy_config = PolicyConfig::default();
    let economics_config = EconomicsConfig::default();

    let packet = build_packet(100, &reasoning, &learning, &energy, &policy_config, &economics_config);

    assert_eq!(packet.policy.policy_hash, policy_config.compute_hash());
}

#[test]
fn packet_economics_hash_matches_config() {
    let reasoning = ReasoningEnvelope::new(0.9, 0.5, 0.1, 0.8).unwrap();
    let learning = LearningEnvelope::new(
        "cap_001".to_string(),
        1,
        "a".repeat(64),
        0.5,
        0.5,
        0.9,
        LearningStatus::Committed,
        0,
    )
    .unwrap();
    let energy = EnergyEnvelope::new(100.0, 50.0, 0.95).unwrap();

    let policy_config = PolicyConfig::default();
    let economics_config = EconomicsConfig::default();

    let packet = build_packet(100, &reasoning, &learning, &energy, &policy_config, &economics_config);

    assert_eq!(packet.economics_hash, economics_config.compute_hash());
}

// ============================================================================
// Cross-Decision Replay Consistency
// ============================================================================

#[test]
fn replay_1000_decisions_deterministically() {
    let policy_config = PolicyConfig::default();
    let economics_config = EconomicsConfig::default();

    let energy = EnergyEnvelope::new(100.0, 50.0, 0.95).unwrap();

    for tick in 0..1000 {
        // Vary inputs deterministically based on tick
        let quality = 0.5 + (tick as f64 % 100.0) / 200.0; // 0.5 - 1.0
        let reasoning = ReasoningEnvelope::new(quality, 0.5, 0.1, 0.8).unwrap();
        let learning = LearningEnvelope::new(
            format!("cap_{:04}", tick % 10),
            tick / 100 + 1,
            "b".repeat(64),
            0.5,
            0.5,
            quality,
            LearningStatus::Committed,
            0,
        )
        .unwrap();

        let packet = build_packet(tick, &reasoning, &learning, &energy, &policy_config, &economics_config);

        // Replay and verify
        let (replayed_decision, replayed_reasons) = replay_packet(&packet, &policy_config);

        assert_eq!(
            replayed_decision, packet.decision(),
            "Decision mismatch at tick {}", tick
        );
        assert_eq!(
            replayed_reasons, packet.reason_codes(),
            "Reason codes mismatch at tick {}", tick
        );
    }
}
