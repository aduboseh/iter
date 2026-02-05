# SDK Lifecycle Contract

## Version: 1.0.0

## Contract Status

**UNDER RECONSTITUTION (evidence-first).**  
As of 2026-02-04, this document defines **requirements** only.  
Any SDK compliance claims are intentionally omitted until verified by file:line evidence and test execution logs.

This document defines the shared lifecycle contract that all Iter SDK implementations must conform to. Deviations from this contract are considered bugs.

## State Machine

```
OPEN ──(close())──> CLOSING ──(drain complete OR timeout)──> CLOSED
```

**States:**
- `OPEN`: Accepting new requests, processing responses
- `CLOSING`: Not accepting new requests, processing existing responses during drain
- `CLOSED`: Not accepting requests, not processing responses

## API Contracts

### `send(method, params, timeoutMs)`

**Preconditions:**
- State must be `OPEN`
- `responseQueue.size < maxInflight`

**Behavior:**
- Queues request and writes to stdin
- Returns Promise that resolves on matching response or rejects on timeout
- Request ID must monotonically increment

**Errors:**
- `ConnectionClosedError` if state is `CLOSING` or `CLOSED`
- `BackpressureError` if maxInflight exceeded
- `RequestTimeoutError` if timeoutMs elapses without response

### `handleStdoutLine(line)`

**Preconditions:**
- None (handler always registered)

**Behavior:**
- MUST process responses when state is `OPEN` or `CLOSING`
- MUST ignore when state is `CLOSED`
- Parses JSON-RPC, matches request ID, resolves/rejects queued promise

**Rationale:**
- Responses arriving during drain window must be observable
- Allows `waitForDrain()` to actually drain in-flight requests

### `close()`

**Preconditions:**
- None (can be called from any state)

**Behavior:**
1. If state is `CLOSING` or `CLOSED`, return existing close promise
2. Set state to `CLOSING`
3. Await `waitForDrain()` (bounded timeout, e.g., 5000ms)
4. Kill subprocess (SIGTERM → wait → SIGKILL if needed)
5. Set state to `CLOSED`

**Drain semantics:**
- Poll `responseQueue.size` at intervals
- Continue processing stdout while draining
- Timeout: reject pending requests with `ConnectionClosedError`

**Errors:**
- `ConnectionError` if process becomes zombie after SIGKILL

## Error Taxonomy

| Error | When Thrown |
|-------|-------------|
| `ConnectionClosedError` | Send attempted in non-OPEN state; drain timeout |
| `BackpressureError` | maxInflight exceeded |
| `RequestTimeoutError` | Individual request timeout |
| `RequestError` | Server returned error response |
| `ConnectionError` | Process failure, startup failure, or zombie |

## Invariants

1. **State monotonicity:** State transitions only forward (OPEN → CLOSING → CLOSED)
2. **Request ID monotonicity:** IDs strictly increase, never reuse
3. **Backpressure enforcement:** Never exceed configured maxInflight
4. **Drain completeness:** Drain window allows responses to complete
5. **Close idempotence:** Multiple calls to `close()` return same Promise
6. **Fail-closed:** Any protocol violation or error rejects all pending requests

## Implementation Checklist

- [x] State machine: OPEN/CLOSING/CLOSED
- [x] `send()` gated on OPEN only
- [x] `handleStdoutLine()` processes in OPEN || CLOSING
- [x] `close()` sets CLOSING, drains, then CLOSED
- [x] Bounded drain timeout
- [x] SIGTERM → SIGKILL escalation
- [x] Close idempotence
- [x] Backpressure enforcement
- [x] Request timeout with eviction
- [x] Stderr capture (ring buffer, 10KB max)
- [x] Fail-closed on protocol violations

## SDK Compliance Claims

**OMITTED.** Compliance status is produced by audit reports, not embedded claims.
