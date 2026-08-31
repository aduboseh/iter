# Productization Evidence

This directory is intentionally empty until a certification run produces
control evidence.

Evidence is not a narrative approval. Each file must use schema
`apex-productization-evidence/v1`, identify one release-matrix control, bind
both exact iter and SCG commits, list commands executed, and include at least
one artifact whose SHA-256 digest is verified by
`scripts/verify_productization_matrix.py`.

Missing evidence, stale commit binding, missing artifacts, and digest mismatch
are all FAIL. Do not commit placeholder PASS evidence.

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
