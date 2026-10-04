use iter_mcp_server::audit::{AuditEvent, DecisionPacket, PersistentAuditLedger};
use iter_mcp_server::economics::EconomicsConfig;
use iter_mcp_server::governed::GovernedRuntime;
use iter_mcp_server::policy::PolicyConfig;
use iter_mcp_server::runtime::{replay_decision, GovernanceRuntime};
use iter_mcp_server::substrate::stub::{GovernanceProposal, StubRuntime};
use serde_json::{json, Value};
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

fn packet() -> DecisionPacket {
    let mut runtime = GovernedRuntime::new(
        StubRuntime::new(),
        PolicyConfig::default(),
        EconomicsConfig::default(),
    );
    runtime
        .evaluate(&GovernanceProposal {
            proposal_id: "history-test".into(),
            state_snapshot_hash: "a".repeat(64),
            requested_action: "inspect".into(),
            constraints: json!({}),
            proposal_c14n: None,
            proposal_hash: None,
        })
        .unwrap()
        .packet
        .unwrap()
}

fn run_tool(mode: &str, path: Option<&Path>, name: &str, args: Value) -> Value {
    run_tools(mode, path, &[(name, args)]).remove(0)
}

fn run_tools(mode: &str, path: Option<&Path>, calls: &[(&str, Value)]) -> Vec<Value> {
    let mut command = Command::new(env!("CARGO_BIN_EXE_iter-server"));
    command
        .args(["--json-only", &format!("--runtime-mode={mode}")])
        .env_remove("ITER_REQUIRE_AUDIT_LEDGER")
        .env_remove("ITER_AUDIT_LEDGER_PATH")
        .env("SCG_ENDPOINT", "http://127.0.0.1:1")
        .env(
            "SCG_GOVERNANCE_HASH_PATH",
            concat!(env!("CARGO_MANIFEST_DIR"), "/governance/governance.hash"),
        )
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(path) = path {
        command
            .env("ITER_AUDIT_LEDGER_PATH", path)
            .env("ITER_REQUIRE_AUDIT_LEDGER", "1");
    }
    let mut child = command.spawn().unwrap();
    let mut stdin = child.stdin.take().unwrap();
    for (index, (name, args)) in calls.iter().enumerate() {
        writeln!(
            stdin,
            "{}",
            json!({"jsonrpc":"2.0", "id":index + 1, "method":"tools/call",
            "params":{"name":name,"arguments":args}})
        )
        .unwrap();
    }
    drop(stdin);
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let results: Vec<Value> = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .enumerate()
        .map(|(index, line)| {
            let response: Value = serde_json::from_str(line).unwrap();
            assert_eq!(response["id"], index + 1);
            assert!(
                response.get("error").is_none(),
                "unexpected transport error: {response}"
            );
            response["result"].clone()
        })
        .collect();
    assert_eq!(results.len(), calls.len());
    results
}

fn payload(response: &Value) -> Value {
    let text = response
        .pointer("/content/0/text")
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("expected tool payload: {response}"));
    serde_json::from_str(text).unwrap()
}

