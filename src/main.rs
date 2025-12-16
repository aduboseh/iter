#[cfg(feature = "full_substrate")]
mod governance;
#[cfg(feature = "full_substrate")]
mod mcp_handler;
#[cfg(feature = "full_substrate")]
mod services;
#[cfg(feature = "full_substrate")]
mod substrate_runtime;
#[cfg(feature = "full_substrate")]
mod types;
#[cfg(feature = "full_substrate")]
mod validation;

#[cfg(feature = "public_stub")]
mod substrate;

use serde_json::json;
use std::io::{BufRead, BufReader, Write};

fn main() {
    // Local identity closure: print the actual executable path and CWD at runtime.
    // (Shows up in Warp's MCP logs as stderr.)
    match std::env::current_exe() {
        Ok(p) => eprintln!("ITER LOCAL PROOF — PATH = {}", p.display()),
        Err(e) => eprintln!("ITER LOCAL PROOF — PATH = <error: {}>", e),
    }
    match std::env::current_dir() {
        Ok(p) => eprintln!("ITER CWD = {}", p.display()),
        Err(e) => eprintln!("ITER CWD = <error: {}>", e),
    }

    print_mode_banner();
    run_stdio_server();
}

fn print_mode_banner() {
    #[cfg(feature = "public_stub")]
    {
        eprintln!("┌────────────────────────────────────────────────────────────┐");
        eprintln!("│ ITER: PUBLIC STUB MODE                                     │");
        eprintln!("│ Proprietary substrate DISABLED                             │");
        eprintln!("│ Responses are deterministic placeholders                   │");
        eprintln!("└────────────────────────────────────────────────────────────┘");
        eprintln!();
    }

    #[cfg(all(feature = "full_substrate", not(feature = "public_stub")))]
    {
        eprintln!("Iter server running (full substrate mode)");
    }
}

#[cfg(feature = "full_substrate")]
fn run_stdio_server() {
    use crate::mcp_handler::handle_rpc;
    use crate::substrate_runtime::SubstrateRuntime;
    use crate::types::RpcRequest;

    let mut runtime = SubstrateRuntime::with_defaults().expect("Failed to initialize execution runtime");
    let stdin = std::io::stdin();
    let mut reader = BufReader::new(stdin.lock());
    let mut stdout = std::io::stdout();

    eprintln!("Iter server running in STDIO mode");

    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => break, // EOF
            Ok(_) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                match serde_json::from_str::<RpcRequest>(line) {
                    Ok(req) => {
                        let resp = handle_rpc(&mut runtime, req);
                        if let Ok(json) = serde_json::to_string(&resp) {
                            writeln!(stdout, "{}", json).expect("stdout write failed");
                            stdout.flush().expect("stdout flush failed");
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to parse JSON-RPC request: {}", e);
                        let error_resp = json!({
                            "jsonrpc": "2.0",
                            "id": null,
                            "error": {
                                "code": -32700,
                                "message": "Parse error"
                            }
                        });
                        if let Ok(json) = serde_json::to_string(&error_resp) {
                            writeln!(stdout, "{}", json).expect("stdout write failed");
                            stdout.flush().expect("stdout flush failed");
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading from stdin: {}", e);
                break;
            }
        }
    }
}

#[cfg(feature = "public_stub")]
fn run_stdio_server() {
    use crate::substrate::stub::StubRuntime;
    use std::io::BufWriter;

    let mut runtime = StubRuntime::new();
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut reader = BufReader::new(stdin.lock());
    let mut writer = BufWriter::new(stdout.lock());

    eprintln!("Iter server running in STDIO mode (stub) — BUILD 2024-12-16-v6-haltra");

    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => break, // EOF
            Ok(_) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                // Simple stub handler - parse JSON-RPC and respond
                match serde_json::from_str::<serde_json::Value>(line) {
                    Ok(req) => {
                        let method = req.get("method").and_then(|m| m.as_str()).unwrap_or("");
                        let id = req.get("id").cloned();
                        
                        // Notifications (no id) get no response per JSON-RPC 2.0 spec
                        if id.is_none() || id.as_ref().map(|v| v.is_null()).unwrap_or(false) {
                            // Still call handler for side effects, but don't respond
                            let _ = handle_stub_request(&mut runtime, method, &req);
                            continue;
                        }
                        
                        // Build response as owned bytes - no shared Value, no reuse
                        let resp = handle_stub_request(&mut runtime, method, &req);
                        let response_bytes = serde_json::to_vec(&json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "result": resp
                        })).unwrap_or_default();
                        
                        // Single atomic write + newline + flush (Haltra pattern)
                        let _ = writer.write_all(&response_bytes);
                        let _ = writer.write_all(b"\n");
                        let _ = writer.flush();
                    }
                    Err(e) => {
                        eprintln!("Failed to parse JSON-RPC request: {}", e);
                        let error_bytes = serde_json::to_vec(&json!({
                            "jsonrpc": "2.0",
                            "id": serde_json::Value::Null,
                            "error": {
                                "code": -32700,
                                "message": "Parse error"
                            }
                        })).unwrap_or_default();
                        let _ = writer.write_all(&error_bytes);
                        let _ = writer.write_all(b"\n");
                        let _ = writer.flush();
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading from stdin: {}", e);
                break;
            }
        }
    }
}

