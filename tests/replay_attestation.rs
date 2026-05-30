// tests/replay_attestation.rs
//
// WO-ITER-REPLAY-ATTEST-001 — SCG-backed replay seam attestation
//
// Seam: response.json() [simulated via serde round-trip]
//        -> verify_replay_id()
//            -> contract_version gate
//            -> validate_semantics()
//                -> validate_schema_version()
//                -> validate_chain()
//                -> validate_sequence()
//                -> validate_completeness()
//                -> verify_hash_bindings_for_all_steps()
//            -> compute_replay_id() == self.replay_id
//
// Dual-hash contract:
//   Step level:   payload_hash() -> UPPERCASE (hex::encode_upper)
//   Replay level: sha256_hex()   -> lowercase (hex::encode)

use governance_bridge::{
    contract::{
        Decision, GovernanceOutcome, GovernanceStateEnvelope, CONTRACT_VERSION_STR,
        STATE_ENVELOPE_SCHEMA,
    },
    trace::{canonicalize, payload_hash, ExecutionTrace, OperationType, TraceStep},
};
use sha2::{Digest, Sha256};
use std::fs;

const ATTESTATION_GOVERNANCE_HASH: &str =
    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const SNAPSHOT_VECTOR_ID: &str = "V3";
const SNAPSHOT_ORACLE_PATH: &str = "tests/snapshots/canonical_v3_nested_sort.bin";

fn canonical_vectors_path() -> &'static str {
    concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/vendor/governance-bridge/CANONICAL_VECTORS.json"
    )
}

fn canonical_vectors_raw_bytes() -> Vec<u8> {
    fs::read(canonical_vectors_path()).expect("CANONICAL_VECTORS.json must be readable")
}

fn canonical_vectors_raw_text() -> String {
    String::from_utf8(canonical_vectors_raw_bytes())
        .expect("CANONICAL_VECTORS.json must be valid UTF-8")
}

fn sha256_lower(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

fn raw_sha256_fields(text: &str) -> Vec<String> {
    let marker = "\"sha256\": \"";
    let mut rest = text;
    let mut values = Vec::new();
    while let Some(start) = rest.find(marker) {
        let after_marker = &rest[start + marker.len()..];
        let end = after_marker
            .find('"')
            .expect("sha256 field must close with a quote");
        values.push(after_marker[..end].to_string());
        rest = &after_marker[end + 1..];
    }
    values
}

fn is_uppercase_hex_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'A'..=b'F').contains(&byte))
}

// Exact order required by validate_sequence() for trace.v1.
// Any deviation fails validate_sequence() or validate_completeness().
const GOVERNED_SEQUENCE: [(OperationType, &str); 5] = [
    (OperationType::HashVerify, "hash_verify"),
    (OperationType::PolicyEval, "policy_eval"),
    (OperationType::StateCheck, "state_check"),
    (OperationType::DecisionEmit, "decision_emit"),
    (OperationType::TraceFinalize, "trace_finalize"),
];

fn load_canonical_vectors() -> Vec<serde_json::Value> {
    let vectors_raw = canonical_vectors_raw_bytes();
    let manifest: serde_json::Value =
        serde_json::from_slice(&vectors_raw).expect("CANONICAL_VECTORS.json must parse");

    manifest["vectors"]
        .as_array()
        .expect("top-level 'vectors' array must exist")
        .clone()
}

#[test]
fn smoke_008_canonical_vectors_raw_byte_hash_locked() {
    let raw = canonical_vectors_raw_bytes();
    assert_eq!(
        sha256_lower(&raw),
        env!("ITER_CANONICAL_VECTORS_SHA256"),
        "SMOKE-008: CANONICAL_VECTORS.json integrity must be computed over exact raw bytes"
    );
}

