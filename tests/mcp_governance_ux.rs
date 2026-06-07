//! MCP Governance UX Tests — Phase 2 (GovernanceOutcome contract)
//!
//! Proves correctness of decision.preview and audit.search tools.
//!
//! INVARIANTS:
//! - decision.preview MUST NOT mutate lineage
//! - decision.preview MUST return GovernanceOutcome with mode, authoritative_pdp, replay_sufficient
//! - decision.preview MUST be deterministic across repeat calls
//! - demo mode: authoritative_pdp=false, replay_sufficient=false, packet=null
//! - audit.search MUST return deterministic ordering
//! - audit.search MUST respect default limit (100) and max limit (1000)
//! - audit.search on empty lineage returns zero results

use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};

const RESOURCE_PATH: &str = "docs/README.md";
const VALID_HASH: &str = "sha256:8e51aaaa299f88b416976abd2a25a7d3a0db01b61b105066013f43a077408e25";

struct McpTestClient {
    server: Child,
    stdin: std::process::ChildStdin,
    reader: BufReader<std::process::ChildStdout>,
    next_id: u64,
}

impl McpTestClient {
    fn spawn() -> Self {
        let bin_path = env!("CARGO_BIN_EXE_iter-server");

        let mut server = Command::new(bin_path)
            .arg("--json-only")
            .arg("--runtime-mode=demo")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to spawn iter-server");

        let stdin = server.stdin.take().expect("stdin");
        let stdout = server.stdout.take().expect("stdout");
        let reader = BufReader::new(stdout);

        let mut client = Self {
            server,
            stdin,
            reader,
            next_id: 1,
        };

        let _ = client.call("initialize", json!({}));
        client
    }

    fn call(&mut self, method: &str, params: Value) -> Value {
        let req = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": self.next_id
        });
        self.next_id += 1;

        writeln!(self.stdin, "{}", req).expect("write");
        self.stdin.flush().expect("flush");

        let mut line = String::new();
        self.reader.read_line(&mut line).expect("read");
        serde_json::from_str(&line).unwrap_or(json!({"error": "parse failed"}))
    }

    fn call_tool(&mut self, tool_name: &str, arguments: Value) -> Value {
        self.call(
            "tools/call",
            json!({
                "name": tool_name,
                "arguments": arguments
            }),
        )
    }

    fn extract_tool_text(&mut self, tool_name: &str, arguments: Value) -> Value {
        let resp = self.call_tool(tool_name, arguments);
        let text = resp
            .pointer("/result/content/0/text")
            .and_then(|t| t.as_str())
            .expect("tool response must contain content[0].text");
        serde_json::from_str(text).expect("tool text must parse as JSON")
    }

    fn register_fixture_resource(&mut self) {
        let registration = self.extract_tool_text(
            "register_resource",
            json!({
                "resource_path": RESOURCE_PATH,
                "expected_hash": VALID_HASH
            }),
        );
        assert_eq!(
            registration.get("registered").and_then(|v| v.as_bool()),
            Some(true),
            "fixture resource registration must succeed"
        );
    }

    fn close(mut self) {
        drop(self.stdin);
        let _ = self.server.wait();
    }
}

fn proposal_args() -> Value {
    json!({
        "proposal_id": "test-preview-001",
        "state_snapshot_hash": VALID_HASH,
        "requested_action": "write to docs/README.md",
        "constraints": { "scope": RESOURCE_PATH }
    })
}

// ---------------------------------------------------------------------------
// decision.preview tests
// ---------------------------------------------------------------------------

#[test]
fn tools_list_contains_phase2_tools() {
    let mut client = McpTestClient::spawn();
    let resp = client.call("tools/list", json!({}));

    let tools = resp
        .get("result")
        .and_then(|r| r.get("tools"))
        .and_then(|t| t.as_array())
        .expect("tools/list must return tools array");

    let names: Vec<&str> = tools
        .iter()
        .filter_map(|t| t.get("name").and_then(|n| n.as_str()))
        .collect();

    assert!(
        names.contains(&"decision.preview"),
        "decision.preview missing from tools/list. Found: {:?}",
        names
    );
    assert!(
        names.contains(&"audit.search"),
        "audit.search missing from tools/list. Found: {:?}",
        names
    );

    client.close();
}

#[test]
fn decision_preview_returns_governance_outcome() {
    let mut client = McpTestClient::spawn();
    let preview = client.extract_tool_text("decision.preview", proposal_args());

    assert!(
        preview.get("verdict").is_some(),
        "decision.preview must return a verdict"
    );
    assert_eq!(
        preview.get("mode").and_then(|v| v.as_str()),
        Some("demo"),
        "decision.preview must return mode"
    );
    assert_eq!(
        preview.get("authoritative_pdp").and_then(|v| v.as_bool()),
        Some(false),
        "demo mode must set authoritative_pdp=false"
    );
    assert_eq!(
        preview.get("replay_sufficient").and_then(|v| v.as_bool()),
        Some(false),
        "demo mode must set replay_sufficient=false"
    );
    assert!(
        preview.get("reason_codes").is_some(),
        "decision.preview must return reason_codes"
    );
    assert_eq!(
        preview.get("schema_version").and_then(|v| v.as_str()),
        Some("1.0"),
        "decision.preview must return schema_version"
    );
    assert!(
        preview.get("packet").is_none() || preview.get("packet").unwrap().is_null(),
        "demo mode must not emit packet"
    );

    client.close();
}

