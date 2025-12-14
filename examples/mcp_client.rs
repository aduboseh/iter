//! Iter Reference Client
//!
//! A true MCP client that communicates with the Iter server via STDIO transport.
//! Demonstrates all available tools without any knowledge of server internals.
//!
//! # Usage
//!
//! ```bash
//! cargo build --release --bin iter-server
//! cargo run --example mcp_client
//! ```
//!
//! # Tools Demonstrated
//!
//! - `node.create` - Create node with belief and energy
//! - `node.query` - Query node state by ID
//! - `node.mutate` - Mutate node belief (debug operation)
//! - `edge.bind` - Bind edge between nodes
//! - `edge.propagate` - Run simulation step
//! - `governor.status` - Query governor status
//! - `governance.status` - Query full governance health
//! - `esv.audit` - Audit node compliance status
//! - `lineage.replay` - Replay lineage history

use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};

fn main() {
    println!("=== Iter Reference Client ===\n");

    let mut client = McpClient::spawn();

    // ========================================================================
    // 1. Protocol Initialization
    // ========================================================================
    println!("--- Protocol Initialization ---");

    let init_resp = client.call("initialize", json!({}));
    println!("initialize: {}\n", format_response(&init_resp));

    // ========================================================================
    // 2. List Available Tools
    // ========================================================================
    println!("--- List Tools ---");

    let list_resp = client.call("tools/list", json!({}));
    if let Some(tools) = list_resp
        .get("result")
        .and_then(|r| r.get("tools"))
        .and_then(|t| t.as_array())
    {
        println!("Available tools ({}):", tools.len());
        for tool in tools {
            if let (Some(name), Some(desc)) = (
                tool.get("name").and_then(|n| n.as_str()),
                tool.get("description").and_then(|d| d.as_str()),
            ) {
                println!("  - {}: {}", name, desc);
            }
        }
    }
    println!();

    // ========================================================================
    // 3. Node Operations
    // ========================================================================
    println!("--- Node Operations ---");

    // Create first node
    let create_resp = client.tool_call("node.create", json!({
        "belief": 0.7,
        "energy": 100.0
    }));
    println!("node.create (belief=0.7, energy=100): {}", extract_text(&create_resp));
    let node1_id = extract_id(&create_resp).unwrap_or(0);
    println!("  -> Created node ID: {}", node1_id);

    // Create second node
    let create_resp2 = client.tool_call("node.create", json!({
        "belief": 0.3,
        "energy": 50.0
    }));
    let node2_id = extract_id(&create_resp2).unwrap_or(1);
    println!("node.create (belief=0.3, energy=50): created node {}", node2_id);

    // Query node
    let query_resp = client.tool_call("node.query", json!({
        "node_id": node1_id.to_string()
    }));
    println!("node.query (node {}): {}", node1_id, extract_text(&query_resp));

    // Mutate node (DEBUG operation)
    let mutate_resp = client.tool_call("node.mutate", json!({
        "node_id": node1_id.to_string(),
        "delta": 0.1
    }));
    println!("node.mutate (node {}, delta=+0.1): {}", node1_id, extract_text(&mutate_resp));
    println!();

    // ========================================================================
    // 4. Edge Operations
    // ========================================================================
    println!("--- Edge Operations ---");

    // Bind edge
    let bind_resp = client.tool_call("edge.bind", json!({
        "src": node1_id.to_string(),
        "dst": node2_id.to_string(),
        "weight": 0.5
    }));
    println!("edge.bind ({}→{}, weight=0.5): {}", node1_id, node2_id, extract_text(&bind_resp));

    // Propagate (run simulation step)
    let prop_resp = client.tool_call("edge.propagate", json!({
        "edge_id": "0"
    }));
    println!("edge.propagate (step): {}", extract_text(&prop_resp));

    // Query node after propagation
    let query_resp2 = client.tool_call("node.query", json!({
        "node_id": node2_id.to_string()
    }));
    println!("node.query (node {} after propagation): {}", node2_id, extract_text(&query_resp2));
    println!();

    // ========================================================================
    // 5. Governance Operations
    // ========================================================================
    println!("--- Governance Operations ---");

    // Governor status
    let gov_resp = client.tool_call("governor.status", json!({}));
    println!("governor.status: {}", extract_text(&gov_resp));

    // Full governance status
    let governance_resp = client.tool_call("governance.status", json!({}));
    println!("governance.status: {}", extract_text(&governance_resp));

    // ESV audit
    let esv_resp = client.tool_call("esv.audit", json!({
        "node_id": node1_id.to_string()
    }));
    println!("esv.audit (node {}): {}", node1_id, extract_text(&esv_resp));
    println!();

    // ========================================================================
    // 6. Lineage Operations
    // ========================================================================
    println!("--- Lineage Operations ---");

    // Lineage replay
    let replay_resp = client.tool_call("lineage.replay", json!({}));
    println!("lineage.replay: {}", extract_text(&replay_resp));
    println!();

    // ========================================================================
    // 7. Error Handling Demo
    // ========================================================================
    println!("--- Error Handling Demo ---");

    // Query non-existent node
    let bad_query_resp = client.tool_call("node.query", json!({
        "node_id": "999999"
    }));
    println!("node.query (non-existent node 999999):");
    if let Some(err) = bad_query_resp.get("error") {
        println!("  Error code: {}", err.get("code").unwrap_or(&json!(-1)));
        println!("  Error message: {}", err.get("message").unwrap_or(&json!("unknown")));
    } else {
        println!("  {}", extract_text(&bad_query_resp));
    }

    // Invalid tool name
    let bad_tool_resp = client.tool_call("invalid.tool", json!({}));
    println!("tools/call (invalid.tool):");
    if let Some(err) = bad_tool_resp.get("error") {
        println!("  Error code: {}", err.get("code").unwrap_or(&json!(-1)));
        println!("  Error message: {}", err.get("message").unwrap_or(&json!("unknown")));
    }
    println!();

    println!("=== Reference Client Complete ===");

    client.close();
}

