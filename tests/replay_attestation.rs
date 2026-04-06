// tests/replay_attestation.rs
//
// WO-ITER-REPLAY-ATTEST-001 — SCG-backed replay seam attestation
//
// Seam: response.json() [simulated via serde round-trip]
//        → verify_replay_id()
//            → contract_version gate
//            → validate_semantics()
//                → validate_schema_version()
//                → validate_chain()
//                → validate_sequence()
//                → validate_completeness()
//                → verify_hash_bindings_for_all_steps()
//            → compute_replay_id() == self.replay_id
//
// Dual-hash contract:
//   Step level:   payload_hash() → UPPERCASE (hex::encode_upper)
//   Replay level: sha256_hex()   → lowercase (hex::encode)

use governance_bridge::{
    contract::{Decision, GovernanceOutcome, CONTRACT_VERSION_STR},
    trace::{payload_hash, ExecutionTrace, OperationType, TraceStep},
};
use std::fs;

const ATTESTATION_GOVERNANCE_HASH: &str =
    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

// Exact order required by validate_sequence() for trace.v1.
// Any deviation fails validate_sequence() or validate_completeness().
const GOVERNED_SEQUENCE: [(OperationType, &str); 5] = [
    (OperationType::HashVerify, "hash_verify"),
    (OperationType::PolicyEval, "policy_eval"),
    (OperationType::StateCheck, "state_check"),
    (OperationType::DecisionEmit, "decision_emit"),
    (OperationType::TraceFinalize, "trace_finalize"),
];

/// Build a fully governed 5-step execution trace over a single canonical payload.
///
/// Chain rule:
///   step[0].input_hash  = step_hash_upper  (no previous step)
///   step[n].input_hash  = step[n-1].output_hash
///   step[n].output_hash = step_hash_upper  (uniform payload)
///
/// Because all output_hashes are identical, chain is trivially valid.
/// verify_hash_binding() confirms payload → hash on every step independently.
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
            input_hash: input_hash,
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
    let vectors_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/vendor/governance-bridge/CANONICAL_VECTORS.json"
    );

    let vectors_raw = fs::read(vectors_path).expect("CANONICAL_VECTORS.json must be readable");

    let manifest: serde_json::Value = serde_json::from_slice(&vectors_raw)
        .expect("CANONICAL_VECTORS.json must parse as valid JSON");

    let vectors = manifest["vectors"]
        .as_array()
        .expect("top-level 'vectors' array must exist");

    assert_eq!(
        vectors.len(),
        4,
        "Expected exactly 4 canonical vectors — schema_version may have changed"
    );

    for vector in vectors {
        let id = vector["id"].as_str().expect("vector.id must be a string");
        let name = vector["name"]
            .as_str()
            .expect("vector.name must be a string");

        let canonical_serialized = vector["canonical_serialized"]
            .as_str()
            .unwrap_or_else(|| panic!("Vector {} ({}) missing canonical_serialized", id, name));

        let expected_sha256 = vector["sha256"]
            .as_str()
            .unwrap_or_else(|| panic!("Vector {} ({}) missing sha256", id, name))
            .to_lowercase();

        // ── Layer 1: payload_hash must match vector sha256 ───────────
        // Fires BEFORE verify_replay_id(). A mismatch here means
        // Iter's canonicalize() diverged from SCG's — the primary risk.
        let (trace, step_hash_upper) = build_governed_trace(canonical_serialized);

        assert_eq!(
            step_hash_upper.to_lowercase(),
            expected_sha256,
            "\nCANONICAL HASH MISMATCH (Layer 1) — Vector {} ({})\
             \n  expected (vector):  {}\
             \n  computed:           {}\
             \n  payload:            {}\
             \nDo not patch. File a new WO.",
            id,
            name,
            expected_sha256,
            step_hash_upper.to_lowercase(),
            canonical_serialized
        );

        // ── Layer 2: compute replay_id over full outcome ──────────────
        // NOT the vector sha256. Covers {contract_version, decision,
        // governance_hash, execution_trace} → lowercase hex.
        let replay_id = GovernanceOutcome::compute_replay_id(
            CONTRACT_VERSION_STR,
            &Decision::Allow,
            ATTESTATION_GOVERNANCE_HASH,
            &trace,
        );

        let outcome = GovernanceOutcome {
            contract_version: CONTRACT_VERSION_STR.to_string(),
            decision: Decision::Allow,
            governance_hash: ATTESTATION_GOVERNANCE_HASH.to_string(),
            execution_trace: trace,
            replay_id,
        };

        // ── Serde round-trip (mirrors response.json() path) ──────────
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

        // ── Production boundary call ──────────────────────────────────
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
            expected_sha256,
            step_hash_upper
        );

        println!(
            "  \u{2705}  Vector {} ({})  sha256: {}  replay_id: {}...",
            id,
            name,
            expected_sha256,
            &deserialized.replay_id[..16]
        );
    }

    println!(
        "\nATTESTATION COMPLETE\
         \n  Seam:    response.json \u{2192} verify_replay_id()\
         \n  Layer 1: payload_hash (UPPERCASE step binding): confirmed\
         \n  Layer 2: compute_replay_id (lowercase outcome digest): confirmed\
         \n  Serde:   round-trip through from_value: confirmed"
    );
}
