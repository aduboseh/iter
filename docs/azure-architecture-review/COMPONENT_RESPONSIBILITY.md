# Iter Component Responsibility Map

**Classification:** External-Safe  
**Version:** 1.0  
**Last Updated:** January 2026

---

## Overview

This document defines what Iter is responsible for versus what it delegates to external systems or does not handle.

---

## Iter Responsibilities

### Core Governance (ITER OWNS)

| Component | Responsibility | Guarantee |
|-----------|---------------|-----------|
| **Policy Evaluation** | Evaluate governance conditions against system state | Deterministic, fail-closed |
| **Decision Emission** | Produce DecisionPackets with full causality | Replay-sufficient, checksummed |
| **Learning Gate** | Permit or deny learning based on policy | Explicit status codes |
| **Economic Enforcement** | Enforce budget and permit constraints | Hard limits, no overdraft |
| **Audit Trail** | Emit structured AuditEvents | Append-only, timestamped |
| **Input Validation** | Reject malformed, out-of-range, or invalid inputs | Fail-closed with error codes |
| **Protocol Compliance** | MCP 2024-11-05, JSON-RPC 2.0 | Versioned, backward-compatible |

### Determinism Guarantees (ITER OWNS)

| Property | Description |
|----------|-------------|
| **Byte-identical outputs** | Same inputs + config → same DecisionPacket bytes |
| **Canonical serialization** | JSON keys sorted, floats normalized, nulls explicit |
| **Checksum verification** | SHA-256 over canonical form; mismatch = hard error |
| **Replay without re-learning** | DecisionPacket contains everything to reconstruct decision |

### Security Boundaries (ITER OWNS)

| Boundary | Enforcement |
|----------|-------------|
| **Input sanitization** | All inputs validated before processing |
| **Output sanitization** | No internal state, paths, or stack traces in responses |
| **Error taxonomy** | Exhaustive, documented error codes; no catch-all errors |
| **Fail-closed default** | Unknown enum values, NaN, Inf → immediate rejection |

---

## External System Responsibilities

### Consumer Systems (NOT ITER)

| System | Responsibility |
|--------|---------------|
| **AI Agent Runtime** | Generate proposals, interpret decisions, execute actions |
| **Workflow Orchestrator** | Sequence calls, handle retries, manage state machine |
| **Application Layer** | Present results to users, implement business logic |
| **Upstream Models** | Produce reasoning signals, learning proposals |

### Infrastructure (NOT ITER)

| Component | Responsibility |
|-----------|---------------|
| **Compute Platform** | Container runtime, scaling, availability |
| **Identity Provider** | Authentication, token issuance, identity lifecycle |
| **Secrets Management** | Store and rotate API keys, certificates |
| **Monitoring Stack** | Collect metrics, aggregate logs, alert on anomalies |
| **Network Security** | TLS termination, firewall rules, DDoS protection |

### Data Persistence (SHARED)

| Data Type | Iter Responsibility | External Responsibility |
|-----------|--------------------|-----------------------|
| **DecisionPackets** | Emit canonical JSON | Store, index, query |
| **AuditEvents** | Emit JSON Lines stream | Ingest, retain, archive |
| **Telemetry** | Emit TraceContext, metrics | Collect, aggregate, visualize |

---

## Explicit Non-Responsibilities

Iter does **NOT**:

| Activity | Why Not |
|----------|---------|
| **Train models** | Iter is a governance layer, not a learning system |
| **Generate content** | Iter evaluates decisions, does not produce text/media |
| **Store user data** | Iter is stateless per-request; persistence is external |
| **Orchestrate workflows** | Iter is a single decision point, not a coordinator |
| **Interpret ethics** | Iter enforces policy; policy authoring, ethical interpretation, and normative judgment are external responsibilities |
| **Compute reasoning signals** | Upstream systems provide signals; Iter evaluates them |

---

## Component Boundary Diagram

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        CONSUMER RESPONSIBILITY                              │
│  • Generate proposals           • Interpret decisions                       │
│  • Execute actions              • Manage workflow state                     │
└─────────────────────────────────────────────────────────────────────────────┘
                                     │
                                     ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                         ITER RESPONSIBILITY                                  │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ • Input validation              • Policy evaluation                  │   │
│  │ • Governance decision           • DecisionPacket emission            │   │
│  │ • Audit event emission          • Determinism enforcement            │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
                                     │
                                     ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                      INFRASTRUCTURE RESPONSIBILITY                          │
│  • Compute (AKS/Container Apps)    • Identity (Entra ID)                   │
│  • Secrets (Key Vault)             • Monitoring (Azure Monitor)            │
│  • Storage (Blob/Table)            • Networking (NSG, WAF)                 │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## SDK Responsibility Boundary

| SDK Responsibility | Server Responsibility |
|-------------------|----------------------|
| Protocol serialization | Protocol parsing |
| Version negotiation (client-side) | Version validation (server-side) |
| TraceContext injection | TraceContext propagation |
| Error handling (client) | Error generation (server) |
| Connection management | Request processing |

SDKs are **thin clients**: they wrap the protocol but contain no business logic.

SDKs do not cache decisions, alter outputs, or bypass server-side governance.

---

## Responsibility Matrix (RACI)

| Activity | Iter | Consumer | Infrastructure | Policy Author |
|----------|------|----------|----------------|---------------|
| Define policy rules | - | - | - | **A/R** |
| Evaluate policy | **A/R** | I | - | C |
| Generate proposals | I | **A/R** | - | - |
| Make governance decision | **A/R** | I | - | - |
| Execute permitted action | I | **A/R** | - | - |
| Store decision artifacts | C | - | **A/R** | - |
| Scale compute | - | - | **A/R** | - |
| Manage identity | - | - | **A/R** | - |

**A** = Accountable, **R** = Responsible, **C** = Consulted, **I** = Informed
