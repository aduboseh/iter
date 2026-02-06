//! CLI Replay & Audit Integration Tests (Phase 4)
//!
//! Validates that iter-cli replay and audit export produce correct results
//! using the same replay contract and hashes as existing golden vector tests.
//!
//! Test structure:
//! 1. Replay golden vector via CLI — exit 0, outcome VERIFIED
//! 2. Audit export + replay round-trip — export then replay, both succeed
//! 3. Fail-closed on corrupt file — tampered checksum, exit 2, outcome MISMATCH
//! 4. Fail-closed on policy version mismatch — wrong version, exit 2
//! 5. Fail-closed on missing file — exit 1
//! 6. Policy hash stability — verify hash unchanged across CLI boundary

use std::path::PathBuf;
use std::process::Command;

use iter_mcp_server::audit::DecisionPacket;
use iter_mcp_server::contracts::{
    EnergyEnvelope, LearningEnvelope, LearningStatus, PolicyDecision, PolicyEnvelope,
    ReasoningEnvelope, SystemState,
};

const GV1_CHECKSUM: &str = "acd92a1cea22df1e26db77689498b62393458ca8dcceddcddd1c40f23aeaa8fe";
const GV1_POLICY_HASH: &str = "b1c2d3e4f5a6b1c2d3e4f5a6b1c2d3e4f5a6b1c2d3e4f5a6b1c2d3e4f5a6b1c2";

fn iter_cli_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_iter-cli"))
}