#[cfg(feature = "public_stub")]
fn handle_stub_request(
    runtime: &mut substrate::stub::StubRuntime,
    method: &str,
    req: &serde_json::Value,
) -> serde_json::Value {
    match method {
        "initialize" => {
            // Warp currently advertises protocolVersion "2025-03-26".
            // Echo the client's requested protocol version for maximum compatibility.
            let client_protocol = req
                .get("params")
                .and_then(|p| p.get("protocolVersion"))
                .and_then(|v| v.as_str())
                .unwrap_or("2024-11-05");

            json!({
                "protocolVersion": client_protocol,
                "serverInfo": {
                    "name": "iter-server",
                    "version": "0.3.0"
                },
                "capabilities": {
                    "tools": {},
                    "resources": {},
                    "prompts": {}
                }
            })
        }
        "resources/list" => json!({
            "resources": []
        }),
        "prompts/list" => json!({
            "prompts": []
        }),
        "notifications/initialized" => json!({}),
        "tools/list" | "tools.list" => json!({
            "tools": [
                {
                    "name": "node.create",
                    "description": "Create a node",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "belief": { "type": "number", "description": "Initial belief value" },
                            "energy": { "type": "number", "description": "Initial energy value" }
                        },
                        "required": ["belief", "energy"]
                    }
                },
                {
                    "name": "node.query",
                    "description": "Query a node",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "node_id": { "type": "string", "description": "Node ID (numeric string)" }
                        },
                        "required": ["node_id"]
                    }
                },
                {
                    "name": "node.mutate",
                    "description": "Mutate node belief",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "node_id": { "type": "string", "description": "Node ID (numeric string)" },
                            "delta": { "type": "number", "description": "Belief delta" }
                        },
                        "required": ["node_id", "delta"]
                    }
                },
                {
                    "name": "edge.bind",
                    "description": "Bind an edge",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "src": { "type": "string", "description": "Source node ID (numeric string)" },
                            "dst": { "type": "string", "description": "Destination node ID (numeric string)" },
                            "weight": { "type": "number", "description": "Edge weight" }
                        },
                        "required": ["src", "dst", "weight"]
                    }
                },
                {
                    "name": "edge.propagate",
                    "description": "Run propagation step",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "edge_id": { "type": "string", "description": "Edge ID (accepted for compatibility, not used)" }
                        }
                    }
                },
                {
                    "name": "governor.status",
                    "description": "Query governor status",
                    "inputSchema": { "type": "object", "properties": {} }
                },
                {
                    "name": "governance.status",
                    "description": "Query governance health",
                    "inputSchema": { "type": "object", "properties": {} }
                },
                {
                    "name": "esv.audit",
                    "description": "Audit node ESV",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "node_id": { "type": "string", "description": "Node ID (numeric string)" }
                        },
                        "required": ["node_id"]
                    }
                },
                {
                    "name": "lineage.replay",
                    "description": "Replay lineage",
                    "inputSchema": { "type": "object", "properties": {} }
                }
            ]
        }),
        "tools/call" => {
            let empty_params = json!({});
            let params = req.get("params").unwrap_or(&empty_params);
            let tool_name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
            let empty_args = json!({});
            let args = params.get("arguments").unwrap_or(&empty_args);
            handle_stub_tool(runtime, tool_name, args)
        }
        _ => json!({"error": "Unknown method"})
    }
}

