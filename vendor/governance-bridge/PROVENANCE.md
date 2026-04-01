# Governance Bridge — Vendored Snapshot

| Field | Value |
|---|---|
| Source | https://github.com/aduboseh/SCG |
| Crate path | crates/scg-governance-bridge |
| SCG source commit | `eb83bf1ca7d761b227a7fc9fa58d3a5d194455b2` (merge of PR #11) |
| SCG merged main/master head at vendor time | `eb83bf1ca7d761b227a7fc9fa58d3a5d194455b2` |
| Vendored | 2026-03-31 |
| CONTRACT_VERSION_STR | `scg.v1` |
| `contract.rs` SHA256 | `ab8671912ac6ab477a87f6a5f91b6031032dfe560557c97b4350c22b45e3a6f4` |
| `trace.rs` SHA256 | `049f8b83afb049e2ef98ad383094ad9e56cee80284281573e08fef78f8c0fc87` |
| `errors.rs` SHA256 | `d1459d2ebfd73dfed7d1bc78990a250b72ec701e7260624e320d824c2397d0af` |
| `lib.rs` SHA256 | `e2556d561acba83914a85b445186d6c6a97d4a75b19a95c37ea552c192f61f36` |

## Update Protocol

This snapshot is cryptographically bound to the Iter build.
Any file modification will cause `cargo build` to fail with an integrity error.

To update this snapshot intentionally:
1. Pull the new contract from SCG at the target commit.
2. Recompute all four SHA256 hashes.
3. Update the constants in `build.rs`.
4. Update this file with the new commit and hashes.
5. Open a PR. Changes under `vendor/` require explicit review.

## Notes

Semantic trace hardening: `validate_semantics()`, `validate_sequence()`,
`validate_completeness()`, `verify_hash_binding()`, `input_payload`, and
`output_payload` fields added to `TraceStep`.

Do not edit files in this directory without following the update protocol.
