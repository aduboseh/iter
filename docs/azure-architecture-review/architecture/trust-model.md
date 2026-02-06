# Iter Trust Model

**Classification:** External-Safe  
**Version:** 1.0  
**Last Updated:** February 2026

---

## Purpose

This document defines where Iter earns trust through verification versus where it must assume trust from external systems.

---

## Trust Boundaries

### What Iter Verifies (Trust Earned)

| Claim | Verification Method | Failure Mode |
|-------|-------------------|--------------|
| Input schema conformance | JSON schema validation | Reject with error code |
| Type safety | Runtime type checks | Reject with error code |
| Range validity | Explicit bound checks | Reject with error code |
| Enum validity | Exhaustive enum matching | Reject with error code |
| Float safety | NaN/Inf detection | Reject with error code |
| Determinism | Checksum verification | Replay mismatch = halt |
| Policy integrity | Cryptographic hash | Policy corruption = halt |

### What Iter Does NOT Verify (Trust Assumed)

| Assumption | Trusted Source | Risk Mitigation |
|------------|----------------|-----------------|
| Upstream reasoning signals are accurate | AI models, agents | Policy gates check signal validity, not accuracy |
| Identity claims are authentic | Entra ID, Managed Identity | Delegated to Azure identity infrastructure |
| Network transport is secure | TLS, Azure networking | Delegated to infrastructure layer |
| Clock accuracy | System clock | Timestamps are audit markers, not security primitives |
| Storage integrity | Azure Storage, Cosmos DB | Delegated to Azure service guarantees |

---

## Cryptographic Guarantees

### DecisionPacket Checksum

```
INPUT:
- Canonical JSON representation
- Policy hash
- State hash

OPERATION:
- SHA-256(canonical_json)

OUTPUT:
- 256-bit checksum

GUARANTEE:
- Collision-resistant
- Any input change produces different checksum
- Checksum match = byte-identical input
```

**Checksum is NOT:**
- A signature (no private key)
- Authentication (no identity binding)
- Encryption (content is readable)

**Checksum IS:**
- Integrity proof
- Replay anchor
- Corruption detector

### Policy Hash

```
OPERATION:
- SHA-256(policy_definition)

GUARANTEE:
- Policy change = hash change
- Hash match = policy unchanged
```

---

## Human Review Requirements

### Iter Cannot Replace Human Judgment

| Decision Type | Iter Role | Human Role |
|---------------|----------|------------|
| Policy definition | Enforce | Author |
| Ethical interpretation | N/A | Interpret |
| Risk tolerance | N/A | Set thresholds |
| Business impact | N/A | Assess |
| Compliance posture | Audit trail | Review |

### When Human Review is Required

- Before deploying new policies
- When reviewing governance denials
- During incident investigation
- For compliance reporting
- When adjusting risk thresholds

Iter provides audit artifacts. Humans provide judgment.

---

## Trust Propagation

### Transitive Trust (Dangerous)

```
ANTI-PATTERN:
Consumer → Iter → Upstream Model → Iter → Action

RISK:
- Circular dependency
- Trust amplification
- Hidden assumptions
```

**Iter does NOT amplify trust.**

### Explicit Trust (Safe)

```
CORRECT PATTERN:
Consumer → Upstream Model
Consumer → Iter (validates model output)
Iter → Consumer (governance decision)
Consumer → Action (if permitted)

BENEFIT:
- Explicit trust boundaries
- No hidden dependencies
- Auditable flow
```

---

## Zero-Trust Posture

### Deny-by-Default

**Rule:** Unknown inputs are rejected, not assumed safe.

| Input Type | Unknown Value | Iter Response |
|------------|---------------|---------------|
| Enum | Unrecognized variant | Reject |
| Float | NaN or Inf | Reject |
| Schema | Missing required field | Reject |
| Range | Out-of-bounds | Reject |

**No fallback behavior exists.**

### Explicit Allow-Lists

**Rule:** Only explicitly permitted inputs proceed to evaluation.

```
ANTI-PATTERN (implicit):
if input.is_safe():
    proceed()

CORRECT PATTERN (explicit):
match input {
    KnownSafeValue => proceed(),
    _ => reject()
}
```

---

## Trust Verification

### At Request Time

1. Validate input schema
2. Check type safety
3. Verify range bounds
4. Confirm enum membership
5. Proceed to governance evaluation

**Any failure → reject immediately.**

### At Replay Time

1. Extract DecisionPacket from archive
2. Extract inputs and configuration
3. Recompute decision
4. Compare checksums

**Checksum mismatch → trust violation detected.**

---

## Trust Boundaries with Azure Services

### Azure Services Iter Trusts

| Service | Trust Assumption | Mitigation |
|---------|-----------------|------------|
| Entra ID | Identity claims are valid | Microsoft's security guarantees |
| Managed Identity | Token exchange is secure | Azure platform guarantees |
| Key Vault | Secrets are protected | HSM-backed storage |
| Azure Storage | Data is persisted durably | Azure SLA + immutable blobs |

### Azure Services Iter Does NOT Trust

| Service | Why Not | Iter Behavior |
|---------|---------|---------------|
| None (special case) | N/A | Iter validates all inputs from any source |

**Key Principle:** Iter validates inputs regardless of source reputation.

---

## Trust vs Verification Matrix

| Claim | Trust | Verify | Method |
|-------|-------|--------|--------|
| Input schema | No | Yes | JSON schema validation |
| Determinism | No | Yes | Checksum comparison |
| Identity | Yes | No | Delegated to Entra ID |
| Storage | Yes | No | Delegated to Azure Storage |
| Network | Yes | No | Delegated to Azure networking |
| Clock | Yes | No | System clock assumed accurate |
| Policy intent | Yes | No | Policy author responsibility |

---

## Failure to Establish Trust

### Untrusted Input

```
REQUEST:
{
  "tool": "node.create",
  "parameters": {
    "value": NaN
  }
}

ITER RESPONSE:
{
  "error": {
    "code": "INVALID_FLOAT",
    "message": "Float value NaN is not permitted"
  }
}
```

### Checksum Mismatch

```
REPLAY OPERATION:
1. Read DecisionPacket from archive
2. Extract inputs
3. Recompute decision
4. Compare checksums

RESULT:
- Expected: a3b2c1...
- Actual:   a3b2d1...

ITER RESPONSE:
Checksum mismatch detected. Trust boundary violated.
```

---

## Trust Model Evolution

Any change to this trust model is a breaking architectural change and requires:
1. Explicit security review
2. Impact analysis on determinism guarantees
3. Update to threat model
4. Documentation update

No silent trust boundary changes are permitted.
