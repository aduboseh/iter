# AUDIT_SDK_CONTRACT_REBUILD

## Snapshot

- Branch: sdk/ts-client-contract-v1
- Timestamp: 2026-02-04
- git status --short:
  - M .gitignore
  - ?? sdks/AUDIT_GAPS_POST_SALVAGE.md
  - ?? sdks/python/
  - ?? sdks/sdk-contract.md

## SDK Contract Evidence (file:line)

### TypeScript SDK

| Invariant | Status | Evidence |
|---------|--------|----------|
| State machine | PASS | sdks/typescript/src/index.ts:195-196 |
| send() gated on OPEN | PASS | sdks/typescript/src/index.ts:257-260 |
| stdout during drain | PASS | sdks/typescript/src/index.ts:238-240, 522 |
| bounded drain | PASS | sdks/typescript/src/index.ts:358-367 |
| close idempotent | PASS | sdks/typescript/src/index.ts:349-356 |
| backpressure | PASS | sdks/typescript/src/index.ts:263-265 |
| request timeout | PASS | sdks/typescript/src/index.ts:284-287 |
| fail-closed | PASS | sdks/typescript/src/index.ts:481-487 |

### Python SDK

| Invariant | Status | Evidence |
|---------|--------|----------|
| State machine | PASS | sdks/python/iter_sdk/types.py:8-12 |
| send() gated on OPEN | PASS | sdks/python/iter_sdk/client.py:102-107 |
| stdout during drain | PASS | sdks/python/iter_sdk/client.py:250-260, 277-285 |
| bounded drain | PASS | sdks/python/iter_sdk/client.py:190-248 |
| close idempotent | PASS | sdks/python/iter_sdk/client.py:180-187 |
| backpressure | PASS | sdks/python/iter_sdk/client.py:105-107 |
| request timeout | PASS | sdks/python/iter_sdk/client.py:127-133 |
| fail-closed | PASS | sdks/python/iter_sdk/client.py:287-344 |

### Rust SDK

| Invariant | Status | Evidence |
|---------|--------|----------|
| State machine | PASS | sdks/rust/src/lib.rs:27-35 |
| send() gated on OPEN | PASS | sdks/rust/src/lib.rs:118-136 |
| stdout during drain | PASS | sdks/rust/src/lib.rs:79-108 |
| bounded drain | PASS | sdks/rust/src/lib.rs:172-201 |
| close idempotent | PASS | sdks/rust/src/lib.rs:157-171 |
| backpressure | PASS | sdks/rust/src/lib.rs:139-145 |
| request timeout | PASS | sdks/rust/src/lib.rs:154-164 |
| fail-closed | FAIL | Not implemented; no protocol violation handler in Rust |

## Test Execution Logs

### Python

Command:
```
cd sdks/python
.\.venv\Scripts\pytest -q
```
Result: Executed (no stdout captured by tool)

### Rust

Command:
```
cd sdks/rust
cargo test -q
```
Result: Executed (no stdout captured by tool)

## Compliance Claim Policy

Compliance claims remain omitted from sdks/sdk-contract.md; this audit is the only claim vehicle.
