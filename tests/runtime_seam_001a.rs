use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};

const GOVERNANCE_HASH: &str = include_str!("../governance/governance.hash");

struct McpTestClient {
    server: Child,
    stdin: std::process::ChildStdin,
    reader: BufReader<std::process::ChildStdout>,
    next_id: u64,
}

impl McpTestClient {
    fn spawn(runtime_mode: Option<&str>) -> Self {
        let bin_path = env!("CARGO_BIN_EXE_iter-server");

        let mut cmd = Command::new(bin_path);
        cmd.arg("--json-only")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());

        if let Some(mode) = runtime_mode {
            cmd.arg(format!("--runtime-mode={}", mode));
        }

        let mut server = cmd.spawn().expect("Failed to spawn iter-server");
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

    fn extract_tool_json(&mut self, tool_name: &str, arguments: Value) -> Value {
        let resp = self.call(
            "tools/call",
            json!({
                "name": tool_name,
                "arguments": arguments
            }),
        );

        let text = resp
            .pointer("/result/content/0/text")
            .and_then(|v| v.as_str())
            .expect("tool response text");
        serde_json::from_str(text).expect("tool text JSON")
    }

    fn close(mut self) {
        drop(self.stdin);
        let _ = self.server.wait();
    }
}

fn proposal_args() -> Value {
    json!({
        "proposal_id": "runtime-seam-001a",
        "state_snapshot_hash": "abc123def456abc123def456abc123def456abc123def456abc123def456abc123",
        "requested_action": "deploy_capsule"
    })
}

#[test]
fn default_runtime_mode_remains_demo_stub() {
    let mut client = McpTestClient::spawn(None);
    let outcome = client.extract_tool_json("decision.check", proposal_args());

    assert_eq!(outcome.get("mode").and_then(|v| v.as_str()), Some("demo"));
    assert_eq!(
        outcome.get("authoritative_pdp").and_then(|v| v.as_bool()),
        Some(false)
    );
    assert!(
        outcome.get("packet").is_none() || outcome.get("packet").unwrap().is_null(),
        "default demo mode must not emit a packet"
    );

    client.close();
}

#[test]
fn governed_local_decision_check_emits_packet_with_hash_and_trace() {
    let mut client = McpTestClient::spawn(Some("governed-local"));
    let outcome = client.extract_tool_json("decision.check", proposal_args());
    let packet = outcome.get("packet").expect("packet");

    assert_eq!(
        outcome.get("mode").and_then(|v| v.as_str()),
        Some("governed")
    );
    assert_eq!(
        outcome.get("authoritative_pdp").and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        outcome.get("replay_sufficient").and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        packet.get("governance_hash").and_then(|v| v.as_str()),
        Some(GOVERNANCE_HASH.trim())
    );

    let trace = packet
        .get("execution_trace")
        .and_then(|v| v.as_array())
        .expect("execution_trace array");
    assert!(!trace.is_empty(), "execution trace must not be empty");

    client.close();
}

#[test]
fn governed_local_packet_and_trace_are_deterministic_across_fresh_servers() {
    let run_once = || {
        let mut client = McpTestClient::spawn(Some("governed-local"));
        let outcome = client.extract_tool_json("decision.check", proposal_args());
        let packet = outcome.get("packet").cloned().expect("packet");
        client.close();
        packet
    };

    let packet1 = run_once();
    let packet2 = run_once();

    assert_eq!(
        packet1.get("checksum"),
        packet2.get("checksum"),
        "governed-local checksum must be deterministic across fresh servers"
    );
    assert_eq!(
        packet1.get("execution_trace"),
        packet2.get("execution_trace"),
        "governed-local execution trace must be deterministic across fresh servers"
    );
}

#[test]
fn invalid_runtime_mode_fails_closed() {
    let bin_path = env!("CARGO_BIN_EXE_iter-server");
    let output = Command::new(bin_path)
        .arg("--json-only")
        .arg("--runtime-mode=invalid-mode")
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .expect("spawn invalid runtime mode");

    assert!(
        !output.status.success(),
        "invalid runtime mode must exit non-zero"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("ERROR_INVALID_RUNTIME_MODE"),
        "stderr must report explicit invalid runtime mode error, got: {}",
        stderr
    );
}
