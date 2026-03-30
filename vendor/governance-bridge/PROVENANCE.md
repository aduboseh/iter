# Governance Bridge — Vendored Snapshot

| Field | Value |
|---|---|
| Source | https://github.com/aduboseh/SCG |
| Crate path | crates/scg-governance-bridge |
| SCG source commit | 4f727ff (codex/wo-scg-contract-001b pre-merge head) |
| SCG merged main/master head at vendor time | 5221a59c2baee6581c927ac1a6d50775c3f1b2c3 |
| Vendored | 2026-03-29 |
| CONTRACT_VERSION_STR | `scg.v1` |
| `contract.rs` SHA256 | `f80501527f9cff1abccf4226afca9cd949f8f44fbc774460c440a26d3ec28605` |
| `trace.rs` SHA256 | `fb7c80bc8afe0f88f4dc2cfc95abea220879bbe5649c9cc44eeadf0c40f1846e` |
| `errors.rs` SHA256 | `d1459d2ebfd73dfed7d1bc78990a250b72ec701e7260624e320d824c2397d0af` |
| `lib.rs` SHA256 | `d508e6c4fa515761fa052e844ca51434dfd8971aaa3fab3a5d7c118787bbf8ac` |

## Update Protocol

This snapshot is cryptographically bound to the Iter build.
Any file modification will cause `cargo build` to fail with an integrity error.

To update this snapshot intentionally:
1. Pull the new contract from SCG at the target commit.
2. Recompute all four SHA256 hashes.
3. Update the constants in [build.rs](/C:/Users/adubo/OneDrive/Documents/New%20project/iter_001b_retry/build.rs).
4. Update this file with the new commit and hashes.
5. Open a PR. Changes under `vendor/` require explicit review.

Do not edit files in this directory without following the update protocol.
