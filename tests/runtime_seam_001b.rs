use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;

use governance_bridge::contract::{
    Decision as ScgDecision, GovernanceOutcome as ScgGovernanceOutcome, CONTRACT_VERSION_STR,
};
use governance_bridge::trace::{ExecutionTrace, TraceStep};
use serde_json::{json, Value};

use iter_mcp_server::governance_connector::ScgRuntime;
use iter_mcp_server::runtime::{GovernanceRuntime, GovernanceRuntimeError};
use iter_mcp_server::substrate::stub::GovernanceProposal;

const GOVERNANCE_HASH: &str = include_str!("../governance/governance.hash");

struct McpTestClient {
    server: Child,
    stdin: std::process::ChildStdin,
    reader: BufReader<std::process::ChildStdout>,
    next_id: u64,
}

impl McpTestClient {
    fn spawn_governed_backed(endpoint: &str) -> Self {
        let bin_path = env!("CARGO_BIN_EXE_iter-server");
        let governance_hash_path =
            format!("{}/governance/governance.hash", env!("CARGO_MANIFEST_DIR"));

        let mut cmd = Command::new(bin_path);
        cmd.arg("--json-only")
            .arg("--runtime-mode=scg-backed")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .env("SCG_ENDPOINT", endpoint)
            .env("SCG_GOVERNANCE_HASH_PATH", governance_hash_path);

        let mut server = cmd.spawn().expect("spawn iter-server");
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

    fn extract_tool_json(&mut self, tool_name: &str, arguments: Value) -> Value {
        let resp = self.call(
            "tools/call",
            json!({
                "name": tool_name,
                "arguments": arguments
            }),
        );

        let text = resp
            .pointer("/result/content/0/text")
            .and_then(|v| v.as_str())
            .expect("tool response text");
        serde_json::from_str(text).expect("tool text JSON")
    }

    fn close(mut self) {
        drop(self.stdin);
        let _ = self.server.wait();
    }
}

struct MockHttpResponse {
    status_code: u16,
    body: String,
}

struct MockScgServer {
    endpoint: String,
    expected_requests: usize,
    handle: Option<thread::JoinHandle<()>>,
}

impl MockScgServer {
    fn spawn(responses: Vec<MockHttpResponse>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock SCG");
        let endpoint = format!("http://{}", listener.local_addr().expect("addr"));
        let expected_requests = responses.len();

        let handle = thread::spawn(move || {
            for response in responses {
                let (mut stream, _) = listener.accept().expect("accept");
                stream
                    .set_read_timeout(Some(Duration::from_secs(2)))
                    .expect("read timeout");

                let _ = read_http_request(&mut stream);

                let status_line = match response.status_code {
                    200 => "200 OK",
                    500 => "500 Internal Server Error",
                    503 => "503 Service Unavailable",
                    code => panic!("unsupported status code {}", code),
                };

                let payload = response.body;
                let reply = format!(
                    "HTTP/1.1 {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    status_line,
                    payload.as_bytes().len(),
                    payload
                );

                stream.write_all(reply.as_bytes()).expect("write response");
                stream.flush().expect("flush response");
            }
        });

        Self {
            endpoint,
            expected_requests,
            handle: Some(handle),
        }
    }

