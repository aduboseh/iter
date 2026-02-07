use std::fs;
use std::path::PathBuf;

use iter_mcp_server::substrate::stub::{
    AuditSearchFilter, AuditSearchResult, DecisionPreview, DecisionSummary, DeterminismProof,
    GovernanceProposal, GovernanceVerdict,
};
use jsonschema::{Draft, JSONSchema};
use serde_json::{json, Value};

const SCHEMA_DIR: &str = "schemas/v1";
const DATA_DIR: &str = "tests/data";

#[test]
fn schema_integrity() {
    validate_decision_packet_fixture();
    validate_decision_preview_sample();
    validate_decision_check_request_sample();
    validate_audit_search_filter_sample();
    validate_audit_search_result_sample();
}

fn schema_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(SCHEMA_DIR)
        .join(name)
}

fn data_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(DATA_DIR)
        .join(name)
}

fn compile_schema(value: &Value) -> JSONSchema {
    JSONSchema::options()
        .with_draft(Draft::Draft7)
        .compile(value)
        .expect("schema compiles")
}

fn validate_value(schema: &JSONSchema, instance: &Value) {
    if let Err(err) = schema.validate(instance) {
        panic!(
            "schema validation failed:\n{}\ninstance:\n{}",
            err.collect::<Vec<_>>()
                .into_iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join("\n"),
            instance
        );
    }
}

fn load_schema(name: &str) -> Value {
    let contents = fs::read_to_string(schema_path(name)).expect("schema exists");
    serde_json::from_str(&contents).expect("valid schema JSON")
}

fn decision_packet_schema() -> JSONSchema {
    let value = load_schema("decision_packet.schema.json");
    compile_schema(&value)
}

fn decision_preview_schema() -> JSONSchema {
    let value = load_schema("decision_preview.schema.json");
    compile_schema(&value)
}

fn decision_check_request_schema() -> JSONSchema {
    let value = load_schema("decision_check_request.schema.json");
    compile_schema(&value)
}

fn audit_search_definition(name: &str) -> Value {
    let value = load_schema("audit_search.schema.json");
    let mut definition = value["definitions"][name].clone();
    definition
        .as_object_mut()
        .unwrap()
        .entry("$schema".to_string())
        .or_insert(Value::String(
            "http://json-schema.org/draft-07/schema#".to_string(),
        ));
    definition
}

fn validate_decision_packet_fixture() {
    let schema = decision_packet_schema();
    let contents =
        fs::read_to_string(data_path("golden_decision_v1.json")).expect("fixture exists");
    let instance: Value = serde_json::from_str(&contents).expect("valid JSON");
    validate_value(&schema, &instance);
}

fn validate_decision_preview_sample() {
    let schema = decision_preview_schema();
    let preview = DecisionPreview {
        preview_version: "1.0".to_string(),
        simulation: true,
        request: json!({
            "proposal_id": "sample-proposal",
            "state_snapshot_hash": "a".repeat(64),
            "requested_action": "deploy"
        }),
        verdict: GovernanceVerdict::Allow,
        determinism: DeterminismProof {
            drift_ok: true,
            energy_drift: 0.0,
            coherence: 0.9,
        },
        constraints: json!({}),
        obligations: json!({}),
        policy_trace: vec!["ENERGY_INTEGRITY_GATE".to_string()],
        checksum_preview: "preview-hash".to_string(),
        derived_from: "decision.check@1".to_string(),
    };
    let instance = serde_json::to_value(&preview).expect("serialize preview");
    validate_value(&schema, &instance);
}

fn validate_decision_check_request_sample() {
    let schema = decision_check_request_schema();
    let proposal = GovernanceProposal {
        proposal_id: "sample-proposal".to_string(),
        state_snapshot_hash: "b".repeat(64),
        constraints: json!({}),
        requested_action: "deploy".to_string(),
        proposal_c14n: None,
        proposal_hash: Some("hash".to_string()),
    };
    let instance = serde_json::to_value(&proposal).expect("serialize proposal");
    validate_value(&schema, &instance);
}

fn validate_audit_search_filter_sample() {
    let schema_value = audit_search_definition("AuditSearchFilter");
    let schema = compile_schema(&schema_value);
    let filter = AuditSearchFilter {
        principal: Some("policy".to_string()),
        action: Some("evaluate".to_string()),
        resource: Some("proposal".to_string()),
        decision: Some("ALLOW".to_string()),
        policy_id: Some("policy-1".to_string()),
        from: Some("2026-01-01T00:00:00Z".to_string()),
        to: Some("2026-12-31T00:00:00Z".to_string()),
        limit: Some(10),
    };
    let instance = serde_json::to_value(&filter).expect("serialize filter");
    validate_value(&schema, &instance);
}

fn validate_audit_search_result_sample() {
    let schema_value = audit_search_definition("AuditSearchResult");
    let schema = compile_schema(&schema_value);
    let result = AuditSearchResult {
        results: vec![DecisionSummary {
            decision_id: "decision-1".to_string(),
            principal: "policy".to_string(),
            action: "evaluate".to_string(),
            resource: "proposal".to_string(),
            decision: "ALLOW".to_string(),
            timestamp: "2026-02-06T00:00:00Z".to_string(),
        }],
        count: 1,
        ordering: "(timestamp_utc,decision_id) ASC".to_string(),
    };
    let instance = serde_json::to_value(&result).expect("serialize result");
    validate_value(&schema, &instance);
}
