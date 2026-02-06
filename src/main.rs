use serde_json::json;
use std::io::{BufRead, BufReader, Write};

/// Server profile controlling which MCP tools are exposed.
///
/// - `Governance`: Production PDP surface. No kernel/graph tools.
/// - `KernelDebug`: Internal-only. Exposes kernel/graph tools for debugging.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ServerProfile {
    Governance,
    KernelDebug,
}

/// Parse --profile flag from CLI args. Default: Governance.
/// Exits with code 1 on unrecognized profile.
fn detect_profile(args: &[String]) -> ServerProfile {
    let profile_arg = args.iter().find(|arg| arg.starts_with("--profile="));
    match profile_arg.map(|s| s.as_str()) {
        Some("--profile=kernel-debug") => ServerProfile::KernelDebug,
        Some("--profile=governance") | None => ServerProfile::Governance,
        Some(other) => {
            eprintln!("FATAL: ERROR_INVALID_PROFILE: {}", other);
            std::process::exit(1);
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let json_only = args.iter().any(|a| a == "--json-only");
    let profile = detect_profile(&args);

    if !json_only {
        match std::env::current_exe() {
            Ok(p) => eprintln!("ITER LOCAL PROOF — PATH = {}", p.display()),
            Err(e) => eprintln!("ITER LOCAL PROOF — PATH = <error: {}>", e),
        }
        match std::env::current_dir() {
            Ok(p) => eprintln!("ITER CWD = {}", p.display()),
            Err(e) => eprintln!("ITER CWD = <error: {}>", e),
        }
        print_mode_banner();
        eprintln!("iter-server profile: {:?}", profile);
    }

    run_stdio_server(json_only, profile);
}

fn print_mode_banner() {
    eprintln!("┌────────────────────────────────────────────────────────────┐");
    eprintln!("│ ITER: PUBLIC STUB MODE                                     │");
    eprintln!("│ Proprietary substrate DISABLED                             │");
    eprintln!("│ Responses are deterministic placeholders                   │");
    eprintln!("└────────────────────────────────────────────────────────────┘");
    eprintln!();
}

fn run_stdio_server(json_only: bool, profile: ServerProfile) {
    use iter_mcp_server::substrate::stub::StubRuntime;
    use std::io::BufWriter;

    let mut runtime = StubRuntime::new();

    // Build and validate tool surface before entering server loop (fail-fast).
    let tools_list = build_tools_list(profile);
    if let Err(msg) = validate_surface(profile, &tools_list) {
        eprintln!("FATAL: {}", msg);
        std::process::exit(1);
    }
    let tools_json = serde_json::to_value(&tools_list).unwrap_or(json!([]));

    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut reader = BufReader::new(stdin.lock());
    let mut writer = BufWriter::new(stdout.lock());

    if !json_only {
        eprintln!(
            "Iter server running in STDIO mode (stub) — v{}",
            env!("CARGO_PKG_VERSION")
        );
    }

    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => break, // EOF
            Ok(_) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                match serde_json::from_str::<serde_json::Value>(line) {
                    Ok(req) => {
                        let method = req.get("method").and_then(|m| m.as_str()).unwrap_or("");
                        let id = req.get("id").cloned();

                        if id.is_none() || id.as_ref().map(|v| v.is_null()).unwrap_or(false) {
                            let _ = handle_stub_request(
                                &mut runtime,
                                method,
                                &req,
                                profile,
                                &tools_json,
                            );
                            continue;
                        }

                        let resp =
                            handle_stub_request(&mut runtime, method, &req, profile, &tools_json);
                        let response_bytes = serde_json::to_vec(&json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "result": resp
                        }))
                        .unwrap_or_default();

                        let _ = writer.write_all(&response_bytes);
                        let _ = writer.write_all(b"\n");
                        let _ = writer.flush();
                    }
                    Err(e) => {
                        if !json_only {
                            eprintln!("Failed to parse JSON-RPC request: {}", e);
                        }
                        let error_bytes = serde_json::to_vec(&json!({
                            "jsonrpc": "2.0",
                            "id": serde_json::Value::Null,
                            "error": {
                                "code": -32700,
                                "message": "Parse error"
                            }
                        }))
                        .unwrap_or_default();
                        let _ = writer.write_all(&error_bytes);
                        let _ = writer.write_all(b"\n");
                        let _ = writer.flush();
                    }
                }
            }
            Err(e) => {
                if !json_only {
                    eprintln!("Error reading from stdin: {}", e);
                }
                break;
            }
        }
    }
}

