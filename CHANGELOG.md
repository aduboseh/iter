# Changelog

All notable changes to Iter will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.2] - 2024-12-19

### Fixed
- Runtime version consistency: use `CARGO_PKG_VERSION` everywhere
- SDK license alignment: all metadata now Apache-2.0
- Unused import and dead code warnings

### Added
- Code quality CI workflow: fmt, clippy, audit enforcement
- SBOM generation for release tags
- Architecture boundary clarification for stub implementation

## [1.0.1] - 2024-12-15

### Changed
- Post-A1 credibility lock; no functional delta
- Added `BUYER_README.md` for corporate evaluation
- Added 12-month surface freeze statement
- Archived credibility artifacts in `governance/credibility_log.md`

## [1.0.0] - 2024-12-15

### Added
- **Phase 5: Release Discipline**
  - `RELEASE.md`: Channels (stable/rc), support window (N, N-1 × 6 months), EOL policy
  - `CHANGELOG.md`: Keep a Changelog format
  - `release_gate.yml`: 6-gate CI workflow (governance, SDK×2, version, changelog, boundary)
  - 10 new governance tests (total: 71)

- **Phase 4: Client SDKs**
  - Rust SDK (`iter-sdk` crate) with STDIO transport
  - TypeScript SDK (`@iter/sdk` package) with async API
  - Version pinning (supports protocol 1.0.0 - 1.99.99)
  - Trace context pass-through hooks
  - SDK CI workflow with isolated builds

- **Phase 3: Telemetry & Audit Hooks**
  - `TraceContext` for distributed tracing correlation
  - `AuditEvent` with phase and outcome tracking
  - `AUDIT_ALLOWLIST` / `AUDIT_DENYLIST` for field-level redaction
  - JSON Lines serialization for audit streams
  - 22 telemetry invariant tests

- **Phase 2: Protocol Versioning**
  - `PROTOCOL_VERSION` constant (1.0.0)
  - `ProtocolVersion` struct with semantic versioning
  - `CompatibilityStatus` enum for version negotiation
  - Compatibility rules: major breaks, minor compatible, patch safe
  - Golden wire format snapshots
  - 15 versioning tests

- **Phase 1: Governance Enforcement**
  - Schema stability tests (15 tests)
  - Error taxonomy tests (9 tests)
  - Governance invariant test harness

- **Phase 0: Identity & IP Boundary**
  - Dual-build system (`public_stub` / `full_substrate`)
  - SCG reference scrubbing
  - Clean IP boundary between public and proprietary code

### Protocol
- Initial protocol version: 1.0.0
- Wire format: JSON-RPC 2.0 over STDIO
- MCP-compliant tool interface

### Governance
- 71 governance invariant tests
- All tests compile in `public_stub` mode

## [0.3.0] - 2024-12-15

### Added
- **Phase 4: Client SDKs**
  - Rust SDK (`iter-sdk` crate) with STDIO transport
  - TypeScript SDK (`@iter/sdk` package) with async API
  - Version pinning (supports protocol 1.0.0 - 1.99.99)
  - Trace context pass-through hooks
  - SDK CI workflow with isolated builds

- **Phase 3: Telemetry & Audit Hooks**
  - `TraceContext` for distributed tracing correlation
  - `AuditEvent` with phase and outcome tracking
  - `AUDIT_ALLOWLIST` / `AUDIT_DENYLIST` for field-level redaction
  - JSON Lines serialization for audit streams
  - 22 telemetry invariant tests

- **Phase 2: Protocol Versioning**
  - `PROTOCOL_VERSION` constant (1.0.0)
  - `ProtocolVersion` struct with semantic versioning
  - `CompatibilityStatus` enum for version negotiation
  - Compatibility rules: major breaks, minor compatible, patch safe
  - Golden wire format snapshots
  - 15 versioning tests

- **Phase 1: Governance Enforcement**
  - Schema stability tests (15 tests)
  - Error taxonomy tests (9 tests)
  - Governance invariant test harness

- **Phase 0: Identity & IP Boundary**
  - Dual-build system (`public_stub` / `full_substrate`)
  - SCG reference scrubbing
  - Clean IP boundary between public and proprietary code

### Protocol
- Initial protocol version: 1.0.0
- Wire format: JSON-RPC 2.0 over STDIO
- MCP-compliant tool interface

### Governance
- 61 governance invariant tests
- All tests compile in `public_stub` mode

## [0.2.0] - 2024-12-01

### Added
- MCP server implementation
- Tool discovery and invocation
- Node and edge operations
- Governor status reporting

### Changed
- Migrated to async runtime

## [0.1.0] - 2024-11-15

### Added
- Initial project structure
- Basic MCP protocol types
- Stub substrate interface

[Unreleased]: https://github.com/aduboseh/iter/compare/v1.0.2...HEAD
[1.0.2]: https://github.com/aduboseh/iter/compare/v1.0.1...v1.0.2
[1.0.1]: https://github.com/aduboseh/iter/compare/v1.0.0...v1.0.1
[1.0.0]: https://github.com/aduboseh/iter/compare/v0.3.0...v1.0.0
[0.3.0]: https://github.com/aduboseh/iter/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/aduboseh/iter/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/aduboseh/iter/releases/tag/v0.1.0
