# Governance Bridge — Vendored Snapshot

| Field | Value |
|---|---|
| Source | https://github.com/aduboseh/SCG |
| Crate path | crates/scg-governance-bridge |
| SCG source commit | `b4b165d71377477b62902a9e3615e8ff5d1d5604` (CANON-001 branch head) |
| SCG merged main/master head at vendor time | `3e0675073a50ce20bdad7c342f7a5caaa3801504` |
| Vendored | 2026-04-03 |
| CONTRACT_VERSION_STR | `scg.v1` |
| `contract.rs` SHA256 | `1179dcdd5e8bc51f88324136fdfb55bfe58be00167cbfe091d0c8731e9b51ab0` |
| `trace.rs` SHA256 | `568cb863df5363ea922187dfd8d379a4396ff387be5ac315f75c923a354cdf05` |
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

## Canonical serialization locked — 2026-04-03
trace.rs canonical_payload() now uses sorted-key JSON (see CANON.md).
serde_json::to_string removed from canonical form.
Symmetric with SCG canonical at commit b4b165d71377477b62902a9e3615e8ff5d1d5604.
No active drift.
