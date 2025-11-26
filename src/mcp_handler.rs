use crate::scg_core::ScgRuntime;
use crate::types::*;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum ScgError {
    #[error("ESV validation failed")]
    EsvFailed,
    #[error("Thermodynamic drift exceeded")]
    DriftExceeded,
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Bad request: {0}")]
    BadRequest(String),
}

fn esv_guard(new_belief: f64, threshold: f64) -> Result<(), ScgError> {
    if new_belief.abs() > threshold {
        Err(ScgError::EsvFailed)
    } else {
        Ok(())
    }
}

fn drift_guard(runtime: &ScgRuntime) -> Result<(), ScgError> {
    if !runtime.energy_drift_ok() {
        Err(ScgError::DriftExceeded)
    } else {
        Ok(())
    }
}

fn rpc_error_from_scg(id: serde_json::Value, err: ScgError) -> RpcResponse {
    let (code, msg) = match err {
        ScgError::EsvFailed => (1000, "ESV_VALIDATION_FAILED".to_string()),
        ScgError::DriftExceeded => (2000, "THERMODYNAMIC_DRIFT_EXCEEDED".to_string()),
        ScgError::NotFound(m) => (4004, format!("NOT_FOUND: {m}")),
        ScgError::BadRequest(m) => (4000, format!("BAD_REQUEST: {m}")),
    };
    RpcResponse::error(id, code, msg)
}

