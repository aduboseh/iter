# Credibility Log

This document archives provenance artifacts for Iter credibility milestones.

---

## ITER-A1: Credibility Lock

**Date**: December 2024
**PR**: [#18 – iter-A1 Credibility Lock](https://github.com/aduboseh/iter/pull/18)
**Merged commit**: See PR for final commit hash
**CI run**: Linked in PR checks

### Scope

ITER-A1 addressed all gaps identified in the pre-freeze assessment:

| Gap | Resolution |
|-----|------------|
| Schema version drift (v0.3 references) | Upgraded all schemas to v1.0.0 with GitHub blob `$id` URLs |
| Stale stability baseline (v0.2.0) | Generated fresh `audits/stability_v1.0.0/` baseline |
| Legacy SCG pilot tags | Documented in `TAG_PROVENANCE.md` |
| CODEOWNERS referencing non-existent files | Fixed to match actual repository structure |
| README commands not validated | Verified `public_stub` builds and tests pass |
| Stub documentation | Expanded `docs/ARCHITECTURE.md` and `docs/GOVERNANCE.md` |
| Missing LICENSE file | Added MIT license |

### Artifacts Created

- `LICENSE` – MIT license
- `TAG_PROVENANCE.md` – Explains all tags in repository history
- `CREDIBILITY_CLOSURE.md` – ITER-A1 summary
- `spec/SCHEMA_VERSION_POLICY.md` – Schema versioning rules
- `.github/workflows/sdk_publish_check.yml` – Packaging dry-run workflow
- `audits/stability_v1.0.0/` – CI-enforced baseline

### Test Results

71 governance tests passing in `public_stub` mode at time of merge.

---

## ITER-A2: Canonicalization Pass

**Date**: December 2024
**Tag**: `v1.0.1`
**Branch**: `canonicalization/iter-a2`

### Scope

- Cut `v1.0.1` (signed tag, no functional delta)
- Created `BUYER_README.md` for corporate evaluation
- Added 12-month freeze statement to `README.md` and `RELEASE.md`
- Archived A1 artifacts in this document

### Surface Freeze

Protocol and SDK surface stable for 12 months (through December 2025) barring security issues.

**Note**: v1.0.1 re-tagged at HEAD to correct pre-merge sequencing; no functional delta.

---

## Audit Trail

All credibility milestones are traceable via:

1. Git tags (signed)
2. Pull request history
3. CI workflow logs
4. This log
