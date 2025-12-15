# Iter Release Discipline

This document defines how Iter ships: channels, versioning, compatibility, EOL policy, and release gates.

## Release Channels

| Channel | Purpose | Cadence | Stability |
|---------|---------|---------|-----------|
| `stable` | Production deployments | On demand (tagged) | Full compatibility guarantees |
| `rc` | Pre-release validation | Before each stable | Feature-complete, may have bugs |

**No nightly channel.** Iter prioritizes stability over bleeding-edge features.

## Versioning

Iter follows [Semantic Versioning 2.0.0](https://semver.org/):

- **MAJOR**: Breaking protocol changes (new major version = new compatibility window)
- **MINOR**: New features, backward-compatible
- **PATCH**: Bug fixes, security patches

### Protocol Version

The protocol version (`src/types/version.rs`) is independent of the crate version:

- Protocol version changes only when wire format changes
- SDKs declare supported protocol versions (N, N-1)
- Protocol major bump = SDK major bump required

## Compatibility Policy

### Support Window

| Version Type | Support Duration |
|--------------|------------------|
| Current stable (N) | Full support |
| Previous stable (N-1) | Security fixes only, 6 months |
| Older (N-2 and below) | Unsupported |

### Deprecation Process

1. **Announce**: Deprecation notice in changelog and docs
2. **Warn**: Runtime warnings for deprecated features (1 minor version)
3. **Remove**: Feature removed in next major version

### SDK Compatibility

SDKs support protocol versions N and N-1:
- `MIN_SERVER_VERSION`: Oldest supported server
- `MAX_SERVER_VERSION`: Newest supported server (same major)

## End-of-Life (EOL) Policy

### Timeline

- **EOL announcement**: 3 months before support ends
- **Security-only period**: Final month before EOL
- **EOL date**: No further updates

### What EOL Means

- No bug fixes or security patches
- No guaranteed compatibility with newer versions
- Upgrade path documented in migration guide

## Release Process

### Pre-Release Checklist

All gates must pass before any release:

- [ ] Governance tests pass (`cargo test --test governance_invariants`)
- [ ] SDK CI passes (Rust + TypeScript)
- [ ] Protocol version unchanged OR migration documented
- [ ] Changelog generated and reviewed
- [ ] No known security vulnerabilities
- [ ] Sanitizer checks pass (private, full_substrate only)

### Release Steps

1. **Branch**: Create `release/vX.Y.Z` from `main`
2. **Version bump**: Update `Cargo.toml`, SDK versions
3. **Changelog**: Generate from merged PRs
4. **RC tag**: `vX.Y.Z-rc.1`
5. **Validation**: Deploy RC to staging, run integration tests
6. **Stable tag**: `vX.Y.Z`
7. **Publish**: Crates.io, npm (when ready)
8. **Announce**: GitHub release with changelog

### Hotfix Process

For critical security fixes:

1. Branch from latest stable tag
2. Apply minimal fix
3. Skip RC (if urgent)
4. Tag as patch release
5. Backport to N-1 if in support window

## Artifact Integrity

### Signing

All release artifacts are signed:

- Git tags: GPG/SSH signed
- Checksums: SHA-256 for all binaries
- SBOM: Software Bill of Materials for dependencies

### Verification

```bash
# Verify git tag signature
git verify-tag vX.Y.Z

# Verify checksum
sha256sum -c iter-vX.Y.Z.sha256
```

## Changelog Format

Changelogs follow [Keep a Changelog](https://keepachangelog.com/):

```markdown
## [X.Y.Z] - YYYY-MM-DD

### Added
- New features

### Changed
- Changes to existing features

### Deprecated
- Features to be removed

### Removed
- Removed features

### Fixed
- Bug fixes

### Security
- Security patches

### Governance
- Schema stability, error taxonomy changes

### Protocol
- Wire format, version compatibility changes
```

## CI Enforcement

Release gates are enforced by CI:

- `release_gate.yml`: Blocks release without passing checks
- `changelog_check.yml`: Ensures changelog entry exists
- `version_check.yml`: Validates version bump consistency

## Emergency Procedures

### Security Incident

1. Assess severity (Critical/High/Medium/Low)
2. Critical/High: Hotfix within 24-48 hours
3. Coordinate disclosure (if external report)
4. Post-mortem and preventive measures

### Rollback

If a release causes production issues:

1. Revert to previous stable tag
2. Document issue in post-mortem
3. Fix forward (no re-release of same version number)