#[cfg(feature = "public_stub")]
fn handle_stub_tool(
    runtime: &mut substrate::stub::StubRuntime,
    tool: &str,
    args: &serde_json::Value,
) -> serde_json::Value {
    match tool {
        "node.create" => {
            let belief = args.get("belief").and_then(|b| b.as_f64()).unwrap_or(0.5);
            let energy = args.get("energy").and_then(|e| e.as_f64()).unwrap_or(100.0);
            let node = runtime.create_node(belief, energy);
            json!({"content": [{"type": "text", "text": serde_json::to_string(&node).unwrap()}]})
        }
        "node.query" => {
            let id_str = args.get("node_id").and_then(|i| i.as_str()).unwrap_or("0");
            let id: u64 = id_str.parse().unwrap_or(0);
            match runtime.query_node(id) {
                Some(node) => json!({"content": [{"type": "text", "text": serde_json::to_string(&node).unwrap()}]}),
                None => json!({"error": {"code": 4004, "message": "Node not found"}})
            }
        }
        "node.mutate" => {
            let id_str = args.get("node_id").and_then(|i| i.as_str()).unwrap_or("0");
            let id: u64 = id_str.parse().unwrap_or(0);
            let delta = args.get("delta").and_then(|d| d.as_f64()).unwrap_or(0.0);
            match runtime.mutate_node(id, delta) {
                Some(node) => json!({"content": [{"type": "text", "text": serde_json::to_string(&node).unwrap()}]}),
                None => json!({"error": {"code": 4004, "message": "Node not found"}})
            }
        }
        "edge.bind" => {
            let src: u64 = args.get("src").and_then(|s| s.as_str()).and_then(|s| s.parse().ok()).unwrap_or(0);
            let dst: u64 = args.get("dst").and_then(|d| d.as_str()).and_then(|d| d.parse().ok()).unwrap_or(0);
            let weight = args.get("weight").and_then(|w| w.as_f64()).unwrap_or(0.5);
            match runtime.bind_edge(src, dst, weight) {
                Some(edge) => json!({"content": [{"type": "text", "text": serde_json::to_string(&edge).unwrap()}]}),
                None => json!({"error": {"code": 4004, "message": "Node not found"}})
            }
        }
        "edge.propagate" => {
            let msg = runtime.propagate();
            json!({"content": [{"type": "text", "text": msg}]})
        }
        "governor.status" | "governance.status" => {
            let status = runtime.governor_status();
            json!({"content": [{"type": "text", "text": serde_json::to_string(&status).unwrap()}]})
        }
        "esv.audit" => {
            let id_str = args.get("node_id").and_then(|i| i.as_str()).unwrap_or("0");
            let id: u64 = id_str.parse().unwrap_or(0);
            match runtime.esv_audit(id) {
                Some(audit) => json!({"content": [{"type": "text", "text": serde_json::to_string(&audit).unwrap()}]}),
                None => json!({"error": {"code": 4004, "message": "Node not found"}})
            }
        }
        "lineage.replay" => {
            let lineage = runtime.lineage_replay();
            json!({"content": [{"type": "text", "text": serde_json::to_string(&lineage).unwrap()}]})
        }
        _ => json!({"error": {"code": 3000, "message": "Unknown tool"}})
    }
}
