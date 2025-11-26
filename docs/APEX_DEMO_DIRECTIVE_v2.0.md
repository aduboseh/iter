# APEX DEMO DIRECTIVE - SCG SUBSTRATE CLI (PRODUCTION EDITION)

**Version:** 2.0
**Status:** Executed and Archived
**Execution Date:** 2025-11-26
**Artifact Hash:** `588153f3c00a95c3296b576a74f4d8bea8ea556fb2a0366236bd7b21b899d1df`

---

## Overview

This directive governed the creation of a cryptographically-certified demonstration package for the SCG substrate. The package validates deterministic graph operations, constraint enforcement, and lineage tracking for enterprise security research.

---

## Execution Contract (Completed)

- [x] Execute in strict sequential order
- [x] No skipped steps
- [x] No placeholders or TODOs
- [x] All code complete and runnable
- [x] Validation matrix emitted
- [x] SHA-256 hash computed
- [x] Determinism proof documented

---

## Deliverables (All Completed)

### Primary Artifacts

| File | Description | Status |
|------|-------------|--------|
| `demos/scg_demo.sh` | Main CLI demonstration script | Completed |
| `demos/prompts_synthetic.txt` | Domain-neutral prompt library | Completed |
| `demos/scg_demo.toml` | Deterministic configuration | Completed |

### Expected Outputs

| File | Description | Status |
|------|-------------|--------|
| `demo_expected/01_start.log` | Baseline invariants | Completed |
| `demo_expected/02_create_nodes.log` | Node creation responses | Completed |
| `demo_expected/03_bind_edges.log` | Edge binding responses | Completed |
| `demo_expected/04_propagate_cycle.log` | Propagation results | Completed |
| `demo_expected/05_violation.log` | Constraint violation | Completed |
| `demo_expected/06_lineage.json` | Cryptographic receipt | Completed |
| `demo_expected/07_checksums.txt` | Determinism checksums | Completed |

### Documentation

| File | Description | Status |
|------|-------------|--------|
| `RUN_CERTIFICATION.md` | Validation methodology | Completed |
| `SUBSTRATE_OVERVIEW.md` | Architecture summary | Completed |
| `DEMO_WALKTHROUGH.md` | 7-minute presentation | Completed |
| `SUBSTRATE_ATTESTATION.txt` | Technical attestation | Completed |
| `RUNBOOK.md` | Operational guide | Completed |
| `Dockerfile` | Container build | Completed |
| `DOCKER_INSTRUCTIONS.md` | Docker usage | Completed |
| `PACKAGING_MANIFEST.txt` | File checksums | Completed |

---

## Demo Flow (Implemented)

### Step A: Server Startup
- MCP server started as background process
- Health probe with 5-second timeout
- Cleanup trap on exit

### Step B: Baseline Invariants
- `governor.status` query
- Log drift, coherence, node_count, edge_count

### Step C: Node Creation
- Create 5 nodes with beliefs: 0.1, 0.3, 0.5, 0.7, 0.9
- Validate belief bounds [0.0, 1.0]
- Store node IDs dynamically

### Step D: Edge Topology
- Bind 5 edges including cycle (N3→N1) and self-loop (N4→N4)
- Store edge IDs dynamically

### Step E: Propagation Tests
- Test acyclic, cycle, and self-loop propagation
- Verify governor stability

### Step F: Constraint Violation
- Trigger synthetic violation (non-existent node)
- Validate error response with code 4000
- Confirm drift_delta = 0.0

### Step G: Lineage Export
- Export operation chain with checksums
- Build structured invariant proof

### Step H: Energy Check
- Verify drift ≤ 1e-10

### Step I: Reproducibility
- Run scenario twice
- Compute SHA-256 for invariant files
- Compare and emit verdict

---

## Determinism Stack (7 Layers)

1. **Locale:** `LC_ALL=C`, `LANG=C`, `TZ=UTC`
2. **Timestamps:** `SCG_TIMESTAMP_MODE=deterministic`
3. **Telemetry:** `SCG_DETERMINISM=1`
4. **JSON-RPC IDs:** Sequential 1..N, reset per run
5. **Normalization:** CRLF→LF, trailing whitespace stripped
6. **Fixed Epoch:** `1700000000`
7. **Checksum Comparison:** SHA-256 for all invariant files

---

## Compliance Audit (Passed)

| Check | Result |
|-------|--------|
| Prohibited keywords | PASS |
| Personal references | PASS |
| Absolute paths | PASS |
| Embedded secrets | PASS |
| Domain fingerprinting | PASS |

---

## Certification

**Artifact Hash:** `588153f3c00a95c3296b576a74f4d8bea8ea556fb2a0366236bd7b21b899d1df`
**Package Hash:** `9FAEA83409F014066EEA2483E364C83A9AACC3F59BA884206A88D5B0BEF07158`
**Package Size:** 21.75 KB
**Total Files:** 18

**Determinism Proof:** Two sequential runs produce identical SHA-256 checksums for all invariant artifacts.

---

## Archive Note

This directive has been fully executed. The resulting package is production-ready and certified for enterprise deployment. See `DIRECTIVE_CHANGELOG.md` for version history.
