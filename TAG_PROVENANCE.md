# Tag Provenance

This document explains the provenance of all tags in the Iter repository.

## Current Release Tags

These tags represent official Iter releases:

| Tag | Type | Description |
|-----|------|-------------|
| `v1.0.0` | Stable | First stable release (2024-12-15) |
| `v1.0.0-rc.1` | RC | Release candidate for v1.0.0 |
| `v0.3.0` | Legacy | Pre-1.0 development milestone |
| `v0.2.0-*` | Legacy | Pre-1.0 development milestones |

## Phase Tags

These tags mark completion of AGED-SCG-001 restructuring phases:

| Tag | Phase | Description |
|-----|-------|-------------|
| `iter-phase0-complete` | 0 | Identity & IP Boundary |
| `iter-phase1-governance-complete` | 1 | Governance Tests |
| `iter-phase2-versioning-complete` | 2 | Protocol Versioning |
| `iter-phase3-telemetry-complete` | 3 | Telemetry & Audit |
| `iter-phase4-sdks-complete` | 4 | Client SDKs |
| `iter-phase5-release-complete` | 5 | Release Discipline |

## Legacy Experimental Tags (Pre-Iter)

The following tags predate the Iter public release and represent internal pilot work during the SCG research phase. They are retained for historical completeness but are **not part of the Iter release lineage**:

| Tag | Status | Notes |
|-----|--------|-------|
| `SG-SCG-PILOT-*` | Archived | Internal pilot experiments |
| `LOAD-BALANCE-01_*` | Archived | Load balancing research |
| `public-hardened-*` | Archived | Pre-release hardening passes |
| `public-sanitize-*` | Archived | Pre-release sanitization work |
| `v1.0.0-substrate` | Archived | Internal substrate milestone |

These tags:
- Were created before Iter's public identity was established
- Use internal project naming conventions
- Are not supported and should not be used as base references
- Remain in history for audit trail purposes only

## Tag Naming Convention (v1.0.0+)

All future tags follow this convention:

- `vX.Y.Z` — Stable releases
- `vX.Y.Z-rc.N` — Release candidates
- `iter-phaseN-*` — Phase completion markers (internal use)

No new tags will use internal project prefixes (SG-, SCG-, etc.).
