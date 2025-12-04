# SCG Governance v1.1.1 — Performance Baseline

**Version:** 1.1.1  
**Release Tag:** v1.1.1-perf-baseline  
**Authority:** Only SG Solutions — Synthetic Cognitive Law (SCL)  
**Synced From:** https://github.com/aduboseh/SCG/releases/tag/v1.1.1-perf-baseline

---

## Baseline Checksums

### Determinism Baseline

```
Checksum: 04f3ddbd9a5e3659ece6df1a5dd7e3d63359dde8936abd95bcf166888fbada60
```

This checksum represents the stable state of:
- 10-node DAG with ring + cross topology
- 5 belief propagation cycles
- 3 governor correction steps
- Full lineage hash chain

### Governance Rules Checksum

```
SHA256: 9D7623E581D982D8F9BC816738EF0880E9631E6FD5789C36AF80698DF2BAA527
```

This checksum covers the 14 governance rules across 6 tables defined in `WARP.md`.

---

## Performance Baseline (2025-12-04)

| Benchmark | Mean | Range |
|-----------|------|-------|
| `single_node_step` | 964 ns | [959 ns, 968 ns] |
| `dag_propagation/10` | 3.19 µs | [3.13 µs, 3.25 µs] |
| `dag_propagation/100` | 18.2 µs | [18.1 µs, 18.3 µs] |
| `dag_propagation/500` | 82.3 µs | [81.5 µs, 83.2 µs] |
| `governor_correction` | 5.05 µs | [5.01 µs, 5.11 µs] |
| `lineage_append` | 3.16 µs | [3.10 µs, 3.23 µs] |

**Regression Thresholds:**
- Warning: >5% slower
- Failure: >10% slower
- Critical: >25% slower

---

## Invariant Compliance

| ID | Name | Status |
|----|------|--------|
| INV-01 | Drift Invariant (ε ≤ 1e-10) | ✅ Enforced |
| INV-02 | Energy Conservation | ✅ Enforced |
| INV-03 | Cycle Idempotence | ✅ Enforced |
| INV-04 | ESV Safety | ✅ Enforced |
| INV-05 | Unsafe Ops State | ✅ Enforced |
| INV-06 | Temporal Invariance | ✅ Enforced |
| INV-07 | Belief Boundary | ✅ Enforced |
| INV-08 | Graph Connectivity | ✅ Enforced |
| INV-09 | Lineage Integrity | ✅ Enforced |
| INV-10 | Governor Coherence | ✅ Enforced |
| INV-11 | DAG Acyclicity | ✅ Enforced |
| INV-12 | Monotonic Tick | ✅ Enforced |
| INV-13 | Consensus Quorum | ✅ Enforced |
| INV-14 | Ethics Veto | ✅ Enforced |
| INV-15 | Lyapunov Stability | ✅ Enforced |

---

## MCP Boundary Requirements

The MCP server MUST:

1. **Validate governance checksum** on startup
2. **Sanitize all responses** via `services/sanitizer.rs`
3. **Never expose** DAG internals, raw ESV values, or topology
4. **Log all mutations** to lineage

---

## Verification

MCP server can verify substrate alignment:

```rust
const SUBSTRATE_DETERMINISM_CHECKSUM: &str = 
    "04f3ddbd9a5e3659ece6df1a5dd7e3d63359dde8936abd95bcf166888fbada60";

const GOVERNANCE_RULES_CHECKSUM: &str = 
    "9D7623E581D982D8F9BC816738EF0880E9631E6FD5789C36AF80698DF2BAA527";

// Performance baseline validation
const PERF_BASELINE_TAG: &str = "v1.1.1-perf-baseline";
```

---

## Changelog

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2025-12-03 | Initial governance rules |
| 1.1.0 | 2025-12-04 | Substrate integrity baseline, determinism checksum |
| 1.1.1 | 2025-12-04 | Performance baseline locked, INV-11..15 added |

---

**This file is synced from SCG substrate release v1.1.1-perf-baseline.**
