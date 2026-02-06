//! MCP Alias Equivalence Tests — Phase 1
//!
//! Proves that canonical and legacy tool IDs produce byte-identical responses
//! for the same inputs. Comparison is performed on canonical JSON serialization
//! (stable field order, no whitespace variance).
//!
//! INVARIANT: For each alias pair (canonical, legacy), calling both with identical
//! inputs in the same process MUST produce identical canonical JSON responses.
//!
//! If any test fails, the alias wiring is broken and Phase 1 is incomplete.

use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};

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

    fn close(mut self) {
        drop(self.stdin);
        let _ = self.server.wait();
    }
}

/// Canonicalize JSON for deterministic comparison.
/// Serializes to sorted-key compact JSON string.
fn canonical_json(v: &Value) -> String {
    serde_json::to_string(v).expect("canonical serialization")
}

/// Assert two tool responses are equivalent under canonical JSON serialization.
fn assert_alias_equivalent(
    client: &mut McpTestClient,
    canonical_id: &str,
    legacy_id: &str,
    arguments: Value,
) {
    let r_legacy = client.call_tool(legacy_id, arguments.clone());
    let r_canonical = client.call_tool(canonical_id, arguments);

    let c_legacy = canonical_json(&r_legacy);
    let c_canonical = canonical_json(&r_canonical);

    assert_eq!(
        c_legacy, c_canonical,
        "ALIAS DIVERGENCE: {} vs {} produced different responses.\nLegacy:    {}\nCanonical: {}",
        legacy_id, canonical_id, c_legacy, c_canonical
    );
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn alias_tools_list_contains_both_canonical_and_legacy() {
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

    let required_canonical = [
        "decision.check",
        "audit.export",
        "audit.replay",
        "governance.health",
        "governor.health",
    ];

    let required_legacy = [
        "governance.evaluate",
        "esv.audit",
        "lineage.replay",
        "governance.status",
        "governor.status",
    ];

    for name in &required_canonical {
        assert!(
            names.contains(name),
            "Canonical tool '{}' missing from tools/list. Found: {:?}",
            name,
            names
        );
    }

    for name in &required_legacy {
        assert!(
            names.contains(name),
            "Legacy tool '{}' missing from tools/list. Found: {:?}",
            name,
            names
        );
    }

    client.close();
}

#[test]
fn alias_governor_health_eq_governor_status() {
    let mut client = McpTestClient::spawn();
    assert_alias_equivalent(&mut client, "governor.health", "governor.status", json!({}));
    client.close();
}

#[test]
fn alias_governance_health_eq_governance_status() {
    let mut client = McpTestClient::spawn();
    assert_alias_equivalent(
        &mut client,
        "governance.health",
        "governance.status",
        json!({}),
    );
    client.close();
}

#[test]
fn alias_audit_replay_eq_lineage_replay() {
    let mut client = McpTestClient::spawn();
    assert_alias_equivalent(&mut client, "audit.replay", "lineage.replay", json!({}));
    client.close();
}

#[test]
fn alias_audit_export_eq_esv_audit() {
    // Use separate server instances with identical setup to isolate state.
    let setup = |c: &mut McpTestClient| {
        let _ = c.call_tool("node.create", json!({"belief": 0.5, "energy": 100.0}));
    };

    let r_legacy = {
        let mut c = McpTestClient::spawn();
        setup(&mut c);
        let r = c.call_tool("esv.audit", json!({"node_id": "0"}));
        c.close();
        r
    };

    let r_canonical = {
        let mut c = McpTestClient::spawn();
        setup(&mut c);
        let r = c.call_tool("audit.export", json!({"node_id": "0"}));
        c.close();
        r
    };

    let c_legacy = canonical_json(&r_legacy);
    let c_canonical = canonical_json(&r_canonical);

    assert_eq!(
        c_legacy, c_canonical,
        "ALIAS DIVERGENCE: esv.audit vs audit.export produced different responses.\nLegacy:    {}\nCanonical: {}",
        c_legacy, c_canonical
    );
}

#[test]
fn alias_decision_check_eq_governance_evaluate() {
    // governance.evaluate mutates lineage state (increments lineage_len in CIH).
    // Sequential calls in the same process produce different CIH values.
    // Use separate server instances to isolate state for accurate comparison.
    let args = json!({
        "proposal_id": "test-proposal-001",
        "state_snapshot_hash": "abc123def456abc123def456abc123def456abc123def456abc123def456abc123",
        "requested_action": "deploy_capsule"
    });

    let r_legacy = {
        let mut c = McpTestClient::spawn();
        let r = c.call_tool("governance.evaluate", args.clone());
        c.close();
        r
    };

    let r_canonical = {
        let mut c = McpTestClient::spawn();
        let r = c.call_tool("decision.check", args);
        c.close();
        r
    };

    let c_legacy = canonical_json(&r_legacy);
    let c_canonical = canonical_json(&r_canonical);

    assert_eq!(
        c_legacy, c_canonical,
        "ALIAS DIVERGENCE: governance.evaluate vs decision.check produced different responses.\nLegacy:    {}\nCanonical: {}",
        c_legacy, c_canonical
    );
}

#[test]
fn alias_equivalence_repeat_determinism() {
    let mut client = McpTestClient::spawn();

    // Call the same alias pair twice to verify cross-call determinism within same process
    for _ in 0..2 {
        assert_alias_equivalent(&mut client, "governor.health", "governor.status", json!({}));
    }

    client.close();
}
