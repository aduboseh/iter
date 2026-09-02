# Productization Evidence

This directory contains the evidence schema only. Certification evidence and
its artifacts MUST NOT be committed here because a tracked evidence file cannot
truthfully contain the commit hash that includes itself.

Evidence is not a narrative approval. Each file must use schema
`apex-productization-evidence/v1`, identify one release-matrix control, bind
both exact iter and SCG commits, list commands executed, and include at least
one artifact whose SHA-256 digest is verified by
`scripts/verify_productization_matrix.py`.

Evidence is transported as a GitHub Actions artifact named
`apex-productization-evidence-<iter-commit>-<scg-commit>`. The producing run
must have the exact iter commit as its `head_sha`. Manual certification requires
that run ID; release certification requires exactly one active artifact with
that name. The artifact is downloaded outside both source trees before the
verifier runs.

Missing or ambiguous evidence, stale commit binding, missing artifacts, and
digest mismatch are all FAIL. Do not commit placeholder or completed PASS
evidence.

Minimal shape:

```json
{
  "schema_version": "apex-productization-evidence/v1",
  "control_id": "G1-01",
  "result": "PASS",
  "subject_commits": {
    "iter": "<40-hex commit>",
    "scg": "<40-hex commit>"
  },
  "commands": [
    "cargo test --locked --workspace"
  ],
  "artifacts": [
    {
      "path": "artifacts/G1-01/certification.json",
      "sha256": "<64 lowercase hex>"
    }
  ]
}
```
