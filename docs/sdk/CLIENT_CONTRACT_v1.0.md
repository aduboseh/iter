# Iter SDK Client Contract v1.0

**Status**: CANONICAL  
**Authority**: Normative for all Iter client implementations  
**Compliance**: Fail-closed (any violation = non-conformant)  
**Version**: 1.0.0  
**Date**: 2026-01-27

---

## 0. Scope

This document defines the **mandatory behavioral contract** for all Iter SDK clients (TypeScript, Rust, Python, future languages). 

**Normative Language**:
- **MUST** / **SHALL**: Absolute requirement
- **MUST NOT** / **SHALL NOT**: Absolute prohibition  
- **SHOULD**: Strong recommendation (deviation requires justification)
- **MAY**: Optional feature

**Compliance**: An SDK is conformant **if and only if** it satisfies every MUST/MUST NOT requirement and passes all stated conformance tests.

---

## 1. Transport

### 1.1 STDIO Protocol

**R1.1.1**: Client MUST communicate with the Iter binary exclusively via STDIO (stdin/stdout).  
**R1.1.2**: Client MUST use JSON-RPC 2.0 message format.  
**R1.1.3**: Client MUST write one JSON object per line, terminated by LF (`\n`).  
**R1.1.4**: Client MUST read one JSON object per line from stdout.  
**R1.1.5**: Client MUST NOT use HTTP, WebSocket, or any other transport.

**Conformance Test CT1.1**:
```
Given: SDK spawns Iter binary
When: Client sends request {"jsonrpc":"2.0","method":"tools/list","id":1}
Then:
- Stdout line MUST parse as valid JSON-RPC 2.0 response
- Response MUST contain "id":1
```

---

### 1.2 Request Correlation

**R1.2.1**: Client MUST assign monotonically increasing integer request IDs starting from 1.  
**R1.2.2**: Client MUST support multiple in-flight requests with distinct IDs.  
**R1.2.3**: Client MUST dispatch responses by matching `response.id` to pending request ID.  
**R1.2.4**: Client MUST ignore responses with unknown IDs (including those evicted due to timeout). Client SHOULD log warning; MUST NOT crash.

**Conformance Test CT1.2**:
```
Given: SDK connected to Iter binary
When: Client sends 3 requests with IDs 1, 2, 3 without awaiting responses
And: Server responds with valid JSON-RPC responses in any order
Then: Each response MUST resolve the correct pending request by matching response.id
And: No response MUST resolve the wrong request
```

---

## 2. Backpressure

### 2.1 In-Flight Limit

**R2.1.1**: Client MUST enforce a configurable `max_inflight` limit (default: 1).  
**R2.1.2**: Client MUST reject new requests if `pending.size() >= max_inflight`.  
**R2.1.3**: Rejection MUST return a typed error (e.g., `BackpressureError`), not hang.  
**R2.1.4**: Client MAY expose `max_inflight` as constructor/config parameter.

**Rationale**: Prevents unbounded memory growth and cascade failures.

**Conformance Test CT2.1**:
```
Given: Client with max_inflight=2
When: Client sends 3 concurrent requests
Then: Third request MUST fail immediately with BackpressureError
```

---

## 3. Timeouts

### 3.1 Per-Request Timeout

**R3.1.1**: Client MUST enforce a per-request timeout (default: 30 seconds).  
**R3.1.2**: Timeout MUST be user-configurable (e.g., `send(method, params, timeout=60.0)`).  
**R3.1.3**: On timeout, client MUST:
  1. Remove pending entry from dispatch map
  2. Return a typed error (e.g., `RequestTimeoutError`)
  3. NOT wait indefinitely for response
**R3.1.4**: Timed-out requests that later arrive MUST be ignored (see R1.2.4).

**Conformance Test CT3.1**:
```
Given: Client with timeout=2s
When: Binary does not respond within 2s
Then: Request MUST fail with RequestTimeoutError after 2s (±100ms)
And: Late response arriving at t=5s MUST be ignored
```

---

## 4. Subprocess Lifecycle

### 4.1 Process Ownership

**R4.1.1**: If SDK spawns the Iter binary, SDK MUST own the process lifecycle.  
**R4.1.2**: Client MUST provide an explicit `close()` / `shutdown()` method.  
**R4.1.3**: On close, client MUST:
  1. Stop accepting new requests
  2. Drain pending requests with bounded timeout (5 seconds)
  3. Send SIGTERM to process
  4. Wait 2 seconds for graceful shutdown
  5. Send SIGKILL if process still alive
  6. `await process.wait()` to reap zombie

