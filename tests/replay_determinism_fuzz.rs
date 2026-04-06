//! tests/replay_determinism_fuzz.rs
//!
//! WO-ITER-INTEGRITY-HARDENING-001 — Property-based fuzz harness
//!
//! Converts coverage from 4 curated vectors -> unbounded valid input space.
//!
//! Properties tested:
//!   P1: canonicalize() is pure — same input always produces same bytes
//!   P2: payload_hash() is deterministic — same bytes always produce same hash
//!   P3: compute_replay_id() is deterministic — same trace always produces
//!       same replay_id
//!   P4: verify_replay_id() is consistent — a valid outcome always verifies
//!
//! Generator constraint: inputs are restricted to valid SCG payload schema
//! (string values only, no floats, valid UTF-8 NFC) to avoid masking real
//! failures with invalid domain states.

use governance_bridge::{
    contract::{Decision, GovernanceOutcome, CONTRACT_VERSION_STR},
    trace::{canonicalize, payload_hash, ExecutionTrace, OperationType, TraceStep},
};
use proptest::prelude::*;

fn arb_json_string() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9_]{1,16}"
}

fn arb_json_object() -> impl Strategy<Value = serde_json::Value> {
    prop::collection::btree_map(arb_json_string(), arb_json_string(), 0..8).prop_map(|map| {
        let object = map
            .into_iter()
            .map(|(k, v)| (k, serde_json::Value::String(v)))
            .collect();
        serde_json::Value::Object(object)
    })
}

fn canonical_bytes(value: &serde_json::Value) -> Vec<u8> {
    let canonical = canonicalize(value);
    serde_json::to_vec(&canonical).expect("canonical serialization must not fail")
}

fn arb_canonical_payload() -> impl Strategy<Value = String> {
    arb_json_object().prop_map(|value| {
        String::from_utf8(canonical_bytes(&value)).expect("canonical bytes must be utf-8")
    })
}

const SEQUENCE: [(OperationType, &str); 5] = [
    (OperationType::HashVerify, "hash_verify"),
    (OperationType::PolicyEval, "policy_eval"),
    (OperationType::StateCheck, "state_check"),
    (OperationType::DecisionEmit, "decision_emit"),
    (OperationType::TraceFinalize, "trace_finalize"),
];

const GOV_HASH: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn build_trace(payload: &str) -> ExecutionTrace {
    let step_hash = payload_hash(payload).expect("generated payload must hash");
    let mut steps = Vec::with_capacity(SEQUENCE.len());
    let mut prev_output = step_hash.clone();

    for (op_type, op_str) in SEQUENCE {
        let input_hash = if steps.is_empty() {
            step_hash.clone()
        } else {
            prev_output.clone()
        };
        let output_hash = step_hash.clone();

        steps.push(TraceStep {
            region_id: "fuzz".to_string(),
            operation: op_str.to_string(),
            input_hash,
            output_hash: output_hash.clone(),
            operation_type: op_type,
            input_payload: payload.to_string(),
            output_payload: payload.to_string(),
        });

        prev_output = output_hash;
    }

    ExecutionTrace::from_steps(steps)
}

proptest! {
    #[test]
    fn p1_canonicalize_is_pure(value in arb_json_object()) {
        let bytes_1 = canonical_bytes(&value);
        let bytes_2 = canonical_bytes(&value);
        prop_assert_eq!(bytes_1, bytes_2,
            "canonicalize() produced different bytes for same input");
    }
}

proptest! {
    #[test]
    fn p2_payload_hash_is_deterministic(payload in arb_canonical_payload()) {
        let hash_1 = payload_hash(&payload).expect("payload_hash must not fail");
        let hash_2 = payload_hash(&payload).expect("payload_hash must not fail");
        prop_assert_eq!(hash_1, hash_2,
            "payload_hash produced different values for same input");
    }
}

proptest! {
    #[test]
    fn p3_replay_id_is_deterministic(payload in arb_canonical_payload()) {
        let trace = build_trace(&payload);
        let id_1 = GovernanceOutcome::compute_replay_id(
            CONTRACT_VERSION_STR, &Decision::Allow, GOV_HASH, &trace,
        );
        let id_2 = GovernanceOutcome::compute_replay_id(
            CONTRACT_VERSION_STR, &Decision::Allow, GOV_HASH, &trace,
        );
        prop_assert_eq!(&id_1, &id_2,
            "compute_replay_id produced different values for same trace");
        prop_assert!(
            !id_1.chars().any(|c| c.is_ascii_uppercase()),
            "replay_id contains uppercase chars — casing contract violated: {}",
            id_1
        );
    }
}

proptest! {
    #[test]
    fn p4_verify_replay_id_is_consistent(payload in arb_canonical_payload()) {
        let trace = build_trace(&payload);
        let replay_id = GovernanceOutcome::compute_replay_id(
            CONTRACT_VERSION_STR, &Decision::Allow, GOV_HASH, &trace,
        );
        let outcome = GovernanceOutcome {
            contract_version: CONTRACT_VERSION_STR.to_string(),
            decision: Decision::Allow,
            governance_hash: GOV_HASH.to_string(),
            execution_trace: trace,
            replay_id,
        };
        prop_assert!(
            outcome.verify_replay_id().is_ok(),
            "verify_replay_id() rejected a correctly constructed outcome: {:?}",
            outcome.verify_replay_id()
        );
    }
}
