# Iter Demo Directive Changelog

## v2.0 (2025-11-26)
**Status:** Production deployment

**Changes from v1.0:**
- Renamed `legacy_demo.sh` to `iter_demo.sh` (audience neutrality)
- Added 7-layer determinism stack (locale hardening, JSON normalization)
- Enhanced packaging with Docker reproducibility bundle
- Added Microsoft audit compliance section
- Integrated dual-run SHA-256 verification protocol
- Added `normalize()` helper for CRLF/whitespace handling
- Sequential JSON-RPC ID requirement (1..N per run)
- Environment hardening: `LC_ALL=C`, `LANG=C`, `TZ=UTC`

**New Deliverables:**
- `RUN_CERTIFICATION.md` - Validation methodology
- `SUBSTRATE_OVERVIEW.md` - Architecture summary
- `DEMO_WALKTHROUGH.md` - 7-minute presentation script
- `SUBSTRATE_ATTESTATION.txt` - Technical attestation
- `RUNBOOK.md` - Operational guide
- `Dockerfile` - Container build
- `DOCKER_INSTRUCTIONS.md` - Docker usage
- `PACKAGING_MANIFEST.txt` - File checksums

**Certification:**
- Artifact Hash: `588153f3c00a95c3296b576a74f4d8bea8ea556fb2a0366236bd7b21b899d1df`
- Package Hash: `9FAEA83409F014066EEA2483E364C83A9AACC3F59BA884206A88D5B0BEF07158`
- Package Size: 21.75 KB
- Compliance: 5/5 security dimensions passed

## v1.0 (2025-11-26)
**Status:** Initial draft (superseded)

**Scope:**
- Core directive structure
- Demo flow specification (Steps A-I)
- Quality gates definition
- Basic expected output samples

**Limitations:**
- No determinism hardening
- No Docker packaging
- No formal attestation documents
- Personal references in filenames


