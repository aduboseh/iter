# Iter Governance Model

**Classification:** External-Safe  
**Version:** 1.0  
**Last Updated:** February 2026

---

## Purpose

This document defines Iter's governance execution model as a deterministic finite state machine.

---

## Governance FSM States

### State 1: VALIDATING

**Entry Condition:** Request received

**Actions:**
- Validate JSON schema
- Check type safety
- Verify range bounds
- Detect NaN/Inf floats
- Validate enum membership

**Transitions:**
- Valid → EVALUATING
- Invalid → REJECTED

---

### State 2: EVALUATING

**Entry Condition:** Validation passed

**Actions:**
- Load policy configuration
- Evaluate governance gates
- Check economic constraints
- Determine learning permission

**Transitions:**
- Policy = ALLOW → PERMITTED
- Policy = DENY → DENIED
- Policy = FREEZE_LEARNING → FROZEN
- Policy = REQUIRE_REVIEW → REVIEW_REQUIRED
- Policy = DEGRADED_MODE → DEGRADED

---

### State 3: PERMITTED

**Entry Condition:** Policy decision = ALLOW

**Actions:**
- Emit DecisionPacket (decision=ALLOW)
- Emit AuditEvent (outcome=PERMITTED)
- Return success to consumer

**Terminal State:** Yes

---

### State 4: DENIED

**Entry Condition:** Policy decision = DENY

**Actions:**
- Emit DecisionPacket (decision=DENY, reason_codes)
- Emit AuditEvent (outcome=DENIED)
- Return rejection to consumer

**Terminal State:** Yes

---

### State 5: FROZEN

**Entry Condition:** Policy decision = FREEZE_LEARNING

**Actions:**
- Emit DecisionPacket (decision=FREEZE_LEARNING, reason_codes)
- Emit AuditEvent (outcome=FROZEN)
- Return freeze status to consumer

**Terminal State:** Yes

---

### State 6: REVIEW_REQUIRED

**Entry Condition:** Policy decision = REQUIRE_REVIEW

**Actions:**
- Emit DecisionPacket (decision=REQUIRE_REVIEW, reason_codes)
- Emit AuditEvent (outcome=REVIEW_REQUIRED)
- Return review status to consumer

**Terminal State:** Yes

---

### State 7: DEGRADED

**Entry Condition:** Policy decision = DEGRADED_MODE

**Actions:**
- Emit DecisionPacket (decision=DEGRADED_MODE, reason_codes)
- Emit AuditEvent (outcome=DEGRADED)
- Return degraded status to consumer

**Terminal State:** Yes

---

### State 8: REJECTED

**Entry Condition:** Validation failed

**Actions:**
- Emit error response (error_code)
- Emit AuditEvent (outcome=REJECTED)
- Return error to consumer

**Terminal State:** Yes

---

## State Transition Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│  REQUEST                                                            │
│     │                                                               │
│     ▼                                                               │
│  ┌─────────────┐                                                   │
│  │ VALIDATING  │                                                   │
│  └─────┬───────┘                                                   │
│        │                                                            │
│   ┌────┴────┐                                                      │
│   │         │                                                      │
│  Valid    Invalid                                                  │
│   │         │                                                      │
│   ▼         ▼                                                      │
│  ┌─────────────┐    ┌──────────────┐                              │
│  │ EVALUATING  │    │  REJECTED    │ (terminal)                   │
│  └─────┬───────┘    └──────────────┘                              │
│        │                                                            │
│   ┌────┴────┬────────────┬─────────────┬─────────┐                │
│   │         │            │             │         │                │
│  ALLOW    DENY    FREEZE_LEARNING  REQUIRE   DEGRADED             │
│   │         │            │         REVIEW       │                │
│   ▼         ▼            ▼             ▼         ▼                │
│  ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐          │
│  │PERMIT  │ │DENIED  │ │FROZEN  │ │REVIEW  │ │DEGRAD  │          │
│  └────────┘ └────────┘ └────────┘ └────────┘ └────────┘          │
│  (terminal) (terminal) (terminal) (terminal) (terminal)           │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Drift vs Violation

### Drift (Soft Signal)

**Definition:** Observable deviation from expected behavior without policy violation.

**Examples:**
- Quality degradation below threshold (DEGRADED_MODE)
- Scarcity streak approaching limit
- Integrity approaching boundary

**Response:**
- Continue execution
- Emit warning codes in DecisionPacket
- Flag for human review (REQUIRE_REVIEW)

**Audit Code:** `DRIFT_DETECTED`

---

### Violation (Hard Error)

**Definition:** Policy rule broken or invalid input detected.

**Examples:**
- NaN/Inf float values
- Budget exceeded
- Permit expired
- Unknown enum value

**Response:**
- Halt execution immediately
- Reject request with error code
- No DecisionPacket emission

**Audit Code:** `VIOLATION_DETECTED`

---

## Governance Truth Table

| Input Valid | Policy Check | Learning OK | Outcome State |
|------------|-------------|------------|---------------|
| No | - | - | REJECTED |
| Yes | DENY | - | DENIED |
| Yes | ALLOW | Yes | PERMITTED |
| Yes | ALLOW | No | FROZEN |
| Yes | REQUIRE_REVIEW | - | REVIEW_REQUIRED |
| Yes | DEGRADED_MODE | - | DEGRADED |

---

## Enforcement

**Determinism Guarantee:**
- Same input + same policy → same state progression
- Same final state → same DecisionPacket checksum

**Fail-Closed Guarantee:**
- Unknown state → REJECTED
- Ambiguous transition → REJECTED

**Audit Guarantee:**
- Every state transition emits AuditEvent
- State progression is reconstructible from audit log

---

## Governance Evaluation Logic

### Policy Gates (Evaluated in Order)

1. **Input Quality Gate**
   - Check input quality threshold
   - Fail → DENY (REASON: INPUT_QUALITY_INSUFFICIENT)

2. **Reasoning Quality Gate**
   - Check reasoning quality threshold
   - Fail → DEGRADED_MODE or REQUIRE_REVIEW

3. **Energy Integrity Gate**
   - Check energy integrity threshold
   - Fail → DENY (REASON: ENERGY_INTEGRITY_BELOW_THRESHOLD)

4. **Economic Control Gate**
   - Check budget limits
   - Check permit validity
   - Fail → DENY (REASON: WINDOW_BUDGET_EXCEEDED, PERMIT_EXPIRED, etc.)

5. **Learning Permission Gate**
   - Check learning eligibility
   - Fail → FREEZE_LEARNING (REASON: SCARCITY_STREAK_EXCEEDED, etc.)

**All gates pass → ALLOW**

---

## State Persistence

**Iter does NOT persist state across requests.**

Each request is evaluated independently using:
- Inputs provided in request
- Policy configuration (versioned)
- Economic configuration (versioned)

No session state, no memory, no accumulation.

---

## Verification

**Claim:** Governance FSM is deterministic and fail-closed.

**Verification:**
1. Run identical request twice
2. Compare DecisionPackets
3. Checksums must match

**Test:**
```rust
#[test]
fn governance_fsm_determinism() {
    let request = /* ... */;
    let packet1 = evaluate(request.clone());
    let packet2 = evaluate(request);
    assert_eq!(packet1.checksum, packet2.checksum);
}
```
