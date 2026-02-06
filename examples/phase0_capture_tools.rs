//! Phase 0 Tools Capture - APEX DIRECTIVE ITER-MCP-TOOL-SURFACE v1
//!
//! Captures raw MCP tools/list response for Phase 0 reality freeze.
//! Outputs unfiltered JSON to stdout for checksumming and archival.

use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};

fn main() {
    let mut client = McpClient::spawn();

    // Initialize protocol
    let _ = client.call("initialize", json!({}));

    // Capture tools/list (the only operation for Phase 0)
    let tools_response = client.call("tools/list", json!({}));

    // Output raw JSON to stdout (no filtering, no formatting beyond pretty-print)
    println!("{}", serde_json::to_string_pretty(&tools_response).unwrap());

    client.close();
}

/// Minimal MCP client for Phase 0 capture
struct McpClient {
    server: Child,
    stdin: std::process::ChildStdin,
    reader: BufReader<std::process::ChildStdout>,
    next_id: u64,
}

impl McpClient {
    fn spawn() -> Self {
        let server_bin = std::env::var_os("ITER_SERVER_BIN")
            .unwrap_or_else(|| "C:\\Users\\adubo\\iter\\target\\release\\iter-server.exe".into());

        let mut server = Command::new(server_bin)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to spawn iter-server");

        let stdin = server.stdin.take().expect("Failed to open stdin");
        let stdout = server.stdout.take().expect("Failed to open stdout");
        let reader = BufReader::new(stdout);

        Self {
            server,
            stdin,
            reader,
            next_id: 1,
        }
    }

    fn call(&mut self, method: &str, params: Value) -> Value {
        let req = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": self.next_id
        });
        self.next_id += 1;

        writeln!(self.stdin, "{}", req).expect("Failed to write to server");
        self.stdin.flush().expect("Failed to flush stdin");

        let mut line = String::new();
        self.reader
            .read_line(&mut line)
            .expect("Failed to read response");
        serde_json::from_str(&line)
            .unwrap_or(json!({"error": {"code": -1, "message": "parse failed"}}))
    }

    fn close(mut self) {
        drop(self.stdin);
        let _ = self.server.wait();
    }
}
