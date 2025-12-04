# SCG MCP Governance

> Change control, integrity verification, and compliance enforcement

---

## Overview

The SCG MCP Server operates under strict governance to ensure:
- **Integrity**: No unauthorized modifications to security boundaries
- **Consistency**: Manifest parity between SCG and MCP repos
- **Auditability**: Complete change history with approval trails

---

## Governance Architecture

```
┌──────────────┐     ┌──────────────┐
│   SCG Repo   │     │  MCP Server  │
│              │     │              │
│ governance/  │────▶│ governance/  │
│ SCG_Gov_v1.0 │     │ SCG_Gov_v1.0 │
└──────────────┘     └──────────────┘
       │                    │
       │    SHA-256 match   │
       └────────┬───────────┘
                │
                ▼
       ┌────────────────┐
       │ CI Verification│
       │  (weekly cron) │
       └────────────────┘
```

---

## Dual-Checksum Verification

### Purpose

The governance manifest must be identical in both repositories:
- `aduboseh/SCG` (core substrate)
- `aduboseh/scg-mcp` (MCP boundary)

### Enforcement

**CI Workflow:** `verify_rules_consistency.yml`

```yaml
- name: Verify cross-repo consistency
  run: |
    diff -q scg/governance/SCG_Governance_v1.0.md \
            scg_mcp_server/governance/SCG_Governance_v1.0.md
```

**Expected Checksum:**
```
9D7623E581D982D8F9BC816738EF0880E9631E6FD5789C36AF80698DF2BAA527
```

### Verification Schedule

| Trigger | Description |
|---------|-------------|
| `push` | On changes to `WARP.md` or `governance/**` |
| `pull_request` | On PRs touching governance files |
| `schedule` | Weekly (Sunday 00:00 UTC) |
| `workflow_dispatch` | Manual trigger |

---

## CODEOWNERS Protection

### Protected Paths

| Path | Owner | Reason |
|------|-------|--------|
| `/src/services/sanitizer/` | @aduboseh | Security boundary |
| `/tests/integration/` | @aduboseh | Test integrity |
| `/tests/snapshots/` | @aduboseh | Golden files |
| `/.github/workflows/` | @aduboseh | CI pipeline |
| `/src/mcp_handler.rs` | @aduboseh | MCP dispatch |

### CODEOWNERS File

```
# .github/CODEOWNERS

# Security-critical sanitizer boundary
/src/services/sanitizer/ @aduboseh

# Integration test suite
/tests/integration/ @aduboseh

# Response snapshots (golden files)
/tests/snapshots/ @aduboseh

# CI/CD workflows
/.github/workflows/ @aduboseh

# MCP handler (tool dispatch)
/src/mcp_handler.rs @aduboseh
```

### Effect

Any PR modifying protected paths requires approval from `@aduboseh` before merge.

---

## Immutable Components

### Forbidden Pattern Registry

**File:** `src/services/sanitizer/forbidden.rs`

```
╔══════════════════════════════════════════════════════════════════════════╗
║  IMMUTABLE REGISTRY — DO NOT MODIFY WITHOUT FOUNDER-LEVEL OVERRIDE       ║
║  Version: 2.0.0 | Sealed: 2025-12-03 | Authority: SCG Governor           ║
║  Any modification requires CODEOWNERS approval and audit trail entry.    ║
╚══════════════════════════════════════════════════════════════════════════╝
```

**Change Process:**
1. Document security justification
2. Add comprehensive test coverage
3. Submit PR with `security` label
4. Obtain founder approval
5. Update version and seal date

### Governance Manifest

**File:** `governance/SCG_Governance_v1.0.md`

**Change Process:**
1. Modify in both SCG and MCP repos
2. Ensure SHA-256 checksums match
3. CI verifies cross-repo consistency
4. Both repos must pass before merge

---

## CI Enforcement

### Required Checks

All PRs must pass:

| Check | Description |
|-------|-------------|
| `integration-tests` | 69 MCP integration tests |
| `boundary-audit` | Forbidden pattern scan |
| `merge-gate` | Final verification |

### Merge Gate

```yaml
merge-gate:
  needs: [integration-tests, boundary-audit]
  if: always()
  steps:
    - name: Check test results
      run: |
        if [ "${{ needs.integration-tests.result }}" != "success" ]; then
          echo "::error::Integration tests failed. Merge blocked."
          exit 1
        fi
```

---

## Branch Protection

### Main Branch Rules

- ✅ Require pull request before merging
- ✅ Require status checks to pass
- ✅ Require CODEOWNERS approval
- ✅ Dismiss stale reviews on new commits
- ❌ Allow force pushes (disabled)
- ❌ Allow deletions (disabled)

---

## Audit Trail

### Lineage Records

Every substrate operation is recorded:
- Operation type
- Parameters
- Timestamp (deterministic mode available)
- SHA-256 hash
- Episode grouping

### Export Format

```json
{
  "entries": [
    {
      "operation": "node.create",
      "params": {"belief": 0.7, "energy": 1.0},
      "timestamp": "2025-12-03T00:00:00Z",
      "hash": "abc123..."
    }
  ],
  "episode_hash": "def456...",
  "checksum": "..."
}
```

---

## Compliance Status

### Current State

| Requirement | Status |
|-------------|--------|
| Governance manifest parity | ✅ Verified |
| CODEOWNERS configured | ✅ Active |
| CI enforcement | ✅ Passing |
| Immutable registry sealed | ✅ v2.0.0 |
| Branch protection | ✅ Enabled |

### Release Milestones

| Version | Milestone |
|---------|-----------|
| `v0.1.0` | Initial MCP server |
| `v0.2.0-mcp-integrity` | MCP Hardening v2.0 - Boundary sealed |

---

## Governance Violations

### Detection

CI will fail if:
- Governance checksums don't match
- Forbidden patterns found in responses
- Protected files modified without approval

### Resolution

1. Identify violation in CI logs
2. Revert unauthorized changes
3. Document incident
4. Review access controls

---

## Contact

**Governance Issues:** governance@onlysgsolutions.com

**Security Issues:** security@onlysgsolutions.com

---

## See Also

- [SECURITY.md](./SECURITY.md) - Security architecture
- [ARCHITECTURE.md](./ARCHITECTURE.md) - System design
- [SCG_Governance_v1.0.md](../governance/SCG_Governance_v1.0.md) - Full manifest