/// MCP client that communicates with iter-server via STDIO.
struct McpClient {
    server: Child,
    stdin: std::process::ChildStdin,
    reader: BufReader<std::process::ChildStdout>,
    next_id: u64,
}

impl McpClient {
    /// Spawn iter-server and return a client handle.
    fn spawn() -> Self {
        let mut server = Command::new("cargo")
            .args(["run", "--release", "--bin", "iter-server"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to spawn iter-server. Did you run `cargo build --release`?");

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

    /// Send a JSON-RPC request and return the response.
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
        self.reader.read_line(&mut line).expect("Failed to read response");
        serde_json::from_str(&line).unwrap_or(json!({"error": {"code": -1, "message": "parse failed"}}))
    }

    /// Convenience method for tools/call.
    fn tool_call(&mut self, tool_name: &str, arguments: Value) -> Value {
        self.call("tools/call", json!({
            "name": tool_name,
            "arguments": arguments
        }))
    }

    /// Close the server process.
    fn close(mut self) {
        drop(self.stdin);
        let _ = self.server.wait();
    }
}

/// Extract text content from MCP response.
fn extract_text(resp: &Value) -> String {
    resp.get("result")
        .and_then(|r| r.get("content"))
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|item| item.get("text"))
        .and_then(|t| t.as_str())
        .unwrap_or("(no content)")
        .to_string()
}

/// Extract node ID from create response.
fn extract_id(resp: &Value) -> Option<u64> {
    let text = resp
        .get("result")?
        .get("content")?
        .as_array()?
        .first()?
        .get("text")?
        .as_str()?;

    let json: Value = serde_json::from_str(text).ok()?;
    json.get("id")?.as_u64()
}

/// Format response for display.
fn format_response(resp: &Value) -> String {
    serde_json::to_string_pretty(resp).unwrap_or_else(|_| resp.to_string())
}
