//! Golden Vector Tests — RFC 8785 JCS Checksums
//!
//! These tests verify that JCS canonicalization produces identical checksums
//! across all platforms. Checksums are hardcoded and enforced.
//!
//! INVARIANT: Same input => same JCS bytes => same checksum.
//! If these tests fail on any platform, canonicalization is broken.

use iter_mcp_server::audit::DecisionPacket;
use iter_mcp_server::contracts::{
    EnergyEnvelope, LearningEnvelope, LearningStatus, PolicyDecision, PolicyEnvelope,
    ReasoningEnvelope, SystemState,
};
use iter_mcp_server::economics::EconomicsConfig;
use iter_mcp_server::governed::GovernedRuntime;
use iter_mcp_server::policy::PolicyConfig;
use iter_mcp_server::runtime::{
    replay_decision, GovernanceMode, GovernanceRuntime, GovernanceVerdict,
};
use iter_mcp_server::substrate::stub::{GovernanceProposal, StubRuntime};

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

    assert_eq!(
        packet.checksum, "7c87b26cd45156097179930ec92596386e975e4073c28788fded11e8ae24092a",
        "GOLDEN_VECTOR_1 checksum mismatch — JCS canonicalization changed"
    );
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

    assert_eq!(
        packet.checksum, "0b6a56cd917a330eca3993a8b5f7c93f7df954269ee16bfe1c9269c08bfeab4a",
        "GOLDEN_VECTOR_2 checksum mismatch — JCS canonicalization changed"
    );
    assert!(packet.verify_checksum().is_ok());
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

    assert_eq!(
        packet.checksum, "0e067ce895d600ec5a561c753747bce264ec2d599c3efff34b4dfcc0879f2b14",
        "GOLDEN_VECTOR_3 checksum mismatch — JCS canonicalization changed"
    );
    assert!(packet.verify_checksum().is_ok());
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

    assert_eq!(
        packet.checksum, "ad48002751bfa7b76bee45c3762426cb9865b423d5e444373c0efc22960789b0",
        "GOLDEN_VECTOR_4 checksum mismatch — JCS canonicalization changed"
    );
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

    assert_eq!(
        packet.checksum, "c2ee5f2a35fe2337c56344357d915f19acdd5847b08a73c8597a2ac839cd08d4",
        "GOLDEN_VECTOR_5 checksum mismatch — JCS canonicalization changed"
    );
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

/// Golden Vector 6: Governed-mode evaluate -> replay cycle
///
/// Proves the full governed path:
/// 1. GovernedRuntime.evaluate() produces a DecisionPacket
/// 2. replay_decision() verifies and reproduces the verdict
/// 3. Packet checksum survives the roundtrip
#[test]
fn golden_vector_6_governed_replay_cycle() {
    let mut graph = StubRuntime::new();
    graph.create_node(0.8, 100.0);
    graph.create_node(0.6, 50.0);

    let policy_config = PolicyConfig::default();
    let economics_config = EconomicsConfig::default();
    let mut rt = GovernedRuntime::new(graph, policy_config.clone(), economics_config);

    let proposal = GovernanceProposal {
        proposal_id: "golden-governed-001".to_string(),
        state_snapshot_hash: "a".repeat(64),
        constraints: serde_json::json!({}),
        requested_action: "deploy".to_string(),
        proposal_c14n: None,
        proposal_hash: None,
    };

    let outcome = rt.evaluate(&proposal).expect("governed evaluate");
    assert_eq!(outcome.mode, GovernanceMode::Governed);
    assert!(outcome.authoritative_pdp);
    assert!(outcome.replay_sufficient);

    let packet = outcome.packet.as_ref().expect("governed must emit packet");
    assert!(packet.verify_checksum().is_ok());

    let policy_version = outcome
        .policy_version
        .as_ref()
        .expect("must have policy_version");
    let schema_version = &outcome.schema_version;

    let replayed =
        replay_decision(packet, policy_version, schema_version).expect("replay must succeed");

    assert_eq!(replayed.verdict, outcome.verdict);
    assert_eq!(replayed.mode, GovernanceMode::Governed);
    assert!(replayed.authoritative_pdp);
    assert!(replayed.replay_sufficient);

    let replayed_packet = replayed.packet.expect("replay must return packet");
    assert_eq!(replayed_packet.checksum, packet.checksum);
}

