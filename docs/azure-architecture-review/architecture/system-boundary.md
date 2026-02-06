# Iter System Boundary

**Classification:** External-Safe  
**Version:** 1.0  
**Last Updated:** February 2026

---

## Purpose

This document defines the boundary between Iter's responsibilities and external system responsibilities. Violating this boundary is a design error.

---

## Boundary Definition

### Inside the Boundary (Iter Owns)

| Responsibility | Enforcement Mechanism |
|----------------|----------------------|
| Input validation | Schema enforcement, type checking, range validation |
| Governance evaluation | Deterministic policy execution |
| Decision emission | DecisionPacket generation with SHA-256 checksum |
| Audit event generation | Structured JSON Lines output |
| Fail-closed enforcement | Hard rejection of invalid inputs |
| Determinism guarantee | Canonical serialization, replay verification |

### Outside the Boundary (Iter Does NOT Own)

| Responsibility | Owned By |
|----------------|----------|
| Proposal generation | Upstream models, AI agents |
| Action execution | Consumer systems |
| Business logic | Application layer |
| Ethical interpretation | Policy authors, human reviewers |
| Long-term storage | External persistence layer (Azure Storage, etc.) |
| Network transport | Infrastructure (Azure networking) |
| Identity management | Entra ID, Managed Identity |
| Orchestration | Workflow systems, agent runtimes |

---

## Ingress Points

### MCP Protocol Layer

```
INPUT (Consumer → Iter):
- JSON-RPC 2.0 request
- Tool name + parameters
- TraceContext (optional)

VALIDATION (Iter):
- Schema conformance
- Type safety
- Range bounds
- Enum validity
- Float safety (NaN/Inf rejected)

OUTCOME:
- Valid → proceed to governance evaluation
- Invalid → reject with error code
```

**Key Invariant:** No execution occurs before validation completes.

---

## Egress Points

### DecisionPacket Output

```
OUTPUT (Iter → Consumer):
- DecisionPacket (canonical JSON)
- SHA-256 checksum
- Reason codes
- Governance status

GUARANTEE:
- Same input → same DecisionPacket
- Same DecisionPacket → same checksum
- Checksum mismatch = corruption detected
```

### Audit Event Stream

```
OUTPUT (Iter → Audit Sink):
- AuditEvent (JSON Lines)
- Phase markers
- Timestamps
- Outcome codes

GUARANTEE:
- Every governance evaluation emits AuditEvent
- AuditEvents are append-only
- No AuditEvent deletion or modification
```

---

## Boundary Violation Table

| Violation | Example | Outcome |
|-----------|---------|---------|
| Consumer bypasses validation | Direct internal API call | Architectural error, not supported |
| Iter executes actions | Iter calls external API | Design violation, never occurs |
| Iter stores state | Iter writes to internal database | Stateless guarantee violated |
| Consumer modifies DecisionPacket | Altering checksum | Replay verification fails |
| Iter interprets ethics | Iter overrides policy | Policy authority violated |

**Rule:** If Iter performs an activity outside its boundary, execution MUST halt.

---

## Boundary Enforcement

### At Build Time

- Type system prevents stateful operations
- No network client libraries in server binary
- No database write paths in governance logic

### At Runtime

- Input sanitization rejects malformed data
- Output sanitization prevents information leakage
- Error taxonomy prevents catch-all errors

### At Audit Time

- DecisionPacket replay verifies boundary integrity
- AuditEvent correlation detects unauthorized paths

---

## Boundary Diagram

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        OUTSIDE ITER BOUNDARY                                 │
│                                                                             │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐            │
│  │   AI Agent      │  │  Workflow       │  │  Application    │            │
│  │   Runtime       │  │  Orchestrator   │  │  Layer          │            │
│  └────────┬────────┘  └────────┬────────┘  └────────┬────────┘            │
│           │                    │                    │                      │
│           └────────────────────┴────────────────────┘                      │
│                                │                                           │
│                                │ JSON-RPC Request                          │
│                                ▼                                           │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                    ITER BOUNDARY (MCP Layer)                        │   │
│  │  ┌────────────────┐  ┌────────────────┐  ┌────────────────┐       │   │
│  │  │   Input        │  │   Governance   │  │   Output       │       │   │
│  │  │   Validation   │  │   Evaluation   │  │   Emission     │       │   │
│  │  └────────────────┘  └────────────────┘  └────────────────┘       │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                │                                           │
│                                │ DecisionPacket                            │
│                                ▼                                           │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                    OUTSIDE ITER BOUNDARY                            │   │
│  │  ┌────────────────┐  ┌────────────────┐  ┌────────────────┐       │   │
│  │  │   Storage      │  │   Audit Sink   │  │   Telemetry    │       │   │
│  │  │   (Blob)       │  │   (Log Analyt) │  │   (App Insights)       │   │
│  │  └────────────────┘  └────────────────┘  └────────────────┘       │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Verification

**Claim:** Iter operates within this boundary.

**Verification Method:**
1. Read DecisionPacket from archive
2. Extract inputs and configuration
3. Replay governance evaluation
4. Compare checksums

**Pass Condition:** Checksums match.

**Fail Condition:** Checksum mismatch indicates boundary violation or corruption.

---

## Boundary Change Process

Any change to this boundary definition is a breaking architectural change and requires:
1. Explicit justification
2. Impact analysis on determinism guarantees
3. Replay verification of existing DecisionPackets
4. Update to version documentation

No silent boundary expansion is permitted.
