use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};

struct McpTestClient {
    server: Child,
    stdin: std::process::ChildStdin,
    reader: BufReader<std::process::ChildStdout>,
    next_id: u64,
}

impl McpTestClient {
    fn spawn_kernel_debug() -> Self {
        let bin_path = env!("CARGO_BIN_EXE_iter-server");
        let mut server = Command::new(bin_path)
            .arg("--json-only")
            .arg("--runtime-mode=demo")
            .arg("--profile=kernel-debug")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn iter-server");

        let stdin = server.stdin.take().expect("stdin");
        let stdout = server.stdout.take().expect("stdout");
        let reader = BufReader::new(stdout);

        let mut client = Self {
            server,
            stdin,
            reader,
            next_id: 1,
        };
        let _ = client.call("initialize", json!({}));
        client
    }

    fn call(&mut self, method: &str, params: Value) -> Value {
        let req = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": self.next_id
        });
        self.next_id += 1;

        writeln!(self.stdin, "{}", req).expect("write");
        self.stdin.flush().expect("flush");

        let mut line = String::new();
        self.reader.read_line(&mut line).expect("read");
        serde_json::from_str(&line).expect("response JSON")
    }

    fn extract_tool_text(&mut self, tool_name: &str, arguments: Value) -> Value {
        let resp = self.call(
            "tools/call",
            json!({
                "name": tool_name,
                "arguments": arguments
            }),
        );
        let text = resp
            .pointer("/result/content/0/text")
            .and_then(|t| t.as_str())
            .expect("tool response must contain content[0].text");
        serde_json::from_str(text).expect("tool text must parse as JSON")
    }

    fn close(mut self) {
        drop(self.stdin);
        let _ = self.server.wait();
    }
}

#[test]
fn node_tools_accept_numeric_ids_and_return_sdk_node_contract() {
    let mut client = McpTestClient::spawn_kernel_debug();
    let created = client.extract_tool_text(
        "node.create",
        json!({
            "belief": 0.7,
            "energy": 100.0
        }),
    );

    let id = created.get("id").and_then(|v| v.as_u64()).expect("node id");
    assert_eq!(created.get("stability").and_then(|v| v.as_f64()), Some(1.0));

    let queried = client.extract_tool_text(
        "node.query",
        json!({
            "node_id": id
        }),
    );
    assert_eq!(queried.get("id").and_then(|v| v.as_u64()), Some(id));
    assert_eq!(queried.get("stability").and_then(|v| v.as_f64()), Some(1.0));

    client.close();
}
