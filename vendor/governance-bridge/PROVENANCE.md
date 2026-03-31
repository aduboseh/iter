# Governance Bridge — Vendored Snapshot

| Field | Value |
|---|---|
| Source | https://github.com/aduboseh/SCG |
| Crate path | crates/scg-governance-bridge |
| SCG source commit | `96236b6a072fddc770158903e65793fce44eec9f` (merge of PR #10) |
| SCG merged main/master head at vendor time | `96236b6a072fddc770158903e65793fce44eec9f` |
| Vendored | 2026-03-31 |
| CONTRACT_VERSION_STR | `scg.v1` |
| `contract.rs` SHA256 | `d7b27a5731c05332f4ef2724bc69b9b34e695d5e7f26b516f3440c01baa4dd94` |
| `trace.rs` SHA256 | `5ca8df091cd0791c806c36ca35849aae53331674c3bd0a5e5c94320fc4ad6979` |
| `errors.rs` SHA256 | `d1459d2ebfd73dfed7d1bc78990a250b72ec701e7260624e320d824c2397d0af` |
| `lib.rs` SHA256 | `718c723b7b84c06e0ffc035126ff21999ca82ac38707b9fb3c6055ffe84ec112` |

## Update Protocol

This snapshot is cryptographically bound to the Iter build.
Any file modification will cause `cargo build` to fail with an integrity error.

To update this snapshot intentionally:
1. Pull the new contract from SCG at the target commit.
2. Recompute all four SHA256 hashes.
3. Update the constants in `build.rs`.
4. Update this file with the new commit and hashes.
5. Open a PR. Changes under `vendor/` require explicit review.

Do not edit files in this directory without following the update protocol.
