Architecture Boundary

This document defines what the public Iter repository certifies versus what is validated in private infrastructure.

Certification Scopes
Public CI Certifies (this repository)
Domain	What's Tested	Guarantee
Protocol	Wire format, JSON-RPC 2.0 compliance, MCP tool interface	Stable 1.0.0 contract
Type Shapes	Schema stability for all public types	Breaking changes require major version bump
Error Taxonomy	Error codes are exhaustive and documented	New codes are additive only
Versioning	Protocol version negotiation, N/N-1 compatibility	SDKs fail fast on incompatible versions
Telemetry	Audit event structure, redaction guarantees	Allowlist and denylist enforced
SDKs	Rust and TypeScript clients compile and pass tests	Thin clients work against any compliant server
Release Discipline	Version consistency, changelog, boundary integrity	Unsafe changes cannot ship

Build mode: public_stub (default)

Test count: 71 governance invariants plus SDK unit tests

Audience: Protocol consumers, SDK users, auditors, integrators

Private CI Certifies (licensed deployments)
Domain	What's Tested	Guarantee
Execution	Node, edge, and governor behavior	Correct substrate semantics
Integration	End-to-end MCP request flows	Real operations work
Performance	Latency, throughput, resource usage	SLA compliance
Security	Penetration testing, threat modeling	Customer-specific validation

Build mode: full_substrate (requires proprietary execution crates)

Test count: Additional integration and property-based tests

Audience: Licensed customers, deployment engineers, security auditors

Why This Separation Exists

IP Protection
Execution semantics are proprietary, protocol surface is open

Independent Auditability
Public contract can be verified without access to internals

Clean Dependency Graph
SDKs never depend on substrate, substrate implements the protocol

Appropriate Trust Boundaries
Different guarantees for different audiences

What Each Audience Should Trust
If you are	Trust this	Validated by
Building a client	Protocol types, SDK behavior	Public CI
Integrating telemetry	Audit event schema, redaction rules	Public CI
Auditing the contract	Type shapes, versioning rules, error taxonomy	Public CI
Deploying Iter	Execution correctness, performance	Private CI
Evaluating security	Boundary integrity (public) plus threat model (private)	Both
How to Verify
Public guarantees (anyone can run)
# Clone the public repo
git clone https://github.com/aduboseh/iter.git
cd iter

# Run governance tests
cargo test --features public_stub --test governance_invariants

# Build and test Rust SDK
cd sdks/rust && cargo test

# Build and test TypeScript SDK
cd sdks/typescript && npm ci && npm test

Private guarantees (licensed access required)

Contact the Iter team under licensed access for:

Access to private CI dashboards

Deployment-specific test results

Security audit reports

Summary

Iter v1.0.0 certifies the public protocol, SDK surface, telemetry contract, and release guarantees.

Execution correctness is validated separately within licensed deployments. This separation is the architecture, not missing work.
