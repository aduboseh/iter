use std::fs;
use std::path::PathBuf;

use iter_mcp_server::audit::DecisionPacket;
use jsonschema::{Draft, JSONSchema};
use serde_json::Value;

const SCHEMA: &str = "schemas/v1/decision_packet.schema.json";
const GOLDEN_PACKET: &str = "tests/data/golden_decision_v1.json";

#[test]
fn golden_decision_packet_matches_documented_schema() {
    let schema = load_schema(SCHEMA);
    let validator = JSONSchema::options()
        .with_draft(Draft::Draft7)
        .compile(&schema)
        .expect("decision packet schema compiles");
    let packet = load_json(GOLDEN_PACKET);
    let errors: Vec<String> = match validator.validate(&packet) {
        Ok(_) => Vec::new(),
        Err(errs) => errs.map(|e| e.to_string()).collect(),
    };
    if !errors.is_empty() {
        panic!(
            "Golden DecisionPacket fixture diverged from schema:\n{}",
            errors.join("\n")
        );
    }
}

#[test]
fn golden_decision_packet_checksum_verifies() {
    let packet: DecisionPacket =
        serde_json::from_value(load_json(GOLDEN_PACKET)).expect("golden packet deserializes");
    packet
        .verify_checksum()
        .expect("golden DecisionPacket checksum verifies");
}

fn load_schema(relative: &str) -> Value {
    let path = manifest_path(relative);
    let data = fs::read_to_string(path).expect("read schema");
    serde_json::from_str(&data).expect("valid schema JSON")
}

fn load_json(relative: &str) -> Value {
    let path = manifest_path(relative);
    let data = fs::read_to_string(path).expect("read json");
    serde_json::from_str(&data).expect("valid JSON")
}

fn manifest_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative)
}
