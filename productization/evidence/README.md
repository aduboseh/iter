# Productization Evidence

This directory contains the Draft 7 `evidence-v2.schema.json` and its contract. Certification evidence and
its artifacts MUST NOT be committed here because a tracked evidence file cannot
truthfully contain the commit hash that includes itself.

Evidence is not a narrative approval. Each file must use schema
`apex-productization-evidence/v2`, identify one release-matrix control, bind
both exact iter and SCG commits, and capture declared execution identity and
artifact bytes. `scripts/verify_productization_matrix.py` validates the record
using only the Python standard library. Legacy v1 and unknown versions fail
closed; there is no automatic migration or fallback. No active producer exists
to migrate. Future collectors must explicitly emit v2.

## Record Contract

Every field below is required. Unknown fields, duplicate JSON keys and nonfinite
JSON numbers are rejected rather than silently defaulted or normalized.

| Field | Required content |
| --- | --- |
| `schema_version`, `control_id`, `result` | Exact v2 version, selected matrix control, `PASS` or `FAIL`; only `PASS` is accepted as passing evidence. |
| `producer` | Exact trusted repository/workflow, positive integer `run_id`, integer `run_attempt: 1`. All must agree with authenticated transport. |
| `subject_commits` | Exact lowercase 40-hex `iter` and `scg` commits. |
| `execution_identity.runner` | Nonblank `os`, `architecture`, `image` identity strings. For containers use the immutable image digest; hosted collectors must record their runner image version. |
| `execution_identity.toolchains` | Nonempty array of `name`, exact `version`, and `version_log` artifact reference. |
| `execution_identity.corpus`, `configuration`, `result` | References to files capturing the actual inputs, configuration and observed result; each is bound by an artifact digest. |
| `commands` | Nonempty array of `repo` (`iter` or `scg`), repository-relative `cwd` (`.` for root), exact `argv`, integer `exit_code` and `expected_exit`, plus `stdout` and `stderr` artifact references. |
| `artifacts` | Nonempty array of unique portable `path` and lowercase SHA-256 digest `sha256`, checked against the authenticated archive bytes. |

Every corpus/configuration/result, toolchain version log and command log reference
must name a declared, verified artifact, not a digest without bytes. Multi-file
corpora need a collector-defined manifest and complete artifact list; validating
its completeness and interpreting control-specific results remain collector duties.
Empty output files and empty non-executable argv operands are valid and preserved.
The executable must be nonblank. Exit codes must match the expected outcome;
intentional negative tests may declare a nonzero expected exit. No command in an
evidence record is executed by the consumer.

The schema is a structural minimum, not the full acceptance policy. Python also
enforces current subjects, the authenticated run, portable paths, unique artifact
declarations, byte digests, cross-field references and exit equality. It requires
integer JSON tokens for run IDs, attempts and exit codes (rejecting `1.0` as well
as booleans); Draft 7 considers `1.0` mathematically integral. Rust tests use the
existing `jsonschema` dependency to independently validate the shared, explicitly
synthetic conformance fixture in `tests/data/productization_evidence_v2.json`.
That fixture is not execution evidence and must never be uploaded as certification.

This contract establishes declared execution identity and byte binding, not proof
that commands ran, image identities are genuine, the result satisfies a control,
or a signer attested to it. Behavioral collectors and attestation policy remain
separate prerequisites. An advisory `--allow-failures` exit zero does not change
a rejected record's report status from `FAIL`.

## Authenticated Transport

Evidence is transported as a GitHub Actions artifact named
`apex-productization-evidence-<iter-commit>-<scg-commit>`. The producing run
must have the exact iter commit as its `head_sha`, conclude successfully, and
come from the trusted workflow
`.github/workflows/apex_productization_evidence.yml` through an independent
`workflow_dispatch` run on a protected source branch (including protected release
branches). The subject must be that branch's current tip or its verified ancestor;
an unprotected PR head or a matching ref name alone is insufficient. Only the first run attempt is
accepted: the transport policy does not authenticate approvals and artifacts for reruns. Dispatch a
new run instead. Manual certification requires that run ID; release
certification requires exactly one active artifact with that name. After authenticating
the run, the verifier selects exactly one matching artifact from that run, checks its
immutable ID and repository identity, and downloads the raw ZIP from GitHub's API.
The exact bytes must match the API's lowercase `sha256:` digest and byte count before
any ZIP member is read. A missing digest fails closed, including for older artifacts.

