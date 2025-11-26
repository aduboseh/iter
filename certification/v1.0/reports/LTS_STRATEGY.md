# SCG LTS Versioning Strategy

**Version**: 1.0.0  
**Status**: ACTIVE  
**Effective Date**: 2025-01-15  
**Substrate Version**: v1.0.0-substrate (FROZEN)  
**Governance Authority**: SCG Substrate Team

---

## Executive Summary

This document defines the **Long-Term Support (LTS) versioning strategy** for the SCG-MCP system, establishing immutable substrate stability while enabling isolated connectome evolution. The strategy guarantees **24-month substrate support** with strict modification policies and **12-month connectome support** with independent versioning.

**Core Principle**: The substrate is **mathematically closed and frozen** — all future cognitive advancement occurs in the isolated connectome layer.

---

## 1. Versioning Model

### 1.1 Substrate Versioning (v1.0.x-substrate)

**Format**: `MAJOR.MINOR.PATCH-substrate`

**Example**: v1.0.0-substrate → v1.0.1-substrate → v1.0.2-substrate

**Semantics**:
- **MAJOR** (1): Breaking changes to substrate API, invariants, or architecture (requires governance approval)
- **MINOR** (0): New features or enhancements (PROHIBITED for substrate — must remain frozen)
- **PATCH** (x): Bug fixes, security patches, clarification implementations (allowed with SUBSTRATE_OVERRIDE)

**Current Line**: v1.0.x-substrate (LTS until 2027-01-15)

---

### 1.2 Connectome Versioning (v2.x)

**Format**: `MAJOR.MINOR.PATCH`

**Example**: v2.0.0 → v2.1.0 → v2.1.1

**Semantics**:
- **MAJOR** (2): Breaking changes to connectome API or architecture
- **MINOR** (x): New cognitive modules, tracts, or regions
- **PATCH** (x): Bug fixes, performance improvements

**Current Line**: v2.0.x (begins after SCG-PILOT-01 certification)

---

## 2. Substrate LTS Policy (v1.0.x-substrate)

### 2.1 Support Duration

**LTS Period**: 24 months from v1.0.0-substrate release (2025-01-15 → 2027-01-15)

**During LTS**:
-  Security patches expedited (v1.0.1, v1.0.2, etc.)
-  Critical bug fixes (energy drift, lineage corruption, etc.)
-  Clarification implementations (if new mathematical proofs emerge)
-  New features (deferred to v2.0.0-substrate or connectome v2.x)
-  Performance enhancements (unless security-critical)
-  API expansions (substrate API is frozen)

---

### 2.2 Modification Policy

**All substrate modifications require**:
1. **Dual Approval**: Technical Lead + Security Lead sign-off
2. **SUBSTRATE_OVERRIDE**: Commit message MUST include this keyword
3. **Governance Documentation**: Issue filed with justification
4. **Test Coverage**: 100% coverage for modified code
5. **Certification Update**: If invariants affected, re-certification required

**Example Commit Message**:
```
SUBSTRATE_OVERRIDE: Fix critical SHA256 hash collision in lineage

Governance Issue: #42
Approval: Technical Lead (Alice), Security Lead (Bob)
Impact: Lineage integrity (ε calculation)
Test Coverage: 100% (3 new tests added)
Certification: Re-certification NOT required (no invariant change)
```

---

### 2.3 Prohibited Modifications

The following substrate components are **ABSOLUTELY FROZEN** and cannot be modified even with SUBSTRATE_OVERRIDE:

1. **Invariant Thresholds**:
   - Energy conservation: ΔE ≤ 1×10⁻¹⁰
   - Coherence: C(t) ≥ 0.97
   - Lineage integrity: ε ≤ 1×10⁻¹⁰
   - Shard rotation: N = 250 operations

2. **Core Substrate Files** (modification triggers v2.0.0):
   - `src/scg_core.rs` (unless critical bug fix)
   - `src/types.rs` (unless critical bug fix)
   - `src/mcp_handler.rs` (tool contracts frozen)

3. **Clarification Implementations** (now canonical):
   - `src/fault/governor_correction.rs`
   - `src/lineage/shard.rs`
   - `src/lineage/replay_episode.rs`

**Rationale**: These components define substrate identity. Modifying them creates a new substrate (v2.0.0).

---

### 2.4 Security Patch Process

**Timeline**:
- P0 (Critical): 24-hour patch deployment
- P1 (High): 7-day patch deployment
- P2 (Medium): 30-day patch deployment
- P3 (Low): Next scheduled release

**Process**:
1. Security issue filed (private repository)
2. Technical + Security Lead review
3. Patch developed with test coverage
4. CI/CD validation (all 46+ tests must pass)
5. Expedited merge with SUBSTRATE_OVERRIDE
6. Tag new version (e.g., v1.0.1-substrate)
7. Notify all substrate deployments

---

## 3. Connectome Release Cycle (v2.x)

### 3.1 Support Duration