#[test]
fn smoke_009_canonical_vector_digest_casing_preserved() {
    let raw_text = canonical_vectors_raw_text();
    let digests = raw_sha256_fields(&raw_text);
    assert_eq!(
        digests.len(),
        4,
        "SMOKE-009: expected four raw sha256 fields in CANONICAL_VECTORS.json"
    );

    for digest in digests {
        assert!(
            is_uppercase_hex_digest(&digest),
            "SMOKE-009: canonical vector digest must preserve uppercase hex exactly: {}",
            digest
        );
    }
}

fn find_vector_by_id<'a>(vectors: &'a [serde_json::Value], id: &str) -> &'a serde_json::Value {
    vectors
        .iter()
        .find(|vector| vector["id"].as_str() == Some(id))
        .unwrap_or_else(|| panic!("Vector {} must exist", id))
}

fn canonical_bytes_from_input(input: &serde_json::Value) -> Vec<u8> {
    let canonical = canonicalize(input);
    serde_json::to_vec(&canonical).expect("canonical serialization must not fail")
}

fn canonical_string_from_input(input: &serde_json::Value) -> String {
    String::from_utf8(canonical_bytes_from_input(input))
        .expect("canonical bytes must be valid UTF-8")
}

fn snapshot_source_materials() -> (String, String, Vec<u8>) {
    let vectors = load_canonical_vectors();
    let vector = find_vector_by_id(&vectors, SNAPSHOT_VECTOR_ID);

    let published_canonical_serialized = vector["canonical_serialized"]
        .as_str()
        .unwrap_or_else(|| panic!("Vector {} missing canonical_serialized", SNAPSHOT_VECTOR_ID))
        .to_string();
    let expected_sha256_upper = vector["sha256"]
        .as_str()
        .unwrap_or_else(|| panic!("Vector {} missing sha256", SNAPSHOT_VECTOR_ID))
        .to_string();
    let actual_bytes = canonical_bytes_from_input(&vector["input"]);
    let actual_canonical =
        String::from_utf8(actual_bytes.clone()).expect("canonical bytes must be valid UTF-8");

    assert_eq!(
        actual_canonical, published_canonical_serialized,
        "snapshot source vector must round-trip to the published canonical string"
    );

    (actual_canonical, expected_sha256_upper, actual_bytes)
}

/// Build a fully governed 5-step execution trace over a single canonical payload.
///
/// Chain rule:
///   step[0].input_hash  = step_hash_upper  (no previous step)
///   step[n].input_hash  = step[n-1].output_hash
///   step[n].output_hash = step_hash_upper  (uniform payload)
///
/// Because all output_hashes are identical, chain is trivially valid.
/// verify_hash_binding() confirms payload -> hash on every step independently.
fn build_governed_trace(canonical_payload: &str) -> (ExecutionTrace, String) {
    let step_hash_upper =
        payload_hash(canonical_payload).expect("canonical_payload must be valid JSON");

    let mut steps: Vec<TraceStep> = Vec::with_capacity(5);
    let mut prev_output = step_hash_upper.clone();

    for (op_type, op_str) in &GOVERNED_SEQUENCE {
        let input_hash = if steps.is_empty() {
            step_hash_upper.clone()
        } else {
            prev_output.clone()
        };
        let output_hash = step_hash_upper.clone();

        steps.push(TraceStep {
            region_id: "attestation.canonical_vectors".to_string(),
            operation: op_str.to_string(),
            operation_type: *op_type,
            input_hash,
            output_hash: output_hash.clone(),
            input_payload: canonical_payload.to_string(),
            output_payload: canonical_payload.to_string(),
        });

        prev_output = output_hash;
    }

    (ExecutionTrace::from_steps(steps), step_hash_upper)
}

