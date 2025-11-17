# Day-1 Replay Episode Results

**Directive**: SG-SCG-PILOT-ACT-04 v1.0.0 §4

---

## Overview

This directory contains replay episode validation results for Day-1 of SCG-PILOT-01.

## Replay Protocol

- **Seed**: DAY1_EPISODE
- **Cycles**: 250
- **Environments**: local, docker, kubernetes
- **Variance Threshold**: ε ≤ 1×10⁻¹⁰

## Expected Files

| File | Environment | Status |
|------|-------------|--------|
| `local_replay.txt` | Local machine | ⏳ Pending |
| `docker_replay.txt` | Docker container | ⏳ Pending |
| `kubernetes_replay.txt` | K8s cluster | ⏳ Pending |
| `variance_analysis.json` | Hash comparison | ⏳ Pending |

## Execution

Run replay episodes using:

```powershell
.\deployment\pilot\replay-episode.ps1 -Day 1
```

## Validation Criteria

- All three environments must produce identical operation checksums
- Hash variance |H_ref - H_test| ≤ 1×10⁻¹⁰
- Lineage integrity |H_global − H_expected| = 0

---

**Last Updated**: 2025-11-17 (Day-1 activation)
