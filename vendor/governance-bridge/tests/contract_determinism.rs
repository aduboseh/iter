use scg_governance_bridge::{
    contract::{
        Decision, GovernanceOutcome, GovernanceRequest, GovernanceStateEnvelope,
        CONTRACT_VERSION_STR, STATE_ENVELOPE_SCHEMA,
    },
    errors::BridgeError,
    trace::{ExecutionTrace, OperationType, TraceStep},
    GovernanceBridge, StubBridge,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

fn req(id: &str) -> GovernanceRequest {
    GovernanceRequest {
        proposal_id: id.into(),
        state_snapshot_hash: "abc123def456".into(),
        requested_action: "approve".into(),
        constraints: BTreeMap::new(),
    }
}

fn bridge() -> StubBridge {
    StubBridge {
        governance_hash: "canonical-sha256-hash-scg-v1".into(),
    }
}

fn test_state_envelope(snapshot_hash: &str) -> GovernanceStateEnvelope {
    GovernanceStateEnvelope::new(snapshot_hash.to_string(), 110.0, 0.0, 1, 0)
}

fn replay_id(
    contract_version: &str,
    decision: &Decision,
    governance_hash: &str,
    state_envelope: &GovernanceStateEnvelope,
    trace: &ExecutionTrace,
) -> String {
    GovernanceOutcome::compute_replay_id(
        contract_version,
        decision,
        governance_hash,
        &state_envelope.state_snapshot_hash,
        &state_envelope.schema,
        &state_envelope.compute_hash(),
        trace,
    )
}

fn canonical_payload<T: Serialize>(value: &T) -> String {
    serde_json::to_string(value).unwrap()
}

fn payload_hash(payload: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(payload.as_bytes());
    hex::encode_upper(hasher.finalize())
}

fn step<TInput: Serialize, TOutput: Serialize>(
    region_id: &str,
    operation: &str,
    operation_type: OperationType,
    input: &TInput,
    output: &TOutput,
) -> TraceStep {
    let input_payload = canonical_payload(input);
    let output_payload = canonical_payload(output);

    TraceStep {
        region_id: region_id.into(),
        operation: operation.into(),
        input_hash: payload_hash(&input_payload),
        output_hash: payload_hash(&output_payload),
        operation_type,
        input_payload,
        output_payload,
    }
}

fn valid_trace() -> ExecutionTrace {
    let hash_input = ("proposal-001", "snapshot-001", "approve");
    let hash_output = ("snapshot-001", true);
    let policy_input = hash_output;
    let policy_output = (policy_input, "approve", "allow");
    let state_input = policy_output;
    let state_output = (state_input, vec!["no-violations"], false);
    let decision_input = state_output.clone();
    let decision_output = ("allow", "approve", true, false);
    let finalize_input = decision_output;
    let finalize_output = (
        "canonical-sha256-hash-scg-v1",
        CONTRACT_VERSION_STR,
        "proposal-001",
        "trace-sealed",
    );

    ExecutionTrace::from_steps(vec![
        step(
            "gateway",
            "snapshot-hash-compare",
            OperationType::HashVerify,
            &hash_input,
            &hash_output,
        ),
        step(
            "gateway",
            "policy-eval",
            OperationType::PolicyEval,
            &policy_input,
            &policy_output,
        ),
        step(
            "gateway",
            "state-check",
            OperationType::StateCheck,
            &state_input,
            &state_output,
        ),
        step(
            "gateway",
            "decision-emit",
            OperationType::DecisionEmit,
            &decision_input,
            &decision_output,
        ),
        step(
            "gateway",
            "trace-finalize",
            OperationType::TraceFinalize,
            &finalize_input,
            &finalize_output,
        ),
    ])
}

#[test]
fn identical_requests_produce_identical_outcomes() {
    let o1 = bridge().evaluate(req("t01")).unwrap();
    let o2 = bridge().evaluate(req("t01")).unwrap();
    assert_eq!(o1, o2);
}

#[test]
fn execution_trace_is_deterministic_across_runs() {
    let o1 = bridge().evaluate(req("t02")).unwrap();
    let o2 = bridge().evaluate(req("t02")).unwrap();
    assert_eq!(o1.execution_trace, o2.execution_trace);
}

#[test]
fn governance_hash_is_bound_and_required() {
    let o = bridge().evaluate(req("t03")).unwrap();
    assert!(!o.governance_hash.is_empty());
    assert_eq!(o.governance_hash, "canonical-sha256-hash-scg-v1");
}

#[test]
fn execution_trace_contains_typed_steps() {
    let o = bridge().evaluate(req("t04")).unwrap();
    assert!(!o.execution_trace.is_empty());
    let steps = o.execution_trace.steps();
    assert!(!steps[0].region_id.is_empty());
    assert!(!steps[0].operation.is_empty());
    assert!(!steps[0].input_hash.is_empty());
    assert!(!steps[0].output_hash.is_empty());
    assert!(!steps[0].input_payload.is_empty());
    assert!(!steps[0].output_payload.is_empty());
}

#[test]
fn btreemap_constraints_preserve_ordering() {
    let mut c: BTreeMap<String, String> = BTreeMap::new();
    c.insert("z_last".into(), "v".into());
    c.insert("a_first".into(), "v".into());
    c.insert("m_middle".into(), "v".into());
    let keys: Vec<&String> = c.keys().collect();
    assert_eq!(keys[0], "a_first");
    assert_eq!(keys[1], "m_middle");
    assert_eq!(keys[2], "z_last");
}

#[test]
fn contract_version_is_set_on_every_outcome() {
    let o = bridge().evaluate(req("t06")).unwrap();
    assert_eq!(o.contract_version, CONTRACT_VERSION_STR);
}

#[test]
fn stale_contract_version_is_detectable() {
    let o = bridge().evaluate(req("t07")).unwrap();
    assert_ne!(o.contract_version, "scg.v0");
}

#[test]
fn replay_id_is_deterministic() {
    let id1 = GovernanceOutcome::compute_replay_id(
        CONTRACT_VERSION_STR,
        &Decision::Allow,
        "test-hash",
        "snapshot-001",
        STATE_ENVELOPE_SCHEMA,
        &test_state_envelope("snapshot-001").compute_hash(),
        &ExecutionTrace::new(),
    );
    let id2 = GovernanceOutcome::compute_replay_id(
        CONTRACT_VERSION_STR,
        &Decision::Allow,
        "test-hash",
        "snapshot-001",
        STATE_ENVELOPE_SCHEMA,
        &test_state_envelope("snapshot-001").compute_hash(),
        &ExecutionTrace::new(),
    );
    assert_eq!(id1, id2);
}

#[test]
fn different_inputs_produce_different_replay_ids() {
    let id1 = GovernanceOutcome::compute_replay_id(
        CONTRACT_VERSION_STR,
        &Decision::Allow,
        "hash-a",
        "snapshot-001",
        STATE_ENVELOPE_SCHEMA,
        &test_state_envelope("snapshot-001").compute_hash(),
        &ExecutionTrace::new(),
    );
    let id2 = GovernanceOutcome::compute_replay_id(
        CONTRACT_VERSION_STR,
        &Decision::Allow,
        "hash-b",
        "snapshot-001",
        STATE_ENVELOPE_SCHEMA,
        &test_state_envelope("snapshot-001").compute_hash(),
        &ExecutionTrace::new(),
    );
    assert_ne!(id1, id2);
}

#[test]
fn verify_replay_id_passes_on_untampered_outcome() {
    let o = bridge().evaluate(req("t10")).unwrap();
    assert!(o.verify_replay_id().is_ok());
}

#[test]
fn verify_replay_id_fails_on_tampered_outcome() {
    let mut o = bridge().evaluate(req("t11")).unwrap();
    o.replay_id = "tampered-value".into();
    assert!(matches!(
        o.verify_replay_id().unwrap_err(),
        BridgeError::ReplayIntegrityViolation(_)
    ));
}

#[test]
fn verify_replay_id_fails_on_stale_contract_version() {
    let mut o = bridge().evaluate(req("t11b")).unwrap();
    o.contract_version = "scg.v0".into();
    o.replay_id = replay_id(
        &o.contract_version,
        &o.decision,
        &o.governance_hash,
        &o.state_envelope,
        &o.execution_trace,
    );
    assert!(matches!(
        o.verify_replay_id().unwrap_err(),
        BridgeError::ContractVersionMismatch {
            expected,
            got
        } if expected == CONTRACT_VERSION_STR && got == "scg.v0"
    ));
}

#[test]
fn version_check_precedes_semantic_validation() {
    let mut broken = valid_trace().into_steps();
    broken[1].input_hash = "d".repeat(64);
    let trace = ExecutionTrace::from_steps(broken);
    let state_envelope = test_state_envelope("snapshot-001");
    let outcome = GovernanceOutcome {
        contract_version: "scg.v99".into(),
        decision: Decision::Allow,
        governance_hash: "canonical-sha256-hash-scg-v1".into(),
        state_snapshot_hash: state_envelope.state_snapshot_hash.clone(),
        state_envelope_schema: state_envelope.schema.clone(),
        state_envelope_hash: state_envelope.compute_hash(),
        state_envelope: state_envelope.clone(),
        replay_id: replay_id(
            "scg.v99",
            &Decision::Allow,
            "canonical-sha256-hash-scg-v1",
            &state_envelope,
            &trace,
        ),
        execution_trace: trace,
    };

    assert!(matches!(
        outcome.verify_replay_id().unwrap_err(),
        BridgeError::ContractVersionMismatch { expected, got }
            if expected == CONTRACT_VERSION_STR && got == "scg.v99"
    ));
}

#[test]
fn escalate_is_a_valid_decision_variant() {
    let id = GovernanceOutcome::compute_replay_id(
        CONTRACT_VERSION_STR,
        &Decision::Escalate,
        "test-hash",
        "snapshot-001",
        STATE_ENVELOPE_SCHEMA,
        &test_state_envelope("snapshot-001").compute_hash(),
        &ExecutionTrace::new(),
    );
    assert!(!id.is_empty());
    let allow_id = GovernanceOutcome::compute_replay_id(
        CONTRACT_VERSION_STR,
        &Decision::Allow,
        "test-hash",
        "snapshot-001",
        STATE_ENVELOPE_SCHEMA,
        &test_state_envelope("snapshot-001").compute_hash(),
        &ExecutionTrace::new(),
    );
    assert_ne!(id, allow_id);
}

#[test]
fn empty_governance_hash_is_rejected() {
    let bad = StubBridge {
        governance_hash: String::new(),
    };
    let result = bad.evaluate(req("t13"));
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        BridgeError::GovernanceHashInvalid(_)
    ));
}