#[test]
fn att_iter_replay_canonical_vectors() {
    let vectors = load_canonical_vectors();

    assert_eq!(
        vectors.len(),
        4,
        "Expected exactly 4 canonical vectors — schema_version may have changed"
    );

    for vector in &vectors {
        let id = vector["id"].as_str().expect("vector.id must be a string");
        let name = vector["name"]
            .as_str()
            .expect("vector.name must be a string");

        let published_canonical_serialized = vector["canonical_serialized"]
            .as_str()
            .unwrap_or_else(|| panic!("Vector {} ({}) missing canonical_serialized", id, name));
        let canonical_serialized = canonical_string_from_input(&vector["input"]);

        assert_eq!(
            canonical_serialized, published_canonical_serialized,
            "\nCANONICAL SERIALIZATION MISMATCH — Vector {} ({})\
             \n  published:  {}\
             \n  derived:    {}\
             \nThis means canonicalize(input) drifted before hashing.",
            id, name, published_canonical_serialized, canonical_serialized
        );

        // The contract artifact publishes uppercase SHA-256 digests.
        // Preserve exact casing here so any casing regression fails closed.
        let expected_sha256_upper = vector["sha256"]
            .as_str()
            .unwrap_or_else(|| panic!("Vector {} ({}) missing sha256", id, name));

        // -- Layer 1: canonicalize(input) -> payload_hash must match vector sha256 --
        // Fires BEFORE verify_replay_id(). A mismatch here means
        // Iter's canonicalize() diverged from SCG's — the primary risk.
        let (trace, step_hash_upper) = build_governed_trace(&canonical_serialized);

        assert_eq!(
            step_hash_upper, expected_sha256_upper,
            "\nCANONICAL HASH MISMATCH (Layer 1) — Vector {} ({})\
             \n  expected (vector):  {}\
             \n  computed:           {}\
             \n  payload:            {}\
             \nDo not patch. File a new WO.",
            id, name, expected_sha256_upper, step_hash_upper, canonical_serialized
        );

        // -- Layer 2: compute replay_id over full outcome -------------------
        // NOT the vector sha256. Covers {contract_version, decision,
        // governance_hash, state envelope proof, execution_trace} -> lowercase hex.
        let state_envelope =
            GovernanceStateEnvelope::new("snapshot-001".to_string(), 110.0, 0.0, 1, 0);
        let state_envelope_hash = state_envelope.compute_hash();
        let replay_id = GovernanceOutcome::compute_replay_id(
            CONTRACT_VERSION_STR,
            &Decision::Allow,
            ATTESTATION_GOVERNANCE_HASH,
            &state_envelope.state_snapshot_hash,
            STATE_ENVELOPE_SCHEMA,
            &state_envelope_hash,
            &trace,
        );

        let outcome = GovernanceOutcome {
            contract_version: CONTRACT_VERSION_STR.to_string(),
            decision: Decision::Allow,
            governance_hash: ATTESTATION_GOVERNANCE_HASH.to_string(),
            state_snapshot_hash: state_envelope.state_snapshot_hash.clone(),
            state_envelope_schema: STATE_ENVELOPE_SCHEMA.to_string(),
            state_envelope_hash,
            state_envelope,
            execution_trace: trace,
            replay_id,
        };

        // -- Serde round-trip (mirrors response.json() path) ----------------
        let fixture_json = serde_json::to_value(&outcome)
            .unwrap_or_else(|e| panic!("Vector {} ({}) serialization failed: {}", id, name, e));

        let deserialized: GovernanceOutcome =
            serde_json::from_value(fixture_json).unwrap_or_else(|e| {
                panic!(
                    "Vector {} ({}) deserialization failed: {}\
                 \nSerde path is broken — not just the hash.",
                    id, name, e
                )
            });

        // -- Production boundary call ---------------------------------------
        let result = deserialized.verify_replay_id();

        assert!(
            result.is_ok(),
            "\nREPLAY VERIFICATION FAILED — Vector {} ({})\
             \n  error:    {:?}\
             \n  payload:  {}\
             \n  sha256:   {}\
             \n  step_hash (UPPER): {}",
            id,
            name,
            result.err(),
            canonical_serialized,
            expected_sha256_upper,
            step_hash_upper
        );

        println!(
            "  ✅  Vector {} ({})  sha256: {}  replay_id: {}...",
            id,
            name,
            expected_sha256_upper,
            &deserialized.replay_id[..16]
        );
    }

    println!(
        "\nATTESTATION COMPLETE\
         \n  Seam:    response.json -> verify_replay_id()\
         \n  Layer 1: canonicalize(input) + payload_hash (UPPERCASE step binding): confirmed\
         \n  Layer 2: compute_replay_id (lowercase outcome digest): confirmed\
         \n  Serde:   round-trip through from_value: confirmed"
    );
}

