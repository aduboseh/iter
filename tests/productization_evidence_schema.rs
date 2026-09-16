use jsonschema::{Draft, JSONSchema};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

fn fixture() -> Value {
    serde_json::from_str(include_str!("data/productization_evidence_v2.json"))
        .expect("synthetic fixture is valid JSON")
}

fn schema() -> JSONSchema {
    let value: Value = serde_json::from_str(include_str!(
        "../productization/evidence/evidence-v2.schema.json"
    ))
    .expect("schema is valid JSON");
    JSONSchema::options()
        .with_draft(Draft::Draft7)
        .compile(&value)
        .expect("evidence schema compiles")
}

fn object_paths(value: &Value, path: &str, paths: &mut Vec<String>) {
    match value {
        Value::Object(fields) => {
            paths.push(path.to_owned());
            for (key, child) in fields {
                object_paths(child, &format!("{path}/{key}"), paths);
            }
        }
        Value::Array(items) => {
            for (index, child) in items.iter().enumerate() {
                object_paths(child, &format!("{path}/{index}"), paths);
            }
        }
        _ => {}
    }
}

#[test]
fn synthetic_fixture_has_valid_schema_and_byte_digests() {
    let fixture = fixture();
    assert!(fixture["purpose"].as_str().unwrap().contains("SYNTHETIC"));
    assert!(schema().is_valid(&fixture["record"]));
    for artifact in fixture["record"]["artifacts"].as_array().unwrap() {
        let path = artifact["path"].as_str().unwrap();
        let bytes = fixture["files"][path].as_str().unwrap().as_bytes();
        assert_eq!(
            artifact["sha256"],
            format!("{:x}", Sha256::digest(bytes)),
            "fixture bytes must remain bound: {path}"
        );
    }
}

#[test]
fn every_object_rejects_missing_null_and_unknown_fields() {
    let record = &fixture()["record"];
    let schema = schema();
    let mut paths = Vec::new();
    object_paths(record, "", &mut paths);
    for path in paths {
        for key in record.pointer(&path).unwrap().as_object().unwrap().keys() {
            for remove in [true, false] {
                let mut invalid = record.clone();
                let target = invalid.pointer_mut(&path).unwrap().as_object_mut().unwrap();
                if remove {
                    target.remove(key);
                } else {
                    target.insert(key.clone(), Value::Null);
                }
                assert!(!schema.is_valid(&invalid), "{path}/{key}, remove={remove}");
            }
        }
        let mut invalid = record.clone();
        invalid
            .pointer_mut(&path)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert("unexpected".to_owned(), Value::Null);
        assert!(!schema.is_valid(&invalid), "unknown field in {path}");
    }
}

#[test]
fn schema_rejects_wrong_versions_identity_types_and_empty_collections() {
    let record = &fixture()["record"];
    let schema = schema();
    for (path, invalid) in [
        ("/schema_version", json!("apex-productization-evidence/v1")),
        ("/producer/run_id", json!(true)),
        ("/producer/run_id", json!(0)),
        ("/producer/run_attempt", json!(2)),
        ("/subject_commits/iter", json!("A".repeat(40))),
        (
            "/subject_commits/iter",
            json!(format!("{}\n", "a".repeat(40))),
        ),
        (
            "/artifacts/0/sha256",
            json!(format!("{}\n", "a".repeat(64))),
        ),
        ("/control_id", json!("G1-01\n")),
        ("/execution_identity/runner/image", json!("  ")),
        ("/execution_identity/toolchains", json!([])),
        ("/commands", json!([])),
        ("/commands/0/repo", json!("host")),
        ("/commands/0/argv", json!(["cargo", 1])),
        ("/commands/0/argv", json!([])),
        ("/commands/0/argv", json!([""])),
        ("/commands/0/exit_code", json!(false)),
        ("/artifacts", json!([])),
    ] {
        let mut mutated = record.clone();
        *mutated.pointer_mut(path).unwrap() = invalid;
        assert!(!schema.is_valid(&mutated), "invalid value at {path}");
    }
}

#[test]
fn schema_allows_failure_reports_and_negative_test_exit_codes() {
    // Shape validation is not authorization; the consumer separately requires PASS,
    // matching observed/expected exits, and authenticated artifact byte bindings.
    let mut record = fixture()["record"].clone();
    record["result"] = json!("FAIL");
    record["commands"][0]["exit_code"] = json!(101);
    record["commands"][0]["expected_exit"] = json!(101);
    assert!(schema().is_valid(&record));
}

#[test]
fn schema_requires_digest_or_version_for_each_runner_kind() {
    let mut record = fixture()["record"].clone();
    let schema = schema();
    let digest = format!("sha256:{}", "c".repeat(64));
    for (kind, image, valid) in [
        ("container", digest.as_str(), true),
        ("container", "ubuntu:latest", false),
        ("container", "ubuntu-24.04@20260914.1.0", false),
        ("github-hosted", "ubuntu-24.04@20260914.1.0", true),
        ("github-hosted", "windows-2025@20260914.3.0", true),
        ("github-hosted", "ubuntu-latest", false),
        ("github-hosted", "ubuntu-24.04@latest", false),
        ("github-hosted", "ubuntu-24.04@20260914.1.0\n", false),
        ("github-hosted", digest.as_str(), false),
        ("workstation", digest.as_str(), false),
    ] {
        record["execution_identity"]["runner"]["kind"] = json!(kind);
        record["execution_identity"]["runner"]["image"] = json!(image);
        assert_eq!(schema.is_valid(&record), valid, "{kind} {image}");
    }
}
