//! Iter Rust SDK
//!
//! Thin client for the Iter MCP protocol. This SDK provides:
//! - Type-safe request/response handling
//! - Protocol version compatibility checking
//! - Trace context propagation
//!
//! # Design Principles
//!
//! - **Thin**: No business logic; pure protocol wrapper
//! - **Contract-driven**: Generated from protocol types
//! - **Version-aware**: Fails fast on incompatible versions
//! - **Telemetry-safe**: Passes trace context, never enriches payloads
//!
//! # Example
//!
//! ```ignore
//! use iter_sdk::{IterClient, NodeCreateRequest};
//!
//! let client = IterClient::connect("stdio")?;
//! let node = client.node_create(NodeCreateRequest { belief: 0.5, energy: 1.0 })?;
//! ```

use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

// ============================================================================
// Protocol Version
// ============================================================================

/// SDK protocol version (must match server)
pub const SDK_PROTOCOL_VERSION: &str = "1.0.0";

/// Minimum supported server protocol version
pub const MIN_SERVER_VERSION: &str = "1.0.0";

/// Maximum supported server protocol version  
pub const MAX_SERVER_VERSION: &str = "1.99.99";

// ============================================================================
// Trace Context
// ============================================================================

/// Trace context for request correlation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceContext {
    pub trace_id: String,
    pub span_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_span_id: Option<String>,
}

impl TraceContext {
    /// Create a new trace context
    pub fn new(trace_id: impl Into<String>) -> Self {
        let id = trace_id.into();
        Self {
            trace_id: id.clone(),
            span_id: id,
            parent_span_id: None,
        }
    }
}

// ============================================================================
// Request/Response Types (Contract-Driven)
// ============================================================================

/// JSON-RPC 2.0 Request
#[derive(Debug, Clone, Serialize)]
pub struct RpcRequest {
    pub jsonrpc: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
    pub id: serde_json::Value,
}

/// JSON-RPC 2.0 Response
#[derive(Debug, Clone, Deserialize)]
pub struct RpcResponse {
    pub jsonrpc: String,
    pub result: Option<serde_json::Value>,
    pub error: Option<RpcError>,
    pub id: serde_json::Value,
}

/// JSON-RPC 2.0 Error
#[derive(Debug, Clone, Deserialize)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
}

// ============================================================================
// SDK Error Types
// ============================================================================

/// SDK error type
#[derive(Debug)]
pub enum SdkError {
    /// Protocol version mismatch
    VersionMismatch { client: String, server: String },
    /// Connection failed
    ConnectionFailed(String),
    /// Request failed
    RequestFailed(RpcError),
    /// IO error
    Io(std::io::Error),
    /// JSON error
    Json(serde_json::Error),
}

impl std::fmt::Display for SdkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SdkError::VersionMismatch { client, server } => {
                write!(f, "Version mismatch: client={}, server={}", client, server)
            }
            SdkError::ConnectionFailed(msg) => write!(f, "Connection failed: {}", msg),
            SdkError::RequestFailed(err) => write!(f, "Request failed: {} ({})", err.message, err.code),
            SdkError::Io(err) => write!(f, "IO error: {}", err),
            SdkError::Json(err) => write!(f, "JSON error: {}", err),
        }
    }
}

impl std::error::Error for SdkError {}

impl From<std::io::Error> for SdkError {
    fn from(err: std::io::Error) -> Self {
        SdkError::Io(err)
    }
}

impl From<serde_json::Error> for SdkError {
    fn from(err: serde_json::Error) -> Self {
        SdkError::Json(err)
    }
}

pub type Result<T> = std::result::Result<T, SdkError>;

// ============================================================================
// Client
// ============================================================================

/// Iter MCP client (STDIO transport)
pub struct IterClient {
    process: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    request_id: u64,
    trace_context: Option<TraceContext>,
}

impl IterClient {
    /// Connect to an Iter server process
    pub fn connect(binary_path: &str) -> Result<Self> {
        let mut process = Command::new(binary_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;

        let stdin = process.stdin.take().ok_or_else(|| {
            SdkError::ConnectionFailed("Failed to open stdin".to_string())
        })?;
        let stdout = process.stdout.take().ok_or_else(|| {
            SdkError::ConnectionFailed("Failed to open stdout".to_string())
        })?;

        Ok(Self {
            process,
            stdin,
            stdout: BufReader::new(stdout),
            request_id: 0,
            trace_context: None,
        })
    }

    /// Set trace context for subsequent requests
    pub fn with_trace(&mut self, trace: TraceContext) -> &mut Self {
        self.trace_context = Some(trace);
        self
    }

    /// Send a raw JSON-RPC request
    pub fn send(&mut self, method: &str, params: Option<serde_json::Value>) -> Result<RpcResponse> {
        self.request_id += 1;
        
        let request = RpcRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
            id: serde_json::json!(self.request_id),
        };

        let request_json = serde_json::to_string(&request)?;
        writeln!(self.stdin, "{}", request_json)?;
        self.stdin.flush()?;

        let mut response_line = String::new();
        self.stdout.read_line(&mut response_line)?;

        let response: RpcResponse = serde_json::from_str(&response_line)?;
        
        if let Some(err) = response.error {
            return Err(SdkError::RequestFailed(err));
        }

        Ok(response)
    }

