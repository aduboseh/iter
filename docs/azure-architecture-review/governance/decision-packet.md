# DecisionPacket Specification

**Classification:** External-Safe  
**Version:** 1.0  
**Last Updated:** February 2026

---

## Purpose

DecisionPacket is the canonical governance artifact. It contains everything required to verify and replay a governance decision without re-inference.

---

## Schema

### Required Fields

| Field | Type | Description |
|-------|------|-------------|
| `iter_build_hash` | string | Iter version identifier (64 hex chars) |
| `scg_build_hash` | string | SCG version identifier (64 hex chars) |
| `tick` | u64 | Decision tick (monotonic counter) |
| `energy` | EnergyEnvelope | Energy state at decision time |
| `reasoning` | ReasoningEnvelope | Reasoning state at decision time |
| `learning` | LearningEnvelope | Learning state at decision time |
| `policy` | PolicyEnvelope | Policy evaluation result |
| `permit_hash` | string? | Active permit hash (null if no permit) |
| `economics_hash` | string | Economics configuration hash (64 hex chars) |
| `evaluated_rules` | string[] | Ordered list of rule IDs evaluated |
| `checksum` | string | SHA-256 of canonical JSON (64 hex chars) |

### Envelopes

**EnergyEnvelope:**
- `nodes` (f64): [0.0, +∞)
- `reservoir` (f64): [0.0, +∞)
- `integrity` (f64): [0.0, 1.0]

**ReasoningEnvelope:**
- `quality` (f64): [0.0, 1.0]
- `value_signal` (f64): [0.0, 1.0]
- `conflict_signal` (f64): [0.0, 1.0]
- `control_signal` (f64): [0.0, 1.0]

**LearningEnvelope:**
- `capsule_id` (string)
- `epoch` (u64)
- `version_hash` (string, 64 hex chars)
- `update_cost` (f64): [0.0, +∞)
- `update_paid` (f64): [0.0, +∞)
- `update_quality` (f64): [0.0, 1.0]
- `status` (LearningStatus enum)
- `scarcity_streak` (u64)

**PolicyEnvelope:**
- `policy_hash` (string, 64 hex chars)
- `decision` (PolicyDecision enum)
- `reason_codes` (string[], ordered, non-empty if decision ≠ ALLOW)

---

## Checksum Generation

### Canonical JSON Rules

1. **Key ordering**: Lexicographic ASCII sort
2. **No whitespace**: No spaces, newlines, indentation
3. **Float encoding**: No trailing zeros, scientific notation for |value| >= 1e21 or < 1e-6
4. **NaN/Inf**: Forbidden (hard error before serialization)
5. **Null handling**: Explicit `null` for Option::None fields
6. **Array ordering**: Preserved as-is

### Checksum Algorithm

```
SHA-256(canonical_json_bytes) → lowercase hex string (64 chars)
```

### Field Ordering for Checksum

Fields appear in this order for canonical form:
```
economics_hash
energy { integrity, nodes, reservoir }
evaluated_rules
iter_build_hash
learning { capsule_id, epoch, scarcity_streak, status, update_cost, update_paid, update_quality, version_hash }
permit_hash
policy { decision, policy_hash, reason_codes }
reasoning { conflict_signal, control_signal, quality, value_signal }
scg_build_hash
tick
```

---

## Immutability Guarantees

**Once emitted, DecisionPackets are immutable.**

| Operation | Permitted |
|-----------|-----------|
| Read | Yes |
| Write | No |
| Modify | No |
| Delete | No |

**Enforcement:**
- Storage uses immutable blob containers (Azure Storage immutable blobs)
- Checksum mismatch = corruption detected
- No in-place updates

---

## Enum Registries

### LearningStatus (closed set)

| Value | Code | Description |
|-------|------|-------------|
| COMMITTED | 0 | Update successfully committed |
| NO_PROPOSAL_NO_DELTA | 1 | No change needed |
| REJECTED_INPUT_QUALITY | 2 | Input quality below threshold |
| REJECTED_SCARCITY | 3 | Insufficient energy |
| REJECTED_INTEGRITY | 4 | Hash verification failed |

### PolicyDecision (closed set)

| Value | Code | Description |
|-------|------|-------------|
| ALLOW | 0 | Action permitted |
| DENY | 1 | Action blocked |
| FREEZE_LEARNING | 2 | Learning suspended |
| DEGRADED_MODE | 3 | Reduced capability |
| REQUIRE_REVIEW | 4 | Human review required |

**Unknown enum values = MUST fail with `UNKNOWN_ENUM` error.**

---

## Replay Contract

### Guarantee

Given a DecisionPacket, replay MUST:
1. Extract inputs and configuration
2. Recompute governance decision
3. Produce identical checksum

### Failure Condition

Checksum mismatch indicates:
- Replay logic error
- Data corruption
- Policy version mismatch

### Replay Process

```
1. Read DecisionPacket from archive
2. Extract: tick, energy, reasoning, learning, policy_hash, economics_hash
3. Reconstruct governance evaluation using extracted data
4. Generate new DecisionPacket
5. Compare checksums

PASS: checksums match
FAIL: checksums differ → investigation required
```

---

## Validation Rules

### At Generation Time

- All floats checked for NaN/Inf (hard error if detected)
- All ranges validated (e.g., integrity ∈ [0.0, 1.0])
- All hashes validated (64 hex chars)
- reason_codes non-empty if decision ≠ ALLOW

### At Consumption Time

- Schema conformance checked
- Enum values validated (unknown = error)
- Checksum verified (recompute and compare)

---

## Example DecisionPacket (Conceptual)

```json
{
  "iter_build_hash": "a1b2c3...",
  "scg_build_hash": "d4e5f6...",
  "tick": 42,
  "energy": {
    "nodes": 100.0,
    "reservoir": 50.0,
    "integrity": 0.95
  },
  "reasoning": {
    "quality": 0.85,
    "value_signal": 0.7,
    "conflict_signal": 0.1,
    "control_signal": 0.6
  },
  "learning": {
    "capsule_id": "cap_001",
    "epoch": 5,
    "version_hash": "abc123...",
    "update_cost": 10.0,
    "update_paid": 10.0,
    "update_quality": 0.9,
    "status": "COMMITTED",
    "scarcity_streak": 0
  },
  "policy": {
    "policy_hash": "def456...",
    "decision": "ALLOW",
    "reason_codes": []
  },
  "permit_hash": null,
  "economics_hash": "ghi789...",
  "evaluated_rules": ["rule_001", "rule_002"],
  "checksum": "a3b2c1d4e5f6..."
}
```

---

## Storage and Retention

**Recommended:**
- Azure Blob Storage (immutable container)
- 7-year retention for compliance
- Indexed by: tick, capsule_id, decision, timestamp

**Access Pattern:**
- Write-once, read-many
- Query by decision outcome (DENY, ALLOW, etc.)
- Replay on-demand
