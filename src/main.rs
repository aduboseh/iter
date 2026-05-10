use serde_json::json;
use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Write};

// Compile-time build attestation baked into the binary.
const ITER_BUILD_MODE: &str = env!("ITER_BUILD_MODE");

/// Server profile controlling which MCP tools are exposed.
///
/// - `Governance`: Production PDP surface. No kernel/graph tools.
/// - `KernelDebug`: Internal-only. Exposes kernel/graph tools for debugging.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ServerProfile {
    Governance,
    KernelDebug,
}

/// Runtime mode controlling which in-crate engine backs MCP governance calls.
///
/// - `Demo`: Default public stub behavior.
/// - `GovernedLocal`: Governed runtime over the local stub substrate. Emits
///   DecisionPackets, but is not yet SCG-backed.
/// - `ScgBacked`: Authoritative mode backed by the live SCG governance endpoint.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RuntimeMode {
    Demo,
    GovernedLocal,
    ScgBacked,
}

#[derive(Default)]
struct ResourceHashRegistry {
    resources: BTreeMap<String, String>,
}

impl ResourceHashRegistry {
    fn register(
        &mut self,
        resource_path: &str,
        expected_hash: &str,
    ) -> Result<(String, String), String> {
        let resource = normalize_resource_path(resource_path)
            .ok_or_else(|| "resource_path must be a non-empty path".to_string())?;
        let hash = normalize_sha256_hash(expected_hash)
            .ok_or_else(|| "expected_hash must be a non-empty SHA-256 hex value".to_string())?;

        self.resources.insert(resource.clone(), hash.clone());
        Ok((resource, hash))
    }

    fn find_match<'a>(&'a self, args: &serde_json::Value) -> Option<(&'a str, &'a str)> {
        let candidates = resource_candidates(args);
        self.resources.iter().find_map(|(resource, hash)| {
            if candidates.iter().any(|candidate| candidate == resource) {
                return Some((resource.as_str(), hash.as_str()));
            }

            let requested_action = args
                .get("requested_action")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim_matches('"')
                .replace('\\', "/")
                .to_ascii_lowercase();
            if requested_action.contains(resource) {
                Some((resource.as_str(), hash.as_str()))
            } else {
                None
            }
        })
    }
}

fn normalize_sha256_hash(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    let hex = trimmed
        .strip_prefix("sha256:")
        .or_else(|| trimmed.strip_prefix("SHA256:"))
        .unwrap_or(trimmed)
        .trim()
        .to_ascii_lowercase();

    if hex.is_empty() || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }

    Some(format!("sha256:{}", hex))
}

fn hash_prefix(hash: &str) -> String {
    if let Some(hex) = hash.strip_prefix("sha256:") {
        format!("sha256:{}", hex.chars().take(8).collect::<String>())
    } else {
        hash.chars().take(15).collect()
    }
}