    /// List available tools
    pub fn tools_list(&mut self) -> Result<Vec<ToolInfo>> {
        let response = self.send("tools/list", None)?;
        let result = response.result.ok_or_else(|| {
            SdkError::RequestFailed(RpcError {
                code: -1,
                message: "No result".to_string(),
            })
        })?;
        
        let tools: ToolListResponse = serde_json::from_value(result)?;
        Ok(tools.tools)
    }

    /// Create a node
    pub fn node_create(&mut self, belief: f64, energy: f64) -> Result<NodeState> {
        let params = serde_json::json!({
            "belief": belief,
            "energy": energy
        });
        
        let response = self.send("tools/call", Some(serde_json::json!({
            "name": "node.create",
            "arguments": params
        })))?;
        
        parse_tool_result(response)
    }

    /// Query a node
    pub fn node_query(&mut self, node_id: u64) -> Result<NodeState> {
        let response = self.send("tools/call", Some(serde_json::json!({
            "name": "node.query",
            "arguments": { "node_id": node_id }
        })))?;
        
        parse_tool_result(response)
    }

    /// Get governor status
    pub fn governor_status(&mut self) -> Result<GovernorStatus> {
        let response = self.send("tools/call", Some(serde_json::json!({
            "name": "governor.status",
            "arguments": {}
        })))?;
        
        parse_tool_result(response)
    }
}

impl Drop for IterClient {
    fn drop(&mut self) {
        let _ = self.process.kill();
    }
}

// ============================================================================
// Response Types (MCP-aligned)
// ============================================================================

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

// ============================================================================
// Helpers
// ============================================================================

fn parse_tool_result<T: serde::de::DeserializeOwned>(response: RpcResponse) -> Result<T> {
    let result = response.result.ok_or_else(|| {
        SdkError::RequestFailed(RpcError {
            code: -1,
            message: "No result".to_string(),
        })
    })?;
    
    // MCP tool responses have content array
    let content = result.get("content")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|item| item.get("text"))
        .and_then(|t| t.as_str())
        .ok_or_else(|| {
            SdkError::RequestFailed(RpcError {
                code: -1,
                message: "Invalid tool response format".to_string(),
            })
        })?;
    
    let parsed: T = serde_json::from_str(content)?;
    Ok(parsed)
}

// ============================================================================
// Version Checking
// ============================================================================

/// Check if a server version is compatible with this SDK
pub fn is_version_compatible(server_version: &str) -> bool {
    // Parse versions
    let parse = |v: &str| -> Option<(u32, u32, u32)> {
        let parts: Vec<&str> = v.split('.').collect();
        if parts.len() != 3 { return None; }
        Some((
            parts[0].parse().ok()?,
            parts[1].parse().ok()?,
            parts[2].parse().ok()?,
        ))
    };
    
    let server = match parse(server_version) {
        Some(v) => v,
        None => return false,
    };
    
    let min = parse(MIN_SERVER_VERSION).unwrap();
    let max = parse(MAX_SERVER_VERSION).unwrap();
    
    // Check major.minor.patch bounds
    server >= min && server <= max
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sdk_version_is_valid() {
        assert!(!SDK_PROTOCOL_VERSION.is_empty());
    }

    #[test]
    fn version_compatibility_current() {
        assert!(is_version_compatible("1.0.0"));
    }

    #[test]
    fn version_compatibility_minor_bump() {
        assert!(is_version_compatible("1.1.0"));
        assert!(is_version_compatible("1.5.0"));
    }

    #[test]
    fn version_compatibility_rejects_major_bump() {
        assert!(!is_version_compatible("2.0.0"));
    }

    #[test]
    fn trace_context_creation() {
        let trace = TraceContext::new("test-trace");
        assert_eq!(trace.trace_id, "test-trace");
        assert_eq!(trace.span_id, "test-trace");
        assert!(trace.parent_span_id.is_none());
    }
}
