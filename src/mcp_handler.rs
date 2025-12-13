use crate::substrate_runtime::SubstrateRuntime;
use crate::types::*;
use crate::validation::{validate_belief, validate_energy, validate_node_id, validate_weight};
use serde::Deserialize;
use serde_json::json;

/// Converts McpError to RPC error response.
fn rpc_error(id: serde_json::Value, err: McpError) -> RpcResponse {
    RpcResponse::error(id, err.error_code(), err.to_string())
}

pub fn handle_rpc(runtime: &mut SubstrateRuntime, req: RpcRequest) -> RpcResponse {
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
                    "name": "iter-server",
                    "version": "0.3.0"
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
                        "description": "Create node with belief and energy values",
                        "version": "0.3.0",
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
                        "description": "Mutate node belief by delta (DEBUG operation - bypasses physics)",
                        "version": "0.3.0",
                        "sideEffects": ["state_mutation", "energy_consumption"],
                        "dependencies": ["node.query"],
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
                        "name": "node.query",
                        "description": "Query node state by ID",
                        "version": "0.3.0",
                        "sideEffects": [],
                        "dependencies": [],
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "node_id": { "type": "string", "description": "Node ID (numeric string)" }
                            },
                            "required": ["node_id"]
                        }
                    },
                    {
                        "name": "edge.bind",
                        "description": "Bind edge between two nodes",
                        "version": "0.3.0",
                        "sideEffects": ["state_mutation", "topology_change", "lineage_append"],
                        "dependencies": ["node.query"],
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
                        "description": "Run a simulation step (propagates beliefs along all edges)",
                        "version": "0.3.0",
                        "sideEffects": ["state_mutation", "energy_transfer", "lineage_append"],
                        "dependencies": [],
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "edge_id": { "type": "string", "description": "Edge ID (accepted for compatibility, not used)" }
                            },
                            "required": ["edge_id"]
                        }
                    },
                    {
                        "name": "governor.status",
                        "description": "Query governor drift and coherence status",
                        "version": "0.3.0",
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
                        "version": "0.3.0",
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
                        "version": "0.3.0",
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
                        "version": "0.3.0",
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
                    },
                    {
                        "name": "governance.status",
                        "description": "Query governance health status including checksum validity, drift, and ESV status",
                        "version": "0.3.0",
                        "sideEffects": [],
                        "dependencies": [],
                        "inputSchema": {
                            "type": "object",
                            "properties": {}
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
                Err(e) => return rpc_error(id, McpError::BadRequest { message: e.to_string() }),
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
                Err(e) => return rpc_error(id, McpError::BadRequest { message: e.to_string() }),
            };

            // Validate inputs at MCP boundary before substrate engagement
            if let Err(e) = validate_belief(p.belief) {
                return rpc_error(id, e);
            }
            if let Err(e) = validate_energy(p.energy) {
                return rpc_error(id, e);
            }

            // Delegate to substrate runtime - it handles energy allocation and lineage
            let mcp_node = match runtime.create_node(p.belief, p.energy) {
                Ok(n) => n,
                Err(e) => return rpc_error(id, e),
            };
            
            RpcResponse::success(
                id,
                json!({
                    "content": [
                        {
                            "type": "text",
                            "text": serde_json::to_string_pretty(&mcp_node).unwrap_or_else(|_| format!("{:?}", mcp_node))
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
                Err(e) => return rpc_error(id, McpError::BadRequest { message: e.to_string() }),
            };
            
            // Validate inputs at MCP boundary
            let node_id = match validate_node_id(&p.node_id) {
                Ok(id) => id,
                Err(e) => return rpc_error(id, e),
            };

            // Delegate mutation to substrate - it handles ESV/drift checks internally
            let mcp_node = match runtime.mutate_node(node_id, p.delta) {
                Ok(n) => n,
                Err(e) => return rpc_error(id, e),
            };

            RpcResponse::success(
                id,
                json!({
                    "content": [
                        {
                            "type": "text",
                            "text": serde_json::to_string_pretty(&mcp_node).unwrap_or_else(|_| format!("{:?}", mcp_node))
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
                Err(e) => return rpc_error(id, McpError::BadRequest { message: e.to_string() }),
            };

            // Validate node ID at MCP boundary
            let node_id = match validate_node_id(&p.node_id) {
                Ok(id) => id,
                Err(e) => return rpc_error(id, e),
            };

            let mcp_node = match runtime.query_node(node_id) {
                Ok(n) => n,
                Err(e) => return rpc_error(id, e),
            };

            RpcResponse::success(
                id,
                json!({
                    "content": [
                        {
                            "type": "text",
                            "text": serde_json::to_string_pretty(&mcp_node).unwrap_or_else(|_| format!("{:?}", mcp_node))
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
                Err(e) => return rpc_error(id, McpError::BadRequest { message: e.to_string() }),
            };

            // Validate inputs at MCP boundary
            let src = match validate_node_id(&p.src) {
                Ok(id) => id,
                Err(e) => return rpc_error(id, e),
            };
            let dst = match validate_node_id(&p.dst) {
                Ok(id) => id,
                Err(e) => return rpc_error(id, e),
            };
            if let Err(e) = validate_weight(p.weight) {
                return rpc_error(id, e);
            }

            // Delegate to substrate - it handles drift checks via governance
            let mcp_edge = match runtime.create_edge(src, dst, p.weight) {
                Ok(e) => e,
                Err(e) => return rpc_error(id, e),
            };

            RpcResponse::success(
                id,
                json!({
                    "content": [
                        {
                            "type": "text",
                            "text": serde_json::to_string_pretty(&mcp_edge).unwrap_or_else(|_| format!("{:?}", mcp_edge))
                        }
                    ]
                }),
            )
        }

        "edge.propagate" => {
            // In the real substrate, propagation happens during simulation steps.
            // This tool now triggers a single simulation step.
            // Note: The edge_id parameter is accepted but not used - all edges propagate.
            #[derive(Deserialize)]
            struct P {
                #[allow(dead_code)]
                edge_id: String,
            }
            let _p: P = match serde_json::from_value(params) {
                Ok(v) => v,
                Err(e) => return rpc_error(id, McpError::BadRequest { message: e.to_string() }),
            };

            // Run a simulation step - this propagates beliefs along all edges
            match runtime.step() {
                Ok(()) => (),
                Err(e) => return rpc_error(id, e),
            };

            RpcResponse::success(
                id,
                json!({
                    "content": [
                        {
                            "type": "text",
                            "text": "Propagation step completed (all edges processed)"
                        }
                    ]
                }),
            )
        }

        "governor.status" => {
            let status = match runtime.governance_status() {
                Ok(s) => s,
                Err(e) => return rpc_error(id, e),
            };
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
            // ESV audit in the substrate model checks energy conservation.
            // The node_id is accepted for API compatibility but the check is global.
            #[derive(Deserialize)]
            struct P {
                #[allow(dead_code)]
                node_id: String,
            }
            let _p: P = match serde_json::from_value(params) {
                Ok(v) => v,
                Err(e) => return rpc_error(id, McpError::BadRequest { message: e.to_string() }),
            };

            // Check energy conservation (ESV in substrate terms)
            let ok = runtime.check_energy_conservation(1e-6).is_ok();

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
            // Get recent lineage entries from the substrate's causal trace
            let entries = runtime.lineage_recent(100);
            RpcResponse::success(
                id,
                json!({
                    "content": [
                        {
                            "type": "text",
                            "text": serde_json::to_string_pretty(&entries).unwrap_or_else(|_| format!("{:?}", entries))
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
                Err(e) => return rpc_error(id, McpError::BadRequest { message: e.to_string() }),
            };

            // Export lineage to file
            let entries = runtime.lineage_all();
            let json_str = match serde_json::to_string_pretty(&entries) {
                Ok(s) => s,
                Err(e) => return rpc_error(id, McpError::SubstrateError { message: format!("JSON serialization failed: {}", e) }),
            };
            
            // Write to file
            if let Err(e) = std::fs::write(&p.path, &json_str) {
                return rpc_error(id, McpError::SubstrateError { message: format!("File write failed: {}", e) });
            }
            
            // Compute simple checksum (sum of bytes mod 2^32)
            let checksum: u32 = json_str.bytes().fold(0u32, |acc, b| acc.wrapping_add(b as u32));

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

        "governance.status" => {
            // Delegate entirely to substrate's governance status
            let status = match runtime.governance_status() {
                Ok(s) => s,
                Err(e) => return rpc_error(id, e),
            };
            
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

        _ => rpc_error(
            id,
            McpError::BadRequest { message: format!("Unknown method: {}", method) },
        ),
    }
}
