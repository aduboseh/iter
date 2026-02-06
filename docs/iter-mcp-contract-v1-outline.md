# Iter MCP Contract v1 — Outline

**APEX DIRECTIVE:** ITER-MCP-TOOL-SURFACE v1 — Phase 0 (updated Phase 1)  
**Status:** Phase 1 aliases ACTIVE  
**Date:** 2026-02-05 (Phase 0) / 2026-02-06 (Phase 1)

---

## Overview

Iter is a deterministic Policy Decision Point (PDP) that evaluates governance conditions and produces immutable, checksummed decision artifacts.

### Naming Families

All public MCP tools follow these canonical families:

- **`decision.*`** — Decision gates, simulations, and explanations
- **`audit.*`** — Audit trail search, export, and replay
- **`governance.*`** / **`governor.*`** — Health and operational status
- **`kernel.*`** — Internal syscall-like operations (non-public)

---

## Canonical Tool Set (Target Public Surface)

### Decision Tools (`decision.*`)

| Canonical ID | Current Implementation | Phase | Description |
|--------------|----------------------|-------|-------------|
| `decision.check` | `governance.evaluate` | 1 (alias) | Authoritative PDP gate. Evaluates governance proposal and returns verdict (ALLOW/DENY). |
| `decision.preview` | *(to be added)* | 2 | Non-authoritative simulation. Returns projected decision without committing. |
| `decision.explain` | *(to be added)* | 3 | Structured explanation of decision rationale with policy trace. |

### Audit Tools (`audit.*`)

| Canonical ID | Current Implementation | Phase | Description |
|--------------|----------------------|-------|-------------|
| `audit.search` | *(to be added)* | 2 | Search decisions by criteria (time range, principal, action, outcome). |
| `audit.export` | `esv.audit` | 1 (alias) | Export audit bundle for compliance/archival. Currently node-scoped; may expand to full bundles. |
| `audit.replay` | `lineage.replay` | 1 (alias) | Deterministic replay of decision history. Verifies checksums and reconstructs lineage. |

### Health Tools (`governance.*` / `governor.*`)

| Canonical ID | Current Implementation | Phase | Description |
|--------------|----------------------|-------|-------------|
| `governance.health` | `governance.status` | 1 (alias) | Governance subsystem health summary. |
| `governor.health` | `governor.status` | 1 (alias) | Governor coherence and drift metrics. |

### Kernel Tools (`kernel.*`)

| Canonical ID | Current Implementation | Phase | Description |
|--------------|----------------------|-------|-------------|
| `kernel.node.create` | `node.create` | 1 (alias) | Internal: create graph node with belief/energy. |
| `kernel.node.query` | `node.query` | 1 (alias) | Internal: query node state. |
| `kernel.node.mutate` | `node.mutate` | 4 (restrict) | Debug-only: unsafe state mutation. Must be gated in production. |
| `kernel.edge.bind` | `edge.bind` | 1 (alias) | Internal: bind edge between nodes. |
| `kernel.edge.propagate` | `edge.propagate` | 1 (alias) | Internal: deterministic propagation step. |

---

## Legacy-to-Canonical Mapping (Alias Plan)

**Phase 1 aliases are ACTIVE as of 2026-02-06.** Legacy IDs remain functional but are marked deprecated.

| Legacy ID | Canonical ID | Status | Retire By |
|-----------|--------------|--------|-----------|
| `governance.evaluate` | `decision.check` | **ACTIVE** (Phase 1) | v3.0 |
| `governance.status` | `governance.health` | **ACTIVE** (Phase 1) | v3.0 |
| `governor.status` | `governor.health` | **ACTIVE** (Phase 1) | v3.0 |
| `esv.audit` | `audit.export` | **ACTIVE** (Phase 1) | v3.0 |
| `lineage.replay` | `audit.replay` | **ACTIVE** (Phase 1) | v3.0 |
| `node.create` | `kernel.node.create` | Planned (Phase 3) | v3.0 |
| `node.query` | `kernel.node.query` | Planned (Phase 3) | v3.0 |
| `node.mutate` | `kernel.node.mutate` | Planned (Phase 3) | v3.0 (remove in production mode) |
| `edge.bind` | `kernel.edge.bind` | Planned (Phase 3) | v3.0 |
| `edge.propagate` | `kernel.edge.propagate` | Planned (Phase 3) | v3.0 |

**Alias Semantics:**
- Aliases are pointers only; canonical IDs own semantics.
- Both legacy and canonical IDs invoke identical behavior during transition period.
- Deprecation warnings are emitted when legacy IDs are used via SDKs.
- `deprecated: true` metadata applies to all legacy IDs with active aliases.
- `retire_by_version: v3.0` for all deprecated IDs.

---

## Core Artifacts

### DecisionPacket

Immutable governance decision artifact.

**Required Fields:**
```json
{
  "decision_packet_version": "1.0",
  "id": "<unique_decision_id>",
  "request": {
    "principal": "<identity>",
    "action": "<requested_action>",
    "resource": "<resource_uri>",
    "context": {}
  },
  "decision": "ALLOW" | "DENY" | "DEFER",
  "constraints": [],
  "obligations": [],
  "policy_trace": {
    "policy_id": "<policy_hash>",
    "rules_evaluated": ["<rule_ids>"],
    "reason_codes": ["<codes>"]
  },
  "checksum": "<sha256>",
  "timestamp": "<iso8601>"
}
```

