# SCG Governance Update Protocol
**Version:** 1.0  
**Status:** ACTIVE  
**Effective:** 2025-12-03  

---

## Overview

This document defines the protocol for updating SCG Governance manifests. All governance changes must follow this versioning scheme and change process to ensure deterministic evolution of the cognitive substrate.

---

## Versioning Scheme

### v1.x — Constraint Layer Changes
**Scope:** Rule modifications, threshold adjustments, new constraints within existing categories.

**Examples:**
- Adding a new rule (e.g., G4, T3, C4)
- Modifying ESV thresholds
- Adjusting drift epsilon bounds
- Adding compliance frameworks
- Clarifying existing rule behavior

**Process:**
1. Propose change via PR with `governance/` changes
2. Document rationale in PR description
3. Update `SCG_Governance_v1.x.md` with new version number
4. Compute new SHA256 checksum
5. Update all checksum references:
   - `scg-governance/src/lib.rs`
   - `scg_mcp_server/src/governance.rs`
   - `.github/workflows/verify_rules_consistency.yml`
   - `lineage/SCG_GOVERNANCE/*/manifest.json`
6. CI must pass checksum validation
7. Create new lineage freeze event

**Breaking:** NO — existing code remains compliant.

---

### v2.x — Structural Governance Changes
**Scope:** New rule categories, new invariants, DAG structure changes, ethics module extensions.

**Examples:**
- Adding new rule category (Table 7+)
- Introducing new global invariant (G4+)
- Modifying DAG acyclicity requirements
- Extending ESV to new domains
- Adding new compliance validation methods

**Process:**
1. Create RFC (Request for Comments) document
2. Review period: minimum 7 days
3. Update governance module interfaces if needed
4. Bump major version in `scg-governance` crate
5. Migration guide required for existing deployments
6. Full audit of both repos required
7. Create tagged release with governance version

**Breaking:** POSSIBLY — may require code changes for compliance.

---

### v3.x — Cognitive Autonomy Features
**Scope:** Self-updating governance, quorum-based rule evolution, autonomous constraint adaptation.

**Examples:**
- Governance self-modification protocols
- Multi-agent consensus for rule changes
- Autonomous drift correction
- Dynamic ESV threshold adaptation
- Self-healing governance mechanisms

**Process:**
1. Extended RFC with safety analysis
2. Ethics review board approval (if applicable)
3. Staged rollout with monitoring
4. Automatic rollback triggers defined
5. Lineage tracking of all autonomous changes
6. Quarterly human review of autonomous adaptations

**Breaking:** SIGNIFICANT — fundamentally changes governance model.

---

## Change Categories

### Non-Breaking Changes
- Documentation clarifications
- Example additions
- Typo corrections
- Threshold relaxations (less strict)

### Breaking Changes
- New required constraints
- Threshold tightening (more strict)
- Removal of override capabilities
- New mandatory validation steps

---

## Override Protocol

Temporary governance relaxation is permitted under strict conditions:

```rust
// SCG_OVERRIDE: G2 (Deterministic Replay)
// Reason: External API integration with non-deterministic responses
// Expires: 2025-03-01 OR when deterministic mock is available
// Tracked: lineage/overrides/2025-12-03_g2_api_integration.json
// Approved: <approver_name>
```

### Override Requirements
1. **Time-bounded:** Must have expiration date or condition
2. **Logged:** Must create lineage entry
3. **Justified:** Must document why override is necessary
4. **Scoped:** Must specify exact rules being overridden
5. **Audited:** Must be reviewed in quarterly audit

### Override Tracking
All active overrides are tracked in:
```
lineage/overrides/YYYY-MM-DD_<rule_id>_<context>.json
```

---

## Checksum Management

### Computing Checksum
```bash
# PowerShell
(Get-FileHash "governance/SCG_Governance_v1.0.md" -Algorithm SHA256).Hash

# Bash
sha256sum governance/SCG_Governance_v1.0.md | cut -d' ' -f1 | tr 'a-f' 'A-F'
```

### Checksum Locations
Update ALL of the following when governance changes:

1. **SCG Repository:**
   - `governance/SCG_Governance_v1.0.md` (header)
   - `crates/scg-governance/src/lib.rs`
   - `crates/scg-governance/build.rs`
   - `.github/workflows/verify_rules_consistency.yml`
   - `.scg-context`
   - `lineage/SCG_GOVERNANCE/*/manifest.json`

2. **scg_mcp_server Repository:**
   - `governance/SCG_Governance_v1.0.md` (header)
   - `src/governance.rs`
   - `.github/workflows/verify_rules_consistency.yml`
   - `.mcp-context`
   - `lineage/SCG_GOVERNANCE/*/manifest.json`

---

## Lineage Freeze Events

Every governance version change creates a lineage freeze event:

```
lineage/SCG_GOVERNANCE/YYYY-MM-DD_frozen/
├── manifest.json      # Version metadata, checksums, commits
├── governance.md      # Snapshot of governance at freeze
└── changelog.md       # Changes from previous version
```

---

## CI/CD Integration

### Pre-merge Checks
- Checksum consistency across repos
- All governance files present
- No override violations
- Audit script passes

### Post-merge Actions
- Tag release with governance version
- Update lineage ledger
- Notify stakeholders of governance change

---

## Audit Schedule

| Audit Type | Frequency | Script |
|------------|-----------|--------|
| Drift check | Every commit | CI workflow |
| Header compliance | Weekly | `scg_audit.sh` |
| Full governance audit | Quarterly | Manual + script |
| Override review | Monthly | Manual |

---

## Emergency Procedures

### Governance Violation Detected
1. System enters quarantine mode
2. Alert sent to maintainers
3. Rollback to last known-good governance state
4. Root cause analysis required
5. Lineage preserved for forensics

### Checksum Mismatch
1. CI fails immediately
2. Cannot merge until resolved
3. Check for unauthorized modifications
4. Verify correct checksum in all locations

---

## Version History

| Version | Date | Type | Summary |
|---------|------|------|---------|
| 1.0 | 2025-12-03 | Initial | Frozen governance manifest with 14 rules |

---

*This protocol is itself governed by SCG Governance v1.0 and follows the same versioning scheme.*
