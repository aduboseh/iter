# Replay and Lineage

**Classification:** External-Safe  
**Version:** 1.0  
**Last Updated:** February 2026

---

## Replay Definition

**Replay:** Reconstructing a governance decision using archived DecisionPacket data without re-running model inference.

**Replay is NOT:**
- Re-running the model
- Re-generating reasoning signals
- Re-computing embeddings
- Re-fetching external data

**Replay IS:**
- Reading DecisionPacket from archive
- Extracting inputs and configuration
- Re-evaluating governance logic
- Comparing checksums

---

## Deterministic Regeneration

### Process

```
1. Read DecisionPacket from archive
2. Extract:
   - tick
   - energy (nodes, reservoir, integrity)
   - reasoning (quality, value_signal, conflict_signal, control_signal)
   - learning (capsule_id, epoch, version_hash, costs, quality, status, scarcity_streak)
   - policy_hash
   - economics_hash
   - permit_hash
3. Load policy configuration (by policy_hash)
4. Load economics configuration (by economics_hash)
5. Re-evaluate governance gates using extracted data
6. Generate new DecisionPacket
7. Compare checksums:
   - Match → replay successful
   - Mismatch → investigation required
```

### Checksum Verification

```rust
fn verify_replay(archived_packet: DecisionPacket) -> Result<(), ReplayError> {
    let replayed = replay_decision(archived_packet.clone())?;
    if archived_packet.checksum == replayed.checksum {
        Ok(())
    } else {
        Err(ReplayError::ChecksumMismatch {
            expected: archived_packet.checksum,
            actual: replayed.checksum,
        })
    }
}
```

---

## Lineage Verification

### What is Lineage?

Lineage is the causal chain from inputs → governance → decision.

**Lineage includes:**
- Which policy version was active
- Which gates were evaluated
- Which reason codes were emitted
- Which configuration hashes were used

**Lineage does NOT include:**
- How the model generated reasoning signals (upstream responsibility)
- How the application used the decision (downstream responsibility)

---

## Lineage Reconstruction

### From DecisionPacket

```
DecisionPacket → Extract:
  - policy_hash → load policy definition
  - economics_hash → load economics config
  - evaluated_rules → identify gate execution order
  - reason_codes → identify failure points
  - decision → identify final outcome

Result: Complete causal chain from policy to decision
```

### Verification Steps

1. **Policy Verification**
   - Load policy by `policy_hash`
   - Confirm policy was active at decision `tick`

2. **Gate Verification**
   - Confirm gates were evaluated in declared order
   - Confirm no gates were skipped

3. **Reason Code Verification**
   - Confirm reason codes correspond to failed gates
   - Confirm no spurious reason codes

4. **Decision Verification**
   - Confirm decision matches gate outcomes
   - Confirm decision is deterministic

---

## Replay vs Re-Inference

| Operation | Replay | Re-Inference |
|-----------|--------|--------------|
| Uses model | No | Yes |
| Uses archived data | Yes | No |
| Deterministic | Yes | No (stochastic models) |
| Verifies governance | Yes | No |
| Verifies model | No | Yes |
| Execution time | ~ms | ~seconds |

**Use replay to verify governance.**

**Use re-inference to verify model.**

---

## Audit Trail Integration

### AuditEvent Correlation

```
DecisionPacket (tick=42) ↔ AuditEvent (tick=42, phase="governance_evaluation")

Verification:
- Every DecisionPacket has corresponding AuditEvent
- AuditEvent timestamps are monotonic
- No missing ticks
```

### Lineage Query

```sql
SELECT 
  packet.tick,
  packet.policy_hash,
  packet.decision,
  packet.reason_codes,
  audit.timestamp,
  audit.phase
FROM decision_packets AS packet
JOIN audit_events AS audit ON packet.tick = audit.tick
WHERE packet.decision = 'DENY'
ORDER BY packet.tick DESC;
```

---

## Replay Failure Modes

| Failure | Cause | Response |
|---------|-------|----------|
| Checksum mismatch | Replay logic error, corruption, policy drift | Investigate, halt operations |
| Missing policy | Policy not archived | Cannot replay, manual review required |
| Missing economics config | Config not archived | Cannot replay, manual review required |
| Invalid enum value | Contract version mismatch | Upgrade client, retry |

---

## Replay Guarantees

**Guarantee 1:** Replay produces identical checksum for identical inputs.

**Guarantee 2:** Replay uses only DecisionPacket contents (no external dependencies).

**Guarantee 3:** Replay failure is detectable (checksum mismatch).

**No silent replay failures.**
