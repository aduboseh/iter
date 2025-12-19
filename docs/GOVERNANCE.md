# Iter Governance

Governance for the Iter public protocol surface.

## Governance Tests

71 invariant tests enforce:

- **Schema stability**: Protocol type shapes cannot change without version bump
- **Error taxonomy**: Error codes are exhaustive and stable
- **Versioning**: Protocol version rules are enforced
- **Telemetry**: Audit event structure and redaction guarantees
- **Release discipline**: Version consistency, changelog, boundary integrity

Run locally:
```bash
cargo test --test governance_invariants
```

## Change Control

- Branch protection on `main`
- Required status checks (CI must pass)
- CODEOWNERS review for security-critical paths
- Signed commits and tags

## Release Policy

See [RELEASE.md](../RELEASE.md) for:
- Release channels (stable, rc)
- Support window (N, N-1 for 6 months)
- EOL policy
- Hotfix procedures

## Certification Scope

See [ARCHITECTURE_BOUNDARY.md](../ARCHITECTURE_BOUNDARY.md) for what public CI certifies vs. private CI.

## Contact

For governance or security inquiries:

Armonti Du-Bose-Hill <armontidubosehill@gmail.com>

