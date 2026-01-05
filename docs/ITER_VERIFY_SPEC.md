# iter-verify CLI Specification

**Version:** 1.0  
**Status:** SPECIFICATION (not yet implemented)  
**Target:** Phase 0 / Phase 3 deliverable

## Purpose

External artifact verification tool for Iter governance decisions. Consumes replay artifacts and verifies determinism without access to Iter internals.

## Usage

```bash
# Verify single artifact
iter-verify verify artifact.bin

# Verify directory of artifacts
iter-verify verify ./artifacts/

# Verify with JSON output
iter-verify verify ./artifacts/ --format json

# Batch verification from manifest
iter-verify batch manifest.json
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | All artifacts verified successfully |
| 1 | One or more artifacts failed verification |
| 2 | Tool error (invalid input, parse failure, etc.) |

## Output Formats

### Default (human-readable)

```
Verifying 3 artifacts...
  [PASS] artifact-001.bin (coherence: 0.92, drift: 6.1e-11)
  [PASS] artifact-002.bin (coherence: 0.88, drift: 0.0)
  [FAIL] artifact-003.bin (fingerprint mismatch)

Result: 2/3 verified, 1 failed
```

### JSON (`--format json`)

```json
{
  "verified": false,
  "total": 3,
  "passed": 2,
  "failed": 1,
  "max_drift": 6.1e-11,
  "coherence_min": 0.88,
  "results": [
    {
      "artifact": "artifact-001.bin",
      "status": "PASS",
      "fingerprint": "abc123...",
      "coherence": 0.92,
      "drift": 6.1e-11
    },
    {
      "artifact": "artifact-002.bin",
      "status": "PASS",
      "fingerprint": "def456...",
      "coherence": 0.88,
      "drift": 0.0
    },
    {
      "artifact": "artifact-003.bin",
      "status": "FAIL",
      "reason": "fingerprint_mismatch",
      "expected": "ghi789...",
      "actual": "xyz000..."
    }
  ]
}
```

## Commands

### `verify`

Verify one or more artifacts.

```
iter-verify verify <PATH> [OPTIONS]

Arguments:
  PATH    File or directory containing artifacts

Options:
  --format <FORMAT>    Output format: text, json [default: text]
  --strict             Fail on any warning
  --quiet              Suppress non-error output
```

### `batch`

Verify artifacts listed in a manifest file.

```
iter-verify batch <MANIFEST> [OPTIONS]

Arguments:
  MANIFEST    JSON manifest listing artifacts to verify

Options:
  --format <FORMAT>    Output format: text, json [default: text]
  --parallel <N>       Parallel verification threads [default: 4]
```

Manifest format:
```json
{
  "artifacts": [
    { "path": "./artifact-001.bin", "expected_cih": "abc123..." },
    { "path": "./artifact-002.bin", "expected_cih": "def456..." }
  ]
}
```

### `info`

Display artifact metadata without full verification.

```
iter-verify info <PATH>

Output:
  Schema version: 1
  Proposal ID: enf-123
  Timestamp: 2026-01-05T01:50:00Z
  CIH: abc123...
  Artifact hash: def456...
```

## Implementation Notes

### Build from existing drift-harness

The `haltra-internal/rust/drift-harness` crate contains artifact parsing and verification logic. `iter-verify` should:

1. Extract verification logic into a standalone binary
2. Remove Haltra-specific dependencies
3. Add CLI argument parsing (clap)
4. Add JSON output formatting

### No domain logic

`iter-verify` MUST NOT contain:
- Haltra business rules
- Ethics interpretation
- ROI economics
- Domain-specific validation

It verifies **determinism** only: fingerprint match, drift bounds, coherence.

### Artifact schema

```
[version: u8]
[input_digest: [u8; 32]]
[policy_hash: [u8; 32]]
[tick_ms: u64]
[state_len: u32][state_snapshot: [u8; state_len]]
[decision_len: u32][decision_output: [u8; decision_len]]
[lineage_hash: [u8; 32]]
```

## CI Integration

```yaml
# Example GitHub Actions usage
- name: Verify governance artifacts
  run: |
    iter-verify verify ./proof/artifacts/ --format json > verify-report.json
    if [ $? -ne 0 ]; then
      echo "Artifact verification failed"
      exit 1
    fi
```

## Security Considerations

- `iter-verify` is read-only; it cannot modify artifacts
- No network access required
- No secrets or credentials consumed
- Safe for use in untrusted CI environments

## Timeline

| Milestone | Target |
|-----------|--------|
| Spec finalized | Phase 0 |
| Implementation | Phase 3 |
| CI integration | Phase 3 |

## Contact

- **Iter protocol:** engineering@haltra.app
- **Repository:** https://github.com/aduboseh/iter
