# Iter Product Boundary

This document records the current, certifiable Iter product boundary. It is subordinate to `APEX_PRODUCTIZATION_V1.md` and `APEX_PRODUCTIZATION_GAP_CLOSURE_001.md`.

## Current Classification

Iter and SCG are release-candidate infrastructure, not Enterprise GA. Enterprise release requires all 30 canonical APEX controls to pass against exact Iter and SCG commits.

## Supported Build

The supported public build is `public_stub`.

The `full_substrate` feature name is reserved to prevent semantic reuse. It is not a supported build or a current product configuration. Enabling it in this repository must fail with `FULL_SUBSTRATE_UNSUPPORTED_IN_PUBLIC_REPO`; a stub-backed binary must never claim full-substrate provenance.

## Runtime Boundary

The public Iter distribution contains the governed verifier/runtime surface:

| Runtime mode | Backing system | Authority |
|---|---|---|
| `demo` | Local stub | Non-authoritative protocol demonstration |
| `governed-local` | Governed local stub | Packet-emitting and replay-capable, but not SCG-backed |
| `scg-backed` | Live SCG gateway over the pinned `scg.v1` contract | Authoritative only when all boot and response attestations pass |

SCG is a separately deployed system. Iter consumes SCG through the narrow, versioned contract and does not embed SCG implementation crates.

## What Public CI Proves

- Rust formatting, linting, tests, and dependency audit gates
- Protocol and schema invariants
- Fail-closed build provenance and governance integrity
- Explicit rejection of unavailable `full_substrate` semantics
- Iter-to-SCG contract parity and the tested `scg-backed` seam
- SDK checks that are present in required workflows

These checks do not by themselves prove production performance, deployment security, cross-platform determinism, AKS operability, artifact signing, or independent operator acceptance. Those remain FAIL until the canonical matrix consumes commit-bound evidence from the trusted evidence workflow.

## Distribution Exclusions

The current public product contract does not include:

- an embedded private substrate build,
- SCG source code as part of the Iter package,
- deployment-specific performance guarantees,
- Enterprise GA certification before a 30/0 matrix result.

## Verification

```bash
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
cargo check --locked --workspace --all-targets --no-default-features --features full_substrate
```

The final command is expected to exit `101` and include `FULL_SUBSTRATE_UNSUPPORTED_IN_PUBLIC_REPO`.
