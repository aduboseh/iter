use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeState {
    pub id: Uuid,
    pub belief: f64,
    pub energy: f64,
    pub esv_valid: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeState {
    pub id: Uuid,
    pub src: Uuid,
    pub dst: Uuid,
    pub weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernorStatus {
    pub energy_drift: f64,
    pub coherence: f64,
    pub node_count: usize,
    pub edge_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageEntry {
    pub id: Uuid,
    pub op: String,
    pub checksum: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
    pub version: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolList {
    pub tools: Vec<ToolInfo>,
}

// JSON-RPC request/response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcRequest {
    pub jsonrpc: String,
    pub method: String,
    #[serde(default)]
    pub params: serde_json::Value,
    #[serde(default)]
    pub id: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcError>,
    pub id: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
}

impl RpcResponse {
    pub fn success(id: serde_json::Value, value: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            result: Some(value),
            error: None,
            id,
        }
    }
    pub fn error(id: serde_json::Value, code: i32, msg: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            result: None,
            error: Some(RpcError { code, message: msg.into() }),
            id,
        }
    }
}
