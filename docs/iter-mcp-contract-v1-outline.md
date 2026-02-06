# Iter MCP Contract v1 — Outline

**APEX DIRECTIVE:** ITER-MCP-TOOL-SURFACE v1 — Phase 0  
**Status:** OUTLINE ONLY (no implementation changes)  
**Date:** 2026-02-05

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
|| `decision.preview` | **ACTIVE** | 2 | Non-authoritative simulation. Returns projected decision without committing. |
| `decision.explain` | *(to be added)* | 3 | Structured explanation of decision rationale with policy trace. |

### Audit Tools (`audit.*`)

| Canonical ID | Current Implementation | Phase | Description |
|--------------|----------------------|-------|-------------|
|| `audit.search` | **ACTIVE** | 2 | Search decisions by criteria (time range, principal, action, outcome). |
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

3. **Phase 2 Status**
   - Ordering and determinism enforced for `audit.search`.
   - Results ordered by `(timestamp_utc, decision_id) ASC`.
   - Default limit: 100. Max limit: 1000.

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

## Phase 2 Status (Applied 2026-02-06)

New governance UX tools registered in MCP server and SDKs:

### `decision.preview`
- Non-authoritative governance simulation.
- Returns `DecisionPreview` artifact (DISTINCT from `DecisionPacket`).
- `simulation: true`, `preview_version: "1.0"`, `derived_from: "decision.check@1"`.
- Uses same verdict logic as `decision.check` but does NOT record lineage.
- Deterministic: identical inputs produce identical previews.
- Error code: `5001` (simulation unavailable).

### `audit.search`
- Searches governance decision history with filters.
- Filters: `principal`, `action`, `resource`, `decision`, `policy_id`, `from`, `to`, `limit`.
- Default limit: 100. Max limit: 1000.
- Results ordered by `(timestamp_utc, decision_id) ASC`.
- Deterministic: identical queries produce identical result sets.

### SDK Coverage
- TypeScript: `decisionPreview()`, `auditSearch()` + `DecisionPreview`, `AuditSearchFilter`, `AuditSearchResult` interfaces.
- Rust: `decision_preview()`, `audit_search()` methods.
- Python: `decision_preview()`, `audit_search()` methods.

### Test Coverage
- `tests/mcp_governance_ux.rs`: 10 integration tests covering Phase 2 invariants.
- Preview: artifact shape, determinism, non-mutation, verdict parity with decision.check.
- Search: empty results, post-evaluate results, deterministic ordering, limit enforcement.

## Phase 3 Status — Kernel Isolation & Profiles (Applied 2026-02-06)

### Server Profiles

Iter runs in one of two explicit profiles, selected via CLI:
- `governance` (default) — production PDP surface.
- `kernel-debug` — internal-only kernel/graph surface for debugging.

Profile selection:
```
iter-server --profile=governance    # Production (default)
iter-server --profile=kernel-debug  # Internal debug only
iter-server                         # Same as --profile=governance
```

Unrecognized `--profile` values cause immediate exit (ERROR_INVALID_PROFILE).

### Authority Scope

The governance profile is the only profile under which authoritative PDP, replay, and audit claims apply. No other profile carries production authority.

### Governance Profile Invariants (Enforced at Startup)

1. No `kernel.*`, `node.*`, or `edge.*` tools may be registered.
   - Startup aborts (exit 1) if this invariant is violated.
2. All canonical governance tools must be present:
   - `decision.check`, `decision.preview`
   - `audit.export`, `audit.replay`, `audit.search`
   - `governance.health`, `governor.health`
   - Startup aborts if any canonical tool is missing.
3. `tools/call` rejects kernel/graph tool invocations with error code 3001.

### Kernel-Debug Profile

Kernel-debug is a non-production profile and may expose diagnostic interfaces. It MUST NOT be used for authoritative governance decisions.

- Exposes kernel tools: `node.create`, `node.query`, `node.mutate`, `edge.bind`, `edge.propagate`.
- Also includes all governance tools (superset for debugging).
- Never used by external tooling, production agents, or published configs.