Evidence is consumed from immutable in-memory bytes, never from an extracted directory.
Both hosted workflows use this same verifier path; there is no separate downloader
that can turn a digest mismatch into a warning. Local bundles with recomputed hashes
are not an authenticated substitute: `--evidence-dir` is rejected for evidence controls,
even with `--allow-failures`. Direct evaluation uses:

```text
python -I -B scripts/verify_productization_matrix.py --iter-root <iter-checkout> --scg-root <scg-checkout> --evidence-run-id <run-id> --report <report-path>
```

The consumer permits at most 32 MiB of ZIP bytes, 128 MiB of expanded data, 32 MiB
per file, and 1,024 members. These conservative limits must be checked against real
collector sizes before producer activation. Only stored/deflated, unencrypted regular
files and directories with portable ASCII relative names are accepted. Duplicate
members, case aliases, file/directory conflicts, traversal, device names and links
are rejected. Nothing is extracted, so failure leaves no partial evidence directory.
Downloads use a 30-second socket timeout and a 120-second stream deadline (plus at
most one socket timeout). Read-only failures may be retried with a new invocation;
no fallback to stale or local evidence is permitted.

Reports include the API-derived artifact ID, run, name, digest, byte count, file count
and exact subject commits. `verifier_authority_commit` identifies the separate
verifier checkout. This binds transport origin, not the truth of declared results.

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
`--iter-root iter`; the resolved authority commit is recorded in the job log and report.
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
same person is not an independent operator. The v2 bundle checks remain structural;
behavioral collectors, trustworthy execution capture, protected producer jobs,
short-lived cross-repository authentication and private artifact transport must
be implemented before producer activation. Do not add a producer that simply
copies PASS declarations or repackages local smoke reports as certification.

Missing or ambiguous evidence, stale commit binding, missing artifacts, and
digest mismatch are all FAIL. Do not commit placeholder or completed PASS
evidence.

Illustrative shape (placeholders are intentionally invalid, not evidence):

```json
{
  "schema_version": "apex-productization-evidence/v2",
  "control_id": "G1-01",
  "result": "PASS",
  "producer": {
    "repository": "aduboseh/iter",
    "workflow": ".github/workflows/apex_productization_evidence.yml",
    "run_id": 123456789,
    "run_attempt": 1
  },
  "subject_commits": {
    "iter": "<40-hex commit>",
    "scg": "<40-hex commit>"
  },
  "execution_identity": {
    "runner": { "os": "<os>", "architecture": "<arch>", "image": "<image version or digest>" },
    "toolchains": [{ "name": "rustc", "version": "<exact version>", "version_log": "logs/rustc.log" }],
    "corpus": "inputs/corpus.json",
    "configuration": "inputs/config.json",
    "result": "outputs/result.json"
  },
  "commands": [{
    "repo": "iter", "cwd": ".", "argv": ["cargo", "test", "--locked", "--workspace"],
    "exit_code": 0, "expected_exit": 0,
    "stdout": "logs/tests.stdout.log", "stderr": "logs/tests.stderr.log"
  }],
  "artifacts": [
    {
      "path": "outputs/result.json",
      "sha256": "<64 lowercase hex>"
    }
  ]
}
```

The example omits the remaining artifact declarations for brevity; every referenced
file must also be declared with its exact digest. Reverting to v1 would reopen
incomplete-record acceptance and must keep certification blocked. Runtime public
APIs, persistence formats, SCG digest casing and canonical payload rules are unchanged.
