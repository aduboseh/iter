# Iter Reference Architecture

**Classification:** External-Safe  
**Version:** 1.0  
**Last Updated:** January 2026

---

## System Overview

Iter is a deterministic governance control plane that evaluates policy conditions, enforces constraints, and emits cryptographically verifiable DecisionPackets as governance evidence.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              CONSUMERS                                       │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐    │
│  │   AI Agent   │  │  Workflow    │  │   CLI Tool   │  │  Vertical    │    │
│  │   Runtime    │  │  Orchestrator│  │              │  │  Application │    │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘    │
│         │                 │                 │                 │             │
│         └─────────────────┴────────┬────────┴─────────────────┘             │
│                                    │                                        │
│                          JSON-RPC 2.0 / MCP                                 │
│                                    │                                        │
└────────────────────────────────────┼────────────────────────────────────────┘
                                     │
                                     ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                           ITER CONTROL PLANE                                 │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                        MCP Protocol Layer                            │   │
│  │  • JSON-RPC 2.0 request/response                                     │   │
│  │  • Tool discovery and invocation                                     │   │
│  │  • Input validation and sanitization                                 │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                        │
│                                    ▼                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                     Governance Evaluation Engine                     │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                  │   │
│  │  │   Policy    │  │  Economic   │  │  Learning   │                  │   │
│  │  │   Gates     │  │  Controls   │  │  Permits    │                  │   │
│  │  └─────────────┘  └─────────────┘  └─────────────┘                  │   │
│  │                           │                                          │   │
│  │              ┌────────────┴────────────┐                            │   │
│  │              ▼                         ▼                            │   │
│  │  ┌─────────────────────┐  ┌─────────────────────┐                   │   │
│  │  │  Deterministic      │  │  Fail-Closed        │                   │   │
│  │  │  Decision Logic     │  │  Enforcement        │                   │   │
│  │  └─────────────────────┘  └─────────────────────┘                   │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                        │
│  ┌ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─│─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┐   │
│                                    │          SCG SERVICE BOUNDARY          │
│  │     ┌───────────────────────────┴───────────────────────────┐     │   │
│        │               SCG (Separately Deployed)                │           │
│  │     │  • Pinned, versioned scg.v1 contract                  │     │   │
│        │  • Independent state and provenance                    │           │
│  │     │  • Not embedded by public Iter                        │     │   │
│        └───────────────────────────────────────────────────────┘           │
│  └ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─┘   │
│                                    │                                        │
└────────────────────────────────────┼────────────────────────────────────────┘
                                     │
                    ┌────────────────┼────────────────┐
                    │                │                │
                    ▼                ▼                ▼
