mod types;
mod scg_core;
mod mcp_handler;

use crate::mcp_handler::handle_rpc;
use crate::scg_core::ScgRuntime;
use crate::types::{RpcRequest, RpcResponse};
use serde_json::json;
use std::io::{BufRead, BufReader, Write};

fn main() {
    run_stdio_server();
}

fn run_stdio_server() {
    let runtime = ScgRuntime::new();
    let stdin = std::io::stdin();
    let mut reader = BufReader::new(stdin.lock());
    let mut stdout = std::io::stdout();

    eprintln!("SCG MCP server running in STDIO mode");

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
                        let resp = handle_rpc(&runtime, req);
                        if let Ok(json) = serde_json::to_string(&resp) {
                            let _ = writeln!(stdout, "{}", json);
                            let _ = stdout.flush();
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
                            let _ = writeln!(stdout, "{}", json);
                            let _ = stdout.flush();
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