    fn endpoint(&self) -> &str {
        &self.endpoint
    }
}

impl Drop for MockScgServer {
    fn drop(&mut self) {
        for _ in 0..self.expected_requests {
            if let Ok(mut stream) =
                std::net::TcpStream::connect(self.endpoint.trim_start_matches("http://"))
            {
                let _ = stream.write_all(
                    b"POST /governance/evaluate HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                );
                let _ = stream.flush();
            }
        }
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

fn read_http_request(stream: &mut std::net::TcpStream) -> Vec<u8> {
    let mut data = Vec::new();
    let mut buf = [0_u8; 1024];
    let mut header_end = None;
    let mut content_length = 0_usize;

    loop {
        match stream.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                data.extend_from_slice(&buf[..n]);
                if header_end.is_none() {
                    header_end = find_header_end(&data);
                    if let Some(end) = header_end {
                        content_length = parse_content_length(&data[..end]);
                    }
                }

                if let Some(end) = header_end {
                    let body_len = data.len().saturating_sub(end + 4);
                    if body_len >= content_length {
                        break;
                    }
                }
            }
            Err(e)
                if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.kind() == std::io::ErrorKind::TimedOut =>
            {
                break;
            }
            Err(e) => panic!("read request: {}", e),
        }
    }

    data
}

fn find_header_end(data: &[u8]) -> Option<usize> {
    data.windows(4).position(|window| window == b"\r\n\r\n")
}

fn parse_content_length(header: &[u8]) -> usize {
    let header = String::from_utf8_lossy(header);
    for line in header.lines() {
        let lower = line.to_ascii_lowercase();
        if let Some(value) = lower.strip_prefix("content-length:") {
            return value.trim().parse().unwrap_or(0);
        }
    }
    0
}

fn proposal_args() -> Value {
    json!({
        "proposal_id": "runtime-seam-001b",
        "state_snapshot_hash": "abc123def456abc123def456abc123def456abc123def456abc123def456abc123",
        "requested_action": "deploy_capsule"
    })
}

fn proposal() -> GovernanceProposal {
    GovernanceProposal {
        proposal_id: "runtime-seam-001b".to_string(),
        state_snapshot_hash: "abc123def456abc123def456abc123def456abc123def456abc123def456abc123"
            .to_string(),
        constraints: json!({}),
        requested_action: "deploy_capsule".to_string(),
        proposal_c14n: None,
        proposal_hash: None,
    }
}

fn make_trace() -> ExecutionTrace {
    ExecutionTrace::from_steps(vec![
        TraceStep {
            region_id: "gateway.snapshot".to_string(),
            operation: "compare_hash".to_string(),
            input_hash: "a".repeat(64),
            output_hash: "b".repeat(64),
        },
        TraceStep {
            region_id: "governance.validator".to_string(),
            operation: "compose_decision".to_string(),
            input_hash: "c".repeat(64),
            output_hash: "d".repeat(64),
        },
    ])
}

fn contract_outcome(
    contract_version: &str,
    decision: ScgDecision,
    governance_hash: &str,
    tamper_replay: bool,
) -> ScgGovernanceOutcome {
    let execution_trace = make_trace();
    let mut outcome = ScgGovernanceOutcome {
        contract_version: contract_version.to_string(),
        decision,
        governance_hash: governance_hash.to_string(),
        execution_trace,
        replay_id: String::new(),
    };
    outcome.replay_id = ScgGovernanceOutcome::compute_replay_id(
        &outcome.contract_version,
        &outcome.decision,
        &outcome.governance_hash,
        &outcome.execution_trace,
    );
    if tamper_replay {
        outcome.replay_id = "tampered-replay-id".to_string();
    }
    outcome
}

fn runtime_for(endpoint: &str, hash: &str) -> ScgRuntime {
    ScgRuntime::connect(endpoint.to_string(), hash.to_string()).expect("connect runtime")
}

#[test]
fn governed_backed_mode_emits_governed_packet() {
    let server = MockScgServer::spawn(vec![MockHttpResponse {
        status_code: 200,
        body: serde_json::to_string(&contract_outcome(
            CONTRACT_VERSION_STR,
            ScgDecision::Allow,
            GOVERNANCE_HASH.trim(),
            false,
        ))
        .expect("serialize outcome"),
    }]);

    let mut client = McpTestClient::spawn_governed_backed(server.endpoint());
    let outcome = client.extract_tool_json("decision.check", proposal_args());
    let packet = outcome.get("packet").expect("packet");

    assert_eq!(
        outcome.get("mode").and_then(|v| v.as_str()),
        Some("governed")
    );
    assert_eq!(
        outcome.get("authoritative_pdp").and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        packet.get("governance_hash").and_then(|v| v.as_str()),
        Some(GOVERNANCE_HASH.trim())
    );
    assert!(
        packet
            .get("execution_trace")
            .and_then(|v| v.as_array())
            .is_some_and(|trace| !trace.is_empty()),
        "scg-backed packet must carry the SCG execution trace"
    );

    let result = client.extract_tool_json("audit.search", json!({ "decision": "ALLOW" }));
    let count = result.get("count").and_then(|v| v.as_u64()).expect("count");
    assert!(
        count >= 1,
        "audit.search must find the SCG-backed ALLOW record"
    );
    let first = result
        .get("results")
        .and_then(|v| v.as_array())
        .and_then(|v| v.first())
        .expect("at least one result");
    assert_eq!(
        first.get("decision").and_then(|v| v.as_str()),
        Some("ALLOW")
    );

    client.close();
}

#[test]
fn governance_endpoint_unavailable_returns_explicit_error() {
    let mut runtime = runtime_for("http://127.0.0.1:1", GOVERNANCE_HASH.trim());
    let err = runtime.evaluate(&proposal()).expect_err("scg unavailable");
    assert!(matches!(err, GovernanceRuntimeError::ScgUnavailable(_)));
}

#[test]
fn replay_trace_is_identical_not_just_output() {
    let body = serde_json::to_string(&contract_outcome(
        CONTRACT_VERSION_STR,
        ScgDecision::Allow,
        GOVERNANCE_HASH.trim(),
        false,
    ))
    .expect("serialize outcome");
    let server = MockScgServer::spawn(vec![
        MockHttpResponse {
            status_code: 200,
            body: body.clone(),
        },
        MockHttpResponse {
            status_code: 200,
            body,
        },
    ]);

    let mut runtime = runtime_for(server.endpoint(), GOVERNANCE_HASH.trim());
    let run1 = runtime.evaluate(&proposal()).expect("first evaluate");
    let run2 = runtime.evaluate(&proposal()).expect("second evaluate");

    assert_eq!(
        serde_json::to_value(run1.packet.as_ref().expect("packet")).expect("packet json"),
        serde_json::to_value(run2.packet.as_ref().expect("packet")).expect("packet json")
    );
    assert_eq!(
        run1.packet.as_ref().expect("packet").execution_trace,
        run2.packet.as_ref().expect("packet").execution_trace
    );
}

#[test]
fn governance_hash_absent_fails_boot() {
    let result = ScgRuntime::connect("http://127.0.0.1:18080".to_string(), String::new());
    assert!(matches!(
        result,
        Err(GovernanceRuntimeError::ConfigMissing(_))
    ));
}

#[test]
fn endpoint_absent_fails_boot() {
    let result = ScgRuntime::connect(String::new(), GOVERNANCE_HASH.trim().to_string());
    assert!(matches!(
        result,
        Err(GovernanceRuntimeError::ConfigMissing(_))
    ));
}

#[test]
fn contract_version_mismatch_fails_closed() {
    let server = MockScgServer::spawn(vec![MockHttpResponse {
        status_code: 200,
        body: serde_json::to_string(&contract_outcome(
            "scg.v0",
            ScgDecision::Allow,
            GOVERNANCE_HASH.trim(),
            false,
        ))
        .expect("serialize outcome"),
    }]);

    let mut runtime = runtime_for(server.endpoint(), GOVERNANCE_HASH.trim());
    let err = runtime.evaluate(&proposal()).expect_err("version mismatch");
    assert!(matches!(
        err,
        GovernanceRuntimeError::ContractVersionMismatch(_)
    ));
}

#[test]
fn governance_hash_mismatch_fails_closed() {
    let server = MockScgServer::spawn(vec![MockHttpResponse {
        status_code: 200,
        body: serde_json::to_string(&contract_outcome(
            CONTRACT_VERSION_STR,
            ScgDecision::Allow,
            &"f".repeat(64),
            false,
        ))
        .expect("serialize outcome"),
    }]);

    let mut runtime = runtime_for(server.endpoint(), GOVERNANCE_HASH.trim());
    let err = runtime.evaluate(&proposal()).expect_err("hash mismatch");
    assert!(matches!(
        err,
        GovernanceRuntimeError::GovernanceHashMismatch(_)
    ));
}

#[test]
fn replay_integrity_violation_fails_closed() {
    let server = MockScgServer::spawn(vec![MockHttpResponse {
        status_code: 200,
        body: serde_json::to_string(&contract_outcome(
            CONTRACT_VERSION_STR,
            ScgDecision::Allow,
            GOVERNANCE_HASH.trim(),
            true,
        ))
        .expect("serialize outcome"),
    }]);

    let mut runtime = runtime_for(server.endpoint(), GOVERNANCE_HASH.trim());
    let err = runtime
        .evaluate(&proposal())
        .expect_err("replay integrity violation");
    assert!(matches!(
        err,
        GovernanceRuntimeError::ReplayIntegrityViolation(_)
    ));
}
