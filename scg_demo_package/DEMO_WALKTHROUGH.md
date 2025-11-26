# SCG Substrate Demo - 7-Minute Walkthrough

**Pre-flight:** Ensure MCP server binary is in PATH and executable.

## Minute 0-1: Startup & Health Validation

```bash
cd scg_mcp_server/demos
./scg_demo.sh
```

**Watch for:** "MCP server health: OK" within 5 seconds.

**Key output:**
```
[SCG-DEMO] SCG Substrate Demo v1.0 (Production Edition)
[SCG-DEMO] Determinism mode: enabled
[SCG-DEMO] Locale: LC_ALL=C
[SCG-DEMO] Starting MCP server...
[SCG-DEMO] MCP server health: OK
```

## Minute 1-2: Baseline Invariants

**Screen output:** `governor.status` showing:
- `drift: 0.0`
- `coherence: 1.0`
- `node_count: 0`
- `edge_count: 0`

**Key point:** "Starting from proven-zero state—no residual operations."

## Minute 2-3: Synthetic Connectome Creation

**Watch:** Five nodes created with beliefs [0.1, 0.3, 0.5, 0.7, 0.9].

**Key output:**
```
[SCG-DEMO] Created node 1/5: <uuid> (belief=0.1)
[SCG-DEMO] Created node 2/5: <uuid> (belief=0.3)
...
```

**Key point:** "Domain-neutral entities—no semantic meaning, pure topology."

## Minute 3-4: Topology Construction & Cycle Propagation

**Highlight:** Edge `N3 → N1` creates intentional cycle.

**Watch:** Five edges bound including:
- Acyclic paths: N1→N2, N2→N3, N4→N5
- Cycle: N3→N1
- Self-loop: N4→N4

**Propagation tests:**
```
[SCG-DEMO] Propagated (acyclic): <edge_uuid>
[SCG-DEMO] Propagated (cycle): <edge_uuid>
[SCG-DEMO] Propagated (selfloop): <edge_uuid>
```

**Key point:** "Substrate handles cycles without instability—drift still ≤ 1e-10."

## Minute 4-5: Constraint Violation Trigger

**Action:** Script sends synthetic unsafe request (edge bind with non-existent node).

**Watch:** Error response with code `4000` and `drift_delta: 0.0`.

**Key output:**
```
[SCG-DEMO] Violation captured: code=4000, msg=BAD_REQUEST: Source or destination not found
```

**Key point:** "Deterministic rejection—no state corruption from bad input."

## Minute 5-6: Lineage Export

**Screen:** `06_lineage.json` displays operation chain with checksums.

**Key structure:**
```json
{
  "episode_id": "synthetic_violation_001",
  "export_checksum": "sha256:...",
  "operation_chain": [...],
  "invariant_proof": {
    "drift_before": 0.0,
    "drift_after": 0.0,
    "coherence_preserved": true
  }
}
```

**Key point:** "Every operation is cryptographically auditable."

## Minute 6-7: Reproducibility Proof

**Watch:** Script runs second iteration and compares checksums.

**Key output:**
```
[SCG-DEMO] ==========================================
[SCG-DEMO] DETERMINISM VERIFIED
[SCG-DEMO] All invariant artifacts match across runs
[SCG-DEMO] ==========================================
```

**Key point:** "Temporal independence proven—can be re-run identically."

---

## Q&A Preparation

| Question | Answer |
|----------|--------|
| "What's the substrate?" | See SUBSTRATE_OVERVIEW.md |
| "Domain applicability?" | Vertical-agnostic; domain envelopes applied post-substrate |
| "Source code access?" | High-level architecture explainable; deep dive under NDA |
| "How is determinism achieved?" | LC_ALL=C, fixed timestamps, sequential IDs, JSON normalization |
| "Error handling?" | Constraint violations return structured JSON-RPC errors |