**Support Period**: 12 months per MINOR version

**Example**:
- v2.0.0 released: 2025-02-01
- v2.1.0 released: 2025-05-01
- v2.0.x support ends: 2026-02-01 (12 months after v2.0.0)
- v2.1.x support ends: 2026-05-01 (12 months after v2.1.0)

---

### 3.2 Release Cadence

**Minor Releases**: Every 3 months (v2.0.0 → v2.1.0 → v2.2.0 → v2.3.0)

**Patch Releases**: As needed (bug fixes, performance improvements)

**Rationale**: Connectome evolves rapidly as new cognitive modules are added; 12-month support balances stability and innovation.

---

### 3.3 Substrate Compatibility

**Guarantee**: All connectome v2.x versions are compatible with substrate v1.0.x

**Mechanism**: Connectome interacts with substrate ONLY via MCP protocol (zero direct coupling)

**Validation**: CI check (`connectome_audit`) verifies zero imports from substrate internals

**Example Compatibility Matrix**:

| Connectome Version | Substrate Version | Status |
|--------------------|-------------------|--------|
| v2.0.0 | v1.0.0-substrate |  Supported |
| v2.0.0 | v1.0.1-substrate |  Supported |
| v2.1.0 | v1.0.0-substrate |  Supported |
| v2.1.0 | v1.0.2-substrate |  Supported |
| v3.0.0 | v1.0.x-substrate |  Not compatible (requires v2.0.0-substrate) |

---

## 4. Version Branching Strategy

### 4.1 Git Branching Model

```
main (substrate v1.0.x-substrate)
  │
  ├─── v1.0.0-substrate (tag, FROZEN)
  │
  ├─── v1.0.1-substrate (tag, security patch)
  │
  └─── connectome/v2.x (branch)
        │
        ├─── v2.0.0 (tag, initial connectome)
        │
        ├─── v2.1.0 (tag, new modules)
        │
        └─── v2.2.0 (tag, enhancements)
```

**Rules**:
- `main` branch contains substrate v1.0.x-substrate (frozen)
- All connectome work occurs in `connectome/v2.x` branch
- Connectome NEVER merges into main (substrate remains pristine)
- Substrate patches (v1.0.1, v1.0.2) tagged on main with SUBSTRATE_OVERRIDE

---

### 4.2 Dependency Management

**Substrate Dependencies** (Cargo.toml):
- Locked during LTS (no version upgrades unless security-critical)
- `tokio`, `serde`, `sha2`, `chrono` pinned to exact versions
- Upgrades require SUBSTRATE_OVERRIDE + regression testing

**Connectome Dependencies**:
- Independent from substrate dependencies
- Can upgrade freely (does not affect substrate stability)
- New dependencies allowed (does not couple to substrate)

---

## 5. End-of-Life (EOL) Policy

### 5.1 Substrate v1.0.x-substrate EOL

**EOL Date**: 2027-01-15 (24 months after release)

**After EOL**:
- No further patches (security or otherwise)
- Deployments must migrate to v2.0.0-substrate
- Documentation archived (marked as legacy)
- Repository branch remains accessible (read-only)

**Migration Path**: v1.0.x-substrate → v2.0.0-substrate (major version bump)

---

### 5.2 Connectome v2.x EOL

**EOL Date**: 12 months after each MINOR version release

**After EOL**:
- No further patches
- Deployments must migrate to latest v2.x version
- Substrate compatibility guaranteed (all v2.x work with v1.0.x-substrate)

**Migration Path**: v2.0.0 → v2.1.0 → v2.2.0 → v2.3.0 (incremental upgrades)

---

## 6. Breaking Change Protocol

### 6.1 Substrate Breaking Changes (v1.0.x → v2.0.0)

**Triggers for v2.0.0-substrate**:
- Invariant threshold changes (e.g., ΔE relaxed or tightened)
- Core architecture redesign (e.g., horizontal sharding)
- MCP protocol breaking changes
- Graph model changes (e.g., hypergraphs instead of directed graphs)

**Process**:
1. Governance proposal filed (3-month review period)
2. Mathematical proof required (for invariant changes)
3. Prototype developed in separate branch
4. Complete re-certification (7-day pilot minimum)
5. Migration guide published
6. v2.0.0-substrate released with 6-month transition period

---

### 6.2 Connectome Breaking Changes (v2.x → v3.0.0)

**Triggers for v3.0.0**:
- Incompatibility with substrate v1.0.x (requires v2.0.0-substrate)
- Region architecture redesign
- Tract protocol changes
- Timestep model changes

**Process**:
1. Technical proposal filed (1-month review)
2. Prototype developed
3. Integration testing against substrate
4. Migration guide published
5. v3.0.0 released with 3-month transition period

---

## 7. Versioning Governance

### 7.1 Substrate Version Authority

**Decision Authority**: Substrate Governance Board (3-member)
- Technical Lead (architecture and invariants)
- Security Lead (audit and compliance)
- Operations Lead (production stability)

