# Iter — Buyer Summary

**Version:** 1.0.1  
**License:** MIT  
**Status:** Stable, frozen surface

---

## What Iter Is

Iter is a governed Model Context Protocol (MCP) server that exposes a deterministic, auditable tool surface over JSON-RPC 2.0 (STDIO transport).

It provides:
- A stable protocol contract (v1.0.0) with semantic versioning
- 71 governance invariant tests enforcing schema stability
- Rust and TypeScript SDKs for client integration
- Release discipline with 6-month N/N-1 support window
- Signed releases and CI-enforced gates

---

## What Iter Guarantees

| Guarantee | Enforcement |
|-----------|-------------|
| Protocol stability | Schema tests fail on breaking changes |
| Error taxonomy | Error codes are exhaustive and additive-only |
| Version compatibility | SDKs reject incompatible server versions at init |
| Audit event structure | Telemetry schema is frozen with redaction guarantees |
| Release integrity | All tags are signed; CI gates block non-compliant releases |

---

## What Iter Does NOT Include

| Exclusion | Reason |
|-----------|--------|
| Execution semantics | Proprietary; validated in private CI only |
| Performance guarantees | SLA-dependent; per-deployment validation |
| End-to-end integration tests | Require `full_substrate` (not in public repo) |
| Substrate internals | IP-protected; available under license |

See [ARCHITECTURE_BOUNDARY.md](ARCHITECTURE_BOUNDARY.md) for certification scope details.

---

## Governance Enforcement

**Tests:** 71 invariant tests run on every PR and release.

**CI Workflows:**
- `mcp_integration.yml` — Protocol and boundary validation
- `sdk_ci.yml` — Isolated SDK builds
- `release_gate.yml` — Version, changelog, and boundary checks
- `sdk_publish_check.yml` — Packaging dry-run

**Branch Protection:**
- `main` requires passing CI
- CODEOWNERS review for security-critical paths
- Signed commits and tags

**Artifacts:**
- `audits/stability_v1.0.0/` — SHA-256 manifest baseline
- `CREDIBILITY_CLOSURE.md` — Audit trail for v1.0.1

---

## Surface Freeze

Protocol and SDK surface are stable for 12 months (through December 2025) barring security issues.

Any breaking change requires:
1. Major version bump
2. Migration documentation
3. 3-month deprecation notice

---

## Evaluation Checklist

```bash
# Clone and verify
git clone https://github.com/aduboseh/iter.git
cd iter
git checkout v1.0.1

# Run governance tests
cargo test --test governance_invariants

# Verify SDK builds
cd sdks/rust && cargo test
cd ../typescript && npm ci && npm test
```

---

## Contact

For technical, licensing, or security inquiries:

Armonti Du-Bose-Hill <armontidubosehill@gmail.com>

---

*Only SG Solutions © 2025*
