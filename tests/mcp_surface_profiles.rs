//! MCP Surface Profile Tests — Phase 3
//!
//! Validates that each server profile exposes exactly the expected tools.
//!
//! INVARIANTS:
//! - Governance profile: no kernel.*, node.*, or edge.* tools.
//! - Governance profile: all canonical governance tools present.
//! - Kernel-debug profile: kernel tools present.
//! - Default (no --profile flag): identical to governance profile.

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
    fn spawn_with_profile(profile: Option<&str>) -> Self {
        let bin_path = env!("CARGO_BIN_EXE_iter-server");

        let mut cmd = Command::new(bin_path);
        cmd.arg("--json-only")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());

        if let Some(p) = profile {
            cmd.arg(format!("--profile={}", p));
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
        serde_json::from_str(&line).unwrap_or(json!({"error": "parse failed"}))
    }

    fn get_tool_names(&mut self) -> Vec<String> {
        let resp = self.call("tools/list", json!({}));
        resp.get("result")
            .and_then(|r| r.get("tools"))
            .and_then(|t| t.as_array())
            .expect("tools/list must return tools array")
            .iter()
            .filter_map(|t| t.get("name").and_then(|n| n.as_str()).map(|s| s.to_string()))
            .collect()
    }

    fn close(mut self) {
        drop(self.stdin);
        let _ = self.server.wait();
    }
}

// ---------------------------------------------------------------------------
// Profile Surface Tests
// ---------------------------------------------------------------------------

#[test]
fn governance_profile_contains_no_kernel_tools() {
    let mut client = McpTestClient::spawn_with_profile(Some("governance"));
    let names = client.get_tool_names();

    for name in &names {
        let is_kernel = name.starts_with("kernel.")
            || name.starts_with("node.")
            || name.starts_with("edge.");
        assert!(
            !is_kernel,
            "Governance profile MUST NOT contain kernel/graph tool '{}'. Found: {:?}",
            name, names
        );
    }

    client.close();
}

#[test]
fn governance_profile_contains_all_canonical_tools() {
    let mut client = McpTestClient::spawn_with_profile(Some("governance"));
    let names = client.get_tool_names();

    let required = [
        "decision.check",
        "decision.preview",
        "audit.export",
        "audit.replay",
        "audit.search",
        "governance.health",
        "governor.health",
    ];

    for tool in &required {
        assert!(
            names.iter().any(|n| n == tool),
            "Governance profile must contain canonical tool '{}'. Found: {:?}",
            tool, names
        );
    }

    client.close();
}

#[test]
fn kernel_debug_profile_contains_kernel_tools() {
    let mut client = McpTestClient::spawn_with_profile(Some("kernel-debug"));
    let names = client.get_tool_names();

    let required_kernel = ["node.create", "node.query", "node.mutate", "edge.bind", "edge.propagate"];

    for tool in &required_kernel {
        assert!(
            names.iter().any(|n| n == tool),
            "Kernel-debug profile must contain kernel tool '{}'. Found: {:?}",
            tool, names
        );
    }

    client.close();
}

#[test]
fn kernel_debug_profile_also_contains_governance_tools() {
    let mut client = McpTestClient::spawn_with_profile(Some("kernel-debug"));
    let names = client.get_tool_names();

    let required_governance = ["decision.check", "decision.preview", "audit.search"];

    for tool in &required_governance {
        assert!(
            names.iter().any(|n| n == tool),
            "Kernel-debug profile must also contain governance tool '{}'. Found: {:?}",
            tool, names
        );
    }

    client.close();
}

#[test]
fn default_profile_is_governance() {
    let mut client_default = McpTestClient::spawn_with_profile(None);
    let mut client_explicit = McpTestClient::spawn_with_profile(Some("governance"));

    let names_default = client_default.get_tool_names();
    let names_explicit = client_explicit.get_tool_names();

    let mut sorted_default = names_default.clone();
    sorted_default.sort();
    let mut sorted_explicit = names_explicit.clone();
    sorted_explicit.sort();

    assert_eq!(
        sorted_default, sorted_explicit,
        "Default profile must produce identical surface to --profile=governance"
    );

    client_default.close();
    client_explicit.close();
}

#[test]
fn governance_profile_rejects_kernel_tool_call() {
    let mut client = McpTestClient::spawn_with_profile(Some("governance"));

    let resp = client.call(
        "tools/call",
        json!({
            "name": "node.create",
            "arguments": {"belief": 0.5, "energy": 100.0}
        }),
    );

    let result = resp.get("result").expect("response must have result");
    assert!(
        result.get("error").is_some(),
        "Calling kernel tool in governance profile must return error. Got: {}",
        result
    );

    client.close();
}