#[test]
fn durable_history_survives_restart_beyond_memory_window() {
    let path = std::env::temp_dir().join(format!(
        "iter-history-{}-{}.jsonl",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let packet = packet();
    let mut ledger = PersistentAuditLedger::open(&path).unwrap();
    for sequence in 0..1002 {
        ledger
            .append(&AuditEvent::from_packet(sequence, &packet), &packet)
            .unwrap();
    }
    let tail = ledger.last_record_hash().to_owned();
    drop(ledger);
    let original = std::fs::read(&path).unwrap();
    for mode in ["governed-local", "scg-backed"] {
        for start in [0, 999, 1002] {
            let page = payload(&run_tool(
                mode,
                Some(&path),
                "audit.history",
                json!({"start_sequence":start, "limit":2}),
            ));
            assert_eq!(page["verification"], "integrity_only");
            assert_eq!(page["verified_next_sequence"], 1002);
            assert_eq!(page["verified_record_hash"], tail);
            let records = page["records"].as_array().unwrap();
            assert_eq!(records.len(), if start == 1002 { 0 } else { 2 });
            for (offset, record) in records.iter().enumerate() {
                assert_eq!(record["ledger_sequence"], start + offset as u64);
                assert_eq!(record["packet"]["checksum"], packet.checksum);
            }
            assert_eq!(
                page["next_sequence"],
                if start == 1002 {
                    Value::Null
                } else {
                    json!(start + 2)
                }
            );
        }
    }
    assert_eq!(
        std::fs::read(&path).unwrap(),
        original,
        "history must not mutate evidence"
    );
    std::fs::remove_file(path).unwrap();
}

#[test]
fn self_consistent_wrong_verdict_is_not_semantic_replay() {
    let mut packet = packet();
    packet.policy.decision = match packet.policy.decision {
        iter_mcp_server::contracts::PolicyDecision::Allow => {
            iter_mcp_server::contracts::PolicyDecision::Deny
        }
        _ => iter_mcp_server::contracts::PolicyDecision::Allow,
    };
    packet.checksum.clear();
    packet.checksum = iter_mcp_server::canonical::hash_str(
        &serde_json_canonicalizer::to_string(&packet).unwrap(),
    );
    packet.verify_checksum().unwrap();
    let result = replay_decision(
        &packet,
        &format!("sha256:{}", packet.policy.policy_hash),
        "decision_packet:v1",
    )
    .unwrap();
    assert!(
        !result.replay_sufficient,
        "checksum inspection cannot certify re-execution"
    );
    assert!(
        !result.authoritative_pdp,
        "a stored verdict is not a newly evaluated decision"
    );
}

#[test]
fn governed_replay_refuses_to_claim_reexecution() {
    for mode in ["governed-local", "scg-backed"] {
        for tool in ["audit.replay", "lineage.replay"] {
            let response = run_tool(mode, None, tool, json!({}));
            assert_eq!(response["error"]["code"], 5003);
            assert!(response["error"]["message"]
                .as_str()
                .unwrap()
                .contains("audit.history"));
        }
    }
}

#[test]
fn history_returns_the_evaluated_packet_after_server_restart() {
    let path = std::env::temp_dir().join(format!(
        "iter-history-evaluated-{}-{}.jsonl",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    drop(PersistentAuditLedger::open(&path).unwrap());
    let resource = "docs/README.md";
    let hash = "sha256:8e51aaaa299f88b416976abd2a25a7d3a0db01b61b105066013f43a077408e25";
    let replies = run_tools(
        "governed-local",
        Some(&path),
        &[
            (
                "register_resource",
                json!({"resource_path":resource,"expected_hash":hash}),
            ),
            (
                "decision.check",
                json!({"proposal_id":"durable-restart","state_snapshot_hash":hash,
            "requested_action":"write to docs/README.md","constraints":{"scope":resource}}),
            ),
            ("audit.history", json!({})),
        ],
    );
    let evaluated = payload(&replies[1]);
    assert_eq!(evaluated["authoritative_pdp"], true);
    assert_eq!(evaluated["replay_sufficient"], false);
    let before = payload(&replies[2]);
    assert_eq!(before["records"][0]["packet"], evaluated["packet"]);
    let after = payload(&run_tool(
        "governed-local",
        Some(&path),
        "audit.history",
        json!({}),
    ));
    assert_eq!(
        after, before,
        "restart must preserve the exact durable page, not reconstruct evidence"
    );
    std::fs::remove_file(path).unwrap();
}

#[test]
fn history_rejects_missing_storage_and_malformed_queries() {
    for mode in ["demo", "governed-local", "scg-backed"] {
        let response = run_tool(mode, None, "audit.history", json!({}));
        assert_eq!(
            response["error"]["code"], 5002,
            "missing storage must not look empty"
        );
    }
    let path = std::env::temp_dir().join(format!(
        "iter-history-query-{}-{}.jsonl",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    drop(PersistentAuditLedger::open(&path).unwrap());
    let invalid = vec![
        json!({"limit":0}),
        json!({"limit":101}),
        json!({"limit":-1}),
        json!({"limit":"1"}),
        json!({"limit":1.5}),
        json!({"limit":null}),
        json!({"start_sequence":1}),
        json!({"start_sequence":u64::MAX}),
        json!({"start_sequence":-1}),
        json!({"start_sequence":"0"}),
        json!({"start_sequence":null}),
        json!({"unexpected":true}),
    ];
    let mut calls: Vec<(&str, Value)> = invalid
        .into_iter()
        .map(|args| ("audit.history", args))
        .collect();
    calls.push(("audit.history", json!({})));
    for mode in ["governed-local", "scg-backed"] {
        let replies = run_tools(mode, Some(&path), &calls);
        for response in &replies[..replies.len() - 1] {
            assert_eq!(response["error"]["code"], 5002);
            assert!(response.get("content").is_none());
        }
        let empty = payload(replies.last().unwrap());
        assert_eq!(empty["verified_next_sequence"], 0);
        assert_eq!(empty["records"], json!([]));
        assert_eq!(empty["next_sequence"], Value::Null);
    }
    std::fs::remove_file(path).unwrap();
}
