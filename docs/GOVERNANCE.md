# Iter Governance

> Visitor-facing change control and integrity posture for the Iter Server boundary.

---

## What governance guarantees (high level)

- public-facing surfaces are reviewed and controlled
- security-critical paths require explicit owner approval
- automated checks run on every change to enforce baseline integrity

---

## Change control

This repository uses:

- branch protection on `main`
- required status checks
- CODEOWNERS-required review for security-critical paths

---

## Integrity checks

Automated checks verify that:

- protected artifacts remain consistent and unmodified
- boundary tests continue to pass
- governance artifacts remain in an expected state

Exact checksum values and non-public verification details are intentionally not documented publicly.

---

## Reporting

- governance: governance@onlysgsolutions.com
- security: security@onlysgsolutions.com

