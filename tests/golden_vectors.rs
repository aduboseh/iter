//! Golden Vector Tests for Cross-Platform Determinism
//!
//! These tests verify that canonicalization produces identical checksums
//! across different platforms (Windows, Linux, macOS) and architectures.
//!
//! INVARIANT: Same input => same canonical bytes => same checksum.
//! If these tests fail on any platform, canonicalization is broken.

use iter_mcp_server::audit::DecisionPacket;
use iter_mcp_server::contracts::{
    EnergyEnvelope, LearningEnvelope, LearningStatus, PolicyDecision, PolicyEnvelope,
    ReasoningEnvelope, SystemState,
};

/// Golden Vector 1: Basic ALLOW decision with committed learning
///
/// This vector represents a healthy system state where:
/// - Reasoning quality is high (0.95)
/// - Learning was committed successfully
/// - Policy allows the action
#[test]
fn golden_vector_1_allow_committed() {
    let energy = EnergyEnvelope::new(100.0, 10.0, 0.95).unwrap();
    let reasoning = ReasoningEnvelope::new(0.95, 0.5, 0.1, 0.8).unwrap();
    let learning = LearningEnvelope::new(
        "capsule_alpha".to_string(),
        42,
        "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2".to_string(),
        0.5,
        0.5,
        1.0,
        LearningStatus::Committed,
        0,
    )
    .unwrap();
    let policy = PolicyEnvelope::new(
        "b1c2d3e4f5a6b1c2d3e4f5a6b1c2d3e4f5a6b1c2d3e4f5a6b1c2d3e4f5a6b1c2".to_string(),
        PolicyDecision::Allow,
        vec![],
    )
    .unwrap();

    let state = SystemState::new(1000, energy, reasoning, learning, policy);

    let packet = DecisionPacket::new(
        "1.0.2".to_string(),
        "scg-11.12.0".to_string(),
        &state,
        None,
        "econ_a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6".to_string(),
        vec![
            "ENERGY_INTEGRITY_GATE".to_string(),
            "REASONING_QUALITY_GATE".to_string(),
            "INPUT_QUALITY_GATE".to_string(),
            "LEARNING_PERMISSION_GATE".to_string(),
            "LEARNING_QUALITY_GATE".to_string(),
        ],
    )
    .unwrap();

    // GOLDEN CHECKSUM: This value MUST be identical across all platforms
    // If this fails on any platform, canonicalization is non-deterministic
    // TODO: Replace placeholder once checksums are stable across CI platforms
    const _EXPECTED_CHECKSUM: &str =
        "a9a8a5e8c38b1d9e3f8c9b7e2a1d4c6f8e9b0a3d5c7e2f1a4b6d8e0c2a4f6b8";

    // Verify checksum structure (64 hex chars)
    assert_eq!(packet.checksum.len(), 64);
    assert!(packet.checksum.chars().all(|c| c.is_ascii_hexdigit()));

    // NOTE: The expected checksum above is a placeholder.
    // Run this test once to get the actual checksum, then update EXPECTED_CHECKSUM.
    // After that, the test becomes a golden vector that must pass on all platforms.
    println!("GOLDEN_VECTOR_1 checksum: {}", packet.checksum);

    // Verify checksum is self-consistent
    assert!(packet.verify_checksum().is_ok());
}

/// Golden Vector 2: FREEZE_LEARNING due to scarcity streak
///
/// This vector represents a system under energy scarcity where:
/// - Learning was rejected due to scarcity
/// - Scarcity streak exceeded threshold
/// - Policy freezes learning
#[test]
fn golden_vector_2_freeze_scarcity() {
    let energy = EnergyEnvelope::new(5.0, 0.5, 0.9).unwrap();
    let reasoning = ReasoningEnvelope::new(0.8, 0.6, 0.2, 0.7).unwrap();
    let learning = LearningEnvelope::new(
        "capsule_beta".to_string(),
        100,
        "deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef".to_string(),
        0.5,
        0.1,
        0.2,
        LearningStatus::RejectedScarcity,
        10, // Scarcity streak at threshold
    )
    .unwrap();
    let policy = PolicyEnvelope::new(
        "cafebabecafebabecafebabecafebabecafebabecafebabecafebabecafebabe".to_string(),
        PolicyDecision::FreezeLearning,
        vec!["SCARCITY_STREAK_EXCEEDED".to_string()],
    )
    .unwrap();

    let state = SystemState::new(5000, energy, reasoning, learning, policy);

    let packet = DecisionPacket::new(
        "1.0.2".to_string(),
        "scg-11.12.0".to_string(),
        &state,
        None,
        "econ_cafebabe00000000000000000000000000000000000000000000000000000000".to_string(),
        vec![
            "ENERGY_INTEGRITY_GATE".to_string(),
            "REASONING_QUALITY_GATE".to_string(),
            "INPUT_QUALITY_GATE".to_string(),
            "LEARNING_PERMISSION_GATE".to_string(),
        ],
    )
    .unwrap();

    println!("GOLDEN_VECTOR_2 checksum: {}", packet.checksum);
    assert!(packet.verify_checksum().is_ok());

    // Verify reason codes present for freeze decision
    assert!(!packet.policy.reason_codes.is_empty());
}

