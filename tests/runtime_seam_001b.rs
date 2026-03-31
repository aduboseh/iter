use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use governance_bridge::contract::{
    Decision as ScgDecision, GovernanceOutcome as ScgGovernanceOutcome, GovernanceRequest,
    CONTRACT_VERSION_STR,
};
use governance_bridge::trace::{ExecutionTrace, OperationType, TraceStep};
use serde_json::{json, Value};

use iter_mcp_server::economics::EconomicsConfig;
use iter_mcp_server::governance_connector::ScgRuntime;
use iter_mcp_server::governed::GovernedRuntime;
use iter_mcp_server::policy::PolicyConfig;
use iter_mcp_server::runtime::{GovernanceRuntime, GovernanceRuntimeError};
use iter_mcp_server::substrate::stub::{
    AuditSearchFilter, GovernanceProposal, ReplayResult, ReplayStatus, StubRuntime,
};

const GOVERNANCE_HASH: &str = include_str!("../governance/governance.hash");

fn seam_guard() -> std::sync::MutexGuard<'static, ()> {
    static SEAM_LOCK: Mutex<()> = Mutex::new(());
    SEAM_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

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

    fn close(mut self) {
        drop(self.stdin);
        let _ = self.server.wait();
    }
}

fn tool_payload(response: &Value) -> Value {
    let text = response
        .pointer("/result/content/0/text")
        .and_then(|value| value.as_str())
        .expect("tool response text");
    serde_json::from_str(text).expect("tool payload json")
}

struct MockHttpResponse {
    status_code: u16,
    body: String,
}

struct MockScgServer {
    endpoint: String,
    expected_requests: usize,
    failures: Arc<Mutex<Vec<String>>>,
    received_requests: Arc<AtomicUsize>,
    shutdown: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}

