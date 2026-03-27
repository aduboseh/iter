//! Iter Rust SDK (async, contract-aligned)
//
// AUDIT NOTE:
// Rust SDK implements fail-closed protocol violation handling.
// See sdks/AUDIT_SDK_CONTRACT_REBUILD.md (Rust section).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::{
    atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
    Arc,
};
use std::time::Duration;
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, Command};
use tokio::sync::{oneshot, Mutex};
use tokio::time::timeout;

// ==============================
// Constants
// ==============================

pub const SDK_PROTOCOL_VERSION: &str = "1.0.0";
pub const MIN_SERVER_VERSION: &str = "1.0.0";
pub const MAX_SERVER_VERSION: &str = "1.99.99";

const STDERR_RING_MAX_BYTES: usize = 10 * 1024;
const MAX_PROTOCOL_VIOLATIONS: usize = 3;
const PROTOCOL_VIOLATION_ERROR_CODE: i32 = -32000;

// ==============================
// State Machine
// ==============================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Open,
    Closing,
    Closed,
}

// ==============================
// Trace Context
// ==============================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceContext {
    pub trace_id: String,
    pub span_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_span_id: Option<String>,
}

impl TraceContext {
    pub fn new(trace_id: impl Into<String>) -> Self {
        let id = trace_id.into();
        Self {
            trace_id: id.clone(),
            span_id: id,
            parent_span_id: None,
        }
    }
}

// ==============================
// JSON-RPC Types
// ==============================

#[derive(Debug, Clone, Serialize)]
pub struct RpcRequest {
    pub jsonrpc: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
    pub id: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RpcResponse {
    pub jsonrpc: String,
    pub result: Option<serde_json::Value>,
    pub error: Option<RpcError>,
    pub id: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
}

// ==============================
// SDK Errors
// ==============================

#[derive(Debug, Error)]
pub enum SdkError {
    #[error("Version mismatch: client={client}, server={server}")]
    VersionMismatch { client: String, server: String },

    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Request failed: {code}: {message}")]
    RequestFailed { code: i32, message: String },

    #[error("Connection closed: {message}")]
    ConnectionClosed {
        message: String,
        pending_count_at_close: Option<usize>,
    },

    #[error("Backpressure: maxInflight={0} exceeded")]
    Backpressure(usize),