/// Governance tool definitions — decision.*, audit.*, governance.*, governor.*
/// Includes canonical IDs and legacy aliases.
#[cfg(feature = "public_stub")]
fn governance_tool_defs() -> Vec<serde_json::Value> {
    vec![
        json!({
            "name": "governor.status",
            "description": "[DEPRECATED: use governor.health] Query governor status",
            "inputSchema": { "type": "object", "properties": {} }
        }),
        json!({
            "name": "governance.status",
            "description": "[DEPRECATED: use governance.health] Query governance health",
            "inputSchema": { "type": "object", "properties": {} }
        }),
        json!({
            "name": "governor.health",
            "description": "Governor coherence and drift metrics (canonical)",
            "inputSchema": { "type": "object", "properties": {} }
        }),
        json!({
            "name": "governance.health",
            "description": "Governance subsystem health summary (canonical)",
            "inputSchema": { "type": "object", "properties": {} }
        }),
        json!({
            "name": "esv.audit",
            "description": "[DEPRECATED: use audit.export] Audit node ESV",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "node_id": { "type": "string", "description": "Node ID (numeric string)" }
                },
                "required": ["node_id"]
            }
        }),
        json!({
            "name": "audit.export",
            "description": "Export audit bundle for compliance/archival (canonical)",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "node_id": { "type": "string", "description": "Node ID (numeric string)" }
                },
                "required": ["node_id"]
            }
        }),
        json!({
            "name": "lineage.replay",
            "description": "[DEPRECATED: use audit.replay] Replay lineage",
            "inputSchema": { "type": "object", "properties": {} }
        }),
        json!({
            "name": "audit.replay",
            "description": "Deterministic replay of decision history (canonical)",
            "inputSchema": { "type": "object", "properties": {} }
        }),
        json!({
            "name": "governance.evaluate",
            "description": "[DEPRECATED: use decision.check] Evaluate governance proposal and return authoritative verdict (PHASE 0: Iter-Haltra Bridge)",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "proposal_id": { "type": "string", "description": "Unique proposal identifier" },
                    "state_snapshot_hash": { "type": "string", "description": "SHA-256 hash of the state snapshot" },
                    "constraints": { "type": "object", "description": "Constraints to evaluate (opaque to Iter)" },
                    "requested_action": { "type": "string", "description": "Requested action (opaque to Iter)" }
                },
                "required": ["proposal_id", "state_snapshot_hash", "requested_action"]
            }
        }),
        json!({
            "name": "decision.check",
            "description": "Authoritative PDP decision gate (canonical)",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "proposal_id": { "type": "string", "description": "Unique proposal identifier" },
                    "state_snapshot_hash": { "type": "string", "description": "SHA-256 hash of the state snapshot" },
                    "constraints": { "type": "object", "description": "Constraints to evaluate (opaque to Iter)" },
                    "requested_action": { "type": "string", "description": "Requested action (opaque to Iter)" }
                },
                "required": ["proposal_id", "state_snapshot_hash", "requested_action"]
            }
        }),
        json!({
            "name": "decision.preview",
            "description": "Non-authoritative governance simulation (canonical, Phase 2)",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "proposal_id": { "type": "string", "description": "Unique proposal identifier" },
                    "state_snapshot_hash": { "type": "string", "description": "SHA-256 hash of the state snapshot" },
                    "constraints": { "type": "object", "description": "Constraints to evaluate (opaque to Iter)" },
                    "requested_action": { "type": "string", "description": "Requested action (opaque to Iter)" }
                },
                "required": ["proposal_id", "state_snapshot_hash", "requested_action"]
            }
        }),
        json!({
            "name": "audit.search",
            "description": "Search governance decision history with filters (canonical, Phase 2)",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "principal": { "type": "string", "description": "Filter by principal" },
                    "action": { "type": "string", "description": "Filter by action" },
                    "resource": { "type": "string", "description": "Filter by resource" },
                    "decision": { "type": "string", "description": "Filter by decision verdict" },
                    "policy_id": { "type": "string", "description": "Filter by policy ID" },
                    "from": { "type": "string", "description": "Start timestamp (RFC3339)" },
                    "to": { "type": "string", "description": "End timestamp (RFC3339)" },
                    "limit": { "type": "integer", "description": "Max results (default 100, max 1000)" }
                }
            }
        }),
    ]
}