impl MockScgServer {
    fn spawn(responses: Vec<MockHttpResponse>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock SCG");
        let endpoint = format!("http://{}", listener.local_addr().expect("addr"));
        let expected_requests = responses.len();
        let failures = Arc::new(Mutex::new(Vec::new()));
        let received_requests = Arc::new(AtomicUsize::new(0));
        let shutdown = Arc::new(AtomicBool::new(false));
        let expected_request = expected_request();
        let thread_failures = Arc::clone(&failures);
        let thread_received_requests = Arc::clone(&received_requests);
        let thread_shutdown = Arc::clone(&shutdown);

        let handle = thread::spawn(move || {
            for response in responses {
                let (mut stream, _) = loop {
                    match listener.accept() {
                        Ok(connection) => break connection,
                        Err(err) => {
                            thread_failures
                                .lock()
                                .expect("mock failures")
                                .push(format!("accept failed: {}", err));
                            return;
                        }
                    }
                };
                if thread_shutdown.load(Ordering::SeqCst) {
                    return;
                }
                stream
                    .set_read_timeout(Some(Duration::from_secs(2)))
                    .expect("read timeout");

                let request = read_http_request(&mut stream);
                if request.is_empty() || find_header_end(&request).is_none() {
                    if thread_shutdown.load(Ordering::SeqCst) {
                        return;
                    }
                    continue;
                }
                if let Err(err) = assert_expected_request(&request, &expected_request) {
                    thread_failures.lock().expect("mock failures").push(err);
                    return;
                }
                thread_received_requests.fetch_add(1, Ordering::SeqCst);

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
            failures,
            received_requests,
            shutdown,
            handle: Some(handle),
        }
    }

    fn endpoint(&self) -> &str {
        &self.endpoint
    }

    fn finish(mut self) {
        self.shutdown.store(true, Ordering::SeqCst);
        let _ = std::net::TcpStream::connect(self.endpoint.trim_start_matches("http://"));
        if let Some(handle) = self.handle.take() {
            handle.join().expect("join mock governance server");
        }

        let failures = self.failures.lock().expect("mock failures");
        assert!(
            failures.is_empty(),
            "mock governance server validation failed: {}",
            failures.join(" | ")
        );
        assert_eq!(
            self.received_requests.load(Ordering::SeqCst),
            self.expected_requests,
            "mock governance server received an unexpected number of requests"
        );
    }
}

impl Drop for MockScgServer {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::SeqCst);
        let _ = std::net::TcpStream::connect(self.endpoint.trim_start_matches("http://"));
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
        let failures = self.failures.lock().expect("mock failures");
        if !failures.is_empty() {
            eprintln!(
                "mock governance server validation failed during drop: {}",
                failures.join(" | ")
            );
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

fn assert_expected_request(request: &[u8], expected: &GovernanceRequest) -> Result<(), String> {
    let header_end = find_header_end(request).ok_or_else(|| "missing HTTP headers".to_string())?;
    let header = String::from_utf8(request[..header_end].to_vec())
        .map_err(|e| format!("invalid request header encoding: {}", e))?;
    let mut lines = header.lines();
    let request_line = lines
        .next()
        .ok_or_else(|| "missing HTTP request line".to_string())?;
    if request_line != "POST /governance/evaluate HTTP/1.1" {
        return Err(format!("unexpected request line: {}", request_line));
    }

    let body = &request[header_end + 4..];
    let actual: GovernanceRequest = serde_json::from_slice(body)
        .map_err(|e| format!("invalid governance request payload: {}", e))?;
    if &actual != expected {
        return Err(format!(
            "unexpected governance request payload: {}",
            serde_json::to_string(&actual).unwrap_or_default()
        ));
    }

    Ok(())
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

fn expected_request() -> GovernanceRequest {
    GovernanceRequest {
        proposal_id: "runtime-seam-001b".to_string(),
        state_snapshot_hash: "abc123def456abc123def456abc123def456abc123def456abc123def456abc123"
            .to_string(),
        requested_action: "deploy_capsule".to_string(),
        constraints: BTreeMap::new(),
    }
}

fn make_trace() -> ExecutionTrace {
    ExecutionTrace::from_steps(vec![
        TraceStep {
            region_id: "gateway.snapshot".to_string(),
            operation: "compare_hash".to_string(),
            input_hash: "a".repeat(64),
            output_hash: "b".repeat(64),
            operation_type: OperationType::HashVerify,
        },
        TraceStep {
            region_id: "governance.validator".to_string(),
            operation: "compose_decision".to_string(),
            input_hash: "b".repeat(64),
            output_hash: "d".repeat(64),
            operation_type: OperationType::TraceFinalize,
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

fn governed_local_runtime() -> GovernedRuntime {
    GovernedRuntime::new(
        StubRuntime::new(),
        PolicyConfig::default(),
        EconomicsConfig::default(),
    )
}

#[test]
fn governed_backed_mode_emits_governed_packet() {
    let _guard = seam_guard();
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

    let mut runtime = runtime_for(server.endpoint(), GOVERNANCE_HASH.trim());
    let outcome = runtime.evaluate(&proposal()).expect("evaluate");
    let packet = outcome.packet.as_ref().expect("packet");

    assert_eq!(
        outcome.mode,
        iter_mcp_server::runtime::GovernanceMode::Governed
    );
    assert!(outcome.authoritative_pdp);
    assert_eq!(packet.governance_hash(), Some(GOVERNANCE_HASH.trim()));
    assert!(
        !packet.execution_trace().is_empty(),
        "scg-backed packet must carry the SCG execution trace"
    );

    let result = runtime.search_decisions(&AuditSearchFilter {
        decision: Some("ALLOW".to_string()),
        ..AuditSearchFilter::default()
    });
    let count = result.count;
    assert!(
        count >= 1,
        "audit.search must find the SCG-backed ALLOW record"
    );
    let first = result.results.first().expect("at least one result");
    assert_eq!(first.decision, "ALLOW");

    server.finish();
}

#[test]
fn governed_backed_bound_fields_not_overwritten_by_adapter() {
    let _guard = seam_guard();
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

    let mut runtime = runtime_for(server.endpoint(), GOVERNANCE_HASH.trim());
    let outcome = runtime.evaluate(&proposal()).expect("evaluate");
    let packet = outcome.packet.as_ref().expect("packet");

    assert_eq!(
        packet.governance_hash.as_deref(),
        Some(GOVERNANCE_HASH.trim())
    );
    assert!(
        !packet.execution_trace.is_empty(),
        "scg-backed packet must retain non-empty SCG execution_trace after adapter binding"
    );

    server.finish();
}

#[test]
fn governance_endpoint_unavailable_returns_explicit_error() {
    let _guard = seam_guard();
    let mut runtime = runtime_for("http://127.0.0.1:1", GOVERNANCE_HASH.trim());
    let err = runtime.evaluate(&proposal()).expect_err("scg unavailable");
    assert!(matches!(err, GovernanceRuntimeError::ScgUnavailable(_)));
}

#[test]
fn replay_trace_is_identical_not_just_output() {
    let _guard = seam_guard();
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

    server.finish();
}

#[test]
fn governance_hash_absent_fails_boot() {
    let _guard = seam_guard();
    let result = ScgRuntime::connect("http://127.0.0.1:18080".to_string(), String::new());
    assert!(matches!(
        result,
        Err(GovernanceRuntimeError::ConfigMissing(_))
    ));
}

#[test]
fn endpoint_absent_fails_boot() {
    let _guard = seam_guard();
    let result = ScgRuntime::connect(String::new(), GOVERNANCE_HASH.trim().to_string());
    assert!(matches!(
        result,
        Err(GovernanceRuntimeError::ConfigMissing(_))
    ));
}

#[test]
fn contract_version_mismatch_fails_closed() {
    let _guard = seam_guard();
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

    server.finish();
}

#[test]
fn governance_hash_mismatch_fails_closed() {
    let _guard = seam_guard();
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

    server.finish();
}

#[test]
fn replay_integrity_violation_fails_closed() {
    let _guard = seam_guard();
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
    assert!(
        matches!(err, GovernanceRuntimeError::ReplayIntegrityViolation(_)),
        "unexpected error: {:?}",
        err
    );

    server.finish();
}

#[test]
fn audit_search_rejects_unsupported_filters_on_governed_backed_mode() {
    let _guard = seam_guard();
    let mut client = McpTestClient::spawn_governed_backed("http://127.0.0.1:1");

    let response = client.call(
        "tools/call",
        json!({
            "name": "audit.search",
            "arguments": { "principal": "alice" }
        }),
    );
    let message = response
        .pointer("/result/error/message")
        .and_then(|value| value.as_str())
        .expect("unsupported audit.search filter error");
    assert!(
        message.contains("audit.search does not support filters in scg-backed mode"),
        "unexpected audit.search error message: {}",
        message
    );

    client.close();
}

#[test]
fn audit_replay_returns_governed_remote_decision_history() {
    let _guard = seam_guard();
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
    let evaluate_response = client.call(
        "tools/call",
        json!({
            "name": "decision.check",
            "arguments": {
                "proposal_id": proposal().proposal_id,
                "state_snapshot_hash": proposal().state_snapshot_hash,
                "requested_action": proposal().requested_action,
                "constraints": {}
            }
        }),
    );
    let evaluate_payload = tool_payload(&evaluate_response);
    let decision_id = evaluate_payload
        .get("packet")
        .and_then(|packet| packet.get("checksum"))
        .and_then(|value| value.as_str())
        .expect("decision packet checksum")
        .to_string();

    let replay_response = client.call(
        "tools/call",
        json!({
            "name": "audit.replay",
            "arguments": {}
        }),
    );
    let replay_results: Vec<ReplayResult> =
        serde_json::from_value(tool_payload(&replay_response)).expect("replay results");

    assert_eq!(
        replay_results.len(),
        1,
        "expected one SCG-backed replay result"
    );
    assert_eq!(replay_results[0].decision_id, decision_id);
    assert_eq!(replay_results[0].replay_status, ReplayStatus::Match);
    assert_eq!(
        replay_results[0].propagation_checksum.as_deref(),
        Some(decision_id.as_str())
    );

    client.close();
    server.finish();
}

#[test]
fn governed_local_and_governed_backed_produce_structurally_equivalent_packets() {
    let _guard = seam_guard();
    let mut local_runtime = governed_local_runtime();
    let local_outcome = local_runtime
        .evaluate(&proposal())
        .expect("governed-local evaluate");
    let local_packet = local_outcome
        .packet
        .expect("governed-local must emit packet");

    let remote_hash = "b".repeat(64);
    let server = MockScgServer::spawn(vec![MockHttpResponse {
        status_code: 200,
        body: serde_json::to_string(&contract_outcome(
            CONTRACT_VERSION_STR,
            ScgDecision::Allow,
            &remote_hash,
            false,
        ))
        .expect("serialize outcome"),
    }]);

    let mut remote_runtime = runtime_for(server.endpoint(), &remote_hash);
    let remote_outcome = remote_runtime
        .evaluate(&proposal())
        .expect("governed-backed evaluate");
    let remote_packet = remote_outcome
        .packet
        .expect("governed-backed must emit packet");

    assert!(
        local_packet
            .governance_hash
            .as_deref()
            .map(|hash| !hash.is_empty())
            .unwrap_or(false),
        "governed-local: governance_hash must be present and non-empty"
    );
    assert!(
        remote_packet
            .governance_hash
            .as_deref()
            .map(|hash| !hash.is_empty())
            .unwrap_or(false),
        "governed-backed: governance_hash must be present and non-empty"
    );
    assert!(
        !local_packet.execution_trace.is_empty(),
        "governed-local: execution_trace must be present and non-empty"
    );
    assert!(
        !remote_packet.execution_trace.is_empty(),
        "governed-backed: execution_trace must be present and non-empty"
    );
    assert_ne!(
        local_packet.governance_hash, remote_packet.governance_hash,
        "governed-local and governed-backed must not be coupled to the same governance_hash in this test fixture"
    );
    assert!(
        !local_packet.iter_build_hash.is_empty(),
        "governed-local packet must carry iter build identity"
    );
    assert!(
        !remote_packet.iter_build_hash.is_empty(),
        "governed-backed packet must carry iter build identity"
    );
    assert!(
        !local_packet.substrate_build_hash.is_empty(),
        "governed-local packet must carry substrate build identity"
    );
    assert!(
        !remote_packet.substrate_build_hash.is_empty(),
        "governed-backed packet must carry substrate build identity"
    );

    server.finish();
}