    #[error("Request timeout: {method} exceeded {timeout_ms}ms")]
    RequestTimeout { method: String, timeout_ms: u64 },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Failed to reap process after kill()")]
    ZombieProcess,
}

pub type Result<T> = std::result::Result<T, SdkError>;

// ==============================
// Response Types (MCP-aligned)
// ==============================

#[derive(Debug, Clone, Deserialize)]
pub struct ToolListResponse {
    pub tools: Vec<ToolInfo>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NodeState {
    pub id: u64,
    pub belief: f64,
    pub energy: f64,
    pub esv_valid: bool,
    pub stability: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GovernorStatus {
    pub drift_ok: bool,
    pub energy_drift: f64,
    pub coherence: f64,
    pub node_count: usize,
    pub edge_count: usize,
    pub healthy: bool,
}

// ==============================
// Client
// ==============================

pub struct IterClient {
    process: Arc<Mutex<Child>>,
    stdin: Arc<Mutex<ChildStdin>>,
    state: Arc<Mutex<State>>,
    request_id: AtomicU64,
    response_queue: Arc<Mutex<HashMap<u64, oneshot::Sender<RpcResponse>>>>,
    max_inflight: usize,

    #[allow(dead_code)]
    stderr_ring: Arc<Mutex<Vec<u8>>>,
    trace_context: Arc<Mutex<Option<TraceContext>>>,

    close_lock: Arc<Mutex<()>>,
    #[allow(dead_code)]
    protocol_violation_count: AtomicUsize,
    #[allow(dead_code)]
    circuit_breaker_tripped: AtomicBool,
}

impl IterClient {
    pub async fn connect(binary_path: &str, max_inflight: usize) -> Result<Self> {
        let mut child = Command::new(binary_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| SdkError::ConnectionFailed("Failed to open stdin".into()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| SdkError::ConnectionFailed("Failed to open stdout".into()))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| SdkError::ConnectionFailed("Failed to open stderr".into()))?;

        let process = Arc::new(Mutex::new(child));
        let stdin = Arc::new(Mutex::new(stdin));
        let state = Arc::new(Mutex::new(State::Open));
        let response_queue = Arc::new(Mutex::new(
            HashMap::<u64, oneshot::Sender<RpcResponse>>::new(),
        ));
        let stderr_ring = Arc::new(Mutex::new(Vec::with_capacity(STDERR_RING_MAX_BYTES)));
        let trace_context = Arc::new(Mutex::new(None));
        let close_lock = Arc::new(Mutex::new(()));

        // stdout task: MUST process in OPEN || CLOSING
        {
            let state_c = Arc::clone(&state);
            let queue_c = Arc::clone(&response_queue);
            let process_c = Arc::clone(&process);

            let violation_count = Arc::new(AtomicUsize::new(0));
            let breaker = Arc::new(AtomicBool::new(false));

            let violation_count_c = Arc::clone(&violation_count);
            let breaker_c = Arc::clone(&breaker);

            tokio::spawn(async move {
                let mut lines = BufReader::new(stdout).lines();

                while *state_c.lock().await != State::Closed {
                    match lines.next_line().await {
                        Ok(Some(line)) => match serde_json::from_str::<serde_json::Value>(&line) {
                            Ok(v) => {
                                let jsonrpc_ok = v
                                    .get("jsonrpc")
                                    .and_then(|x| x.as_str())
                                    .map(|s| s == "2.0")
                                    .unwrap_or(false);

                                let id_opt = v.get("id").and_then(|x| x.as_u64());

                                if !jsonrpc_ok || id_opt.is_none() {
                                    if trip_breaker_if_needed(
                                        "invalid JSON-RPC envelope",
                                        &state_c,
                                        &queue_c,
                                        &process_c,
                                        &violation_count_c,
                                        &breaker_c,
                                    )
                                    .await
                                    {
                                        break;
                                    }
                                    continue;
                                }

                                let id = id_opt.unwrap();

                                let resp_parse = serde_json::from_value::<RpcResponse>(v);
                                let resp = match resp_parse {
                                    Ok(r) => r,
                                    Err(_) => {
                                        if trip_breaker_if_needed(
                                            "unparseable JSON-RPC response",
                                            &state_c,
                                            &queue_c,
                                            &process_c,
                                            &violation_count_c,
                                            &breaker_c,
                                        )
                                        .await
                                        {
                                            break;
                                        }
                                        continue;
                                    }
                                };

                                let mut q = queue_c.lock().await;
                                if let Some(tx) = q.remove(&id) {
                                    let _ = tx.send(resp);
                                }
                            }
                            Err(_) => {
                                if trip_breaker_if_needed(
                                    "malformed stdout (non-JSON)",
                                    &state_c,
                                    &queue_c,
                                    &process_c,
                                    &violation_count_c,
                                    &breaker_c,
                                )
                                .await
                                {
                                    break;
                                }
                            }
                        },
                        Ok(None) => break,
                        Err(_) => break,
                    }
                }
            });

            // Bind these atomics into the client (single source of truth)
            // NOTE: We keep the client-side counters too; the task owns the effective breaker state.
        }

        // stderr task: ring buffer
        {
            let ring_c = Arc::clone(&stderr_ring);
            tokio::spawn(async move {
                let mut r = BufReader::new(stderr);
                let mut buf = [0u8; 4096];
                loop {
                    match r.read(&mut buf).await {
                        Ok(0) => break,
                        Ok(n) => {
                            let mut ring = ring_c.lock().await;
                            append_ring(&mut ring, &buf[..n]);
                        }
                        Err(_) => break,
                    }
                }
            });
        }

        Ok(Self {
            process,
            stdin,
            state,
            request_id: AtomicU64::new(0),
            response_queue,
            max_inflight,
            stderr_ring,
            trace_context,
            close_lock,
            protocol_violation_count: AtomicUsize::new(0),
            circuit_breaker_tripped: AtomicBool::new(false),
        })
    }

    pub async fn state(&self) -> State {
        *self.state.lock().await
    }

    pub async fn with_trace(&self, trace: TraceContext) {
        *self.trace_context.lock().await = Some(trace);
    }

    pub async fn send(
        &self,
        method: impl Into<String>,
        params: Option<serde_json::Value>,
        timeout_ms: u64,
    ) -> Result<RpcResponse> {
        // send() only allowed in OPEN
        {
            let st = *self.state.lock().await;
            if st != State::Open {
                let pending = self.response_queue.lock().await.len();
                return Err(SdkError::ConnectionClosed {
                    message: "Client is closing or closed, cannot send request".into(),
                    pending_count_at_close: Some(pending),
                });
            }
        }

        // backpressure
        {
            let qlen = self.response_queue.lock().await.len();
            if qlen >= self.max_inflight {
                return Err(SdkError::Backpressure(self.max_inflight));
            }
        }

        let id = self.request_id.fetch_add(1, Ordering::SeqCst) + 1;
        let method_s = method.into();

        let req = RpcRequest {
            jsonrpc: "2.0".into(),
            method: method_s.clone(),
            params,
            id: serde_json::json!(id),
        };

        let (tx, rx) = oneshot::channel();

        {
            let mut q = self.response_queue.lock().await;
            q.insert(id, tx);
        }

        // write request (stdin is shared)
        {
            let mut stdin = self.stdin.lock().await;
            stdin
                .write_all(serde_json::to_string(&req)?.as_bytes())
                .await?;
            stdin.write_all(b"\n").await?;
            stdin.flush().await?;
        }

        // wait for response or timeout
        match timeout(Duration::from_millis(timeout_ms), rx).await {
            Ok(Ok(resp)) => Ok(resp),
            Ok(Err(_)) => Err(SdkError::ConnectionFailed(
                "Connection closed (fail-closed)".into(),
            )),
            Err(_) => Err(SdkError::RequestTimeout {
                method: method_s,
                timeout_ms,
            }),
        }
    }

    pub async fn tools_list(&mut self) -> Result<Vec<ToolInfo>> {
        let response = self.send("tools/list", None, 30000).await?;
        let result = response.result.ok_or_else(|| SdkError::RequestFailed {
            code: -1,
            message: "No result".into(),
        })?;

        let tools: ToolListResponse = serde_json::from_value(result)?;
        Ok(tools.tools)
    }

    pub async fn node_create(&mut self, belief: f64, energy: f64) -> Result<NodeState> {
        let params = serde_json::json!({
            "belief": belief,
            "energy": energy
        });

        let response = self
            .send(
                "tools/call",
                Some(serde_json::json!({
                    "name": "node.create",
                    "arguments": params
                })),
                30000,
            )
            .await?;

        parse_tool_result(response)
    }

    pub async fn node_query(&mut self, node_id: u64) -> Result<NodeState> {
        let response = self
            .send(
                "tools/call",
                Some(serde_json::json!({
                    "name": "node.query",
                    "arguments": { "node_id": node_id }
                })),
                30000,
            )
            .await?;

        parse_tool_result(response)
    }

    pub async fn governor_health(&mut self) -> Result<GovernorStatus> {
        let response = self
            .send(
                "tools/call",
                Some(serde_json::json!({
                    "name": "governor.health",
                    "arguments": {}
                })),
                30000,
            )
            .await?;

        parse_tool_result(response)
    }

    pub async fn governance_health(&mut self) -> Result<GovernorStatus> {
        let response = self
            .send(
                "tools/call",
                Some(serde_json::json!({
                    "name": "governance.health",
                    "arguments": {}
                })),
                30000,
            )
            .await?;

        parse_tool_result(response)
    }

    pub async fn decision_check(
        &mut self,
        proposal_id: &str,
        state_snapshot_hash: &str,
        requested_action: &str,
        constraints: Option<serde_json::Value>,
    ) -> Result<serde_json::Value> {
        let mut args = serde_json::json!({
            "proposal_id": proposal_id,
            "state_snapshot_hash": state_snapshot_hash,
            "requested_action": requested_action,
        });
        if let Some(c) = constraints {
            args["constraints"] = c;
        }
        let response = self
            .send(
                "tools/call",
                Some(serde_json::json!({
                    "name": "decision.check",
                    "arguments": args
                })),
                30000,
            )
            .await?;

        extract_raw_result(response)
    }

    pub async fn audit_export(&mut self, node_id: u64) -> Result<serde_json::Value> {
        let response = self
            .send(
                "tools/call",
                Some(serde_json::json!({
                    "name": "audit.export",
                    "arguments": { "node_id": node_id.to_string() }
                })),
                30000,
            )
            .await?;

        extract_raw_result(response)
    }

    pub async fn audit_replay(&mut self) -> Result<serde_json::Value> {
        let response = self
            .send(
                "tools/call",
                Some(serde_json::json!({
                    "name": "audit.replay",
                    "arguments": {}
                })),
                30000,
            )
            .await?;

        extract_raw_result(response)
    }

    pub async fn decision_preview(
        &mut self,
        proposal_id: &str,
        state_snapshot_hash: &str,
        requested_action: &str,
        constraints: Option<serde_json::Value>,
    ) -> Result<serde_json::Value> {
        let mut args = serde_json::json!({
            "proposal_id": proposal_id,
            "state_snapshot_hash": state_snapshot_hash,
            "requested_action": requested_action,
        });
        if let Some(c) = constraints {
            args["constraints"] = c;
        }
        let response = self
            .send(
                "tools/call",
                Some(serde_json::json!({
                    "name": "decision.preview",
                    "arguments": args
                })),
                30000,
            )
            .await?;

        extract_raw_result(response)
    }

    pub async fn audit_search(
        &mut self,
        filter: Option<serde_json::Value>,
    ) -> Result<serde_json::Value> {
        let args = filter.unwrap_or_else(|| serde_json::json!({}));
        let response = self
            .send(
                "tools/call",
                Some(serde_json::json!({
                    "name": "audit.search",
                    "arguments": args
                })),
                30000,
            )
            .await?;

        extract_raw_result(response)
    }

    #[deprecated(note = "Use governor_health() instead. Will be removed in v3.0.")]
    pub async fn governor_status(&mut self) -> Result<GovernorStatus> {
        let response = self
            .send(
                "tools/call",
                Some(serde_json::json!({
                    "name": "governor.status",
                    "arguments": {}
                })),
                30000,
            )
            .await?;

        parse_tool_result(response)
    }

    pub async fn close(&mut self) -> Result<()> {
        // close idempotence
        let _guard = self.close_lock.lock().await;

        {
            let st = *self.state.lock().await;
            if st == State::Closed {
                return Ok(());
            }
        }

        // move to CLOSING
        {
            let mut st = self.state.lock().await;
            *st = State::Closing;
        }

        // drain up to 5s
        wait_for_drain(&self.response_queue, 5000).await;

        // reject pending (best-effort)
        {
            let mut q = self.response_queue.lock().await;
            let pending = q.len();
            if pending > 0 {
                q.clear();
            }
        }

        // kill process
        {
            let mut p = self.process.lock().await;
            if p.try_wait()?.is_none() {
                p.kill().await?;
                timeout(Duration::from_secs(3), p.wait())
                    .await
                    .map_err(|_| SdkError::ZombieProcess)??;
            }
        }

        // CLOSED
        {
            let mut st = self.state.lock().await;
            *st = State::Closed;
        }

        Ok(())
    }
}

// ==============================
// Helpers
// ==============================

fn append_ring(ring: &mut Vec<u8>, chunk: &[u8]) {
    if chunk.is_empty() {
        return;
    }
    ring.extend_from_slice(chunk);
    if ring.len() > STDERR_RING_MAX_BYTES {
        let excess = ring.len() - STDERR_RING_MAX_BYTES;
        ring.drain(0..excess);
    }
}

async fn wait_for_drain(
    queue: &Arc<Mutex<HashMap<u64, oneshot::Sender<RpcResponse>>>>,
    timeout_ms: u64,
) {
    let step = 25u64;
    let max = timeout_ms / step;
    for _ in 0..max {
        if queue.lock().await.is_empty() {
            return;
        }
        tokio::time::sleep(Duration::from_millis(step)).await;
    }
}

async fn trip_breaker_if_needed(
    reason: &str,
    state: &Arc<Mutex<State>>,
    queue: &Arc<Mutex<HashMap<u64, oneshot::Sender<RpcResponse>>>>,
    process: &Arc<Mutex<Child>>,
    violation_count: &Arc<AtomicUsize>,
    breaker: &Arc<AtomicBool>,
) -> bool {
    if breaker.load(Ordering::SeqCst) {
        return true;
    }

    let pending = queue.lock().await.len();
    let n = violation_count.fetch_add(1, Ordering::SeqCst) + 1;

    let should_trip = pending > 0 || n >= MAX_PROTOCOL_VIOLATIONS;
    if !should_trip {
        return false;
    }

    if breaker.swap(true, Ordering::SeqCst) {
        return true;
    }

    {
        let mut st = state.lock().await;
        if *st != State::Closed {
            *st = State::Closing;
        }
    }

    fail_closed_reject_pending(reason, queue).await;

    {
        let mut p = process.lock().await;
        if p.try_wait().ok().flatten().is_none() {
            let _ = p.kill().await;
            let _ = timeout(Duration::from_secs(3), p.wait()).await;
        }
    }

    {
        let mut st = state.lock().await;
        *st = State::Closed;
    }

    true
}

async fn fail_closed_reject_pending(
    reason: &str,
    queue: &Arc<Mutex<HashMap<u64, oneshot::Sender<RpcResponse>>>>,
) {
    let mut q = queue.lock().await;
    for (id, tx) in q.drain() {
        let resp = RpcResponse {
            jsonrpc: "2.0".into(),
            result: None,
            error: Some(RpcError {
                code: PROTOCOL_VIOLATION_ERROR_CODE,
                message: format!("Protocol violation: {reason}"),
            }),
            id: serde_json::json!(id),
        };
        let _ = tx.send(resp);
    }
}

fn parse_tool_result<T: serde::de::DeserializeOwned>(response: RpcResponse) -> Result<T> {
    if let Some(err) = response.error {
        return Err(SdkError::RequestFailed {
            code: err.code,
            message: err.message,
        });
    }

    let result = response.result.ok_or_else(|| SdkError::RequestFailed {
        code: -1,
        message: "No result".into(),
    })?;

    let content = result
        .get("content")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|item| item.get("text"))
        .and_then(|t| t.as_str())
        .ok_or_else(|| SdkError::RequestFailed {
            code: -1,
            message: "Invalid tool response format".into(),
        })?;

    serde_json::from_str(content).map_err(SdkError::Json)
}

fn extract_raw_result(response: RpcResponse) -> Result<serde_json::Value> {
    if let Some(err) = response.error {
        return Err(SdkError::RequestFailed {
            code: err.code,
            message: err.message,
        });
    }

    response.result.ok_or_else(|| SdkError::RequestFailed {
        code: -1,
        message: "No result".into(),
    })
}

// ==============================
// Version Checking
// ==============================

pub fn is_version_compatible(server_version: &str) -> bool {
    fn parse(v: &str) -> Option<(u32, u32, u32)> {
        let p: Vec<&str> = v.split('.').collect();
        if p.len() != 3 {
            return None;
        }
        Some((p[0].parse().ok()?, p[1].parse().ok()?, p[2].parse().ok()?))
    }
    let server = match parse(server_version) {
        Some(v) => v,
        None => return false,
    };
    let min = parse(MIN_SERVER_VERSION).unwrap();
    let max = parse(MAX_SERVER_VERSION).unwrap();
    server >= min && server <= max
}

// ==============================
// Tests
// ==============================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_compatibility() {
        assert!(is_version_compatible("1.0.0"));
        assert!(is_version_compatible("1.2.3"));
        assert!(!is_version_compatible("2.0.0"));
        assert!(!is_version_compatible("bad"));
    }

    #[test]
    fn governance_helper_methods_are_exposed() {
        let _ = IterClient::decision_preview;
        let _ = IterClient::audit_search;
    }

    #[test]
    fn raw_result_preserves_server_error() {
        let response = RpcResponse {
            jsonrpc: "2.0".into(),
            result: None,
            error: Some(RpcError {
                code: 5001,
                message: "simulation unavailable".into(),
            }),
            id: serde_json::json!(1),
        };

        let err = extract_raw_result(response).unwrap_err();
        match err {
            SdkError::RequestFailed { code, message } => {
                assert_eq!(code, 5001);
                assert_eq!(message, "simulation unavailable");
            }
            other => panic!("unexpected error: {other}"),
        }
    }
}