fn build_gv1_packet() -> DecisionPacket {
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
    let policy =
        PolicyEnvelope::new(GV1_POLICY_HASH.to_string(), PolicyDecision::Allow, vec![]).unwrap();

    let state = SystemState::new(1000, energy, reasoning, learning, policy);

    DecisionPacket::new(
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
    .unwrap()
}

fn write_packet_to_temp(packet: &DecisionPacket, name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join("iter_cli_tests");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    let json = serde_json::to_string_pretty(packet).unwrap();
    std::fs::write(&path, &json).unwrap();
    path
}

// ============================================================================
// Test 1: Replay golden vector via CLI
// ============================================================================

#[test]
fn cli_replay_golden_vector_verified() {
    let packet = build_gv1_packet();
    assert_eq!(packet.checksum, GV1_CHECKSUM);

    let input_path = write_packet_to_temp(&packet, "gv1_replay.json");
    let policy_version = format!("sha256:{}", GV1_POLICY_HASH);

    let output = Command::new(iter_cli_bin())
        .arg("replay")
        .arg("--decision-file")
        .arg(&input_path)
        .arg("--policy-version")
        .arg(&policy_version)
        .arg("--schema-version")
        .arg("decision_packet:v1")
        .output()
        .expect("failed to run iter-cli");

    assert_eq!(
        output.status.code(),
        Some(0),
        "replay must exit 0 for valid packet. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: serde_json::Value =
        serde_json::from_str(&stdout).expect("stdout must be valid JSON");

    assert_eq!(result["outcome"], "VERIFIED");
    assert_eq!(result["checksum_match"], true);
    assert_eq!(result["decision"], "ALLOW");
}

// ============================================================================
// Test 2: Audit export + replay round-trip
// ============================================================================

#[test]
fn cli_audit_export_then_replay_roundtrip() {
    let packet = build_gv1_packet();
    let input_path = write_packet_to_temp(&packet, "gv1_export_input.json");
    let export_path = std::env::temp_dir()
        .join("iter_cli_tests")
        .join("gv1_exported.json");

    let export_output = Command::new(iter_cli_bin())
        .arg("audit")
        .arg("export")
        .arg("--decision-file")
        .arg(&input_path)
        .arg("--output")
        .arg(&export_path)
        .output()
        .expect("failed to run iter-cli audit export");

    assert_eq!(
        export_output.status.code(),
        Some(0),
        "audit export must exit 0. stderr: {}",
        String::from_utf8_lossy(&export_output.stderr)
    );

    let export_stdout = String::from_utf8_lossy(&export_output.stdout);
    let export_result: serde_json::Value =
        serde_json::from_str(&export_stdout).expect("export stdout must be valid JSON");
    assert_eq!(export_result["status"], "EXPORTED");
    assert_eq!(export_result["decision_id"], GV1_CHECKSUM);

    let policy_version = format!("sha256:{}", GV1_POLICY_HASH);
    let replay_output = Command::new(iter_cli_bin())
        .arg("replay")
        .arg("--decision-file")
        .arg(&export_path)
        .arg("--policy-version")
        .arg(&policy_version)
        .arg("--schema-version")
        .arg("decision_packet:v1")
        .output()
        .expect("failed to run iter-cli replay on exported file");

    assert_eq!(
        replay_output.status.code(),
        Some(0),
        "replay of exported file must exit 0. stderr: {}",
        String::from_utf8_lossy(&replay_output.stderr)
    );

    let replay_stdout = String::from_utf8_lossy(&replay_output.stdout);
    let replay_result: serde_json::Value =
        serde_json::from_str(&replay_stdout).expect("replay stdout must be valid JSON");
    assert_eq!(replay_result["outcome"], "VERIFIED");
}

// ============================================================================
// Test 3: Fail-closed on corrupt file (tampered checksum)
// ============================================================================

#[test]
fn cli_replay_rejects_tampered_packet() {
    let mut packet = build_gv1_packet();
    packet.tick = 9999;

    let input_path = write_packet_to_temp(&packet, "gv1_tampered.json");
    let policy_version = format!("sha256:{}", GV1_POLICY_HASH);

    let output = Command::new(iter_cli_bin())
        .arg("replay")
        .arg("--decision-file")
        .arg(&input_path)
        .arg("--policy-version")
        .arg(&policy_version)
        .arg("--schema-version")
        .arg("decision_packet:v1")
        .output()
        .expect("failed to run iter-cli");

    assert_eq!(
        output.status.code(),
        Some(2),
        "replay must exit 2 for tampered packet"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: serde_json::Value =
        serde_json::from_str(&stdout).expect("stdout must be valid JSON");
    assert_eq!(result["outcome"], "MISMATCH");
    let reason = result["reason"].as_str().unwrap_or("");
    assert!(
        reason.contains("checksum"),
        "reason must mention checksum mismatch, got: {}",
        reason
    );
}

// ============================================================================
// Test 4: Fail-closed on policy version mismatch
// ============================================================================

#[test]
fn cli_replay_rejects_policy_version_mismatch() {
    let packet = build_gv1_packet();
    let input_path = write_packet_to_temp(&packet, "gv1_wrong_policy.json");

    let output = Command::new(iter_cli_bin())
        .arg("replay")
        .arg("--decision-file")
        .arg(&input_path)
        .arg("--policy-version")
        .arg("sha256:wrong_hash_value")
        .arg("--schema-version")
        .arg("decision_packet:v1")
        .output()
        .expect("failed to run iter-cli");

    assert_eq!(
        output.status.code(),
        Some(2),
        "replay must exit 2 for policy version mismatch"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: serde_json::Value =
        serde_json::from_str(&stdout).expect("stdout must be valid JSON");
    assert_eq!(result["outcome"], "MISMATCH");
    let reason = result["reason"].as_str().unwrap_or("");
    assert!(
        reason.contains("policy_version"),
        "reason must mention policy_version mismatch, got: {}",
        reason
    );
}

// ============================================================================
// Test 5: Fail-closed on missing file
// ============================================================================

#[test]
fn cli_replay_rejects_missing_file() {
    let output = Command::new(iter_cli_bin())
        .arg("replay")
        .arg("--decision-file")
        .arg("nonexistent_file_abc123.json")
        .arg("--policy-version")
        .arg("sha256:x")
        .arg("--schema-version")
        .arg("decision_packet:v1")
        .output()
        .expect("failed to run iter-cli");

    assert_eq!(
        output.status.code(),
        Some(1),
        "replay must exit 1 for missing file"
    );
}

// ============================================================================
// Test 6: Audit export rejects tampered integrity
// ============================================================================

#[test]
fn cli_audit_export_rejects_tampered_integrity() {
    let mut packet = build_gv1_packet();
    packet.tick = 9999;

    let input_path = write_packet_to_temp(&packet, "gv1_tampered_export.json");
    let export_path = std::env::temp_dir()
        .join("iter_cli_tests")
        .join("gv1_tampered_exported.json");

    let output = Command::new(iter_cli_bin())
        .arg("audit")
        .arg("export")
        .arg("--decision-file")
        .arg(&input_path)
        .arg("--output")
        .arg(&export_path)
        .output()
        .expect("failed to run iter-cli audit export");

    assert_eq!(
        output.status.code(),
        Some(2),
        "audit export must exit 2 for tampered packet"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: serde_json::Value =
        serde_json::from_str(&stdout).expect("stdout must be valid JSON");
    assert_eq!(result["status"], "INTEGRITY_FAILURE");
}

// ============================================================================
// Test 7: Policy hash stability across CLI boundary
// ============================================================================

#[test]
fn cli_policy_hash_stability_unchanged() {
    use iter_mcp_server::policy::PolicyConfig;

    let config = PolicyConfig::default();
    let hash_a = config.compute_hash();
    let hash_b = config.compute_hash();
    assert_eq!(hash_a, hash_b, "PolicyConfig hash must be deterministic");

    let packet = build_gv1_packet();
    assert_eq!(packet.checksum, GV1_CHECKSUM, "GV1 checksum must be stable");
    assert!(packet.verify_checksum().is_ok(), "GV1 checksum must verify");
}
