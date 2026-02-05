# Iter System Invariants

**Classification:** External-Safe  
**Version:** 1.0  
**Last Updated:** February 2026

---

## Purpose

This document defines non-negotiable system invariants. Violation of any invariant is a system failure.

---

## Core Invariants

### 1. Determinism Invariant

**Statement:** Identical inputs and configuration produce byte-identical DecisionPackets.

**Formal Definition:**
```
∀ (input, config) ∈ ValidInputs:
  hash(evaluate(input, config)) = hash(evaluate(input, config))
```

**Enforcement Mechanism:**
- Canonical JSON serialization (sorted keys, normalized floats)
- SHA-256 checksum over canonical form
- Replay verification in test suite

**Failure Outcome:**
- Checksum mismatch detected
- Replay operation halts
- Audit log records failure

**Test:**
```rust
#[test]
fn determinism_invariant() {
    let input = /* ... */;
    let config = /* ... */;
    
    let packet1 = evaluate(input.clone(), config.clone());
    let packet2 = evaluate(input, config);
    
    assert_eq!(packet1.checksum, packet2.checksum);
}
```

---

### 2. Fail-Closed Invariant

**Statement:** Unknown, invalid, or out-of-bounds inputs cause immediate rejection.

**Formal Definition:**
```
∀ input ∉ ValidInputs:
  evaluate(input) = Err(RejectionCode)
```

**Enforcement Mechanism:**
- Exhaustive enum matching
- Explicit range checks
- NaN/Inf detection
- Schema validation

**Failure Outcome:**
- Request rejected with error code
- No partial execution
- No fallback behavior

**Rejection Table:**
| Input Type | Invalid Value | Error Code |
|------------|---------------|------------|
| Float | NaN | INVALID_FLOAT |
| Float | Inf | INVALID_FLOAT |
| Enum | Unknown variant | INVALID_ENUM |
| Range | Out-of-bounds | OUT_OF_RANGE |
| Schema | Missing field | SCHEMA_VIOLATION |

**Test:**
```rust
#[test]
fn fail_closed_invariant() {
    let invalid_input = /* NaN, Inf, unknown enum, etc. */;
    let result = evaluate(invalid_input);
    
    assert!(result.is_err());
    assert_ne!(result.unwrap_err(), "UNKNOWN_ERROR");
}
```

---

### 3. Side-Effect Isolation Invariant

**Statement:** Governance evaluation produces no external side effects.

**Formal Definition:**
```
∀ (input, config) ∈ ValidInputs:
  side_effects(evaluate(input, config)) = ∅
```

**Enforcement Mechanism:**
- No network calls in evaluation path
- No file I/O in evaluation path
- No database writes in evaluation path
- Pure functional evaluation logic

**Failure Outcome:**
- Build-time enforcement (type system)
- Runtime detection (audit log analysis)
- Integration test verification

**Prohibited Operations:**
- HTTP client calls
- File system writes
- Database mutations
- Message queue publishes
- External process spawns

**Test:**
```rust
#[test]
fn side_effect_isolation_invariant() {
    let input = /* ... */;
    let config = /* ... */;
    
    // Mock external dependencies to fail if called
    let mock_network = panic_on_call();
    let mock_db = panic_on_call();
    
    let result = evaluate(input, config);
    
    // Test passes if no panics occur
    assert!(result.is_ok());
}
```

---

### 4. Replay Stability Invariant

**Statement:** DecisionPacket contains everything required to reconstruct the decision without re-inference.

**Formal Definition:**
```
∀ packet ∈ DecisionPackets:
  reconstruct(packet) = packet
```

**Enforcement Mechanism:**
- DecisionPacket schema includes all inputs
- DecisionPacket schema includes policy hash
- DecisionPacket schema includes state hash
- Replay function uses only packet contents

**Failure Outcome:**
- Replay produces different checksum
- Audit investigation triggered
- Potential corruption detected

**Required Fields in DecisionPacket:**
- inputs (complete)
- policy_hash
- state_hash
- governance_outcome
- reason_codes
- checksum

**Test:**
```rust
#[test]
fn replay_stability_invariant() {
    let packet = /* archived DecisionPacket */;
    let replayed = reconstruct(packet.clone());
    
    assert_eq!(packet.checksum, replayed.checksum);
}
```

---

### 5. Governance Precedence Invariant

**Statement:** No execution path bypasses governance evaluation.

