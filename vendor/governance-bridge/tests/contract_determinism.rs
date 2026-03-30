use scg_governance_bridge::{
    contract::{Decision, GovernanceOutcome, GovernanceRequest, CONTRACT_VERSION_STR},
    errors::BridgeError,
    trace::ExecutionTrace,
    GovernanceBridge, StubBridge,
};
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
        &ExecutionTrace::new(),
    );
    let id2 = GovernanceOutcome::compute_replay_id(
        CONTRACT_VERSION_STR,
        &Decision::Allow,
        "test-hash",
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
        &ExecutionTrace::new(),
    );
    let id2 = GovernanceOutcome::compute_replay_id(
        CONTRACT_VERSION_STR,
        &Decision::Allow,
        "hash-b",
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
    o.replay_id = GovernanceOutcome::compute_replay_id(
        &o.contract_version,
        &o.decision,
        &o.governance_hash,
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
fn escalate_is_a_valid_decision_variant() {
    let id = GovernanceOutcome::compute_replay_id(
        CONTRACT_VERSION_STR,
        &Decision::Escalate,
        "test-hash",
        &ExecutionTrace::new(),
    );
    assert!(!id.is_empty());
    let allow_id = GovernanceOutcome::compute_replay_id(
        CONTRACT_VERSION_STR,
        &Decision::Allow,
        "test-hash",
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