/// Golden Vector 3: DEGRADED_MODE due to low reasoning quality
///
/// This vector represents a system with degraded cortex where:
/// - Reasoning quality is below threshold
/// - Learning rejected due to input quality
/// - Policy enters degraded mode
#[test]
fn golden_vector_3_degraded_mode() {
    let energy = EnergyEnvelope::new(80.0, 20.0, 0.98).unwrap();
    let reasoning = ReasoningEnvelope::new(0.3, 0.4, 0.5, 0.3).unwrap();
    let learning = LearningEnvelope::new(
        "capsule_gamma".to_string(),
        200,
        "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
        0.5,
        0.0,
        0.0,
        LearningStatus::RejectedInputQuality,
        0,
    )
    .unwrap();
    let policy = PolicyEnvelope::new(
        "fedcba0987654321fedcba0987654321fedcba0987654321fedcba0987654321".to_string(),
        PolicyDecision::DegradedMode,
        vec!["REASONING_QUALITY_BELOW_THRESHOLD".to_string()],
    )
    .unwrap();

    let state = SystemState::new(10000, energy, reasoning, learning, policy);

    let packet = DecisionPacket::new(
        "1.0.2".to_string(),
        "scg-11.12.0".to_string(),
        &state,
        Some("permit_abc123def456abc123def456abc123def456abc123def456abc123def456".to_string()),
        "econ_00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff".to_string(),
        vec![
            "ENERGY_INTEGRITY_GATE".to_string(),
            "REASONING_QUALITY_GATE".to_string(),
        ],
    )
    .unwrap();

    println!("GOLDEN_VECTOR_3 checksum: {}", packet.checksum);
    assert!(packet.verify_checksum().is_ok());

    // Verify permit hash is included
    assert!(packet.permit_hash.is_some());
}

/// Golden Vector 4: Boundary values test
///
/// This vector tests edge cases:
/// - Tick at 0
/// - All floats at exact boundary values (0.0, 1.0)
/// - Empty capsule_id
#[test]
fn golden_vector_4_boundary_values() {
    let energy = EnergyEnvelope::new(0.0, 0.0, 1.0).unwrap();
    let reasoning = ReasoningEnvelope::new(1.0, 0.0, 0.0, 1.0).unwrap();
    let learning = LearningEnvelope::new(
        "".to_string(), // Empty capsule ID
        0,
        "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
        0.0,
        0.0,
        0.0,
        LearningStatus::NoProposalNoDelta,
        0,
    )
    .unwrap();
    let policy = PolicyEnvelope::new(
        "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".to_string(),
        PolicyDecision::Allow,
        vec![],
    )
    .unwrap();

    let state = SystemState::new(0, energy, reasoning, learning, policy);

    let packet = DecisionPacket::new(
        "1.0.2".to_string(),
        "scg-11.12.0".to_string(),
        &state,
        None,
        "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
        vec![],
    )
    .unwrap();

    println!("GOLDEN_VECTOR_4 checksum: {}", packet.checksum);
    assert!(packet.verify_checksum().is_ok());
}

/// Golden Vector 5: Large values test
///
/// This vector tests handling of large numbers:
/// - Large tick value
/// - Large energy values
/// - Large epoch
/// - Large scarcity streak
#[test]
fn golden_vector_5_large_values() {
    let energy = EnergyEnvelope::new(1e12, 1e11, 0.999999).unwrap();
    let reasoning = ReasoningEnvelope::new(0.999999, 0.000001, 0.5, 0.5).unwrap();
    let learning = LearningEnvelope::new(
        "capsule_large_values_test_with_long_name".to_string(),
        u64::MAX - 1,
        "abcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcd".to_string(),
        1e10,
        1e10,
        1.0,
        LearningStatus::Committed,
        1000000,
    )
    .unwrap();
    let policy = PolicyEnvelope::new(
        "1111111111111111111111111111111111111111111111111111111111111111".to_string(),
        PolicyDecision::Allow,
        vec![],
    )
    .unwrap();

    let state = SystemState::new(u64::MAX - 1, energy, reasoning, learning, policy);

    let packet = DecisionPacket::new(
        "1.0.2".to_string(),
        "scg-11.12.0".to_string(),
        &state,
        None,
        "2222222222222222222222222222222222222222222222222222222222222222".to_string(),
        vec![
            "RULE_A".to_string(),
            "RULE_B".to_string(),
            "RULE_C".to_string(),
        ],
    )
    .unwrap();

    println!("GOLDEN_VECTOR_5 checksum: {}", packet.checksum);
    assert!(packet.verify_checksum().is_ok());
}

/// Determinism test: Same input must produce same checksum across 10000 iterations
#[test]
fn determinism_iteration_test() {
    let mut checksums = Vec::with_capacity(10000);

    for _ in 0..10000 {
        let energy = EnergyEnvelope::new(100.0, 10.0, 0.95).unwrap();
        let reasoning = ReasoningEnvelope::new(0.9, 0.5, 0.1, 0.8).unwrap();
        let learning = LearningEnvelope::new(
            "test_capsule".to_string(),
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

        let state = SystemState::new(100, energy, reasoning, learning, policy);

        let packet = DecisionPacket::new(
            "1.0.2".to_string(),
            "scg-test".to_string(),
            &state,
            None,
            "c".repeat(64),
            vec!["RULE1".to_string()],
        )
        .unwrap();

        checksums.push(packet.checksum);
    }

    // All checksums must be identical
    let first = &checksums[0];
    for (i, checksum) in checksums.iter().enumerate() {
        assert_eq!(
            checksum, first,
            "Checksum mismatch at iteration {}: {} != {}",
            i, checksum, first
        );
    }
}