**Voting**: Unanimous approval required for substrate changes

---

### 7.2 Connectome Version Authority

**Decision Authority**: Connectome Technical Lead

**Approval**: Single-party approval (faster iteration)

**Rationale**: Connectome isolation ensures changes cannot corrupt substrate

---

## 8. Certification Linkage

### 8.1 Substrate Certification

**Certification Required**:
- v1.0.0-substrate (SCG-PILOT-01 field validation)
- v1.0.x-substrate (if invariants affected by patch)
- v2.0.0-substrate (full re-certification)

**Certification Process**:
- 7-day pilot minimum
- All 7 invariants validated
- Complete telemetry and lineage audit trail
- 4-way sign-off (Technical, Ops, Security, Product)

---

### 8.2 Connectome Certification

**Certification Required**:
- v2.0.0 (initial connectome, validated against certified substrate)
- v3.0.0 (major version, re-validation)

**Certification Process**:
- Integration testing against substrate v1.0.x
- Cognitive task validation (attention, memory, reasoning)
- Zero substrate coupling verified
- Technical Lead sign-off

---

## 9. Communication Strategy

### 9.1 Version Announcements

**Substrate Releases**:
- Public announcement via GitHub releases
- Email notification to all substrate deployments
- Documentation update (CHANGELOG.md)
- Security advisory (if applicable)

**Connectome Releases**:
- GitHub releases
- Blog post with new cognitive capabilities
- Migration guide (if breaking changes)

---

### 9.2 Deprecation Warnings

**Timeline**:
- 6 months before substrate EOL
- 3 months before connectome EOL
- Monthly reminders during final 3 months

---

## 10. Compliance and Audit

### 10.1 Version Tracking

**All deployments must**:
- Report substrate version to telemetry
- Report connectome version to telemetry
- Log version at startup

**Example Log**:
```
SCG-STARTUP] substrate_version=v1.0.0-substrate connectome_version=v2.1.0 deployment_id=SCG-PILOT-01
```

---

### 10.2 Audit Trail

**LTS Audit Requirements**:
- All substrate patches documented in CHANGELOG.md
- SUBSTRATE_OVERRIDE commits logged with governance issue reference
- Version compatibility matrix maintained in this document
- Certification artifacts archived per version

---

## 11. Version Compatibility Matrix (Live)

### 11.1 Current Supported Versions

| Substrate Version | Release Date | EOL Date | Status |
|-------------------|--------------|----------|--------|
| v1.0.0-substrate | 2025-01-15 | 2027-01-15 |  LTS Active |

| Connectome Version | Release Date | EOL Date | Substrate Compatibility | Status |
|--------------------|--------------|----------|-------------------------|--------|
| v2.0.0-alpha | TBD | TBD | v1.0.x-substrate |  Development |
| v2.0.0 | TBD | TBD | v1.0.x-substrate | ⏳ Planned (post-pilot) |

---

### 11.2 Future Versions (Planned)

| Version | Planned Release | Key Features | Breaking Changes |
|---------|----------------|--------------|------------------|
| v1.0.1-substrate | As needed | Security patches | None |
| v2.0.0 (connectome) | 2025-02-01 | Full cognitive modules | None (v1.0.x compatible) |
| v2.1.0 (connectome) | 2025-05-01 | Enhanced tracts | None (v1.0.x compatible) |
| v2.0.0-substrate | 2027+ | Horizontal sharding | Breaking (requires migration) |

---

## 12. Emergency Override Protocol

### 12.1 Critical Security Vulnerability

**If substrate has P0 security vulnerability**:
1. Immediate SUBSTRATE_OVERRIDE authority granted to Security Lead
2. 24-hour patch development and testing
3. Expedited merge without dual approval (post-hoc review required)
4. Emergency notification to all deployments
5. Rollback plan prepared (use v1.0.x-1 if patch fails)

---

### 12.2 Production Failure

**If substrate causes production failure**:
1. Immediate rollback to last known good version
2. Root cause analysis (24-hour turnaround)
3. Patch developed with SUBSTRATE_OVERRIDE
4. Re-deployment with enhanced monitoring

---

## Document Control

**Version**: 1.0.0  
**Status**: ACTIVE  
**Last Updated**: 2025-01-15  
**Next Review**: 2025-07-15 (6-month review cycle)  
**Owner**: SCG Substrate Governance Board  
**Approval**: Technical Lead, Security Lead, Operations Lead (unanimous)

---

## Appendix A: Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2025-01-15 | Initial LTS strategy for v1.0.0-substrate |

---

## Appendix B: Contact Information

**Substrate Issues**: https://github.com/aduboseh/scg-mcp/issues  
**Security Vulnerabilities**: security@scg-substrate.org (private)  
**Governance Proposals**: governance@scg-substrate.org

---

**END OF LTS STRATEGY**

*This strategy guarantees substrate stability for 24 months while enabling isolated connectome evolution. All modifications follow strict governance protocols.*