#[test]
fn outcome_survives_serde_round_trip() {
    let original = bridge().evaluate(req("t14")).unwrap();
    let json = serde_json::to_vec(&original).unwrap();
    let restored: GovernanceOutcome = serde_json::from_slice(&json).unwrap();
    assert_eq!(original, restored);
}

#[test]
fn verify_replay_id_catches_chain_violation() {
    let mut broken = valid_trace().into_steps();
    broken[1].input_hash = "d".repeat(64);
    let broken_trace = ExecutionTrace::from_steps(broken);
    let state_envelope = test_state_envelope("snapshot-001");

    let outcome = GovernanceOutcome {
        contract_version: CONTRACT_VERSION_STR.to_string(),
        decision: Decision::Allow,
        governance_hash: "trace-chain-hash".into(),
        state_snapshot_hash: state_envelope.state_snapshot_hash.clone(),
        state_envelope_schema: state_envelope.schema.clone(),
        state_envelope_hash: state_envelope.compute_hash(),
        state_envelope: state_envelope.clone(),
        replay_id: replay_id(
            CONTRACT_VERSION_STR,
            &Decision::Allow,
            "trace-chain-hash",
            &state_envelope,
            &broken_trace,
        ),
        execution_trace: broken_trace,
    };

    assert!(matches!(
        outcome.verify_replay_id().unwrap_err(),
        BridgeError::TraceDeterminismViolation(_)
    ));
}

#[test]
fn full_chain_determinism() {
    let trace1 = valid_trace();
    let trace2 = valid_trace();

    assert!(trace1.validate_chain().is_ok());
    assert!(trace2.validate_chain().is_ok());
    assert_eq!(trace1, trace2);
}
