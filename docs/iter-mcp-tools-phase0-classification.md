# Iter MCP Tools — Phase 0 Classification

**APEX DIRECTIVE:** ITER-MCP-TOOL-SURFACE v1 — Phase 0  
**Date:** 2026-02-05  
**Classification:** Current reality freeze (no behavior changes)

---

## Tool Inventory

| Tool Name | Category | Target Role (Future) | Notes (Current Behavior) |
|-----------|----------|---------------------|--------------------------|
| `governance.evaluate` | PUBLIC_GOVERNANCE | `decision.check` | PDP gate; returns authoritative verdict on governance proposals. Accepts `proposal_id`, `state_snapshot_hash`, `requested_action`, optional `constraints`. Marked "PHASE 0: Iter-Haltra Bridge" in description. |
| `governance.status` | OPS/STATUS | `governance.health` | Returns governance health summary. No required parameters. |
| `governor.status` | OPS/STATUS | `governor.health` | Returns governor coherence/drift metrics. No required parameters. |
| `esv.audit` | AUDIT/REPLAY | `audit.export` | Audit node ESV (Energy-State-Value). Requires `node_id` (string). Returns compliance status for a specific node. |
| `lineage.replay` | AUDIT/REPLAY | `audit.replay` | Replay lineage history. No required parameters. Returns lineage reconstruction. |
| `node.create` | KERNEL_INTERNAL | `kernel.node.create` | Create graph node with initial `belief` (number) and `energy` (number). Returns node ID. |
| `node.query` | KERNEL_INTERNAL | `kernel.node.query` | Query node state by `node_id` (string). Returns node belief/energy state. |
| `node.mutate` | DEBUG_ONLY | `kernel.node.mutate` (debug) | Mutate node belief by `delta` (number). Requires `node_id`. **Unsafe state mutation**—governed operation; availability may be restricted by policy. |
| `edge.bind` | KERNEL_INTERNAL | `kernel.edge.bind` | Bind edge between `src` and `dst` nodes (both strings) with `weight` (number). Returns edge ID. |
| `edge.propagate` | KERNEL_INTERNAL | `kernel.edge.propagate` | Run deterministic propagation step. Accepts `edge_id` (string, accepted for compatibility but not used per schema description). |

---

## Category Definitions

### PUBLIC_GOVERNANCE
Tools intended for external consumption by Policy Decision Points (PDPs) and governance consumers.

**Count:** 1 (`governance.evaluate`)

### OPS/STATUS
Operational health/status queries for monitoring and observability.

**Count:** 2 (`governance.status`, `governor.status`)

### AUDIT/REPLAY
Audit trail export and deterministic replay operations.

**Count:** 2 (`esv.audit`, `lineage.replay`)

### KERNEL_INTERNAL
Internal graph/kernel operations. Not intended for external governance consumers. Should be namespaced under `kernel.*` in future phases.

**Count:** 4 (`node.create`, `node.query`, `edge.bind`, `edge.propagate`)

### DEBUG_ONLY
Unsafe or debug-only operations that mutate state without full governance review. Should be restricted in production mode.

**Count:** 1 (`node.mutate`)

---

## Target Canonical Families (Future Phases)

### `decision.*` (PDP / Governance Gate)
- `decision.check` ← current: `governance.evaluate`
- `decision.preview` ← to be added (non-authoritative simulation)
- `decision.explain` ← to be added (structured explanation)

### `audit.*` (Audit & Replay)
- `audit.search` ← to be added (search decisions by criteria)
- `audit.export` ← current: `esv.audit`
- `audit.replay` ← current: `lineage.replay`

### `governance.*` / `governor.*` (Health / Ops)
- `governance.health` ← current: `governance.status`
- `governor.health` ← current: `governor.status`

### `kernel.*` (Internal / Syscall-Like)
- `kernel.node.create` ← current: `node.create`
- `kernel.node.query` ← current: `node.query`
- `kernel.node.mutate` ← current: `node.mutate` (debug-only)
- `kernel.edge.bind` ← current: `edge.bind`
- `kernel.edge.propagate` ← current: `edge.propagate`

---

## Phase 0 Observations

### Tool Count
**Total:** 10 tools

### Tool Order Stability
Tool order is stable across multiple `tools/list` invocations.

### Schema Completeness
All tools have:
- ✓ `name` (string)
- ✓ `description` (string)
- ✓ `inputSchema` (JSON Schema object)

All required fields are documented in schemas.

### Notable Observations

1. **`governance.evaluate` is the only PUBLIC_GOVERNANCE tool**
   - This is the canonical PDP gate
   - Description explicitly marks it as "PHASE 0: Iter-Haltra Bridge"
   - Target canonical name: `decision.check`

2. **`node.mutate` is marked as governed operation**
   - Schema description: "governed operation; availability may be restricted by policy"
   - Classified as DEBUG_ONLY due to unsafe state mutation
   - Should be gated or removed in production mode

3. **`edge.propagate` accepts but does not use `edge_id`**
   - Schema description: "Edge ID (accepted for compatibility, not used)"
   - Suggests legacy parameter carryover
   - Future phases may clean up unused parameters

4. **No `lineage.export` tool exists**
   - Target canonical: `audit.export`
   - Current closest match: `esv.audit` (node-scoped) and `lineage.replay`
   - May need new tool in Phase 1 for full audit bundle export

---

## Conclusion

Phase 0 classification complete. All 10 tools categorized and mapped to target canonical IDs. No behavior changes made; this is a reality freeze only.
