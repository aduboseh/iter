# Audit Ledger Ownership and Recovery

An enabled `PersistentAuditLedger` owns one open read/append file handle and a
nonblocking exclusive OS lock until the instance is dropped. Open acquires the
lock before reading or syncing the ledger. A second writer, including another
Iter process or a hard-link alias, fails startup rather than sharing a cached
sequence. The ledger is not reopened by pathname for each append. No new lock
sidecar or persistent record format is introduced.

The native file-lock API requires Rust 1.89 or newer; the repository's pinned
Docker builder uses 1.93.0. On Windows this exclusive lock can also prevent
other handles from reading the ledger. On Unix the lock is advisory: all writers
must cooperate. Use a filesystem that supports these locks and durability
operations; unsupported locking is an error, never a fallback. Do not rotate,
unlink, replace, truncate or externally edit an active ledger. Restrict write
access to its parent directory. This does not defend against a privileged or
noncooperating filesystem writer, nor establish rollback freshness.

## Failure Semantics

The packet/event pair is verified and serialized before attempting I/O. The
instance is marked unhealthy before writing the complete JSONL record and stays
unhealthy if write, flush, sync, or unwinding interrupts persistence. Cached
sequence/hash are advanced only after successful `sync_all`. No decision packet
or in-memory audit event is published by that failing call. Subsequent governed
evaluate and preview operations fail closed; SCG-backed calls fail before
upstream access. Validation errors before I/O do not poison an otherwise healthy
ledger and may be corrected and retried.

A failed sync does not prove that no bytes were written. Never retry a failed
append within the same instance or infer an absent decision from the returned
error. Existing API error variants/codes are retained; the error reports an
uncertain write or poisoned ledger.

## Recovery Procedure

1. Stop the writer and preserve the ledger bytes and failure evidence. Do not
   delete a ledger to clear its lock; the OS releases ownership when the process
   closes or exits, including forced termination.
2. Reopen under exclusive ownership. The entire existing chain is verified before
   serving. Valid records retain their sequences; no cached tail is reused.
3. A partial JSON record, missing final newline, invalid checksum or broken chain
   prevents startup. Do not automatically truncate or append a newline. Recover
   from an independently verified copy under the deployment's recovery procedure.
4. A fully written but previously unacknowledged record can survive an I/O error.
   Reopening must retain it and continue after it, not overwrite it. Reconcile the
   caller's uncertain outcome before resubmitting: this patch adds no request
   deduplication or exactly-once guarantee.

Well-formed `iter.audit.ledger.v1` JSONL remains compatible within the 16 MiB
record limit, including its hashes, float encoding and fsync-before-acknowledgement
order. Unterminated or oversized records are deliberately rejected. The
[`audit.history` tool](AUDIT_HISTORY.md) verifies durable history after restart;
read/integrity failures also poison the writer. Semantic replay and independent
rollback protection remain separate work.
