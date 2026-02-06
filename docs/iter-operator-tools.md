# Iter Operator Tools

Consumption-grade CLI for replaying and exporting Iter governance decisions.

All commands operate strictly on governed-mode artifacts and do not introduce new governance decisions. This command performs no state mutation and does not create new audit entries.

## `iter-cli replay`

Replay a DecisionPacket under specified policy/schema versions and verify the outcome.

### Usage

```
iter-cli replay \
  --decision-file <path/to/decision_packet.json> \
  --policy-version <policy_version_id> \
  --schema-version <schema_version_id>
```

### Inputs

- `--decision-file`: Path to a JSON file containing a previously emitted `DecisionPacket`.
- `--policy-version`: Identifier for the policy set against which the decision was originally evaluated. Format: `sha256:<hex_hash>` where the hash is `DecisionPacket.policy.policy_hash`.
- `--schema-version`: Identifier for the DecisionPacket schema version. Currently must be `decision_packet:v1`.

### Behavior

1. Reads and parses the DecisionPacket from disk.
2. Validates JSON well-formedness and required fields.
3. Calls the same `replay_decision()` function used by CI golden vector tests.
4. Verifies: recomputed checksum vs stored checksum, policy/schema version match.

### Output

On success (exit 0):

```json
{
  "outcome": "VERIFIED",
  "decision": "ALLOW",
  "checksum_match": true,
  "policy_version": "sha256:<hash>",
  "schema_version": "decision_packet:v1"
}
```

On mismatch (exit 2):

```json
{
  "outcome": "MISMATCH",
  "reason": "checksum verification failed: ...",
  "policy_version": "sha256:<hash>",
  "schema_version": "decision_packet:v1"
}
```

### Guarantees

- Uses the same replay contract and hashes as CI golden vectors; no semantics changed.
- Fail-closed: any checksum mismatch, policy version mismatch, or schema version mismatch produces exit 2 with a stable reason string.

---

## `iter-cli audit export`

Export and validate a DecisionPacket file. This command performs no state mutation and does not create new audit entries.

### Usage

```
iter-cli audit export \
  --decision-file <path/to/decision_packet.json> \
  --output <path/to/output.json>
```

### Inputs

- `--decision-file`: Path to a JSON file containing a DecisionPacket.
- `--output`: File path to write the validated canonical JSON.

### Behavior

1. Reads and parses the DecisionPacket from disk.
2. Verifies checksum integrity via `verify_checksum()`.
3. Writes the DecisionPacket as canonical JSON to `--output` with identical structure and checksum.

### Output

On success (exit 0):

```json
{
  "status": "EXPORTED",
  "decision_id": "<packet_checksum>",
  "output_file": "<path>"
}
```

On integrity failure (exit 2):

```json
{
  "status": "INTEGRITY_FAILURE",
  "reason": "checksum mismatch: expected ..., got ..."
}
```

### Guarantees

- Does NOT recompute or change the DecisionPacket checksum.
- Does NOT alter the meaning or presence of any DecisionPacket field.
- The exported file is byte-identical to `serde_json::to_string_pretty` of the validated packet.

---

## Exit Codes (Both Commands)

| Code | Meaning |
|------|---------|
| 0 | Success (VERIFIED / EXPORTED) |
| 1 | Input error (file missing, unreadable, malformed JSON, missing required flags) |
| 2 | Replay/contract mismatch or integrity failure |
| 3 | Internal error (unexpected failure not attributable to user input) |

---

## Policy Hash Determinism

`PolicyConfig::compute_hash` uses deterministic `serde_json` field ordering within Rust. Cross-language canonicalization (JCS / RFC 8785) is deferred to a future phase; any change will bump versions and regenerate golden vectors.

The `policy_hash_stability` golden vector test guards current hash behavior. DecisionPacket checksums use JCS canonicalization via `serde_json_canonicalizer`.

---

## Golden Fixture

A reference DecisionPacket is checked in at `tests/data/golden_decision_v1.json`. This file is identical to Golden Vector 1 from `tests/golden_vectors.rs` and can be used for manual replay verification:

```
iter-cli replay \
  --decision-file tests/data/golden_decision_v1.json \
  --policy-version sha256:b1c2d3e4f5a6b1c2d3e4f5a6b1c2d3e4f5a6b1c2d3e4f5a6b1c2d3e4f5a6b1c2 \
  --schema-version decision_packet:v1
```
