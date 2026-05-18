# Iter-SCG Contract

Status: canonical for the vendored `scg.v1` governance bridge.

Iter vendors a small SCG bridge crate so the product-facing runtime can emit replayable proof packets without making SCG a direct runtime build dependency. SCG remains the authoritative governance substrate. Iter remains the product and MCP control plane.

## Contract-Critical Artifacts

The Iter build script validates these files before compiling the runtime:

- `vendor/governance-bridge/src/contract.rs`
- `vendor/governance-bridge/src/trace.rs`
- `vendor/governance-bridge/src/errors.rs`
- `vendor/governance-bridge/src/lib.rs`
- `vendor/governance-bridge/CANONICAL_VECTORS.json`

Cargo must rerun `build.rs` when any of those files changes. The build script also watches `vendor/governance-bridge` and `build.rs` itself so Cargo cache reuse cannot bypass governance integrity checks.

## Runtime Proof Packet Provenance

Every `DecisionPacket` includes:

- `replay_scope`: same-binary replay scope, build platform, rustc version, and `cross_platform_replay_claimed=false`
- `contract_provenance`: actual compile-time values exported by `build.rs` via `cargo:rustc-env`
- `provenance_source`: source declarations for contract values, decision values, proof-critical numeric encoding, canonical vector integrity, and vector digest casing

Static contract facts must come from compile-time build exports. Dynamic decision values must come from runtime execution. Proof-critical numeric values must be serialized as exact IEEE-754 lowercase hex strings (`ieee754-f64-bits-lowerhex`), not JSON float numbers. A proof packet must not mix hand-maintained literals with runtime values.

Required `provenance_source` block:

```json
{
  "contract_values": "compile_time_build_rs_rustc_env",
  "decision_values": "runtime_execution",
  "numeric_encoding": "ieee754-f64-bits-lowerhex",
  "canonical_vector_integrity": "raw_byte_sha256",
  "vector_digest_casing": "raw_text_validation"
}
```

## Canonical Vector Validation

Canonical vector integrity has two independent gates:

- SMOKE-008 reads `CANONICAL_VECTORS.json` as raw bytes and computes SHA-256 over the exact file bytes.
- SMOKE-009 reads `CANONICAL_VECTORS.json` as raw UTF-8 text and verifies the embedded `sha256` fields preserve uppercase hex exactly.

Integrity validation must not rely on serde round trips, parsed struct comparison only, formatter-normalized JSON, newline-normalized hashing, or lowercase-normalized vector digests.

## Why there are two SCG commits

This contract records two SCG commits because they prove different things.

The SCG source commit identifies the exact bridge code state that was vendored into Iter.

The SCG vendor master head proves that the vendored bridge logic entered SCG's governed master lineage, rather than being pulled from an unreviewed or private branch.

Together, they answer two separate audit questions:

1. What exact code did Iter vendor?
2. Was that code accepted into SCG's governed source history?

Both are required. The source commit proves byte-level origin. The master head proves governed lineage.

In code, `ITER_SCG_VENDOR_MASTER_HEAD` means the governed SCG master head at vendor acceptance time. It is not a claim about the current SCG master head.

## Failure Modes

