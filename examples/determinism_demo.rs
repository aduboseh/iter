//! Determinism Demo
//!
//! Demonstrates that the Iter server produces identical outputs for identical inputs.
//! This example treats the server as a black box, communicating only via MCP protocol.
//!
//! # Usage
//!
//! ```bash
//! cargo build --release --bin iter-server
//! cargo run --example determinism_demo
//! ```
//!
//! # What this demonstrates
//!
//! - Same inputs → same outputs (deterministic execution)
//! - Observable determinism from the client perspective
//! - No knowledge of server internals required

use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

fn main() {
    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║             ITER DETERMINISM DEMONSTRATION                       ║");
    println!("║     Same inputs → Same outputs (client perspective)              ║");
    println!("╚══════════════════════════════════════════════════════════════════╝\n");

    // Run the demo twice and compare outputs
    println!("Running identical operations twice to verify determinism...\n");

    let run1 = run_demo_sequence();
    let run2 = run_demo_sequence();

    println!("┌─────────────────────────────────────────────────────────────────┐");
    println!("│ DETERMINISM VERIFICATION                                       │");
    println!("└─────────────────────────────────────────────────────────────────┘");

    if run1 == run2 {
        println!("  ✓ Both runs produced IDENTICAL outputs");
        println!("  ✓ Determinism verified from client perspective\n");
    } else {
        println!("  ✗ Outputs differ — determinism NOT verified\n");
        println!("  Run 1: {:?}", run1);
        println!("  Run 2: {:?}", run2);
    }
}

/// Run a sequence of MCP operations and return key outputs for comparison.
fn run_demo_sequence() -> Vec<String> {
    let mut outputs = Vec::new();

    // Spawn iter-server
    let mut server = Command::new("cargo")
        .args(["run", "--release", "--bin", "iter-server"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to spawn iter-server. Did you run `cargo build --release`?");

    let mut stdin = server.stdin.take().expect("Failed to open stdin");
    let stdout = server.stdout.take().expect("Failed to open stdout");
    let mut reader = BufReader::new(stdout);

    // Helper to send request and read response
    let mut rpc_id = 1;
    let mut send_rpc = |method: &str, params: Value| -> Value {
        let req = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": rpc_id
        });
        rpc_id += 1;

        writeln!(stdin, "{}", req).expect("Failed to write to server");
        stdin.flush().expect("Failed to flush stdin");

        let mut line = String::new();
        reader
            .read_line(&mut line)
            .expect("Failed to read response");
        serde_json::from_str(&line).unwrap_or(json!({"error": "parse failed"}))
    };

    // Initialize
    let _ = send_rpc("initialize", json!({}));

    // Create two nodes with fixed parameters
    let node_a = send_rpc(
        "tools/call",
        json!({
            "name": "node.create",
            "arguments": { "belief": 0.7, "energy": 100.0 }
        }),
    );
    outputs.push(extract_text(&node_a));

    let node_b = send_rpc(
        "tools/call",
        json!({
            "name": "node.create",
            "arguments": { "belief": 0.3, "energy": 50.0 }
        }),
    );
    outputs.push(extract_text(&node_b));

    // Bind edge
    let edge = send_rpc(
        "tools/call",
        json!({
            "name": "edge.bind",
            "arguments": { "src": "0", "dst": "1", "weight": 0.7 }
        }),
    );
    outputs.push(extract_text(&edge));

    // Query node
    let query = send_rpc(
        "tools/call",
        json!({
            "name": "node.query",
            "arguments": { "node_id": "0" }
        }),
    );
    outputs.push(extract_text(&query));

    // Propagate
    let _ = send_rpc(
        "tools/call",
        json!({
            "name": "edge.propagate",
            "arguments": { "edge_id": "0" }
        }),
    );

    // Query after propagation
    let query_after = send_rpc(
        "tools/call",
        json!({
            "name": "node.query",
            "arguments": { "node_id": "1" }
        }),
    );
    outputs.push(extract_text(&query_after));

    // Governor status
    let gov = send_rpc(
        "tools/call",
        json!({
            "name": "governor.status",
            "arguments": {}
        }),
    );
    outputs.push(extract_text(&gov));

    // Close server
    drop(stdin);
    let _ = server.wait();

    outputs
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
