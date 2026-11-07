# APEX v1.2 — Repository Governance (PROPOSED)

The root `AGENTS.md` steers Codex; it cannot enforce CI, secret custody, approvals, or runtime guarantees. This directory contains a supplemental **heuristic** Git diff guard and synthetic tests. It does not certify security or compliance.

## Contents
- `apex-guard-v1.2.sh`: read-only diff scanner. Must run from trusted base-branch code against the candidate commit.
- `protected-paths.txt`: repository-specific sensitive paths; the scanner also has built-ins.
- `test_apex_guard_v1.2.py`: synthetic tests for the guard's known behavior.
- `../.github/workflows/apex-guard.yml`: candidate CI wiring; not independently trusted until base-branch review and required GitHub rulesets.

## Local validation (Bash + Git + Python 3)
```bash
python3 .apex/test_apex_guard_v1.2.py
APEX_GLOBAL_BYTES=0 bash .apex/apex-guard-v1.2.sh <base-commit>
```
Set `APEX_GLOBAL_BYTES` to the confirmed byte contribution of globally loaded Codex project instructions if applicable. Zero means *unknown/not counted*, not proof that the global prompt loaded.

## Initial bootstrap is intentionally blocked
The first PR adding this guard will receive a failing "APEX bootstrap" check, because there is no base-branch copy to trust. A human must review the proposed files and use a separately authorized, explicitly documented repository governance action to install the trusted baseline. **Do not weaken this check to make the bootstrap green.** Subsequent protected-path changes are review findings, not automatic approvals.

## Manual controls required in GitHub
Set branch rules for the actual default branch: block force pushes/direct writes, require PR reviews and CODEOWNER approvals where supported, require appropriate CI status checks, enable secret scanning/push protection where available, and restrict deploy secrets and environments. Verify CODEOWNERS handles used in this repository resolve to real reviewers.

## Scope of evidence
The included tests exercise synthetic repositories only. Before asserting operational enforcement, capture the exact ruleset configuration, CI run, reviewer decision and Codex canary output for the installed commit. Agent-written policy proposals are not approvals.
