# Iter Failure Modes and Safe-Fail Behavior

**Classification:** External-Safe  
**Version:** 1.0  
**Last Updated:** January 2026

---

## Purpose

This document defines Iter's behavior under failure conditions. All failure modes result in **execution denial**, not degraded execution or policy bypass.

---

## Design Principle: Fail-Closed Architecture

**Core Invariant:** If governance cannot be verified, execution must not proceed.

Iter implements this through:
- No fallback paths that bypass policy evaluation
- Explicit denial as default response to any ambiguous state
- Timeout-enforced evaluation bounds

---

## Failure Mode Matrix

| Failure Condition | Iter Behavior | Consumer Impact | Detection |
|-------------------|---------------|-----------------|-----------|
| **Iter service unavailable** | Return HTTP 503 + JSON-RPC error | Consumer receives DENY equivalent | Azure Monitor health probe failure |
| **Policy evaluation timeout** | Abort evaluation, return DENY | Request rejected | Prometheus `evaluation_timeout` metric |
| **Invalid policy state** | Refuse to evaluate, return error | Consumer receives DENY | Policy validation at load-time |
| **Identity verification failure** | Reject request, return 401 | Consumer cannot authenticate | Entra ID audit logs |
| **Network partition** | Consumer cannot reach Iter | Consumer treats as DENY | Network connectivity monitoring |
| **Cosmos DB unavailable** | Cannot retrieve policy → DENY | All requests denied | Cosmos DB availability metric |
| **Storage account unavailable** | Cannot persist audit → DENY | Governance paused until recovery | Storage account health check |
| **Invalid input (NaN/Inf)** | Reject input, return error | Consumer receives DENY | Input validation at ingestion |
| **Checksum mismatch (replay)** | Abort replay, return error | Replay operation fails | Replay validation logic |
| **Policy version not found** | Cannot evaluate → DENY | Consumer receives DENY | Policy retrieval logic |

---

## Detailed Failure Scenarios

### 1. Iter Service Unavailable

**Cause:** AKS pod crash, node failure, or deliberate scale-to-zero

**Behavior:**
```json
HTTP 503 Service Unavailable

{
  "jsonrpc": "2.0",
  "error": {
    "code": -32000,
    "message": "Governance service unavailable"
  },
  "id": null
}
```

**Consumer Response:**
- Treat as hard denial
- Implement exponential backoff
- Alert operations team

**Recovery:**
- AKS auto-scales or restarts pods
- Health checks confirm service recovery
- Consumers resume normal operation

---

### 2. Policy Evaluation Timeout

**Cause:** Governance logic exceeds configured timeout (default: 5 seconds)

**Behavior:**
- Evaluation thread terminated
- Partial results discarded
- DENY decision returned with reason code `TIMEOUT`

**AuditEvent:**
```json
{
  "phase": "EVALUATION_TIMEOUT",
  "outcome": "DENY",
  "reason_codes": ["EVALUATION_TIMEOUT"],
  "timestamp": "2026-01-25T10:25:00Z"
}
```

**Prevention:**
- Iter enforces complexity bounds on policies
- Customers test policies in staging with representative workloads

---

### 3. Invalid Policy State

**Cause:** Policy file corrupted, schema violation, or conflicting rules

**Behavior:**
- Policy loader rejects invalid policy at load-time (not at evaluation-time)
- Previous valid policy remains active
- New requests continue using last-known-good policy

**Operational Alert:**
```
CRITICAL: Policy validation failed for tenant_id=abc123
Action: Revert to last valid policy version
```

**Protection:**
- Policies validated in CI/CD before deployment
- Rollback mechanism ensures service continuity

---

### 4. Identity Verification Failure

**Cause:** Entra ID token expired, invalid, or missing required claims

**Behavior:**
```json
HTTP 401 Unauthorized

{
  "jsonrpc": "2.0",
  "error": {
    "code": -32001,
    "message": "Identity verification failed"
  },
  "id": null
}
```

**Consumer Response:**
- Refresh Entra ID token
- Retry request with valid credentials
- Escalate if persistent

**Audit Implication:**
- Failed authentication attempts logged to Entra ID audit stream
- Rate-limiting applied to prevent brute-force

---

### 5. Network Partition

**Cause:** Consumer cannot reach Iter due to network failure

**Behavior:**
- Consumer receives connection timeout
- No response from Iter (service unreachable)

**Consumer Response:**
- **Critical:** Treat unreachable Iter as DENY
- Do NOT proceed with execution assuming ALLOW
- Implement circuit-breaker pattern to avoid cascading failures

**Recovery:**
- Network operations restore connectivity
- Circuit breaker re-tests Iter availability
- Normal operation resumes

---