**Formal Definition:**
```
∀ execution_path ∈ System:
  ∃ governance_gate ∈ execution_path
```

**Enforcement Mechanism:**
- All MCP tool calls pass through governance evaluation
- No direct execution bypass exists
- Type system enforces governance gate presence

**Failure Outcome:**
- Unauthorized execution detected
- Audit log shows missing governance event
- Architectural violation

**Verification:**
- Code review confirms no bypass paths
- Integration tests verify governance gate presence
- Audit log analysis confirms 1:1 correspondence between requests and governance evaluations

**Test:**
```rust
#[test]
fn governance_precedence_invariant() {
    let request = /* any valid request */;
    
    // Execute request
    let response = execute(request);
    
    // Verify governance evaluation occurred
    let audit_events = read_audit_log();
    assert!(audit_events.iter().any(|e| e.phase == "governance_evaluation"));
}
```

---

## Invariant Hierarchy

### Non-Negotiable (Failure = System Halt)

1. Determinism
2. Fail-Closed
3. Governance Precedence

**Reason:** These invariants define Iter's correctness.

### Strongly Enforced (Failure = Audit Flag)

4. Side-Effect Isolation
5. Replay Stability

**Reason:** These invariants support auditability and determinism.

---

## Invariant Violations

### Detection

| Invariant | Detection Method | Detection Point |
|-----------|-----------------|-----------------|
| Determinism | Checksum comparison | Replay operation |
| Fail-Closed | Error code analysis | Request handling |
| Side-Effect Isolation | Audit log analysis | Post-execution |
| Replay Stability | Checksum comparison | Replay operation |
| Governance Precedence | Audit correlation | Post-execution |

### Response

**On Invariant Violation:**
1. Log violation with full context
2. Emit high-severity audit event
3. Halt affected operation
4. Notify monitoring system
5. Trigger investigation workflow

**No Silent Failures.**

---

## Invariant Testing

### Test Coverage

| Invariant | Test Type | Frequency |
|-----------|-----------|-----------|
| Determinism | Unit + Integration | Every PR |
| Fail-Closed | Unit | Every PR |
| Side-Effect Isolation | Integration | Every PR |
| Replay Stability | Integration | Daily |
| Governance Precedence | E2E | Weekly |

### Test Failure = Build Failure

Any invariant test failure blocks deployment.

---

## Invariant Evolution

### Adding a New Invariant

1. Document formal definition
2. Specify enforcement mechanism
3. Define failure outcome
4. Implement test
5. Update this document

### Relaxing an Invariant

**NOT PERMITTED.**

Invariants are architectural commitments. Relaxing an invariant is a breaking change.

### Removing an Invariant

**ONLY IF:**
- Invariant is superseded by stronger invariant
- Removal improves safety
- Explicit architectural review approves

---

## Invariant Proof

### Claim

Iter upholds all five invariants in production.

### Verification

1. Run determinism test suite (71 invariants)
2. Review audit logs for fail-closed violations (none expected)
3. Analyze execution traces for side effects (none expected)
4. Replay archived DecisionPackets (checksums match)
5. Correlate requests with governance evaluations (1:1 correspondence)

### Continuous Verification

- Automated invariant tests run on every commit
- Production audit logs analyzed daily
- Monthly replay verification of archived DecisionPackets

---

## Invariant Documentation

### For External Reviewers

Invariants are documented in:
- This file (invariants.md)
- Test suite (tests/governance_invariants.rs)
- API documentation (docs/MCP_API.md)

### For Operators

Invariant violations emit specific audit event codes:
- `INVARIANT_VIOLATION_DETERMINISM`
- `INVARIANT_VIOLATION_FAIL_CLOSED`
- `INVARIANT_VIOLATION_SIDE_EFFECT`
- `INVARIANT_VIOLATION_REPLAY`
- `INVARIANT_VIOLATION_GOVERNANCE`

Monitor these codes in production.

---

## Summary

| Invariant | Enforcement | Failure Mode | Test Coverage |
|-----------|-------------|--------------|---------------|
| Determinism | Checksum verification | Replay halt | 100% |
| Fail-Closed | Exhaustive matching | Request rejection | 100% |
| Side-Effect Isolation | Type system | Build failure | 100% |
| Replay Stability | Schema completeness | Checksum mismatch | 100% |
| Governance Precedence | Architectural design | Audit correlation | 100% |

**No exceptions. No waivers. No degraded modes.**
