# Iter Buyer Summary

**Version:** 1.0.2

**License:** Apache-2.0

**Status:** Release-candidate infrastructure, not Enterprise GA

## Product Surface

Iter is a governed MCP/JSON-RPC runtime and independent verification surface for AI decisions. The public distribution provides protocol types, governance enforcement, DecisionPacket generation and verification, replay checks, audit interfaces, and Rust and TypeScript SDKs.

SCG is a separately deployed cognitive graph system. Authoritative `scg-backed` operation uses the pinned `scg.v1` contract and fails closed when endpoint configuration, governance identity, contract identity, state envelopes, or replay evidence are invalid.

## Current Guarantees

| Guarantee | Enforcement |
|---|---|
| Public build identity | Compile-time `ITER_BUILD_MODE=PUBLIC_STUB` attestation |
| Unavailable substrate rejection | `full_substrate` exits with `FULL_SUBSTRATE_UNSUPPORTED_IN_PUBLIC_REPO` |
| Governance integrity | Build-time and runtime governance hash checks |
| Contract stability | Schema and governance invariant tests |
| Replay integrity | Canonical packet and trace verification |
| Cross-system boundary | Pinned SCG commit plus byte-equal contract checks |

## Current Exclusions

Iter does not currently claim:

- an embedded or licensed `full_substrate` build,
- Enterprise GA status,
- deployment-independent performance guarantees,
- completed cross-platform, AKS, security, signing, supply-chain, or external-operator certification.

Those claims remain unavailable until their canonical APEX controls pass using commit-bound evidence from the trusted evidence workflow.

## Evaluation

```bash
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
python scripts/verify_productization_matrix.py --validate-only
```

See [ARCHITECTURE_BOUNDARY.md](ARCHITECTURE_BOUNDARY.md) and [APEX_PRODUCTIZATION_V1.md](APEX_PRODUCTIZATION_V1.md) for the binding boundary and release criteria.