### 6. Cosmos DB Unavailable

**Cause:** Azure region outage or deliberate failover

**Behavior:**
- Iter cannot retrieve policy definitions
- All evaluation requests return DENY with reason code `POLICY_UNAVAILABLE`

**Operational Impact:**
- Governance paused until Cosmos DB recovers
- Consumers experience full denial of AI operations

**Mitigation:**
- Deploy Cosmos DB with multi-region write capability (production recommendation)
- Iter fails over to secondary region automatically

---

### 7. Storage Account Unavailable

**Cause:** Storage account maintenance or network isolation failure

**Behavior:**
- Iter cannot persist DecisionPackets or AuditEvents
- **Critical Decision:** Iter refuses to evaluate if audit cannot be written
- The runtime returns a fail-closed audit-ledger persistence error; managed deployments should map this to `AUDIT_PERSISTENCE_FAILURE`

**Rationale:**
- Governance without audit is compliance-invalid
- Fail-closed prevents unaudited decisions

**Current executable control:**
- `ITER_AUDIT_LEDGER_PATH` enables a file-backed JSONL ledger
- `ITER_REQUIRE_AUDIT_LEDGER=1` makes that ledger mandatory for governed and `scg-backed` runtime construction
- Existing ledger hash chains are verified on open
- Every governed decision append is flushed and synced before the successful outcome is returned

**Recovery:**
- Storage account health restored
- Iter resumes normal operation
- No decisions lost (all denied during outage)

---

### 8. Invalid Input (NaN / Infinity)

**Cause:** Consumer submits floating-point edge cases

**Behavior:**
```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32602,
    "message": "Invalid params: reasoning_quality contains non-finite value"
  },
  "id": 1
}
```

**Consumer Response:**
- Validate input before submission
- Treat error response as DENY

**Protection:**
- Iter validates all numeric inputs for finiteness
- Rejects requests with NaN, Inf, or -Inf

---

### 9. Checksum Mismatch (Replay)

**Cause:** Attempt to replay tampered or corrupted DecisionPacket

**Behavior:**
- Replay operation aborted
- Error returned: `CHECKSUM_VERIFICATION_FAILED`
- Incident logged for security review

**Audit Implication:**
- Potential tampering detected
- Forensic investigation initiated

---

### 10. Policy Version Not Found

**Cause:** Consumer requests governance using policy version that does not exist

**Behavior:**
- Evaluation refused
- DENY returned with reason code `POLICY_NOT_FOUND`

**Consumer Response:**
- Verify policy version deployed to Iter
- Synchronize policy state between systems

---

## Testing Failure Modes

Iter includes failure mode tests in CI:

```bash
# Run failure mode test suite
cargo test --test failure_modes

# Specific failure scenarios
cargo test test_service_unavailable
cargo test test_policy_timeout
cargo test test_invalid_input_nan
```

---

## Operational Runbook

### Detection → Response → Recovery

| Failure | Detection Time | Response SLA | Recovery Action |
|---------|---------------|--------------|-----------------|
| Service unavailable | <1 min | Alert ops immediately | Scale/restart pods |
| Policy timeout | Real-time | Log + monitor pattern | Optimize policy |
| Identity failure | Real-time | Consumer-side retry | Refresh token |
| Network partition | <2 min | Circuit breaker activates | Network ops restore |
| Cosmos DB down | <1 min | Fail to secondary region | Azure region recovery |
| Storage down | <1 min | Pause governance | Restore storage health |

---

## Customer Guidance

**Iter operates as a critical control plane dependency.** Customers must:

1. **Implement Circuit Breakers**
   - Detect Iter unavailability within 3 failed requests
   - Halt AI execution until connectivity restored
   - Do NOT assume ALLOW on timeout

2. **Monitor Iter Health**
   - Subscribe to Iter health probe metrics
   - Alert on >1% error rate or >99th percentile latency spike

3. **Deploy High-Availability**
   - Multi-AZ AKS deployment (minimum 3 nodes)
   - Cosmos DB multi-region write enabled
   - Storage account geo-redundant (GRS or GZRS)

4. **Test Failure Modes**
   - Conduct chaos engineering exercises
   - Verify consumer behavior when Iter unavailable
   - Confirm no execution bypass paths exist

---

## Non-Negotiable Guarantees

Iter guarantees the following under all failure conditions:

1. **No Policy Bypass:** Execution cannot proceed without verified governance
2. **No Silent Failures:** All failures produce explicit errors
3. **Audit Completeness:** If DecisionPacket cannot be persisted, evaluation does not proceed
4. **Deterministic Behavior:** Identical failures produce identical responses

These guarantees enable safe, compliant AI deployment in regulated environments.

---

*Iter: Governance that fails safely*