fn normalize_resource_path(raw: &str) -> Option<String> {
    let mut path = raw
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .replace('\\', "/");
    while path.starts_with("./") {
        path = path[2..].to_string();
    }

    let lower = path.to_ascii_lowercase();
    let repo_root = std::env::current_dir()
        .ok()
        .map(|p| p.to_string_lossy().replace('\\', "/").to_ascii_lowercase());

    let relative = if let Some(root) = repo_root {
        lower
            .strip_prefix(&root)
            .map(|s| s.trim_start_matches('/').to_string())
            .unwrap_or(lower)
    } else {
        lower
    };

    let normalized = relative
        .split('/')
        .filter(|part| !part.is_empty() && *part != ".")
        .collect::<Vec<_>>()
        .join("/");

    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

fn resource_candidates(args: &serde_json::Value) -> Vec<String> {
    let mut candidates = Vec::new();
    let mut push_candidate = |value: Option<&str>| {
        if let Some(raw) = value {
            if let Some(normalized) = normalize_resource_path(raw) {
                candidates.push(normalized);
            }
        }
    };

    push_candidate(args.get("resource_path").and_then(|v| v.as_str()));
    push_candidate(args.get("resource").and_then(|v| v.as_str()));

    if let Some(constraints) = args.get("constraints") {
        push_candidate(constraints.get("scope").and_then(|v| v.as_str()));
        push_candidate(constraints.get("resource").and_then(|v| v.as_str()));
        push_candidate(constraints.get("resource_path").and_then(|v| v.as_str()));
    }

    candidates
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

/// Parse --runtime-mode flag from CLI args. Default: demo.
/// Exits with code 1 on unrecognized runtime mode.
fn detect_runtime_mode(args: &[String]) -> RuntimeMode {
    let runtime_arg = args.iter().find(|arg| arg.starts_with("--runtime-mode="));
    match runtime_arg.map(|s| s.as_str()) {
        Some("--runtime-mode=scg-backed") => RuntimeMode::ScgBacked,
        Some("--runtime-mode=governed-local") => RuntimeMode::GovernedLocal,
        Some("--runtime-mode=demo") | None => RuntimeMode::Demo,
        Some(other) => {
            eprintln!("FATAL: ERROR_INVALID_RUNTIME_MODE: {}", other);
            std::process::exit(1);
        }
    }
}

fn assert_build_mode() {
    match ITER_BUILD_MODE {
        "PUBLIC_STUB" | "FULL_SUBSTRATE" => {}
        other => panic!("ITER_BUILD_MODE is invalid or tampered: {:?}", other),
    }
}

fn main() {
    assert_build_mode();

    let args: Vec<String> = std::env::args().collect();
    let json_only = args.iter().any(|a| a == "--json-only");
    let profile = detect_profile(&args);
    let runtime_mode = detect_runtime_mode(&args);

    if !json_only {
        match std::env::current_exe() {
            Ok(p) => eprintln!("ITER LOCAL PROOF — PATH = {}", p.display()),
            Err(e) => eprintln!("ITER LOCAL PROOF — PATH = <error: {}>", e),
        }
        match std::env::current_dir() {
            Ok(p) => eprintln!("ITER CWD = {}", p.display()),
            Err(e) => eprintln!("ITER CWD = <error: {}>", e),
        }
        print_mode_banner(runtime_mode);
        eprintln!("iter-server profile: {:?}", profile);
        eprintln!("iter-server runtime mode: {:?}", runtime_mode);
    }

    if let Err(err) = run_stdio_server(json_only, profile, runtime_mode) {
        eprintln!("FATAL: {}", err);
        std::process::exit(1);
    }
}

fn print_mode_banner(runtime_mode: RuntimeMode) {
    eprintln!("┌────────────────────────────────────────────────────────────┐");
    match runtime_mode {
        RuntimeMode::Demo => {
            if ITER_BUILD_MODE == "PUBLIC_STUB" {
                eprintln!("│ ITER: PUBLIC STUB MODE                                     │");
            } else {
                eprintln!("│ ITER: DEMO MODE (FULL SUBSTRATE BUILD)                     │");
            }
            eprintln!("│ Proprietary substrate DISABLED                             │");
            eprintln!("│ Responses are deterministic placeholders                   │");
            eprintln!("└────────────────────────────────────────────────────────────┘");
            eprintln!(
                "WARNING: Server running in stub mode. SCG execution path not active. See WO-ITER-RUNTIME-001B."
            );
        }
        RuntimeMode::GovernedLocal => {
            eprintln!("│ ITER: GOVERNED LOCAL MODE                                  │");
            eprintln!("│ Policy runtime ACTIVE over local stub substrate            │");
            eprintln!("│ SCG execution path still inactive                          │");
            eprintln!("└────────────────────────────────────────────────────────────┘");
            eprintln!(
                "WARNING: Governed local mode is packet-emitting and replay-capable, but not SCG-backed. See WO-ITER-RUNTIME-001B."
            );
        }
        RuntimeMode::ScgBacked => {
            eprintln!("│ ITER: SCG-BACKED GOVERNED RUNTIME                          │");
            eprintln!("│ POST /governance/evaluate — fail-closed                    │");
            eprintln!("│ governance_hash bound at boot                              │");
            eprintln!("│ replay integrity verified on every response                │");
            eprintln!("└────────────────────────────────────────────────────────────┘");
        }
    }
    eprintln!();
}

fn run_stdio_server(
    json_only: bool,
    profile: ServerProfile,
    runtime_mode: RuntimeMode,
) -> Result<(), iter_mcp_server::runtime::GovernanceRuntimeError> {
    use std::io::BufWriter;

    let mut runtime = ServerRuntime::new(runtime_mode)?;
    let mut resource_registry = ResourceHashRegistry::default();

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
            "Iter server running in STDIO mode ({:?}) — v{}",
            runtime.mode(),
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
                            let _ = handle_request(
                                &mut runtime,
                                &mut resource_registry,
                                method,
                                &req,
                                profile,
                                &tools_json,
                            );
                            continue;
                        }

                        let resp = handle_request(
                            &mut runtime,
                            &mut resource_registry,
                            method,
                            &req,
                            profile,
                            &tools_json,
                        );
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

    Ok(())
}

enum ServerRuntime {
    Demo(iter_mcp_server::substrate::stub::StubRuntime),
    GovernedLocal(iter_mcp_server::governed::GovernedRuntime),
    ScgBacked(iter_mcp_server::governance_connector::ScgRuntime),
}

impl ServerRuntime {
    fn new(mode: RuntimeMode) -> Result<Self, iter_mcp_server::runtime::GovernanceRuntimeError> {
        match mode {
            RuntimeMode::Demo => Ok(Self::Demo(
                iter_mcp_server::substrate::stub::StubRuntime::new(),
            )),
            RuntimeMode::GovernedLocal => Ok(Self::GovernedLocal(
                iter_mcp_server::governed::GovernedRuntime::new(
                    iter_mcp_server::substrate::stub::StubRuntime::new(),
                    iter_mcp_server::policy::PolicyConfig::default(),
                    iter_mcp_server::economics::EconomicsConfig::default(),
                ),
            )),
            RuntimeMode::ScgBacked => {
                let endpoint = std::env::var("SCG_ENDPOINT").map_err(|_| {
                    iter_mcp_server::runtime::GovernanceRuntimeError::ConfigMissing(
                        "SCG_ENDPOINT".to_string(),
                    )
                })?;
                let hash_path = std::env::var("SCG_GOVERNANCE_HASH_PATH")
                    .unwrap_or_else(|_| "governance/governance.hash".to_string());
                let boot_hash = std::fs::read_to_string(&hash_path)
                    .map_err(|e| {
                        iter_mcp_server::runtime::GovernanceRuntimeError::ConfigMissing(format!(
                            "governance.hash not found at {}: {}",
                            hash_path, e
                        ))
                    })?
                    .trim()
                    .to_string();

                if boot_hash.is_empty() {
                    return Err(
                        iter_mcp_server::runtime::GovernanceRuntimeError::ConfigMissing(
                            "governance.hash is empty".to_string(),
                        ),
                    );
                }

                Ok(Self::ScgBacked(
                    iter_mcp_server::governance_connector::ScgRuntime::connect(
                        endpoint, boot_hash,
                    )?,
                ))
            }
        }
    }

    fn mode(&self) -> RuntimeMode {
        match self {
            Self::Demo(_) => RuntimeMode::Demo,
            Self::GovernedLocal(_) => RuntimeMode::GovernedLocal,
            Self::ScgBacked(_) => RuntimeMode::ScgBacked,
        }
    }

    fn graph(&self) -> &iter_mcp_server::substrate::stub::StubRuntime {
        match self {
            Self::Demo(runtime) => runtime,
            Self::GovernedLocal(runtime) => runtime.graph(),
            Self::ScgBacked(runtime) => runtime.graph(),
        }
    }

    fn graph_mut(&mut self) -> &mut iter_mcp_server::substrate::stub::StubRuntime {
        match self {
            Self::Demo(runtime) => runtime,
            Self::GovernedLocal(runtime) => runtime.graph_mut(),
            Self::ScgBacked(runtime) => runtime.graph_mut(),
        }
    }

    fn evaluate(
        &mut self,
        proposal: &iter_mcp_server::substrate::stub::GovernanceProposal,
    ) -> Result<
        iter_mcp_server::runtime::GovernanceOutcome,
        iter_mcp_server::runtime::GovernanceRuntimeError,
    > {
        use iter_mcp_server::runtime::GovernanceRuntime as GovernanceRuntimeTrait;

        match self {
            Self::Demo(runtime) => GovernanceRuntimeTrait::evaluate(runtime, proposal),
            Self::GovernedLocal(runtime) => GovernanceRuntimeTrait::evaluate(runtime, proposal),
            Self::ScgBacked(runtime) => GovernanceRuntimeTrait::evaluate(runtime, proposal),
        }
    }

    fn preview(
        &self,
        proposal: &iter_mcp_server::substrate::stub::GovernanceProposal,
    ) -> Result<
        iter_mcp_server::runtime::GovernanceOutcome,
        iter_mcp_server::runtime::GovernanceRuntimeError,
    > {
        use iter_mcp_server::runtime::GovernanceRuntime as GovernanceRuntimeTrait;

        match self {
            Self::Demo(runtime) => GovernanceRuntimeTrait::preview(runtime, proposal),
            Self::GovernedLocal(runtime) => GovernanceRuntimeTrait::preview(runtime, proposal),
            Self::ScgBacked(runtime) => GovernanceRuntimeTrait::preview(runtime, proposal),
        }
    }

    fn search_decisions(
        &self,
        filter: &iter_mcp_server::substrate::stub::AuditSearchFilter,
    ) -> iter_mcp_server::substrate::stub::AuditSearchResult {
        use iter_mcp_server::runtime::GovernanceRuntime as GovernanceRuntimeTrait;

        match self {
            Self::Demo(runtime) => GovernanceRuntimeTrait::search_decisions(runtime, filter),
            Self::GovernedLocal(runtime) => {
                GovernanceRuntimeTrait::search_decisions(runtime, filter)
            }
            Self::ScgBacked(runtime) => GovernanceRuntimeTrait::search_decisions(runtime, filter),
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
                    "node_id": { "type": ["string", "integer"], "description": "Node ID (numeric string or integer)" }
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
                    "node_id": { "type": ["string", "integer"], "description": "Node ID (numeric string or integer)" }
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
            "name": "register_resource",
            "description": "Session-scoped resource hash registry. Registers a resource path and its expected SHA-256 baseline hash for contract-layer validation in decision.check. Registry is cleared on server restart. Must be called before decision.check for hash validation to be enforced.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "resource_path": { "type": "string", "description": "Resource path to bind, normalized relative to the current repository root" },
                    "expected_hash": { "type": "string", "description": "Expected SHA-256 baseline hash, with or without sha256: prefix" }
                },
                "required": ["resource_path", "expected_hash"]
            }
        }),
        json!({
            "name": "governance.evaluate",
            "description": "[DEPRECATED: use decision.check] Governance decision gate. Default runtime is demo stub mode; `--runtime-mode=governed-local` emits governed packets over the local stub substrate; `--runtime-mode=scg-backed` calls the live SCG governance endpoint fail-closed",
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
            "description": "Governance decision gate. Default runtime is demo stub mode; `--runtime-mode=governed-local` emits governed packets over the local stub substrate; `--runtime-mode=scg-backed` calls the live SCG governance endpoint fail-closed",
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
            "description": "Governance preview through the active runtime. Demo is non-authoritative; governed-local is read-only but packet-capable on evaluate; scg-backed previews call the live SCG endpoint without emitting a packet",
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
                    "node_id": { "type": ["string", "integer"], "description": "Node ID (numeric string or integer)" }
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
                    "node_id": { "type": ["string", "integer"], "description": "Node ID (numeric string or integer)" },
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
                    "src": { "type": ["string", "integer"], "description": "Source node ID (numeric string or integer)" },
                    "dst": { "type": ["string", "integer"], "description": "Destination node ID (numeric string or integer)" },
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
            "register_resource",
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
fn handle_request(
    runtime: &mut ServerRuntime,
    registry: &mut ResourceHashRegistry,
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
            "tools": tools_list
        }),
        "tools/call" => {
            let empty_params = json!({});
            let params = req.get("params").unwrap_or(&empty_params);
            let tool_name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
            let empty_args = json!({});
            let args = params.get("arguments").unwrap_or(&empty_args);
            handle_tool(runtime, registry, tool_name, args, profile)
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
fn tool_text(value: serde_json::Value) -> serde_json::Value {
    json!({"content": [{"type": "text", "text": serde_json::to_string(&value).unwrap()}]})
}

#[cfg(feature = "public_stub")]
fn contract_rejection(value: serde_json::Value) -> serde_json::Value {
    tool_text(value)
}

#[cfg(feature = "public_stub")]
fn validate_decision_contract(
    args: &serde_json::Value,
    registry: &ResourceHashRegistry,
) -> Option<serde_json::Value> {
    let submitted_hash = args
        .get("state_snapshot_hash")
        .and_then(|v| v.as_str())
        .and_then(normalize_sha256_hash);

    if submitted_hash.is_none() {
        return Some(contract_rejection(json!({
            "decision": "CONTRACT_REJECTED",
            "reason_code": "INCOMPLETE_DECLARATION",
            "policy_engine_invoked": false,
            "missing_fields": ["state_snapshot_hash"],
        })));
    }

    let submitted_hash = submitted_hash.expect("checked above");
    let Some((resource, expected_hash)) = registry.find_match(args) else {
        return None;
    };

    if submitted_hash != expected_hash {
        return Some(contract_rejection(json!({
            "decision": "CONTRACT_REJECTED",
            "reason_code": "STATE_HASH_MISMATCH",
            "policy_engine_invoked": false,
            "resource_matched": resource,
            "submitted_hash_prefix": hash_prefix(&submitted_hash),
            "expected_hash_prefix": hash_prefix(expected_hash),
        })));
    }

    None
}

#[cfg(feature = "public_stub")]
fn u64_arg(args: &serde_json::Value, name: &str) -> Option<u64> {
    args.get(name).and_then(|value| {
        value
            .as_u64()
            .or_else(|| value.as_str().and_then(|s| s.parse::<u64>().ok()))
    })
}

#[cfg(feature = "public_stub")]
fn handle_tool(
    runtime: &mut ServerRuntime,
    registry: &mut ResourceHashRegistry,
    tool: &str,
    args: &serde_json::Value,
    profile: ServerProfile,
) -> serde_json::Value {
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
        "register_resource" => {
            let Some(resource_path) = args.get("resource_path").and_then(|v| v.as_str()) else {
                return json!({"error": {"code": 4000, "message": "resource_path is required"}});
            };
            let Some(expected_hash) = args.get("expected_hash").and_then(|v| v.as_str()) else {
                return json!({"error": {"code": 4000, "message": "expected_hash is required"}});
            };

            match registry.register(resource_path, expected_hash) {
                Ok((resource, hash)) => tool_text(json!({
                    "registered": true,
                    "resource": resource,
                    "baseline_hash": hash_prefix(&hash),
                    "expected_hash": hash,
                })),
                Err(message) => json!({"error": {"code": 4000, "message": message}}),
            }
        }
        "node.create" => {
            let belief = args.get("belief").and_then(|b| b.as_f64()).unwrap_or(0.5);
            let energy = args.get("energy").and_then(|e| e.as_f64()).unwrap_or(100.0);
            let node = runtime.graph_mut().create_node(belief, energy);
            json!({"content": [{"type": "text", "text": serde_json::to_string(&node).unwrap()}]})
        }
        "node.query" => {
            let id = u64_arg(args, "node_id").unwrap_or(0);
            match runtime.graph().query_node(id) {
                Some(node) => {
                    json!({"content": [{"type": "text", "text": serde_json::to_string(&node).unwrap()}]})
                }
                None => json!({"error": {"code": 4004, "message": "Node not found"}}),
            }
        }
        "node.mutate" => {
            let id = u64_arg(args, "node_id").unwrap_or(0);
            let delta = args.get("delta").and_then(|d| d.as_f64()).unwrap_or(0.0);
            match runtime.graph_mut().mutate_node(id, delta) {
                Some(node) => {
                    json!({"content": [{"type": "text", "text": serde_json::to_string(&node).unwrap()}]})
                }
                None => json!({"error": {"code": 4004, "message": "Node not found"}}),
            }
        }
        "edge.bind" => {
            let src = u64_arg(args, "src").unwrap_or(0);
            let dst = u64_arg(args, "dst").unwrap_or(0);
            let weight = args.get("weight").and_then(|w| w.as_f64()).unwrap_or(0.5);
            match runtime.graph_mut().bind_edge(src, dst, weight) {
                Some(edge) => {
                    json!({"content": [{"type": "text", "text": serde_json::to_string(&edge).unwrap()}]})
                }
                None => json!({"error": {"code": 4004, "message": "Node not found"}}),
            }
        }
        "edge.propagate" => {
            let msg = runtime.graph_mut().propagate();
            json!({"content": [{"type": "text", "text": msg}]})
        }
        "governor.status" | "governance.status" | "governor.health" | "governance.health" => {
            let status = runtime.graph().governor_status();
            json!({"content": [{"type": "text", "text": serde_json::to_string(&status).unwrap()}]})
        }
        "esv.audit" | "audit.export" => {
            let id = u64_arg(args, "node_id").unwrap_or(0);
            match runtime.graph().esv_audit(id) {
                Some(audit) => {
                    json!({"content": [{"type": "text", "text": serde_json::to_string(&audit).unwrap()}]})
                }
                None => json!({"error": {"code": 4004, "message": "Node not found"}}),
            }
        }
        "lineage.replay" | "audit.replay" => {
            let lineage = match runtime {
                ServerRuntime::ScgBacked(runtime) => runtime.replay_decisions(),
                _ => runtime.graph().lineage_replay(),
            };
            json!({"content": [{"type": "text", "text": serde_json::to_string(&lineage).unwrap()}]})
        }
        "governance.evaluate" | "decision.check" => {
            if let Some(rejection) = validate_decision_contract(args, registry) {
                return rejection;
            }

            let proposal = parse_governance_proposal(args);

            match runtime.evaluate(&proposal) {
                Ok(outcome) => {
                    json!({"content": [{"type": "text", "text": serde_json::to_string(&outcome).unwrap()}]})
                }
                Err(e) => {
                    json!({"error": {"code": 1001, "message": e.to_string(), "data": serde_json::to_value(&e).ok()}})
                }
            }
        }
        "decision.preview" => {
            let proposal = parse_governance_proposal(args);
            match runtime.preview(&proposal) {
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
            if let ServerRuntime::ScgBacked(_) = runtime {
                let unsupported =
                    iter_mcp_server::governance_connector::ScgRuntime::unsupported_audit_filters(
                        &filter,
                    );
                if !unsupported.is_empty() {
                    return json!({
                        "error": {
                            "code": 5002,
                            "message": format!(
                                "audit.search does not support filters in scg-backed mode: {}",
                                unsupported.join(", ")
                            ),
                            "data": {
                                "unsupported_filters": unsupported,
                            }
                        }
                    });
                }
            }
            let result = runtime.search_decisions(&filter);
            json!({"content": [{"type": "text", "text": serde_json::to_string(&result).unwrap()}]})
        }
        _ => json!({"error": {"code": 3000, "message": "Unknown tool"}}),
    }
}
