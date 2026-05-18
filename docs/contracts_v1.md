# Iter-SCG Contracts Specification v1.0

**Status**: STABLE  
**Effective**: 2026-01-11  
**Applies to**: iter_mcp_server >= 1.0.2, SCG-CTX-03, SCG-INT-04

## 1. Versioning

### 1.1 Packet Version
```
PACKET_VERSION = "1.0"
```
Every `DecisionPacket` MUST include an `iter_build_hash` that encodes the producing version.
Consumers MUST reject packets with unknown major versions.

### 1.2 Contract Version
```
CONTRACT_VERSION = "1.0"
```
The contract version governs:
- Envelope field sets
- Enum value sets
- Canonicalization rules
- Checksum algorithm

### 1.3 Evolution Rules
- **PATCH** (1.0.x): Bug fixes, documentation. No field or enum changes.
- **MINOR** (1.x.0): Additive fields only. New enum values require minor bump.
- **MAJOR** (x.0.0): Breaking changes. Old readers MUST reject with `UNKNOWN_CONTRACT_VERSION`.

New enum values added in minor versions:
- Old readers that encounter unknown values MUST fail closed with `UNKNOWN_ENUM` error
- The error MUST include the field name and unknown value

## 2. Canonicalization Rules

### 2.1 JSON Encoding
All checksummed payloads use **Canonical JSON** with these rules:

1. **Key ordering**: Lexicographic ASCII sort (a-z, case-sensitive)
2. **No whitespace**: No spaces, newlines, or indentation
3. **Float encoding**: 
   - No trailing zeros after decimal point
   - No leading zeros before decimal point
   - Scientific notation for |value| >= 1e21 or |value| < 1e-6
   - NaN and Infinity are FORBIDDEN (hard error before serialization)
