use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};

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
            .arg("--profile=governance")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn iter-server");

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
        serde_json::from_str(&line).expect("response JSON")
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

    fn close(mut self) {
        drop(self.stdin);
        let _ = self.server.wait();
    }
}

#[test]
fn tools_list_exposes_register_resource_with_canonical_description() {
    let mut client = McpTestClient::spawn();
    let resp = client.call("tools/list", json!({}));

    let tools = resp
        .pointer("/result/tools")
        .and_then(|t| t.as_array())
        .expect("tools/list must return tools array");
    let register_tool = tools
        .iter()
        .find(|t| t.get("name").and_then(|n| n.as_str()) == Some("register_resource"))
        .expect("register_resource must exist in tools/list");

    assert_eq!(
        register_tool.get("description").and_then(|d| d.as_str()),
        Some("Session-scoped resource hash registry. Registers a resource path and its expected SHA-256 baseline hash for contract-layer validation in decision.check. Registry is cleared on server restart. Must be called before decision.check for hash validation to be enforced.")
    );

    client.close();
}

#[test]
fn registered_resource_allows_matching_hash_and_rejects_mismatch_before_policy() {
    let mut client = McpTestClient::spawn();

    let registration = client.extract_tool_text(
        "register_resource",
        json!({
            "resource_path": "\"docs/README.md\"",
            "expected_hash": VALID_HASH
        }),
    );
    assert_eq!(
        registration.get("registered").and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        registration.get("resource").and_then(|v| v.as_str()),
        Some("docs/readme.md")
    );
    assert_eq!(
        registration.get("baseline_hash").and_then(|v| v.as_str()),
        Some("sha256:8e51aaaa")
    );

    let allow = client.extract_tool_text(
        "decision.check",
        json!({
            "proposal_id": "hash-gate-allow",
            "requested_action": "write to docs/README.md",
            "state_snapshot_hash": VALID_HASH,
            "constraints": { "scope": "docs/README.md" }
        }),
    );
    assert_eq!(allow.get("verdict").and_then(|v| v.as_str()), Some("ALLOW"));

    let reject = client.extract_tool_text(
        "decision.check",
        json!({
            "proposal_id": "hash-gate-reject",
            "requested_action": "write to docs/README.md",
            "state_snapshot_hash": "sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            "constraints": { "scope": "docs/README.md" }
        }),
    );
    assert_eq!(
        reject.get("decision").and_then(|v| v.as_str()),
        Some("CONTRACT_REJECTED")
    );
    assert_eq!(
        reject.get("reason_code").and_then(|v| v.as_str()),
        Some("STATE_HASH_MISMATCH")
    );
    assert_eq!(
        reject
            .get("policy_engine_invoked")
            .and_then(|v| v.as_bool()),
        Some(false)
    );
    assert_eq!(
        reject.get("resource_matched").and_then(|v| v.as_str()),
        Some("docs/readme.md")
    );

    client.close();
}

#[test]
fn missing_state_hash_rejects_before_policy() {
    let mut client = McpTestClient::spawn();
    let rejection = client.extract_tool_text(
        "decision.check",
        json!({
            "proposal_id": "missing-hash",
            "requested_action": "write to docs/README.md"
        }),
    );

    assert_eq!(
        rejection.get("decision").and_then(|v| v.as_str()),
        Some("CONTRACT_REJECTED")
    );
    assert_eq!(
        rejection.get("reason_code").and_then(|v| v.as_str()),
        Some("INCOMPLETE_DECLARATION")
    );
    assert_eq!(
        rejection
            .get("policy_engine_invoked")
            .and_then(|v| v.as_bool()),
        Some(false)
    );
    assert!(rejection
        .get("missing_fields")
        .and_then(|v| v.as_array())
        .map(|arr| arr
            .iter()
            .any(|v| v.as_str() == Some("state_snapshot_hash")))
        .unwrap_or(false));

    client.close();
}

#[test]
fn unregistered_resource_mismatch_remains_fail_open_for_compatibility() {
    let mut client = McpTestClient::spawn();
    let response = client.extract_tool_text(
        "decision.check",
        json!({
            "proposal_id": "unregistered-mismatch",
            "requested_action": "write to docs/UNREGISTERED.md",
            "state_snapshot_hash": "sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
        }),
    );

    assert_eq!(
        response.get("verdict").and_then(|v| v.as_str()),
        Some("ALLOW")
    );
    client.close();
}

#[test]
fn register_resource_rejects_incomplete_params() {
    let mut client = McpTestClient::spawn();

    let missing_path = client.call_tool(
        "register_resource",
        json!({
            "expected_hash": VALID_HASH
        }),
    );
    assert_eq!(
        missing_path
            .pointer("/result/error/code")
            .and_then(|v| v.as_i64()),
        Some(4000)
    );

    let missing_hash = client.call_tool(
        "register_resource",
        json!({
            "resource_path": "docs/README.md"
        }),
    );
    assert_eq!(
        missing_hash
            .pointer("/result/error/code")
            .and_then(|v| v.as_i64()),
        Some(4000)
    );

    client.close();
}
