# Iter Failure Modes

**Classification:** External-Safe  
**Version:** 1.0  
**Last Updated:** February 2026

---

## Purpose

This document enumerates failure modes, system responses, and observability hooks.

---

## Failure Mode Catalog

### 1. Drift Detected

**Condition:** Observable quality degradation without policy violation.

**Triggers:**
- Reasoning quality < soft threshold
- Energy integrity approaching boundary
- Scarcity streak increasing

**System Response:**
- Continue execution
- Emit DecisionPacket with `decision=DEGRADED_MODE` or `REQUIRE_REVIEW`
- Log warning-level AuditEvent

**Observability:**
- AuditEvent: `phase=governance_evaluation, outcome=DRIFT_DETECTED`
- Reason codes: `REASONING_QUALITY_BELOW_THRESHOLD`, etc.

---

### 2. Violation Detected

**Condition:** Policy rule broken or invalid input.

**Triggers:**
- NaN/Inf float value
- Budget exceeded
- Permit expired
- Unknown enum value
- Out-of-range value

**System Response:**
- Halt execution immediately
- Reject request with error code
- Emit AuditEvent with `outcome=VIOLATION_DETECTED`
- No DecisionPacket emission

**Observability:**
- Error response: `{"error": {"code": "INVALID_FLOAT", ...}}`
- AuditEvent: `phase=validation, outcome=REJECTED`

---

### 3. Replay Mismatch

**Condition:** Archived DecisionPacket cannot be replayed with matching checksum.

**Triggers:**
- Replay logic error
- Data corruption
- Policy version mismatch
- Contract version incompatibility

**System Response:**
- Log error-level AuditEvent
- Trigger investigation workflow
- Halt affected operations

**Observability:**
- AuditEvent: `phase=replay, outcome=CHECKSUM_MISMATCH`
- Alert: `REPLAY_INTEGRITY_VIOLATION`

---

### 4. Tool Misuse

**Condition:** MCP tool called with invalid parameters or out-of-order.

**Triggers:**
- Missing required parameter
- Parameter type mismatch
- Tool call sequence violation

**System Response:**
- Reject request with JSON-RPC error
- Emit AuditEvent with `outcome=TOOL_MISUSE`

**Observability:**
- JSON-RPC error: `{"error": {"code": -32602, "message": "Invalid params"}}`
- AuditEvent: `phase=mcp_validation, outcome=REJECTED`

---

## Fail-Closed Semantics

### Principle

**Unknown → Reject**

Iter never assumes safety. Unknown inputs, states, or conditions cause immediate rejection.

### Examples

| Input | Expected | Actual | Response |
|-------|----------|--------|----------|
| Enum | "ALLOW", "DENY" | "MAYBE" | Reject with `UNKNOWN_ENUM` |
| Float | [0.0, 1.0] | NaN | Reject with `INVALID_FLOAT` |
| Range | [0, 100] | 150 | Reject with `OUT_OF_RANGE` |

---

## Observability Integration

### AuditEvent Schema (Failure-Specific Fields)

```json
{
  "tick": 42,
  "phase": "governance_evaluation",
  "outcome": "VIOLATION_DETECTED",
  "reason_codes": ["WINDOW_BUDGET_EXCEEDED"],
  "timestamp": "2026-02-05T17:00:00Z",
  "severity": "ERROR"
}
```

### Metrics (Azure Monitor)

| Metric | Type | Alert Threshold |
|--------|------|-----------------|
| `iter_violations_total` | Counter | > 10/min |
| `iter_replay_mismatches_total` | Counter | > 0 |
| `iter_drift_events_total` | Counter | > 100/hour |

### Logs (Log Analytics)

```kusto
AuditEvents
| where outcome == "VIOLATION_DETECTED"
| summarize count() by reason_codes, bin(timestamp, 5m)
| order by timestamp desc
```

---

## Failure Response Matrix

| Failure | Severity | System Response | External Observability |
|---------|----------|----------------|------------------------|
| Drift | WARNING | Continue + DEGRADED_MODE | AuditEvent + Reason codes |
| Violation | ERROR | Reject + Halt | Error response + AuditEvent |
| Replay Mismatch | CRITICAL | Halt + Investigate | Alert + AuditEvent |
| Tool Misuse | ERROR | Reject | JSON-RPC error + AuditEvent |

---

## Incident Response Workflow

### On Violation

1. Iter emits AuditEvent
2. Monitoring system triggers alert
3. Operator retrieves DecisionPacket (if emitted)
4. Operator reviews reason codes
5. Operator adjusts policy or inputs
6. Operator resubmits request

### On Replay Mismatch

1. Iter logs REPLAY_INTEGRITY_VIOLATION
2. Monitoring system triggers critical alert
3. Operator halts replay operations
4. Operator retrieves archived DecisionPacket
5. Operator compares expected vs actual checksums
6. Operator identifies root cause (corruption, logic error, etc.)
7. Operator remediates and verifies

---

## No Silent Failures

**Rule:** Every failure emits at least one AuditEvent.

**Verification:**
```
Total Requests = Successful DecisionPackets + Rejected Requests + Errors

Every request → at least one AuditEvent
```
