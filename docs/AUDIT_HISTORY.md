# Durable History and Verification Scope

`audit.history` reads the configured `ITER_AUDIT_LEDGER_PATH` in governed-local
and scg-backed modes. It does not contact SCG or use the current-process cache.
An absent ledger configuration is an error, not an empty archive. Production
deployments should retain `ITER_REQUIRE_AUDIT_LEDGER=1` and protect that path.
Demo mode has no durable history API.

## Query Contract

```json
{"start_sequence": 0, "limit": 100}
```

Both fields are optional with the defaults shown. `start_sequence` is an
inclusive unsigned ledger sequence; `limit` must be an integer from 1 to 100.
Unknown fields, nulls, invalid types and a cursor past the verified tail are
errors. A cursor exactly at the tail returns a verified empty page.

The response contains:

- `verification: "integrity_only"`: checksums and the ledger chain were verified.
- `records`: contiguous `iter.audit.ledger.v1` records in sequence order, including
  the original event and packet. Duplicate packet checksums at different ledger
  sequences remain separate records; sequence is the pagination identity.
- `next_sequence`: pass this value as the next `start_sequence`; null marks the
  end of the ledger at this read, not a promise that no future records will exist.
- `verified_next_sequence`: the exclusive end of the entire verified ledger.
- `verified_record_hash`: the hash of that tail, or the zero hash for an empty ledger.

The entire chain is verified before any page is returned, including records
outside the requested page. Reads use the writer's locked handle and require
exclusive mutable access; no second filesystem reader or unlocked snapshot is
opened. A failed read or chain/tail verification poisons the instance, refuses
the whole response, and blocks later evaluation/append until recovery. Invalid
query parameters do not poison it. See [recovery](AUDIT_LEDGER_RECOVERY.md).

Every query scans the ledger: O(total ledger bytes), not O(page size). Memory is
bounded by a page plus individual records and their verification copies, rather
than loading the entire file. Records are limited to 16 MiB of JSONL bytes
including the terminating newline, on append and reopen. A page is capped at
16 MiB of original serialized record bytes and 100 records, so it can contain
fewer records than requested. This is not an exact bound on JSON response bytes
or process memory. Follow `next_sequence`, not `records.len() == limit`.

## Integrity Is Not Semantic Replay

Packet checksums and chain links prove internal byte consistency. They do not
prove that a recorded verdict is correct, identify an independent signer, or
detect a fully rewritten/rolled-back chain across restart without a trusted
external checkpoint. The current packet format does not capture all original
request, evaluator configuration and state needed for semantic re-execution.

Consequently evaluated outcomes and runtime metadata report
`replay_sufficient: false`. Live authoritative evaluations still report
`authoritative_pdp: true`; inspecting a stored packet does not.

- Governed `audit.replay` and `lineage.replay` now return error 5003 directing
  clients to `audit.history`. They never return an empty cache as complete history.
- `audit.search` remains a current-process summary search, not a durable archive.
- The Rust `replay_decision` name is retained; it checks integrity and expected
  versions, returns the stored verdict, and reports both replay sufficiency and
  authoritative PDP as false.
- `iter-cli replay` retains its flags and exit codes. Successful inspection now
  emits `outcome: "INTEGRITY_VERIFIED"`, `verification: "integrity_only"`,
  `semantic_replay: false`, and `authoritative_pdp: false`.
- The legacy Rust `ScgRuntime::replay_decisions` inspects only its last 1,000
  session packets. Its success status is now `integrity_verified`, not `match`.
  Use `GovernanceRuntime::history` for durable evidence.

## Compatibility and Operations

No packet/record hashes, canonicalization rules, digest casing, JSONL schema or
release pins change. Existing records up to the byte limit remain readable.
Oversized existing records fail startup; preserve them for reviewed recovery,
never truncate or delete evidence to force a start. Clients consuming the old
replay success labels must update deliberately; the old semantic claim was not
supported by execution. New Rust enum variants can require exhaustive match updates.

The history tool exposes original packet/event contents under the existing MCP
access boundary. Treat them as sensitive audit data; this endpoint adds neither
redaction nor a new authorization system. Copying pages is not a retention policy.

Reads do not mutate ledger bytes, advance decision ticks, or invoke evaluation.
Rollback requires stopping the writer; older binaries can read compatible bytes
but restore the old missing-history and overstated-verification behavior.
