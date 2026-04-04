# Governance Bridge — Vendored Snapshot

| Field | Value |
|---|---|
| Source | https://github.com/aduboseh/SCG |
| Crate path | crates/scg-governance-bridge |
| SCG source commit | `a95d164` |
| SCG merged main/master head at vendor time | `7f6877409b9d720928379bbecbfd035621eeda11` |
| Vendored | 2026-04-04 |
| CONTRACT_VERSION_STR | `scg.v1` |
| `contract.rs` SHA256 | `1179dcdd5e8bc51f88324136fdfb55bfe58be00167cbfe091d0c8731e9b51ab0` |
| `trace.rs` SHA256 | `f09664dba3b51d0ce2115d5b3258f93ae4ed75065cc8646c7c3696b9367e31bc` |
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
Seam audit mirror: `verify_replay_id()` now gates on `contract_version` before
`validate_semantics()`, matching the canonical SCG seam-audit fix.

Do not edit files in this directory without following the update protocol.

## Canonical serialization locked — 2026-04-04
Canonical serialization with sorted-key JSON, NFC guard at ingress, adversarial corpus attested.
unicode-normalization dep added for vendor closure.
Full bridge test suite: 64/0/0.
Symmetric with SCG canonical at commit a95d164.
No active drift.