pub fn handle_rpc(runtime: &ScgRuntime, req: RpcRequest) -> RpcResponse {
    let id = req.id.clone().unwrap_or(serde_json::Value::Null);
    let method = req.method.as_str();
    let params = req.params.clone();

    match method {
        "initialize" => {
            // MCP protocol initialize response
            let response = json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {},
                    "resources": {},
                    "prompts": {}
                },
                "serverInfo": {
                    "name": "scg_mcp_server",
                    "version": "0.1.0"
                }
            });
            RpcResponse::success(id, response)
        }

        "notifications/initialized" => {
            // Client has finished initialization, no response needed
            RpcResponse::success(id, json!({}))
        }

        "resources/list" => {
            // Return empty resources list (we don't provide resources)
            RpcResponse::success(id, json!({ "resources": [] }))
        }

        "prompts/list" => {
            // Return empty prompts list (we don't provide prompts)
            RpcResponse::success(id, json!({ "prompts": [] }))
        }

        "tools/list" | "tools.list" => RpcResponse::success(
            id,
            json!({
                "tools": [
                    {
                        "name": "node.create",
                        "description": "Create SCG node with belief and energy values",
                        "version": "0.1.0",
                        "sideEffects": ["state_mutation", "energy_allocation", "lineage_append"],
                        "dependencies": [],
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
                        "name": "node.mutate",
                        "description": "Mutate node belief by delta",
                        "version": "0.1.0",
                        "sideEffects": ["state_mutation", "esv_validation", "lineage_append"],
                        "dependencies": ["node.query"],
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "node_id": { "type": "string", "description": "Node UUID" },
                                "delta": { "type": "number", "description": "Belief delta" }
                            },
                            "required": ["node_id", "delta"]
                        }
                    },
                    {
                        "name": "node.query",
                        "description": "Query node state by ID",
                        "version": "0.1.0",
                        "sideEffects": [],
                        "dependencies": [],
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "node_id": { "type": "string", "description": "Node UUID" }
                            },
                            "required": ["node_id"]
                        }
                    },
                    {
                        "name": "edge.bind",
                        "description": "Bind edge between two nodes",
                        "version": "0.1.0",
                        "sideEffects": ["state_mutation", "topology_change", "lineage_append"],
                        "dependencies": ["node.query"],
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "src": { "type": "string", "description": "Source node UUID" },
                                "dst": { "type": "string", "description": "Destination node UUID" },
                                "weight": { "type": "number", "description": "Edge weight" }
                            },
                            "required": ["src", "dst", "weight"]
                        }
                    },
                    {
                        "name": "edge.propagate",
                        "description": "Propagate belief along edge",
                        "version": "0.1.0",
                        "sideEffects": ["state_mutation", "energy_transfer", "lineage_append"],
                        "dependencies": ["node.query"],
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "edge_id": { "type": "string", "description": "Edge UUID" }
                            },
                            "required": ["edge_id"]
                        }
                    },
                    {
                        "name": "governor.status",
                        "description": "Query governor drift and coherence status",
                        "version": "0.1.0",
                        "sideEffects": [],
                        "dependencies": [],
                        "inputSchema": {
                            "type": "object",
                            "properties": {}
                        }
                    },
                    {
                        "name": "esv.audit",
                        "description": "Audit node ethical state vector",
                        "version": "0.1.0",
                        "sideEffects": ["esv_validation"],
                        "dependencies": ["node.query"],
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "node_id": { "type": "string", "description": "Node UUID" }
                            },
                            "required": ["node_id"]
                        }
                    },
                    {
                        "name": "lineage.replay",
                        "description": "Replay lineage checksum history",
                        "version": "0.1.0",
                        "sideEffects": [],
                        "dependencies": [],
                        "inputSchema": {
                            "type": "object",
                            "properties": {}
                        }
                    },
                    {
                        "name": "lineage.export",
                        "description": "Export lineage log to file and return checksum",
                        "version": "0.1.0",
                        "sideEffects": ["filesystem_write"],
                        "dependencies": [],
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "path": {
                                    "type": "string",
                                    "description": "Filesystem path to write lineage JSON log"
                                }
                            },
                            "required": ["path"]
                        }
                    }
                ]
            }),
        ),

        "tools/call" => {
            // MCP protocol tools/call - extract tool name and arguments
            #[derive(Deserialize)]
            struct ToolCall {
                name: String,
                arguments: serde_json::Value,
            }
            let call: ToolCall = match serde_json::from_value(params) {
                Ok(v) => v,
                Err(e) => return rpc_error_from_scg(id, ScgError::BadRequest(e.to_string())),
            };

            // Create a new RPC request with the tool method and arguments
            let tool_req = RpcRequest {
                jsonrpc: "2.0".into(),
                method: call.name,
                params: call.arguments,
                id: Some(id.clone()),
            };

            // Recursively handle the tool call
            handle_rpc(runtime, tool_req)
        }

        "node.create" => {
            #[derive(Deserialize)]
            struct P {
                belief: f64,
                energy: f64,
            }
            let p: P = match serde_json::from_value(params) {
                Ok(v) => v,
                Err(e) => return rpc_error_from_scg(id, ScgError::BadRequest(e.to_string())),
            };

            let node = runtime.node_create(p.belief, p.energy);
            RpcResponse::success(
                id,
                json!({
                    "content": [
                        {
                            "type": "text",
                            "text": serde_json::to_string_pretty(&node).unwrap_or_else(|_| format!("{:?}", node))
                        }
                    ]
                }),
            )
        }

        "node.mutate" => {
            #[derive(Deserialize)]
            struct P {
                node_id: String,
                delta: f64,
            }
            let p: P = match serde_json::from_value(params) {
                Ok(v) => v,
                Err(e) => return rpc_error_from_scg(id, ScgError::BadRequest(e.to_string())),
            };
            let uuid = match Uuid::parse_str(&p.node_id) {
                Ok(u) => u,
                Err(e) => return rpc_error_from_scg(id, ScgError::BadRequest(e.to_string())),
            };

            // 1) Fetch current node
            let current = match runtime.node_query(uuid) {
                Some(n) => n,
                None => return rpc_error_from_scg(id, ScgError::NotFound("node".into())),
            };

            // 2) Compute new belief (clamped to [0,1]) and run ESV guard
            let threshold = runtime.get_esv_threshold();
            let new_belief = (current.belief + p.delta).clamp(0.0, 1.0);
            if let Err(e) = esv_guard(new_belief, threshold) {
                return rpc_error_from_scg(id, e); // -> code 1000
            }

            // 3) Drift guard THEN mutate
            if let Err(e) = drift_guard(runtime) {
                return rpc_error_from_scg(id, e);
            }

            let node = match runtime.node_mutate(uuid, p.delta) {
                Ok(n) => n,
                Err(e) => return rpc_error_from_scg(id, ScgError::NotFound(e)),
            };

            RpcResponse::success(
                id,
                json!({
                    "content": [
                        {
                            "type": "text",
                            "text": serde_json::to_string_pretty(&node).unwrap_or_else(|_| format!("{:?}", node))
                        }
                    ]
                }),
            )
        }

        "node.query" => {
            #[derive(Deserialize)]
            struct P {
                node_id: String,
            }
            let p: P = match serde_json::from_value(params) {
                Ok(v) => v,
                Err(e) => return rpc_error_from_scg(id, ScgError::BadRequest(e.to_string())),
            };

            let uuid = match Uuid::parse_str(&p.node_id) {
                Ok(u) => u,
                Err(e) => return rpc_error_from_scg(id, ScgError::BadRequest(e.to_string())),
            };

            let node = match runtime.node_query(uuid) {
                Some(n) => n,
                None => return rpc_error_from_scg(id, ScgError::NotFound("node".into())),
            };

            RpcResponse::success(
                id,
                json!({
                    "content": [
                        {
                            "type": "text",
                            "text": serde_json::to_string_pretty(&node).unwrap_or_else(|_| format!("{:?}", node))
                        }
                    ]
                }),
            )
        }

        "edge.bind" => {
            #[derive(Deserialize)]
            struct P {
                src: String,
                dst: String,
                weight: f64,
            }
            let p: P = match serde_json::from_value(params) {
                Ok(v) => v,
                Err(e) => return rpc_error_from_scg(id, ScgError::BadRequest(e.to_string())),
            };

            let src = match Uuid::parse_str(&p.src) {
                Ok(u) => u,
                Err(e) => return rpc_error_from_scg(id, ScgError::BadRequest(e.to_string())),
            };
            let dst = match Uuid::parse_str(&p.dst) {
                Ok(u) => u,
                Err(e) => return rpc_error_from_scg(id, ScgError::BadRequest(e.to_string())),
            };

            if let Err(e) = drift_guard(runtime) {
                return rpc_error_from_scg(id, e);
            }

            let edge = match runtime.edge_bind(src, dst, p.weight) {
                Ok(e) => e,
                Err(e) => return rpc_error_from_scg(id, ScgError::BadRequest(e)),
            };

            RpcResponse::success(
                id,
                json!({
                    "content": [
                        {
                            "type": "text",
                            "text": serde_json::to_string_pretty(&edge).unwrap_or_else(|_| format!("{:?}", edge))
                        }
                    ]
                }),
            )
        }

        "edge.propagate" => {
            #[derive(Deserialize)]
            struct P {
                edge_id: String,
            }
            let p: P = match serde_json::from_value(params) {
                Ok(v) => v,
                Err(e) => return rpc_error_from_scg(id, ScgError::BadRequest(e.to_string())),
            };

            let uuid = match Uuid::parse_str(&p.edge_id) {
                Ok(u) => u,
                Err(e) => return rpc_error_from_scg(id, ScgError::BadRequest(e.to_string())),
            };

            if let Err(e) = drift_guard(runtime) {
                return rpc_error_from_scg(id, e);
            }

            if let Err(e) = runtime.edge_propagate(uuid) {
                return rpc_error_from_scg(id, ScgError::BadRequest(e));
            }

            RpcResponse::success(
                id,
                json!({
                    "content": [
                        {
                            "type": "text",
                            "text": "Edge propagation successful"
                        }
                    ]
                }),
            )
        }

        "governor.status" => {
            let status = runtime.governor_status();
            RpcResponse::success(
                id,
                json!({
                    "content": [
                        {
                            "type": "text",
                            "text": serde_json::to_string_pretty(&status).unwrap_or_else(|_| format!("{:?}", status))
                        }
                    ]
                }),
            )
        }

        "esv.audit" => {
            #[derive(Deserialize)]
            struct P {
                node_id: String,
            }
            let p: P = match serde_json::from_value(params) {
                Ok(v) => v,
                Err(e) => return rpc_error_from_scg(id, ScgError::BadRequest(e.to_string())),
            };

            let uuid = match Uuid::parse_str(&p.node_id) {
                Ok(u) => u,
                Err(e) => return rpc_error_from_scg(id, ScgError::BadRequest(e.to_string())),
            };

            let ok = match runtime.esv_audit(uuid) {
                Ok(v) => v,
                Err(e) => return rpc_error_from_scg(id, ScgError::NotFound(e)),
            };

            RpcResponse::success(
                id,
                json!({
                    "content": [
                        {
                            "type": "text",
                            "text": format!("ESV audit result: {}", if ok { "VALID" } else { "INVALID" })
                        }
                    ]
                }),
            )
        }

        "lineage.replay" => {
            let entry = runtime.replay_lineage();
            RpcResponse::success(
                id,
                json!({
                    "content": [
                        {
                            "type": "text",
                            "text": serde_json::to_string_pretty(&entry).unwrap_or_else(|_| format!("{:?}", entry))
                        }
                    ]
                }),
            )
        }

        "lineage.export" => {
            #[derive(Deserialize)]
            struct P {
                path: String,
            }
            let p: P = match serde_json::from_value(params) {
                Ok(v) => v,
                Err(e) => return rpc_error_from_scg(id, ScgError::BadRequest(e.to_string())),
            };

            let checksum = match runtime.export_lineage_to_file(&p.path) {
                Ok(h) => h,
                Err(e) => return rpc_error_from_scg(id, ScgError::BadRequest(e)),
            };

            RpcResponse::success(
                id,
                json!({
                    "content": [
                        {
                            "type": "text",
                            "text": serde_json::to_string_pretty(&json!({
                                "status": "ok",
                                "path": p.path,
                                "checksum": checksum
                            })).unwrap_or_default()
                        }
                    ]
                }),
            )
        }

        _ => rpc_error_from_scg(
            id,
            ScgError::BadRequest(format!("Unknown method: {}", method)),
        ),
    }
}
