# Architecture

Iter has one supported public build mode: `public_stub`. This build contains the governed verifier/runtime and the versioned SCG gateway contract.

`full_substrate` is a reserved feature name, not a supported or advertised product configuration. The public build script rejects it with `FULL_SUBSTRATE_UNSUPPORTED_IN_PUBLIC_REPO` so stub semantics cannot be mislabeled as full-substrate provenance.

## Runtime Modes

- `demo`: explicit, non-authoritative local stub behavior.
- `governed-local`: governed local evaluation with DecisionPackets and replay checks, but no live SCG authority.
- `scg-backed`: live SCG gateway evaluation through the pinned `scg.v1` contract, with fail-closed boot and response validation.

SCG remains a separate deployment and implementation boundary. Iter depends on the versioned contract, not SCG implementation crates.

## Certification Boundary

The repositories remain release-candidate infrastructure until the canonical 30-control APEX matrix reports 30 PASS / 0 FAIL for exact Iter and SCG commits. See [ARCHITECTURE_BOUNDARY.md](../ARCHITECTURE_BOUNDARY.md).