**R4.1.4**: Client MUST support context manager / RAII pattern (`async with`, `using`, `Drop`).  
**R4.1.5**: Undrained requests at close MUST fail with `ConnectionClosedError`.

**Conformance Test CT4.1**:
```
Given: Client with 2 pending requests
When: User calls close() while requests in-flight
Then: Both requests MUST complete OR timeout within 5s
And: Process MUST NOT appear in ps aux after close() returns
And: No zombie processes remain
```

---

### 4.2 Process Failure Handling

**R4.2.1**: If subprocess exits unexpectedly, client MUST:
  1. Fail all pending requests with `ConnectionError`
  2. Mark client as closed
  3. Reject subsequent requests

**R4.2.2**: Client MUST NOT attempt automatic reconnection.

**Conformance Test CT4.2**:
```
Given: Client with 1 pending request
When: Binary process crashes (exit code 1)
Then: Pending request MUST fail with ConnectionError
And: Next request MUST fail immediately (not hang)
```

---

## 5. stderr Capture

### 5.1 Diagnostic Capture

**R5.1.1**: Client MUST NOT discard stderr.  
**R5.1.2**: Client MUST capture stderr to a bounded buffer (minimum: last 100 lines or 10KB).  
**R5.1.3**: On connection failure OR request failure, client MUST attach stderr tail to error.  
**R5.1.4**: Client MAY expose `get_stderr_tail()` method for diagnostic access.

**Rationale**: Rust panics and binary diagnostics go to stderr. Silent discard = forensic blindness.

**Conformance Test CT5.1**:
```
Given: Iter binary writes "PANIC: index out of bounds" to stderr
When: Binary exits with code 101
Then: ConnectionError MUST contain "PANIC: index out of bounds" in error message or metadata
```

---

## 6. Malformed stdout

### 6.1 Non-JSON Handling

**R6.1.1**: Client MUST NOT silently discard non-JSON lines from stdout.  
**R6.1.2**: Client MUST either:
  - **Option A**: Store malformed lines in a bounded diagnostic buffer (last 50 lines), OR
  - **Option B**: Invoke a user-provided callback `on_malformed_line(line, error)`
**R6.1.3**: Client MUST log malformed lines at WARN level (if no callback).  
**R6.1.4**: Malformed lines MUST NOT crash the dispatcher loop.

**Rationale**: Binary debug logs mixed with JSON-RPC responses should be observable, not invisible.

**Conformance Test CT6.1**:
```
Given: Binary writes "DEBUG: initializing graph\n" to stdout
When: Client reads this line
Then: Line MUST appear in diagnostic buffer OR trigger on_malformed_line callback
And: Dispatcher MUST continue processing subsequent valid JSON-RPC responses
```

---

## 7. Versioning

### 7.1 Compatibility Checking

**R7.1.1**: Client MUST expose `is_version_compatible(server_version: str) -> bool`.  
**R7.1.2**: Version check MUST use semantic versioning (major.minor.patch).  
**R7.1.3**: Client MUST define `SDK_PROTOCOL_VERSION`, `MIN_SERVER_VERSION`, `MAX_SERVER_VERSION`.  
**R7.1.4**: Compatibility check MUST pass if:
```
MIN_SERVER_VERSION <= server_version <= MAX_SERVER_VERSION
```

**R7.1.5**: Client SHOULD perform version handshake at connect time IF server supports `protocol/version` method.  
**R7.1.6**: If handshake fails, client MUST close connection and raise `VersionMismatchError`.

**Conformance Test CT7.1**:
```
Given: Client SDK_PROTOCOL_VERSION=1.0.0, MIN=1.0.0, MAX=1.99.99
Then: is_version_compatible("1.5.0") MUST return true
And: is_version_compatible("2.0.0") MUST return false
And: is_version_compatible("0.9.0") MUST return false
```

**Conformance Test CT7.2**:
```
Given: Server responds to protocol/version with {"version":"2.0.0"}
When: Client calls connect()
Then: Connect MUST fail with VersionMismatchError(client=1.0.0, server=2.0.0)
And: Subprocess MUST be terminated
```

---

## 8. TraceContext

### 8.1 Client-Side Only

**R8.1.1**: Client MUST support a `TraceContext` type with fields:
  - `trace_id: str`
  - `span_id: str`
  - `parent_span_id: Optional[str]`

