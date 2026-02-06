# Iter Control Plane — Facts Sheet

**Classification:** External-Safe  
**Audience:** Executive / Architect  
**Date:** February 2026

---

## What Iter Is

Iter is a deterministic governance control plane for AI systems.

**Core Function:**
- Evaluates governance conditions against system state
- Enforces constraints before execution
- Emits cryptographically verified DecisionPackets

**Primary Output:**
- DecisionPacket: replay-sufficient, immutable, checksummed governance artifact

---

## What Iter Is NOT

Iter does not:
- Reason, infer, plan, or learn
- Orchestrate agents or workflows
- Train models or optimize prompts
- Store long-term memory or state
- Decide business outcomes or interpret ethics
- Generate content or reasoning signals

---

## Architectural Invariants (Non-Negotiable)

1. **Determinism:** Identical inputs + configuration produce byte-identical DecisionPackets
2. **Fail-Closed:** Unknown inputs, NaN, Inf, invalid enums = immediate rejection
3. **Side-Effect Isolation:** Governance evaluation produces no external side effects
4. **Replay Stability:** DecisionPacket contains everything required to reconstruct decision without re-inference
5. **Governance Precedence:** No execution path bypasses governance evaluation

---

## Trust Boundary

**Iter Verifies (Trust Earned):**
- Input schema conformance
- Type safety, range validity, enum validity
- Float safety (NaN/Inf detection)
- Determinism (checksum verification)
- Policy integrity (cryptographic hash)

**Iter Does NOT Verify (Trust Assumed):**
- Upstream reasoning signals are accurate (checked for validity, not accuracy)
- Identity claims are authentic (delegated to Entra ID)
- Network transport is secure (delegated to Azure networking)
- Storage integrity (delegated to Azure Storage)

---

## Governance Execution Model

**FSM States:**
1. VALIDATING → 2. EVALUATING → Terminal States:
   - PERMITTED (allow)
   - DENIED (block)
   - FROZEN (learning suspended)
   - REVIEW_REQUIRED (human review)
   - DEGRADED (reduced capability)
   - REJECTED (validation failed)

**Policy Gates (Evaluated in Order):**
1. Input Quality Gate
2. Reasoning Quality Gate
3. Energy Integrity Gate
4. Economic Control Gate
5. Learning Permission Gate

**All gates pass → ALLOW**

---

## DecisionPacket Structure

**Required Fields:**
- `iter_build_hash`, `scg_build_hash`, `tick`
- `energy` (nodes, reservoir, integrity)
- `reasoning` (quality, value_signal, conflict_signal, control_signal)
- `learning` (capsule_id, epoch, status, costs, quality)
- `policy` (policy_hash, decision, reason_codes)
- `permit_hash`, `economics_hash`, `evaluated_rules`
- `checksum` (SHA-256 of canonical JSON)

**Immutability:** Once emitted, DecisionPackets cannot be modified or deleted.

**Replay Contract:** Same input → same checksum.

---

## Failure Modes

| Failure | System Response | Observability |
|---------|----------------|---------------|
| Drift (soft) | Continue + DEGRADED_MODE | AuditEvent + reason codes |
| Violation (hard) | Reject + Halt | Error response + AuditEvent |
| Replay Mismatch | Halt + Investigate | Critical alert + AuditEvent |
| Tool Misuse | Reject | JSON-RPC error + AuditEvent |

**Fail-Closed:** Unknown → Reject. No silent failures.

---

## Azure Integration

**Protocol:** MCP (Model Context Protocol) over JSON-RPC 2.0

**Deployment:**
- AKS (primary) or Azure Container Apps
- Managed Identity for service-to-service
- Entra ID for authentication (roadmap)

**Storage:**
- DecisionPackets: Azure Blob Storage (immutable)
- Audit Events: Log Analytics
- Telemetry: Application Insights

**Security:**
- Fail-closed default
- Input sanitization
- Output sanitization
- No stack traces in responses

---

## Key Differentiators

| Aspect | Iter | Typical Agent Stack |
|--------|------|---------------------|
| Determinism | Guaranteed | Probabilistic |
| Replay | Exact, checksum-verified | Best-effort |
| Audit | Native artifacts | Logs only |
| Drift Control | External governance | Prompt discipline |

---

## One-Sentence Summary

Iter makes AI decisions behave like infrastructure—deterministic, replayable, and auditable—so enterprises can trust them.

---

## Review Contact

For architecture questions: See `docs/azure-architecture-review/README.md`

For technical verification: Run `cargo test --test governance_invariants` (71 invariants)