/// Kernel/graph tool definitions — node.*, edge.*
/// Only available in kernel-debug profile.
#[cfg(feature = "public_stub")]
fn kernel_tool_defs() -> Vec<serde_json::Value> {
    vec![
        json!({
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
        }),
        json!({
            "name": "node.query",
            "description": "Query a node",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "node_id": { "type": "string", "description": "Node ID (numeric string)" }
                },
                "required": ["node_id"]
            }
        }),
        json!({
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
        }),
        json!({
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
        }),
        json!({
            "name": "edge.propagate",
            "description": "Run propagation step",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "edge_id": { "type": "string", "description": "Edge ID (accepted for compatibility, not used)" }
                }
            }
        }),
    ]
}

/// Build the tools list for the given profile.
///
/// - Governance: governance tools only.
/// - KernelDebug: kernel tools + governance tools (superset for debugging).
#[cfg(feature = "public_stub")]
fn build_tools_list(profile: ServerProfile) -> Vec<serde_json::Value> {
    match profile {
        ServerProfile::Governance => governance_tool_defs(),
        ServerProfile::KernelDebug => {
            let mut tools = kernel_tool_defs();
            tools.extend(governance_tool_defs());
            tools
        }
    }
}

/// Validate tool surface invariants at startup.
///
/// In governance profile:
/// - No kernel/graph tools (node.*, edge.*, kernel.*) may be registered.
/// - All canonical governance tools must be present.
///
/// Returns Err with message on violation. Caller must exit.
#[cfg(feature = "public_stub")]
fn validate_surface(profile: ServerProfile, tools: &[serde_json::Value]) -> Result<(), String> {
    if profile == ServerProfile::Governance {
        for tool in tools {
            let name = tool["name"].as_str().unwrap_or("");
            let is_kernel = name.starts_with("kernel.")
                || name.starts_with("node.")
                || name.starts_with("edge.");
            if is_kernel {
                return Err(format!(
                    "Kernel/debug tool '{}' registered in governance profile",
                    name
                ));
            }
        }

        let required = [
            "decision.check",
            "decision.preview",
            "audit.export",
            "audit.replay",
            "audit.search",
            "governance.health",
            "governor.health",
        ];
        let names: Vec<&str> = tools.iter().filter_map(|t| t["name"].as_str()).collect();
        for tool_name in &required {
            if !names.contains(tool_name) {
                return Err(format!(
                    "ERROR_MISSING_CANONICAL_TOOL: '{}' not found in governance profile",
                    tool_name
                ));
            }
        }
    }
    Ok(())
}