**R8.1.2**: Client MUST provide `with_trace(ctx: TraceContext)` method for setting context.  
**R8.1.3**: TraceContext MUST be stored in client state but MUST NOT be serialized to wire protocol unless future protocol versions define semantics.  
**R8.1.4**: Client MAY emit TraceContext fields in structured logs.

**Rationale**: TraceContext exists for future OpenTelemetry integration but has no current wire protocol. Making it "work" without server support creates false expectations.

**Conformance Test CT8.1**:
```
Given: Client with trace context set
When: Client sends request
Then: Request JSON MUST NOT contain trace_id in JSON-RPC message
And: Client logs SHOULD contain trace_id in structured fields
```

---

## 9. Error Types

### 9.1 Typed Error Hierarchy

**R9.1.1**: Client MUST expose distinct error types:
  - `SdkError` (base class)
  - `VersionMismatchError(client_version, server_version)`
  - `ConnectionError(message, stderr_tail?)`
  - `RequestError(rpc_error: RpcError)`
  - `RequestTimeoutError(method, timeout)`
  - `BackpressureError(max_inflight)`

**R9.1.2**: Errors MUST be language-idiomatic (exceptions in Python/TS, Result in Rust).  
**R9.1.3**: All error messages MUST be actionable (include diagnostic hints).

**Conformance Test CT9.1**:
```
Given: User code catches error
When: Error is ConnectionError
Then: Error MUST expose stderr_tail as accessible field/property
```

---

## 10. Non-Goals (Explicit Prohibitions)

### 10.1 Out of Scope

**R10.1.1**: Client MUST NOT implement agent orchestration (planning, reflection, tool chaining).  
**R10.1.2**: Client MUST NOT embed framework adapters (LangChain, Google ADK, Semantic Kernel).  
**R10.1.3**: Client MUST NOT manage connection pools (future consideration only).  
**R10.1.4**: Client MUST NOT implement caching, retries, or circuit breakers (orthogonal concerns).

**Rationale**: SDK is a thin protocol client, not an agent framework.

---

## 11. Conformance

### 11.1 Test Requirements

**R11.1.1**: Every conformance test CT* listed above MUST be implemented in SDK test suite.  
**R11.1.2**: All conformance tests MUST pass before SDK release.  
**R11.1.3**: Conformance tests MUST run in CI on every commit.

### 11.2 Compliance Matrix

| SDK | Transport | Backpressure | Timeouts | Lifecycle | stderr | Versioning | Malformed | Errors |
|-----|-----------|--------------|----------|-----------|--------|------------|-----------|--------|
| TypeScript | ❓ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ✅ |
| Rust | ❓ | ❌ | ❌ | ⚠️ | ❌ | ✅ | ❌ | ✅ |
| Python | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |

Legend: ✅ Conformant | ⚠️ Partial | ❌ Non-conformant | ❓ Needs verification

---

## 12. Maintenance

### 12.1 Contract Evolution

**R12.1.1**: Changes to this contract MUST increment version (major for breaking, minor for additive).  
**R12.1.2**: All SDKs MUST converge to conformance within one release cycle of contract update.  
**R12.1.3**: Deviations MUST be documented in SDK README with justification and timeline.

---

## Appendix A: Reference Implementation Checklist

Use this checklist to audit existing SDKs:

- [ ] CT1.1: STDIO JSON-RPC line protocol
- [ ] CT1.2: Out-of-order response dispatch by ID
- [ ] CT2.1: Backpressure enforcement with max_inflight
- [ ] CT3.1: Per-request timeout with eviction
- [ ] CT4.1: Graceful shutdown with drain → SIGTERM → SIGKILL
- [ ] CT4.2: Fail pending on subprocess crash
- [ ] CT5.1: stderr capture in error messages
- [ ] CT6.1: Malformed stdout handling (log or callback)
- [ ] CT7.1: Semantic version compatibility check
- [ ] CT7.2: Optional version handshake at connect
- [ ] CT8.1: TraceContext stored but not serialized
- [ ] CT9.1: Typed error hierarchy with diagnostics
- [ ] CT10.1: No orchestration/framework coupling
- [ ] CT11.1: All conformance tests pass in CI

---

**END OF SPECIFICATION**

**Signature**: Armonti Du-Bose-Hill  
**Date**: 2026-01-27  
**Next Action**: Audit TS/Rust against this contract → minimal patches → Python follows mechanically