/// Golden Vector 7: replay_decision rejects policy_version mismatch
#[test]
fn golden_vector_7_replay_rejects_version_mismatch() {
    let mut graph = StubRuntime::new();
    graph.create_node(0.8, 100.0);

    let mut rt = GovernedRuntime::new(graph, PolicyConfig::default(), EconomicsConfig::default());

    let proposal = GovernanceProposal {
        proposal_id: "mismatch-test".to_string(),
        state_snapshot_hash: "b".repeat(64),
        constraints: serde_json::json!({}),
        requested_action: "deploy".to_string(),
        proposal_c14n: None,
        proposal_hash: None,
    };

    let outcome = rt.evaluate(&proposal).expect("evaluate");
    let packet = outcome.packet.as_ref().expect("packet");

    let result = replay_decision(packet, "sha256:wrong_hash", "decision_packet:v1");
    assert!(
        result.is_err(),
        "replay must reject policy_version mismatch"
    );

    let result = replay_decision(
        packet,
        outcome.policy_version.as_ref().unwrap(),
        "decision_packet:v999",
    );
    assert!(
        result.is_err(),
        "replay must reject schema_version mismatch"
    );
}

/// RFC 8785 JCS canonicalization regression test.
///
/// Validates that serde_json_canonicalizer produces expected output for
/// a known input. If this test fails after a dependency update, the
/// canonicalizer behavior has changed and all checksums are suspect.
#[test]
fn rfc8785_canonicalizer_regression() {
    let input = serde_json::json!({
        "z_last": true,
        "a_first": 1,
        "m_middle": [3, 2, 1],
        "nested": {
            "beta": "b",
            "alpha": "a"
        }
    });

    let canonical =
        serde_json_canonicalizer::to_string(&input).expect("canonicalization must succeed");

    assert_eq!(
        canonical,
        r#"{"a_first":1,"m_middle":[3,2,1],"nested":{"alpha":"a","beta":"b"},"z_last":true}"#,
        "JCS must sort keys alphabetically and preserve array order"
    );

    let input_reordered = serde_json::json!({
        "m_middle": [3, 2, 1],
        "nested": {
            "alpha": "a",
            "beta": "b"
        },
        "a_first": 1,
        "z_last": true
    });

    let canonical_reordered = serde_json_canonicalizer::to_string(&input_reordered)
        .expect("canonicalization must succeed");

    assert_eq!(
        canonical, canonical_reordered,
        "JCS must produce identical output regardless of input key order"
    );
}

/// Policy hash stability test.
///
/// Confirms that PolicyConfig::compute_hash is deterministic across
/// repeat invocations on identical config. If this fails, policy_version
/// comparisons in replay_decision will reject valid packets.
#[test]
fn policy_hash_stability() {
    let config1 = PolicyConfig::default();
    let config2 = PolicyConfig::default();

    let hash1_a = config1.compute_hash();
    let hash1_b = config1.compute_hash();
    let hash2 = config2.compute_hash();

    assert_eq!(
        hash1_a, hash1_b,
        "PolicyConfig::compute_hash must be deterministic across repeat calls"
    );
    assert_eq!(
        hash1_a, hash2,
        "Identical PolicyConfig values must produce identical hashes"
    );
    assert!(!hash1_a.is_empty(), "PolicyConfig hash must not be empty");
}

/// Golden Vector 8: Governed verdict matches policy evaluator decision
#[test]
fn golden_vector_8_governed_verdict_from_policy() {
    let mut graph = StubRuntime::new();
    graph.create_node(0.9, 100.0);
    graph.create_node(0.8, 80.0);

    let mut rt = GovernedRuntime::new(graph, PolicyConfig::default(), EconomicsConfig::default());

    let proposal = GovernanceProposal {
        proposal_id: "verdict-test".to_string(),
        state_snapshot_hash: "c".repeat(64),
        constraints: serde_json::json!({}),
        requested_action: "deploy".to_string(),
        proposal_c14n: None,
        proposal_hash: None,
    };

    let outcome = rt.evaluate(&proposal).expect("evaluate");

    assert_eq!(
        outcome.verdict,
        GovernanceVerdict::Allow,
        "healthy graph with default policy must produce Allow"
    );

    for code in &outcome.reason_codes {
        assert!(
            code.as_str().starts_with("policy."),
            "governed reason code must use policy.* namespace"
        );
    }
}
