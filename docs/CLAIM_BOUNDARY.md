# Claim Boundary

This repository uses the SCG Claim Registry v1.0 as the ceiling for public
product claims.

Evaluator-facing Iter docs may claim:

- same-binary replay under the packet's declared build environment
- `scg.v1` bridge integrity through vendored hashes and compile-time provenance
- fail-closed replay verification for `DecisionPacket`
- exact IEEE-754 hex encoding for proof-critical DecisionPacket numeric fields
- telemetry surfaces where implemented

Evaluator-facing Iter docs must not claim:

- cross-platform replay or cross-compiler determinism: not claimed
- observability-complete operation: not an active product claim
- distributed coordination guarantees: not claimed
- all invariants holding simultaneously
- proof-critical float equivalence outside exact packet encodings and the declared same-binary scope

Older architecture-review material is non-authoritative unless it is updated to
reference this boundary explicitly. The active product proof starts at
`./scripts/golden_path.ps1`.