| Failure mode | Trigger | Expected behavior | Detection method | Recovery path |
|---|---|---|---|---|
| `BUILD_SCRIPT_RERUN_TRIGGER_MISSING` | A contract-critical file changes without a matching `cargo:rerun-if-changed` declaration. | CI fails review of build-script coverage. | Inspect `build.rs` for all watched paths. | Add the missing rerun trigger before changing the artifact. |
| `CARGO_CACHE_GOVERNANCE_BYPASS` | Cargo reuses stale build-script output after a governance artifact changes. | Build must rerun `build.rs` and fail closed on mismatch. | `ITER_SIMULATE_DRIFT=1 cargo build` must fail with `BRIDGE_INTEGRITY_MISMATCH_SIMULATED`. | Add or fix rerun triggers; never add warning-only behavior. |
| `RUSTC_ENV_PROVENANCE_EXPORT_MISSING` | A contract-critical hash, commit, or identifier is not exported through `cargo:rustc-env`. | Runtime compile or provenance tests fail. | Compile-time `env!(...)` access and packet assertions. | Export the missing value in `build.rs` and bind it into `DecisionPacket`. |
| `PROOF_PACKET_PROVENANCE_DRIFT` | Runtime proof packets use duplicated literals instead of compile-time exports. | Packet provenance tests fail. | Compare `DecisionPacket.contract_provenance` with `env!(...)` values. | Replace literals with `env!(...)`-backed provenance fields. |
| `PROOF_CRITICAL_NUMERIC_ENCODING_BYPASSED` | DecisionPacket proof-critical numeric fields are serialized as JSON numbers or deserialize without validation. | Packet schema, fixture, or replay tests fail. | Assert numeric fields are 16-char lowercase hex strings and packet verification rejects invalid ranges. | Restore `ieee754-f64-bits-lowerhex` serialization and fail-closed packet validation. |
| `RAW_BYTE_INTEGRITY_VALIDATION_BYPASSED` | Canonical vector integrity is checked only through parsed JSON or normalized output. | SMOKE-008/009 fail. | Raw byte hash and raw text casing tests. | Restore raw byte hashing and raw text digest-casing checks. |
| `DRIFT_SIMULATION_FAILED` | `ITER_SIMULATE_DRIFT=1 cargo build` succeeds or fails with the wrong code. | CI fails. | CI grep for `BRIDGE_INTEGRITY_MISMATCH_SIMULATED`. | Fix in-memory drift path in `build.rs`. |
| `WORKING_TREE_MUTATED_BY_DRIFT_TEST` | Drift simulation writes to vendored files, vectors, or build script. | CI fails. | `git diff --quiet -- vendor/governance-bridge build.rs`. | Keep drift simulation in memory only. |
| `SCG_MASTER_LINEAGE_UNEXPLAINED` | Contract docs omit why source commit and master-lineage head are both required. | Documentation review fails. | This document must contain the two-commit explanation above. | Restore the explanation before release. |

## Golden Path Evidence

Successful product-finish validation must emit:

```text
GOLDEN_PATH_PASS
contract_version=scg.v1
claim_registry_version=1.0
determinism_scope=same_binary_only
platform=<target_triple>
rustc_version=<version>
cross_platform_replay_claimed=false
scg_source_commit=da14c8390ba8ceeb0ab15d85c598d2042a2029cf
scg_vendor_master_head=3e0675073a50ce20bdad7c342f7a5caaa3801504
build_rerun_triggers=verified
rustc_env_exports=verified
bridge_integrity=verified
canonical_vectors_raw_byte_hash=verified
canonical_vector_uppercase_digests=verified
trace_semantics=verified
proof_packet=<actual_path>
proof_packet_provenance=compile_time_exports+runtime_decision
proof_numeric_encoding=ieee754-f64-bits-lowerhex
replay_verification=verified
drift_simulation=verified
working_tree_mutated=false
```

## AAVP Hard Gate

1. Can an evaluator understand the product without SCG theory?
2. Can an evaluator run the golden path without help?
3. Does the output prove the bridge is pinned?
4. Does the output prove the contract version is `scg.v1`?
5. Does `build.rs` rerun when contract-critical vendor files change?
6. Does `build.rs` export contract provenance into the runtime binary?
7. Does the proof packet use compile-time provenance exports?
8. Does canonical vector validation use raw bytes and raw text?
9. Does the drift simulation prove fail-closed behavior without mutating the working tree?
10. Does the contract doc explain why both SCG source commit and vendor master head matter?
11. Would a skeptical evaluator view this as reproducible evidence rather than architecture theater?

Acceptance threshold: all 11 pass. Any failure returns the work to execution.