// -- LAYER 3: BYTE-LEVEL CANONICAL SNAPSHOT ----------------------------------
//
// Closes failure class: serde nuance or whitespace change that passes
// semantic tests but mutates the actual bytes used in hash computation.
//
// Bootstrap: run with REGEN_SNAPSHOT=1 cargo test snapshot_regen -- --nocapture
//   to regenerate the oracle. Only do this under a new WO after full
//   attestation re-run. Never regenerate silently.
//
// Oracle location: tests/snapshots/canonical_v3_nested_sort.bin
// Source vector:   CANONICAL_VECTORS.json V3 (nested_sort)
//   input:                {"outer":{"q":2,"p":1},"a":true}
//   canonical_serialized: {"a":true,"outer":{"p":1,"q":2}}
//   expected sha256:      A21309223AD721E35FCBE45F1C4DAD9DC35A53971C2FD5BA1B98F0BDAA93E859

#[test]
fn snapshot_regen() {
    // Only runs when REGEN_SNAPSHOT=1 is set.
    // Never runs in CI. Gate is explicit, not accidental.
    if std::env::var("REGEN_SNAPSHOT").unwrap_or_default() != "1" {
        println!("snapshot_regen: skipped (set REGEN_SNAPSHOT=1 to run)");
        return;
    }

    let (canonical_serialized, expected_sha256_upper, bytes) = snapshot_source_materials();
    let actual_sha256_upper = payload_hash(&canonical_serialized).expect("payload_hash failed");

    assert_eq!(
        actual_sha256_upper, expected_sha256_upper,
        "\nSNAPSHOT REGEN REFUSED\
         \n  expected sha256: {}\
         \n  actual sha256:   {}\
         \n  payload:         {}\
         \nOracle was not written.",
        expected_sha256_upper, actual_sha256_upper, canonical_serialized
    );

    std::fs::write(SNAPSHOT_ORACLE_PATH, &bytes).expect("failed to write snapshot oracle");
    println!("SNAPSHOT ORACLE WRITTEN: {} bytes", bytes.len());
    println!("canonical_serialized: {}", canonical_serialized);
    println!("sha256 of oracle: {}", actual_sha256_upper);
    println!("Expected:         {}", expected_sha256_upper);
}

#[test]
fn canonical_json_byte_snapshot_stable() {
    // Enforces that canonicalize() produces byte-identical output to
    // the committed oracle. Fails if serde_json internals change byte
    // layout even while semantic equivalence is preserved.
    //
    // If this test fails after a serde_json version change:
    //   That is the pin working correctly. Do not update the snapshot
    //   without re-running full attestation. File WO-SCG-SERDE-PIN-002.
    let (canonical_serialized, expected_sha256_upper, actual_bytes) = snapshot_source_materials();
    let oracle = include_bytes!("snapshots/canonical_v3_nested_sort.bin");

    assert_eq!(
        payload_hash(&canonical_serialized).expect("payload_hash failed"),
        expected_sha256_upper,
        "snapshot source vector hash must match the vendored contract artifact"
    );

    assert_eq!(
        actual_bytes.as_slice(),
        oracle.as_ref(),
        "\nCANONICAL BYTE DRIFT DETECTED\
         \n  actual len:   {}\
         \n  oracle len:   {}\
         \n  actual hex:   {}\
         \n  oracle hex:   {}\
         \nThis is a cryptographic contract violation.\
         \nDo not update the snapshot without re-running full attestation.\
         \nFile WO-SCG-SERDE-PIN-002 if serde_json was updated.",
        actual_bytes.len(),
        oracle.len(),
        hex::encode(actual_bytes.as_slice()),
        hex::encode(oracle.as_ref())
    );
}
