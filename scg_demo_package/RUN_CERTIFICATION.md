# SCG Substrate Demo - Certification Report

**Package Version:** 1.0
**Artifact Hash:** `588153f3c00a95c3296b576a74f4d8bea8ea556fb2a0366236bd7b21b899d1df`
**Certification Date:** 2025-11-26
**Scope:** Security research and substrate validation

## Determinism Proof

- Dual-run SHA-256 validation implemented
- Environment: `SCG_TIMESTAMP_MODE=deterministic`
- Locale hardening: `LC_ALL=C`, `LANG=C`, `TZ=UTC`
- JSON normalization: CRLF→LF, trailing whitespace stripped
- Sequential JSON-RPC IDs: 1..N (reset per run)
- Temporal variance: **Zero** (cryptographically verified)

## Compliance Attestation

- ✓ Zero domain-specific logic
- ✓ No customer IP references
- ✓ Synthetic data only
- ✓ POSIX-compliant execution
- ✓ JSON-RPC 2.0 conformant
- ✓ No prohibited keywords (healthcare, automotive, financial)
- ✓ No personal identifiers or references

## Environment Specification

- **MCP Server Version:** 0.1.0
- **Protocol:** JSON-RPC 2.0
- **Transport:** stdin/stdout
- **Platform:** POSIX-compliant (bash 4.0+, jq 1.6+)

## Reproducibility Chain

1. Start MCP server (health probe validates <5s)
2. Execute demo script → `demo_output/`
3. Normalize outputs (whitespace, line endings)
4. Compute SHA-256 checksums for invariant files
5. Compare checksums → Binary equality expected

## Validation Methodology

| Check | Command | Expected |
|-------|---------|----------|
| Prohibited keywords | `grep -Ri "haltra\|nodetic\|patient"` | No output |
| Absolute paths | `grep -r "^/" --include="*.sh"` | No output |
| Personal references | `grep -Ri "andrei"` | No output |
| Docker tag consistency | `grep "scg-demo-package:v1.0"` | 2 matches |

**Invariant Guarantee:** Drift ≤ 1e-10 across all operations.

## Files Certified

| File | Purpose | Status |
|------|---------|--------|
| `demos/scg_demo.sh` | Main CLI demonstration | ✓ |
| `demos/scg_demo.toml` | Deterministic configuration | ✓ |
| `demos/prompts_synthetic.txt` | Domain-neutral scenarios | ✓ |
| `demo_expected/01_start.log` | Baseline invariants | ✓ |
| `demo_expected/02_create_nodes.log` | Node creation responses | ✓ |
| `demo_expected/03_bind_edges.log` | Edge topology responses | ✓ |
| `demo_expected/04_propagate_cycle.log` | Propagation results | ✓ |
| `demo_expected/05_violation.log` | Constraint rejection | ✓ |
| `demo_expected/06_lineage.json` | Cryptographic receipt | ✓ |
| `demo_expected/07_checksums.txt` | Determinism checksums | ✓ |