#[cfg(feature = "public_stub")]
fn handle_stub_request(
    runtime: &mut iter_mcp_server::substrate::stub::StubRuntime,
    method: &str,
    req: &serde_json::Value,
    profile: ServerProfile,
    tools_list: &serde_json::Value,
) -> serde_json::Value {
    match method {
        "initialize" => {
            let client_protocol = req
                .get("params")
                .and_then(|p| p.get("protocolVersion"))
                .and_then(|v| v.as_str())
                .unwrap_or("2024-11-05");

            json!({
                "protocolVersion": client_protocol,
                "serverInfo": {
                    "name": "iter-server",
                    "version": env!("CARGO_PKG_VERSION")
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
                    "description": "[DEPRECATED: use governor.health] Query governor status",
                    "inputSchema": { "type": "object", "properties": {} }
                },
                {
                    "name": "governance.status",
                    "description": "[DEPRECATED: use governance.health] Query governance health",
                    "inputSchema": { "type": "object", "properties": {} }
                },
                {
                    "name": "governor.health",
                    "description": "Governor coherence and drift metrics (canonical)",
                    "inputSchema": { "type": "object", "properties": {} }
                },
                {
                    "name": "governance.health",
                    "description": "Governance subsystem health summary (canonical)",
                    "inputSchema": { "type": "object", "properties": {} }
                },
                {
                    "name": "esv.audit",
                    "description": "[DEPRECATED: use audit.export] Audit node ESV",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "node_id": { "type": "string", "description": "Node ID (numeric string)" }
                        },
                        "required": ["node_id"]
                    }
                },
                {
                    "name": "audit.export",
                    "description": "Export audit bundle for compliance/archival (canonical)",
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
                    "description": "[DEPRECATED: use audit.replay] Replay lineage",
                    "inputSchema": { "type": "object", "properties": {} }
                },
                {
                    "name": "audit.replay",
                    "description": "Deterministic replay of decision history (canonical)",
                    "inputSchema": { "type": "object", "properties": {} }
                },
                {
                    "name": "governance.evaluate",
                    "description": "[DEPRECATED: use decision.check] Evaluate governance proposal and return authoritative verdict (PHASE 0: Iter-Haltra Bridge)",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "proposal_id": { "type": "string", "description": "Unique proposal identifier" },
                            "state_snapshot_hash": { "type": "string", "description": "SHA-256 hash of the state snapshot" },
                            "constraints": { "type": "object", "description": "Constraints to evaluate (opaque to Iter)" },
                            "requested_action": { "type": "string", "description": "Requested action (opaque to Iter)" }
                        },
                        "required": ["proposal_id", "state_snapshot_hash", "requested_action"]
                    }
                },
                {
                    "name": "decision.check",
                    "description": "Authoritative PDP decision gate (canonical)",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "proposal_id": { "type": "string", "description": "Unique proposal identifier" },
                            "state_snapshot_hash": { "type": "string", "description": "SHA-256 hash of the state snapshot" },
                            "constraints": { "type": "object", "description": "Constraints to evaluate (opaque to Iter)" },
                            "requested_action": { "type": "string", "description": "Requested action (opaque to Iter)" }
                        },
                        "required": ["proposal_id", "state_snapshot_hash", "requested_action"]
                    }
                }
            ]
        }),
        "tools/call" => {
            let empty_params = json!({});
            let params = req.get("params").unwrap_or(&empty_params);
            let tool_name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
            let empty_args = json!({});
            let args = params.get("arguments").unwrap_or(&empty_args);
            handle_stub_tool(runtime, tool_name, args, profile)
        }
        _ => json!({"error": "Unknown method"}),
    }
}

#[cfg(feature = "public_stub")]
fn parse_governance_proposal(
    args: &serde_json::Value,
) -> iter_mcp_server::substrate::stub::GovernanceProposal {
    let proposal_id = args
        .get("proposal_id")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    let state_snapshot_hash = args
        .get("state_snapshot_hash")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let constraints = args.get("constraints").cloned().unwrap_or(json!({}));
    let requested_action = args
        .get("requested_action")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    let proposal_c14n = args
        .get("proposal_c14n")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let proposal_hash = args
        .get("proposal_hash")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    iter_mcp_server::substrate::stub::GovernanceProposal {
        proposal_id,
        state_snapshot_hash,
        constraints,
        requested_action,
        proposal_c14n,
        proposal_hash,
    }
}