### Surface Partitioning (No Semantic Changes)

Existing tool behavior is byte-identical. This phase changes only which tools are visible and callable per profile. No changes to:
- DecisionPacket / DecisionPreview formats.
- `decision.*` or `audit.*` logic.
- Policy hashing or checksums.
- Legacy alias behavior.

### Test Coverage
- `tests/mcp_surface_profiles.rs`: 6 integration tests validating profile surfaces.
- Governance: no kernel tools, all canonical tools present, kernel calls rejected.
- Kernel-debug: kernel tools present, governance tools also present.
- Default: identical to governance.

## Phase 4 Status — Consumption-Grade Replay & Audit CLI (Applied 2026-02-06)

All CLI commands operate strictly on governed-mode artifacts and do not introduce new governance decisions.

### `iter-cli replay`

Operator command to replay a DecisionPacket under specified policy/schema versions.

- Wraps `replay_decision()` — the same function and code path used by CI golden vector tests.
- Inputs: `--decision-file`, `--policy-version` (sha256:<hash>), `--schema-version` (decision_packet:v1).
- Output: structured JSON to stdout (`outcome: VERIFIED` or `outcome: MISMATCH`).
- Fail-closed: checksum mismatch, policy version mismatch, or schema version mismatch → exit 2.

### `iter-cli audit export`

Operator command to validate and export a DecisionPacket file.

- Reads DecisionPacket JSON, verifies checksum integrity, writes canonical copy to `--output`.
- This command performs no state mutation and does not create new audit entries.
- Fail-closed: integrity failure → exit 2.

### Exit Codes (Both Commands)

- 0: Success (VERIFIED / EXPORTED)
- 1: Input error (file missing, malformed JSON, missing required flags)
- 2: Replay/contract mismatch or integrity failure
- 3: Internal error

### Semantic Guarantees

- Zero semantic change: CLI calls existing replay and audit functions. No alteration to DecisionPacket shape, contents, or checksum algorithm.
- `PolicyConfig::compute_hash` continues to use `serde_json::to_string` with struct field order. Cross-language canonicalization (JCS) is deferred; any change will bump versions and regenerate golden vectors.
- DecisionPacket checksums use JCS canonicalization via `serde_json_canonicalizer`.

### Test Coverage

- `tests/cli_replay_audit.rs`: 7 integration tests.
- Replay: golden vector verified, fail-closed on tampered packet, fail-closed on policy version mismatch, fail-closed on missing file.
- Export: round-trip (export → replay), integrity rejection on tampered packet.
- Policy hash stability across CLI boundary.

### Golden Fixture

- `tests/data/golden_decision_v1.json`: materialized Golden Vector 1 DecisionPacket.
- Checksum: `acd92a1cea22df1e26db77689498b62393458ca8dcceddcddd1c40f23aeaa8fe`.
- Operator documentation: `docs/iter-operator-tools.md`.

## Phase 5 Status — Spec Hardening & Externalization (Applied 2026-02-06)

- JSON Schemas (Draft 7, `additionalProperties: false`) are checked into `schemas/v1/`:
  - `decision_packet`, `decision_preview`, `decision_check_request`, and `audit_search` (filter/result definitions).
- `examples/generate_schemas.rs` regenerates schemas via `cargo run --features schema-gen --example generate_schemas`.
- Claim gates:
  - `tests/schema_integrity.rs` serializes canonical structs and validates them against the committed schemas (DecisionPacket fixture + decision preview/check/audit samples).
  - `tests/doc_examples_integrity.rs` validates `tests/data/golden_decision_v1.json` against the DecisionPacket schema; markdown-tag extraction will be added when docs include machine-tagged blocks.
- External consumers should follow `docs/iter-external-spec-v1.md` for integration workflows and schema links.

Any struct change that alters the wire contract must regenerate the schemas (new versions will live under `schemas/v{N}`).
