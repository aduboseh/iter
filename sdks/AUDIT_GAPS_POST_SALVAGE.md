# AUDIT_GAPS_POST_SALVAGE

## Snapshot

- Branch: sdk/ts-client-contract-v1
- Timestamp: 2026-02-04
- git status --short:
  - ?? sdks/python/
  - ?? sdks/sdk-contract.md

## SDK Inventory (Path-Level)

| Path | Exists | Implementation Present | Notes |
|------|--------|------------------------|-------|
| sdks/typescript/src/index.ts | YES | YES | Verified implementation present |
| sdks/rust/src/lib.rs | YES | YES | Blocking sync; missing contract invariants |
| sdks/python/README.md | YES | N/A | README only |
| sdks/python/iter_sdk/ | NO | NO | Absent |

## Contract Claim Status

- Contract defines requirements only.
- Compliance claims removed/omitted pending audit.

## Gap Register (Ranked)

1. [CRITICAL] Python SDK implementation absent
   - Evidence: sdks/python contains README only
2. [HIGH] Rust SDK blocking synchronous implementation
   - Evidence: sdks/rust/src/lib.rs uses std::io, no lifecycle state machine
3. [HIGH] Contract previously contained unverified compliance claims
   - Evidence: sdks/sdk-contract.md prior SDK status matrix
4. [MEDIUM] Build artifacts present under sdks/rust/target and sdks/typescript/node_modules/dist
   - Evidence: repository file listing