#[cfg(feature = "public_stub")]
fn handle_stub_tool(
    runtime: &mut iter_mcp_server::substrate::stub::StubRuntime,
    tool: &str,
    args: &serde_json::Value,
    profile: ServerProfile,
) -> serde_json::Value {
    use iter_mcp_server::runtime::GovernanceRuntime as GovernanceRuntimeTrait;

    // Reject kernel/graph tools in governance profile.
    if profile == ServerProfile::Governance {
        let is_kernel =
            tool.starts_with("kernel.") || tool.starts_with("node.") || tool.starts_with("edge.");
        if is_kernel {
            return json!({
                "error": {
                    "code": 3001,
                    "message": format!("Tool '{}' not available in governance profile", tool)
                }
            });
        }
    }

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
                Some(node) => {
                    json!({"content": [{"type": "text", "text": serde_json::to_string(&node).unwrap()}]})
                }
                None => json!({"error": {"code": 4004, "message": "Node not found"}}),
            }
        }
        "node.mutate" => {
            let id_str = args.get("node_id").and_then(|i| i.as_str()).unwrap_or("0");
            let id: u64 = id_str.parse().unwrap_or(0);
            let delta = args.get("delta").and_then(|d| d.as_f64()).unwrap_or(0.0);
            match runtime.mutate_node(id, delta) {
                Some(node) => {
                    json!({"content": [{"type": "text", "text": serde_json::to_string(&node).unwrap()}]})
                }
                None => json!({"error": {"code": 4004, "message": "Node not found"}}),
            }
        }
        "edge.bind" => {
            let src: u64 = args
                .get("src")
                .and_then(|s| s.as_str())
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            let dst: u64 = args
                .get("dst")
                .and_then(|d| d.as_str())
                .and_then(|d| d.parse().ok())
                .unwrap_or(0);
            let weight = args.get("weight").and_then(|w| w.as_f64()).unwrap_or(0.5);
            match runtime.bind_edge(src, dst, weight) {
                Some(edge) => {
                    json!({"content": [{"type": "text", "text": serde_json::to_string(&edge).unwrap()}]})
                }
                None => json!({"error": {"code": 4004, "message": "Node not found"}}),
            }
        }
        "edge.propagate" => {
            let msg = runtime.propagate();
            json!({"content": [{"type": "text", "text": msg}]})
        }
        "governor.status" | "governance.status" | "governor.health" | "governance.health" => {
            let status = runtime.governor_status();
            json!({"content": [{"type": "text", "text": serde_json::to_string(&status).unwrap()}]})
        }
        "esv.audit" | "audit.export" => {
            let id_str = args.get("node_id").and_then(|i| i.as_str()).unwrap_or("0");
            let id: u64 = id_str.parse().unwrap_or(0);
            match runtime.esv_audit(id) {
                Some(audit) => {
                    json!({"content": [{"type": "text", "text": serde_json::to_string(&audit).unwrap()}]})
                }
                None => json!({"error": {"code": 4004, "message": "Node not found"}}),
            }
        }
        "lineage.replay" | "audit.replay" => {
            let lineage = runtime.lineage_replay();
            json!({"content": [{"type": "text", "text": serde_json::to_string(&lineage).unwrap()}]})
        }
        "governance.evaluate" | "decision.check" => {
            // PHASE 0+: Iter-Haltra Bridge with JCS verification
            // This is the authoritative governance entry point.
            // Haltra proposes, Iter decides.
            let proposal_id = args
                .get("proposal_id")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            let state_snapshot_hash = args
                .get("state_snapshot_hash")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let constraints = args.get("constraints").cloned().unwrap_or(json!({}));
            let requested_action = args
                .get("requested_action")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            // Patch A: JCS canonical bytes and hash
            let proposal_c14n = args
                .get("proposal_c14n")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let proposal_hash = args
                .get("proposal_hash")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let proposal = iter_mcp_server::substrate::stub::GovernanceProposal {
                proposal_id,
                state_snapshot_hash,
                constraints,
                requested_action,
                proposal_c14n,
                proposal_hash,
            };

            match runtime.evaluate_governance(&proposal) {
                Ok(evaluation) => {
                    json!({"content": [{"type": "text", "text": serde_json::to_string(&evaluation).unwrap()}]})
                }
                Err(e) => {
                    json!({"error": {"code": 1001, "message": e.to_string(), "data": serde_json::to_value(&e).ok()}})
                }
            }
        }
        "decision.preview" => {
            let proposal = parse_governance_proposal(args);
            match GovernanceRuntimeTrait::preview(runtime, &proposal) {
                Ok(outcome) => {
                    json!({"content": [{"type": "text", "text": serde_json::to_string(&outcome).unwrap()}]})
                }
                Err(e) => {
                    json!({"error": {"code": 5001, "message": e.to_string(), "data": serde_json::to_value(&e).ok()}})
                }
            }
        }
        "audit.search" => {
            let filter: iter_mcp_server::substrate::stub::AuditSearchFilter =
                serde_json::from_value(args.clone()).unwrap_or_default();
            let result = GovernanceRuntimeTrait::search_decisions(runtime, &filter);
            json!({"content": [{"type": "text", "text": serde_json::to_string(&result).unwrap()}]})
        }
        _ => json!({"error": {"code": 3000, "message": "Unknown tool"}}),
    }
}