┌──────────────────────┐ ┌──────────────────┐ ┌──────────────────────────────┐
│   DECISION OUTPUT    │ │   AUDIT STREAM   │ │      TELEMETRY               │
│  ┌────────────────┐  │ │  ┌────────────┐  │ │  ┌─────────────────────────┐ │
│  │ DecisionPacket │  │ │  │AuditEvent  │  │ │  │ TraceContext propagation│ │
│  │ • State hash   │  │ │  │• Phase     │  │ │  │ Metrics (latency, etc.) │ │
│  │ • Policy hash  │  │ │  │• Outcome   │  │ │  │ Structured logs         │ │
│  │ • Reason codes │  │ │  │• Timestamp │  │ │  └─────────────────────────┘ │
│  │ • SHA-256 sum  │  │ │  └────────────┘  │ │                              │
│  └────────────────┘  │ │  JSON Lines      │ │                              │
│  Canonical JSON      │ │  (append-only)   │ │                              │
└──────────────────────┘ └──────────────────┘ └──────────────────────────────┘
```

---

## Data Flow

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           REQUEST FLOW                                       │
│                                                                             │
│   Client                    Iter Server                    Output           │
│     │                            │                            │             │
│     │  1. JSON-RPC Request       │                            │             │
│     │  (tools/call)              │                            │             │
│     │ ──────────────────────────►│                            │             │
│     │                            │                            │             │
│     │                     2. Input Validation                 │             │
│     │                     3. Policy Evaluation                │             │
│     │                     4. Governance Decision              │             │
│     │                            │                            │             │
│     │                            │  5. Emit DecisionPacket    │             │
│     │                            │ ──────────────────────────►│             │
│     │                            │                            │             │
│     │                            │  6. Emit AuditEvent        │             │
│     │                            │ ──────────────────────────►│             │
│     │                            │                            │             │
│     │  7. JSON-RPC Response      │                            │             │
│     │◄──────────────────────────│                            │             │
│     │                            │                            │             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Mermaid Diagram (for tooling)

```mermaid
flowchart TB
    subgraph Consumers["Consumers"]
        Agent["AI Agent Runtime"]
        Orch["Workflow Orchestrator"]
        CLI["CLI Tool"]
        App["Vertical Application"]
    end

    subgraph IterControlPlane["Iter Control Plane"]
        MCP["MCP Protocol Layer<br/>JSON-RPC 2.0"]
        
        subgraph Governance["Governance Evaluation"]
            Policy["Policy Gates"]
            Econ["Economic Controls"]
            Learn["Learning Permits"]
        end
        
        Decision["Deterministic Decision Logic"]
        Failsafe["Fail-Closed Enforcement"]
        
        subgraph Sealed["Sealed IP Boundary"]
            Substrate["Execution Substrate<br/>(Private/Licensed)"]
        end
    end

    subgraph Outputs["Outputs"]
        DP["DecisionPacket<br/>Canonical JSON + SHA-256"]
        Audit["AuditEvent Stream<br/>JSON Lines"]
        Telem["Telemetry<br/>TraceContext + Metrics"]
    end

    Consumers --> MCP
    MCP --> Governance
    Governance --> Decision
    Decision --> Failsafe
    Failsafe --> Substrate
    
    Substrate --> DP
    Substrate --> Audit
    Substrate --> Telem

    style Sealed fill:#f9f,stroke:#333,stroke-dasharray: 5 5
```

---

## Key Boundaries

| Boundary | What Crosses | What Does NOT Cross |
|----------|--------------|---------------------|
| MCP Protocol | JSON-RPC requests/responses, tool invocations | Internal state, raw policy inputs |
| Governance Output | DecisionPackets, AuditEvents, reason codes | Reasoning math, heuristics, weights |
| Sealed IP | Nothing (isolated) | Substrate internals, SCG logic |

---

## Security Posture

- **Fail-closed**: Unknown inputs rejected; invalid floats (NaN/Inf) cause hard errors
- **Deterministic**: Identical inputs produce byte-identical outputs
- **Auditable**: Every decision has a cryptographic checksum and reason codes
- **Sanitized**: No stack traces, internal paths, or debug info in responses

---

## Not Shown (Intentionally)

This diagram does NOT show:
- SCG (Substrate Compute Graph) internals
- Reasoning signal computation
- Proprietary invariant logic
- Performance/optimization details

SCG implementation evidence is produced in the SCG repository. Iter does not claim private-CI certification for an unavailable embedded substrate.

---

## System Invariants

### Execution-Blocking Semantics

**Failure Mode Guarantee:** Iter unavailability results in execution denial, not degraded execution.

```
┌────────────────────────────────────────────────────────────────┐
│                    FAILURE MODE BEHAVIOR                        │
├────────────────────────────────────────────────────────────────┤
│ Condition                    │ Result                          │
├──────────────────────────────┼─────────────────────────────────┤
│ Iter service unavailable     │ Consumer execution DENIED       │
│ Policy evaluation timeout    │ Consumer execution DENIED       │
│ Invalid policy state         │ Consumer execution DENIED       │
│ Identity verification failure│ Consumer execution DENIED       │
│ Network partition            │ Consumer execution DENIED       │
└────────────────────────────────────────────────────────────────┘
```

**Design Principle:** Deny-by-default is architecturally enforced. No execution path exists that bypasses governance evaluation.

**Operational Impact:**
- Iter operates as a control plane dependency, not an optional enhancement
- Consumers must implement circuit-breaker patterns for Iter availability
- High-availability deployment (multi-AZ, health-checked) is required for production use

This fail-safe posture ensures AI systems cannot operate outside governance boundaries under any failure scenario.
