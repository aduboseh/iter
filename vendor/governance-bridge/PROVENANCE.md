# Governance Bridge — Vendored Snapshot

| Field | Value |
|---|---|
| Source | https://github.com/aduboseh/SCG |
| Crate path | crates/scg-governance-bridge |
| SCG source commit | `0306feb600e12c627dc4b10963fc8f7781dc0e18` (state-envelope seam head) |
| SCG merged main/master head at vendor time | `b6c9a3b641291631358fcf9f8deace74d71e7615` |
| Vendored | 2026-05-23 |
| CONTRACT_VERSION_STR | `scg.v1` |
| `contract.rs` SHA256 | `82800952f5e03851422fe4469fd159738b927e93b6a284c79f2703070516b3db` |
| `trace.rs` SHA256 | `620892e1986dc22a2a5c17f60ec650e6da70dbe90b847a2862e13c1bf14bce20` |
| `errors.rs` SHA256 | `d1459d2ebfd73dfed7d1bc78990a250b72ec701e7260624e320d824c2397d0af` |
| `lib.rs` SHA256 | `52b4e25270e9b4b001175f9beba8bb020b6d64f9140fbc47b67789e9fa1badd8` |

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
Symmetric with SCG canonical at commit 0306feb600e12c627dc4b10963fc8f7781dc0e18.
No active drift.

## Trace diagnostic hardening — 2026-05-16
trace.rs includes payload context in invalid JSON errors and object-key byte
context in NFC violations. The vendored hash above matches the Iter build.rs
integrity pin.

## Canonical vector contract — 2026-04-05

### Provenance anchor
  SCG commit: edf2e239bfd6760bddbe686febffcce947149564
  Role: replay boundary reference, audit surface, contract origin
  This is a provenance binding, not a comment.
  This commit is on SCG master — it is a governed reference.
  Artifact-generation commit: 1b410f71cf951647376500098d1056b6f0872fb2
  Source: CANONICAL_VECTORS.json field `scg_commit`
  These commits are intentionally different:
  `scg_commit` records where the vector artifact content was generated,
  while the master commit above records where that governed artifact
  entered SCG master as a replay boundary reference.

### Vendored artifact
  File: vendor/governance-bridge/CANONICAL_VECTORS.json
  sha256: 1e804ac4342da71251d4a404bfcee5ef65a2f5b46d599e0fe9d73c80830c1d75
  Line endings: LF
  Hash computed on: raw bytes (fs::read + ReadAllBytes, no normalization)
  Build-time integrity hash encoding: lowercase hex (both PowerShell and Rust)
  This lowercase encoding applies to the vendored file integrity hash above,
  not to the four vector digests stored inside CANONICAL_VECTORS.json.

### Cross-language contract declaration
  Any implementation claiming SCG canonical equivalence must
  reproduce all four vector hashes in CANONICAL_VECTORS.json
  exactly. Input encoding: UTF-8 NFC. Hash algorithm: SHA-256 hex.
  Non-NFC input must be rejected at ingress before hashing.
  Vector digest casing is part of the SCG contract:
  the `sha256` values inside CANONICAL_VECTORS.json are uppercase because
  SCG canonical payload hashing emits uppercase hex. Do not lowercase,
  normalize, or rewrite those values during vendoring.

### Build-time enforcement
  build.rs fails cargo build if CANONICAL_VECTORS.json is modified,
  reformatted, or tampered after vendoring. This is intentional.
  "Fixing formatting" = contract break. Open a new WO.
  Vendored file integrity comparison is case-sensitive lowercase on both sides.
  That lowercase integrity check is distinct from the uppercase vector
  digests preserved inside the JSON artifact itself.

### Schema version upgrade protocol
  If SCG canonical rules evolve:
    1. SCG increments schema_version in CANONICAL_VECTORS.json
    2. New PROVENANCE.md entry required — not a silent patch
    3. build.rs hash updated to new value
    4. All language bindings must re-validate against new vectors
  Upgrading schema_version without this process = governance violation.
