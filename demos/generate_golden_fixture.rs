//! Generates golden DecisionPacket fixture for CLI integration tests.
//!
//! Writes tests/data/golden_decision_v1.json — identical to Golden Vector 1
//! in tests/golden_vectors.rs. The fixture file is deterministic and the
//! checksum is hardcoded in the golden vector test.
//!
//! Run: cargo run --example generate_golden_fixture

use iter_mcp_server::audit::DecisionPacket;
use iter_mcp_server::contracts::{
    EnergyEnvelope, LearningEnvelope, LearningStatus, PolicyDecision, PolicyEnvelope,
    ReasoningEnvelope, SystemState,
};
use std::fs;
use std::path::Path;

fn main() {
    let energy = EnergyEnvelope::new(100.0, 10.0, 0.95).expect("valid energy");
    let reasoning = ReasoningEnvelope::new(0.95, 0.5, 0.1, 0.8).expect("valid reasoning");
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
    .expect("valid learning");
    let policy = PolicyEnvelope::new(
        "b1c2d3e4f5a6b1c2d3e4f5a6b1c2d3e4f5a6b1c2d3e4f5a6b1c2d3e4f5a6b1c2".to_string(),
        PolicyDecision::Allow,
        vec![],
    )
    .expect("valid policy");

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
    .expect("valid packet");

    assert_eq!(
        packet.checksum, "7c87b26cd45156097179930ec92596386e975e4073c28788fded11e8ae24092a",
        "Generated packet checksum does not match Golden Vector 1"
    );

    let dir = Path::new("tests/data");
    fs::create_dir_all(dir).expect("failed to create tests/data directory");

    let json = serde_json::to_string_pretty(&packet).expect("serialization");
    let output_path = dir.join("golden_decision_v1.json");
    fs::write(&output_path, &json).expect("failed to write fixture");

    eprintln!("Written: {}", output_path.display());
    eprintln!("Checksum: {}", packet.checksum);
    eprintln!("Policy version: sha256:{}", packet.policy.policy_hash);
}
