# APEX Productization Controls

The canonical release definition is
[`APEX_RELEASE_MATRIX_V1.json`](APEX_RELEASE_MATRIX_V1.json). It contains
exactly 30 release controls derived from
[`APEX-SCG-ITER-PROD-001`](../APEX_PRODUCTIZATION_V1.md).

## Semantics

- PASS means every declared check passed against the exact iter and SCG commits.
- FAIL includes command failure, zero matched Cargo tests, missing repositories,
  missing evidence, stale commit binding, missing artifacts, or hash mismatch.
- There is no pending, skipped, waived, or assumed-pass state.
- The matrix does not grant release authority. It supplies evidence to the
  release authority.

## Validate The Definition

```text
python scripts/verify_productization_matrix.py --validate-only
```

## Run One Control

From the iter repository, with SCG checked out as a sibling:

```text
python scripts/verify_productization_matrix.py --control G0-10
```

## Run The Complete Matrix

```text
python scripts/verify_productization_matrix.py \
  --iter-root /path/to/iter \
  --scg-root /path/to/SCG \
  --evidence-dir /path/to/evidence \
  --evidence-run-id 123456789 \
  --report /path/to/apex-productization-report.json
```

Replace `123456789` with the actual successful trusted evidence workflow run ID.

The command exits nonzero unless every selected control and the byte-identical
directive-mirror check pass. A successful selected subset reports `PARTIAL`,
never full certification `PASS`. `--allow-failures` is permitted only for
generating a baseline report; it does not change any reported FAIL status.

## Evidence

See [`evidence/README.md`](evidence/README.md). Evidence must bind both source
commits and SHA-256-bind every referenced artifact. Evidence is supplied as an
external GitHub Actions artifact so it can name the exact iter commit without
self-reference. Only a successful run of the declared trusted evidence workflow
is accepted. Placeholder or repository-tracked PASS evidence is prohibited.

## SCG Subject Pin

productization/SCG_RELEASE_REF contains the exact 40-hex SCG commit certified with
the iter release. The verifier and both release workflows fail if the checked-out
SCG subject differs from this pin. Moving branch names are prohibited.

## CI

Pull requests test the verifier and validate the matrix definition. Full
certification is manually dispatched with an explicit immutable SCG commit and
the run ID that produced the external evidence bundle. Release branches and
tags locate exactly one non-expired evidence artifact whose name contains both
subject commits. Release-PR certification consistently uses the PR head commit,
not GitHub's synthetic merge commit. Both repositories must be clean before the
controls run and remain clean at the same commits afterward. A moving branch is
not valid release evidence.