4. **String escaping**: Minimal escaping (only required: `"`, `\`, control chars)
5. **Null handling**: Explicit `null` for Option::None fields (not omitted)
6. **Array ordering**: Preserved as-is (order is semantic)

### 2.2 Checksum Algorithm
```
SHA-256(canonical_json_bytes) -> lowercase hex string (64 chars)
```

### 2.3 Field Ordering for DecisionPacket Canonical Form
Fields MUST appear in this order for checksum computation:
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

## 3. Envelope Specifications

### 3.1 EnergyEnvelope
| Field | Type | Range | Required |
|-------|------|-------|----------|
| nodes | f64 encoded as `ieee754-f64-bits-lowerhex` | [0.0, +∞) | yes |
| reservoir | f64 encoded as `ieee754-f64-bits-lowerhex` | [0.0, +∞) | yes |
| integrity | f64 encoded as `ieee754-f64-bits-lowerhex` | [0.0, 1.0] | yes |

Validation: NaN/Inf = hard error `INVALID_FLOAT`

### 3.2 ReasoningEnvelope
| Field | Type | Range | Required |
|-------|------|-------|----------|
| quality | f64 encoded as `ieee754-f64-bits-lowerhex` | [0.0, 1.0] | yes |
| value_signal | f64 encoded as `ieee754-f64-bits-lowerhex` | [0.0, 1.0] | yes |
| conflict_signal | f64 encoded as `ieee754-f64-bits-lowerhex` | [0.0, 1.0] | yes |
| control_signal | f64 encoded as `ieee754-f64-bits-lowerhex` | [0.0, 1.0] | yes |

Validation: NaN/Inf = hard error `INVALID_FLOAT`

### 3.3 LearningEnvelope
| Field | Type | Range | Required |
|-------|------|-------|----------|
| capsule_id | string | any | yes |
| epoch | u64 | [0, 2^64) | yes |
| version_hash | string | 64 hex chars | yes |
| update_cost | f64 encoded as `ieee754-f64-bits-lowerhex` | [0.0, +∞) | yes |
| update_paid | f64 encoded as `ieee754-f64-bits-lowerhex` | [0.0, +∞) | yes |
| update_quality | f64 encoded as `ieee754-f64-bits-lowerhex` | [0.0, 1.0] | yes |
| status | LearningStatus | enum | yes |
| scarcity_streak | u64 | [0, 2^64) | yes |

### 3.4 PolicyEnvelope
| Field | Type | Range | Required |
|-------|------|-------|----------|
| policy_hash | string | 64 hex chars | yes |
| decision | PolicyDecision | enum | yes |
| reason_codes | string[] | ordered | yes |

Invariant: `reason_codes` MUST NOT be empty when `decision` ∈ {DENY, FREEZE_LEARNING, DEGRADED_MODE, REQUIRE_REVIEW}

### 3.5 SystemState
| Field | Type | Required |
|-------|------|----------|
| tick | u64 | yes |
| energy | EnergyEnvelope | yes |
| reasoning | ReasoningEnvelope | yes |
| learning | LearningEnvelope | yes |
| policy | PolicyEnvelope | yes |

## 4. Enum Value Registry

### 4.1 LearningStatus (closed)
| Value | Code | Description |
|-------|------|-------------|
| COMMITTED | 0 | Update successfully committed |
| NO_PROPOSAL_NO_DELTA | 1 | No change needed (delta below threshold) |
| REJECTED_INPUT_QUALITY | 2 | Cortex input quality below threshold |
| REJECTED_SCARCITY | 3 | Insufficient energy to fund update |
| REJECTED_INTEGRITY | 4 | Hash verification or arithmetic error |

Unknown values: MUST fail with `UNKNOWN_ENUM`

### 4.2 PolicyDecision (closed)
| Value | Code | Description |
|-------|------|-------------|
| ALLOW | 0 | Action permitted |
| DENY | 1 | Action blocked |
| FREEZE_LEARNING | 2 | Learning suspended |
| DEGRADED_MODE | 3 | Reduced capability due to quality |
| REQUIRE_REVIEW | 4 | Human review required |

Unknown values: MUST fail with `UNKNOWN_ENUM`

## 5. Reason Code Registry

### 5.1 Policy Gate Codes
| Code | Gate | Condition |
|------|------|-----------|
| REASONING_QUALITY_BELOW_THRESHOLD | ReasoningQualityGate | quality < threshold |
| ENERGY_INTEGRITY_BELOW_THRESHOLD | EnergyIntegrityGate | integrity < threshold |
| INPUT_QUALITY_INSUFFICIENT | InputQualityGate | input quality < threshold |
| INTEGRITY_VIOLATION | LearningPermissionGate | status == RejectedIntegrity |
| SCARCITY_STREAK_EXCEEDED | LearningPermissionGate | scarcity_streak >= max |
| LEARNING_QUALITY_BELOW_THRESHOLD | LearningQualityGate | update_quality < threshold |

### 5.2 Economics Codes
| Code | Source | Condition |
|------|--------|-----------|
| WINDOW_BUDGET_EXCEEDED | EconomicsController | spent + cost > max_per_window |
| PERMIT_EXPIRED | EconomicsController | tick >= expiry_tick |
| CAPSULE_NOT_PERMITTED | EconomicsController | capsule not in allowed list |
| PERMIT_BUDGET_EXCEEDED | EconomicsController | spent + cost > permit.max |

### 5.3 Validation Codes
| Code | Source | Condition |
|------|--------|-----------|
| INVALID_FLOAT_NAN | validate_bounded_float | value.is_nan() |
| INVALID_FLOAT_INF | validate_bounded_float | value.is_infinite() |
| INVALID_FLOAT_RANGE | validate_bounded_float | value < min or value > max |
| INVALID_HASH_LENGTH | validate_hash | len != 64 |
| INVALID_HASH_CHARS | validate_hash | non-hex characters |
| UNKNOWN_ENUM | from_str_closed | unrecognized value |

## 6. DecisionPacket Specification

### 6.1 Required Fields
| Field | Type | Description |
|-------|------|-------------|
| iter_build_hash | string | Iter version identifier |
| scg_build_hash | string | SCG version identifier |
| tick | u64 | Decision tick |
| energy | EnergyEnvelope | Energy state at decision |
| reasoning | ReasoningEnvelope | Reasoning state at decision |
| learning | LearningEnvelope | Learning state at decision |
| policy | PolicyEnvelope | Policy evaluation result |
| permit_hash | string? | Active permit hash (null if none) |
| economics_hash | string | Economics config hash |
| evaluated_rules | string[] | Rule IDs evaluated (ordered) |
| checksum | string | SHA-256 of canonical form |

### 6.2 Replay Contract (INV-ITER-05)
A `DecisionPacket` MUST contain everything needed to:
1. Determine what the system knew at decision time
2. Verify whether learning was allowed
3. Explain why learning did or did not occur
4. Identify which policy decided the outcome

Verification: Given identical packet content (excluding checksum), recomputing checksum MUST yield identical result. Packet verification MUST reject malformed numeric hex, NaN/Inf bit patterns, and out-of-range decoded values before trusting a packet.

### 6.3 Checksum Verification
```
1. Parse packet JSON
2. Decode and validate proof-critical numeric fields from 16-character lowercase IEEE-754 hex strings
3. Extract checksum field
4. Remove checksum field from object
5. Canonicalize remaining fields per Section 2
6. Compute SHA-256 of canonical bytes
7. Compare computed hash to extracted checksum
8. Mismatch = hard error CHECKSUM_MISMATCH
```

## 7. Error Codes

| Code | Numeric | Description |
|------|---------|-------------|
| UNKNOWN_CONTRACT_VERSION | 1000 | Contract version not supported |
| UNKNOWN_ENUM | 1001 | Unrecognized enum value |
| INVALID_FLOAT_NAN | 1002 | NaN value in float field |
| INVALID_FLOAT_INF | 1003 | Infinite value in float field |
| INVALID_FLOAT_RANGE | 1004 | Float out of valid range |
| INVALID_HASH | 1005 | Hash format invalid |
| CHECKSUM_MISMATCH | 1006 | Packet checksum verification failed |
| MISSING_REASON_CODES | 1007 | Reject/freeze decision without reason |

All errors MUST include:
- Error code (string and numeric)
- Field name (if applicable)
- Actual value (if safe to expose)
- Expected range/format (if applicable)

## 8. Compliance Checklist

For a consumer to be spec-compliant:
- [ ] Reject unknown contract versions
- [ ] Fail closed on unknown enum values
- [ ] Validate all floats for NaN/Inf before processing
- [ ] Verify DecisionPacket checksum before trusting content
- [ ] Require reason_codes on non-ALLOW decisions
- [ ] Use specified canonicalization for any local checksum computation
