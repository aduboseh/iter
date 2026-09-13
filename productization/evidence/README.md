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
must have the exact iter commit as its `head_sha`, conclude successfully, and
come from the trusted workflow
`.github/workflows/apex_productization_evidence.yml` through an independent
`workflow_dispatch` run on a protected source branch (including protected release
branches). The subject must be that branch's current tip or its verified ancestor;
an unprotected PR head or a matching ref name alone is insufficient. Only the first run attempt is
accepted: v1 does not bind artifacts or approvals to a rerun attempt. Dispatch a
new run instead. Manual certification requires that run ID; release
certification requires exactly one active artifact with that name. The artifact
is downloaded outside both source trees before the verifier runs.

The trusted producer workflow is a GA prerequisite and is intentionally absent
while the external certification controls remain incomplete. Until it is added
with protected-environment approval and control-specific collectors, evidence
controls fail closed. An artifact from any other workflow is never accepted.

Both evidence-consuming workflows and direct matrix execution authenticate the
run against GitHub's read-only API, not fields in the evidence JSON. They require
the `production-certification` environment to restrict deployments to protected
branches, prevent self-review, and name individual required reviewers. The run's
review history must contain approval for that exact environment ID by a configured
human account other than either initiating actor. Rejected reviews, missing
metadata, and API failures fail closed. Team-only reviewer policies are not yet
supported. GitHub CLI and read access to Actions and contents are
required. The preflight can be exercised separately:

```text
python -B scripts/verify_productization_matrix.py --verify-evidence-run --evidence-run-id <run-id>
```

The hosted consumers execute the verifier and pinned control matrix from a
separate `aduboseh/iter` `main` checkout, never the candidate's verifier. Both
preflight and evaluation use isolated Python (`-I -B`) and take the subject via
`--iter-root iter`; the resolved authority commit is recorded in the job log.
An unavailable or incompatible authority fails closed. A local CLI invocation
is trustworthy only to the extent that the caller trusts its verifier checkout.

Release PRs run readiness checks only: no certification, private SCG checkout,
or evidence download. Post-merge release pushes require certification success;
skipped, cancelled, and failed certification cannot pass the final release gate.
Manual certification dispatches must originate on `main`. These workflow guards
do not replace repository protection of workflow changes or independent approval;
a PR readiness result is not a release authorization or a certification result.

This authenticates account-level approval, not independent human acceptance or
the scientific validity of a control claim. An alternate account owned by the
same person is not an independent operator. The v1 bundle checks remain structural;
behavioral collectors, complete execution identity, protected producer jobs,
short-lived cross-repository authentication and private artifact transport must
be implemented before producer activation. Do not add a producer that simply
copies PASS declarations or repackages local smoke reports as certification.

Missing or ambiguous evidence, stale commit binding, missing artifacts, and
digest mismatch are all FAIL. Do not commit placeholder or completed PASS
evidence.

Minimal shape:

```json
{
  "schema_version": "apex-productization-evidence/v1",
  "control_id": "G1-01",
  "result": "PASS",
  "producer": {
    "repository": "aduboseh/iter",
    "workflow": ".github/workflows/apex_productization_evidence.yml",
    "run_id": 123456789
  },
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