**PEP Statement:**
> Iter does not enforce obligations. Obligations are advisory only; enforcement is the responsibility of the Policy Enforcement Point (PEP) caller.

### AuditBundle

Exportable audit artifact for compliance/archival.

**Required Fields:**
```json
{
  "bundle_version": "1.0",
  "bundle_id": "<unique_bundle_id>",
  "time_range": {
    "start": "<iso8601>",
    "end": "<iso8601>"
  },
  "decisions": [
    "<DecisionPacket_1>",
    "<DecisionPacket_2>"
  ],
  "policies": [
    {
      "policy_id": "<policy_hash>",
      "policy_snapshot": "<base64_or_uri>"
    }
  ],
  "replay_results": {
    "<decision_id>": {
      "original_checksum": "<sha256>",
      "replay_checksum": "<sha256>",
      "match": true | false
    }
  }
}
```

---

## Canonical ID Immutability Rule

**Rule:** Once a canonical MCP tool ID is public, its semantics are immutable.

**Enforcement:**
- Semantic changes require a new versioned ID (e.g., `decision.check.v2` or `decision.check@2`).
- In-place renames or behavior changes are forbidden.
- Aliases may point to canonical IDs, but never become the semantic source of truth.

**Example Violation:**
```
❌ FORBIDDEN: Rename `decision.check` to `decision.evaluate` in v2
✓ PERMITTED: Introduce `decision.check.v2` with new semantics; deprecate `decision.check`
```

---

## Ordering and Determinism Rules (List/Search APIs)

**For all list-returning or search APIs:**

1. **Explicit Ordering**
   - Results MUST be ordered deterministically.
   - Order MUST be documented in tool schema or contract.
   - Example orderings:
     - Lexicographic by `(timestamp, decision_id)`
     - Ascending by `tick` for lineage replay
     - Alphabetical by `tool_name` for `tools/list`

2. **Determinism Requirement**
   - Identical query parameters → identical result set and order.
   - Applies to fixed underlying store (no concurrent writes).

3. **Phase 0 Status**
   - Currently documented as requirement; not yet enforced.
   - Implementation in Phase 2.

**Example:**
```
audit.search(time_range=[T1, T2], principal="alice")
→ Results ordered by (timestamp ASC, decision_id ASC)
```

---

## Production Invariants (To Be Enforced in Later Phases)

**These invariants are documented now, implemented in Phases 3-4.**

### Invariant 1: Kernel Tool Restriction
- In production mode, `kernel.*` tools MUST NOT be registered in the public MCP surface.
- Startup MUST abort (panic/exit) if `kernel.*` tools are present in production config.

### Invariant 2: Debug Tool Restriction
- `node.mutate` (or `kernel.node.mutate`) MUST be unavailable in production mode.
- Attempts to register debug-only tools in production MUST cause startup failure.

### Invariant 3: DecisionPacket Governance Backing
- All state mutations in the governed domain MUST be backed by a valid DecisionPacket from `decision.check`.
- DecisionPacket MUST have:
  - Matching checksum
  - Valid version
  - Timestamp within acceptable drift window

### Invariant 4: Checksum Stability
- `decision.check` invoked with identical inputs MUST produce identical checksums.
- Checksum mismatch on replay = governance fault (halt).

---

## Version Strategy

### Contract Versions
- **Format:** `MAJOR.MINOR.PATCH`
- **Current:** `1.0.0`

**Version Bump Rules:**
- **PATCH:** Bug fixes, documentation only.
- **MINOR:** Add new tools, add optional parameters, add new enum values (breaking for old readers).
- **MAJOR:** Remove tools, change semantics, change required parameters.

### Tool Versions
- Tools inherit contract version initially.
- If a tool's semantics change, introduce a versioned tool ID (e.g., `decision.check.v2`).

---

## Phase Roadmap (Summary)

| Phase | Focus | Changes |
|-------|-------|---------|
| **Phase 0** | Reality freeze | Document current state, no code changes |
| **Phase 1** | Aliasing | Register canonical IDs as aliases; deprecate legacy IDs |
| **Phase 2** | New tools | Add `decision.preview`, `audit.search` |
| **Phase 3** | Kernel isolation | Enforce production invariants; restrict `kernel.*` tools |
| **Phase 4** | Legacy removal | Remove deprecated legacy IDs |

---

## Conclusion

This outline defines the target Iter MCP Contract v1. Implementation occurs in Phases 1–4. Phase 0 establishes the frozen baseline against which all future changes are measured.

**No behavior changes are permitted under Phase 0.**

---

## Phase 1 Status (Applied 2026-02-06)

Canonical aliases registered in MCP server and SDKs:

- `decision.check` ↔ `governance.evaluate` — same handler, same schema
- `audit.export` ↔ `esv.audit` — same handler, same schema
- `audit.replay` ↔ `lineage.replay` — same handler, same schema
- `governance.health` ↔ `governance.status` — same handler, same schema
- `governor.health` ↔ `governor.status` — same handler, same schema

All alias pairs share identical handlers and schemas. No behavior changes introduced.
Legacy IDs remain active but marked deprecated (`retire_by_version: v3.0`).
SDKs (TypeScript, Rust, Python) expose canonical methods as primary API surface.
Phase 0 JSON inventory and checksum are unchanged.