#[test]
fn decision_preview_deterministic_across_repeat_calls() {
    let mut client = McpTestClient::spawn();

    let p1 = client.extract_tool_text("decision.preview", proposal_args());
    let p2 = client.extract_tool_text("decision.preview", proposal_args());

    assert_eq!(
        p1.get("verdict"),
        p2.get("verdict"),
        "decision.preview verdict must be deterministic"
    );
    assert_eq!(
        p1.get("reason_codes"),
        p2.get("reason_codes"),
        "decision.preview reason_codes must be deterministic"
    );
    assert_eq!(
        p1.get("mode"),
        p2.get("mode"),
        "decision.preview mode must be deterministic"
    );
    assert_eq!(
        p1.get("authoritative_pdp"),
        p2.get("authoritative_pdp"),
        "decision.preview authoritative_pdp must be deterministic"
    );

    client.close();
}

#[test]
fn decision_preview_does_not_mutate_lineage() {
    let mut client = McpTestClient::spawn();

    let lineage_before = client.extract_tool_text("audit.replay", json!({}));

    let _ = client.extract_tool_text("decision.preview", proposal_args());
    let _ = client.extract_tool_text("decision.preview", proposal_args());

    let lineage_after = client.extract_tool_text("audit.replay", json!({}));

    assert_eq!(
        lineage_before, lineage_after,
        "decision.preview MUST NOT mutate lineage.\nBefore: {}\nAfter: {}",
        lineage_before, lineage_after
    );

    client.close();
}

#[test]
fn decision_preview_verdict_matches_decision_check() {
    let args = proposal_args();

    let preview_verdict = {
        let mut c = McpTestClient::spawn();
        let p = c.extract_tool_text("decision.preview", args.clone());
        let v = p
            .get("verdict")
            .cloned()
            .expect("preview must have verdict");
        c.close();
        v
    };

    let check_verdict = {
        let mut c = McpTestClient::spawn();
        c.register_fixture_resource();
        let evaluation = c.extract_tool_text("decision.check", args);
        let v = evaluation
            .get("verdict")
            .cloned()
            .expect("decision.check must have verdict");
        c.close();
        v
    };

    assert_eq!(
        preview_verdict, check_verdict,
        "decision.preview verdict must match decision.check verdict"
    );
}

// ---------------------------------------------------------------------------
// audit.search tests
// ---------------------------------------------------------------------------

#[test]
fn audit_search_empty_lineage_returns_zero_results() {
    let mut client = McpTestClient::spawn();
    let result = client.extract_tool_text("audit.search", json!({}));

    let count = result
        .get("count")
        .and_then(|c| c.as_u64())
        .expect("audit.search must return count");
    assert_eq!(
        count, 0,
        "audit.search on empty lineage must return 0 results"
    );

    let results = result
        .get("results")
        .and_then(|r| r.as_array())
        .expect("audit.search must return results array");
    assert!(results.is_empty(), "results array must be empty");

    assert_eq!(
        result.get("ordering").and_then(|o| o.as_str()),
        Some("(timestamp_utc,decision_id) ASC"),
        "audit.search must return ordering field"
    );

    client.close();
}

#[test]
fn audit_search_returns_decisions_after_governance_evaluate() {
    let mut client = McpTestClient::spawn();
    client.register_fixture_resource();

    let _ = client.call_tool(
        "governance.evaluate",
        json!({
            "proposal_id": "search-test-001",
            "state_snapshot_hash": VALID_HASH,
            "requested_action": "write to docs/README.md",
            "constraints": { "scope": RESOURCE_PATH }
        }),
    );

    let result = client.extract_tool_text("audit.search", json!({}));

    let count = result.get("count").and_then(|c| c.as_u64()).expect("count");
    assert!(
        count >= 1,
        "audit.search must return at least 1 result after governance.evaluate. Got: {}",
        count
    );

    let results = result
        .get("results")
        .and_then(|r| r.as_array())
        .expect("results array");
    let first = &results[0];
    assert!(
        first.get("decision_id").is_some(),
        "result must have decision_id"
    );
    assert!(
        first.get("timestamp").is_some(),
        "result must have timestamp"
    );

    client.close();
}

#[test]
fn audit_search_deterministic_ordering() {
    let mut client = McpTestClient::spawn();
    client.register_fixture_resource();

    for i in 0..3 {
        let _ = client.call_tool(
            "governance.evaluate",
            json!({
                "proposal_id": format!("order-test-{:03}", i),
                "state_snapshot_hash": VALID_HASH,
                "requested_action": "write to docs/README.md",
                "constraints": { "scope": RESOURCE_PATH }
            }),
        );
    }

    let r1 = client.extract_tool_text("audit.search", json!({}));
    let r2 = client.extract_tool_text("audit.search", json!({}));

    assert_eq!(
        r1.get("results"),
        r2.get("results"),
        "audit.search ordering must be deterministic across repeat calls"
    );

    client.close();
}

#[test]
fn audit_search_respects_limit() {
    let mut client = McpTestClient::spawn();
    client.register_fixture_resource();

    for i in 0..5 {
        let _ = client.call_tool(
            "governance.evaluate",
            json!({
                "proposal_id": format!("limit-test-{:03}", i),
                "state_snapshot_hash": VALID_HASH,
                "requested_action": "write to docs/README.md",
                "constraints": { "scope": RESOURCE_PATH }
            }),
        );
    }

    let result = client.extract_tool_text("audit.search", json!({"limit": 2}));

    let count = result.get("count").and_then(|c| c.as_u64()).expect("count");
    assert!(
        count <= 2,
        "audit.search with limit=2 must return at most 2 results. Got: {}",
        count
    );

    client.close();
}
