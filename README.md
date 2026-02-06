# Iter Server

**Deterministic governance control plane for auditable decision-making, replayable reasoning paths, and policy-enforced execution.**

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![Protocol](https://img.shields.io/badge/protocol-MCP%20%7C%20JSON--RPC%202.0-green.svg)](https://modelcontextprotocol.io/)

**Determinism** · **Governance** · **MCP**

---

## What is Iter?

Iter Server is a hardened Model Context Protocol (MCP) server (JSON-RPC 2.0) for deterministic governance evaluation.

Iter operates in two modes:

- **Demo mode** (default): Threshold-based governance using stub graph state. Non-authoritative — `authoritative_pdp=false`, `replay_sufficient=false`, no `DecisionPacket` at the MCP edge. Suitable for protocol validation and integration testing.
- **Governed mode**: PolicyEvaluator-based governance with typed contract envelopes. Authoritative PDP — `authoritative_pdp=true`, `replay_sufficient=true`, emits `DecisionPacket` with RFC 8785 JCS checksums.

**MCP is the transport. Governance mode determines what claims are valid.**

---

## Surface Freeze

The protocol and SDK surface are **stable for 12 months** (through January 2027), barring security issues.

See [RELEASE.md](RELEASE.md) for the compatibility policy.

---
## External Spec (v1)

Machine-readable JSON Schemas live under `schemas/v1`. They freeze the payloads for:

- DecisionPacket responses
- DecisionPreview responses
- `decision.check` requests
- `audit.search` filters/results

See [docs/iter-external-spec-v1.md](docs/iter-external-spec-v1.md) for integration guidance.

Regenerate schemas only when intentionally changing the contract:

```bash
cargo run --features schema-gen --example generate_schemas
```

Any structural change that does not come with a regenerated schema will fail the schema integrity tests.

---

## Designed For

- **Deterministic governance evaluation**  
  Policy decisions as pure functions of state and constraints

- **Replayable decision paths**  
  Reconstruct outcomes without re-learning or re-inference

- **Policy-enforced reasoning and learning gates**  
  Explicit control over when models may learn or execute

- **Audit-ready causality**  
  Cryptographic verification of decision lineage

---

## Iter is NOT

Iter is **not**:
- A general-purpose agent runtime
- An orchestration framework
- A learning or training system
- A low-latency execution engine

**Iter does not compute reasoning signals or perform learning.**

Iter governs and proves decisions produced by upstream systems.

---

## Governance Artifacts

### GovernanceOutcome

All MCP decision endpoints (`decision.check`, `decision.preview`, `audit.search`) return a `GovernanceOutcome` containing:
- `verdict` (ALLOW / BLOCK / REVIEW)
- `mode` (demo / governed)
- `authoritative_pdp` (true only in governed mode)
- `replay_sufficient` (true only for governed evaluate)
- `reason_codes` (namespaced: `demo.thresholds.*` or `policy.*`)
- `packet` (DecisionPacket, governed evaluate only)

### DecisionPacket (governed mode only)

In governed mode, `decision.check` emits a `DecisionPacket` — a replay-sufficient, immutable record containing everything required to reconstruct the governance outcome.

Each packet includes:
- System state snapshot (energy, reasoning, learning, policy envelopes)
- Capsule identity and version hashes
- Policy decisions and explicit reason codes
- Learning permissions and economic constraints
- RFC 8785 JCS canonical JSON with SHA-256 checksum

**Demo mode does not emit DecisionPackets.** Demo verdicts are non-authoritative threshold checks.

#### Determinism Guarantee

DecisionPackets are **deterministic by construction**:
- Identical inputs and configuration produce **byte-identical packets**
- Checksums use RFC 8785 JCS canonicalization
- Replay verifies policy_version and schema_version (fail-closed on mismatch)
- **Platform determinism:** Golden vectors enforced on: **Linux (x86_64, stable Rust)**. Cross-platform determinism (Windows, macOS) will be validated in CI before claiming.

---

## MCP Tools

The MCP surface is intentionally small. All tools are deterministic, side-effect constrained, and auditable.

### State Operations

| Tool | Description |
|------|-------------|
| `node.create` | Create a node with initial values |
| `node.query` | Query node state by ID |
| `node.mutate` | Mutate node belief by delta (debug only) |

### Propagation

| Tool | Description |
|------|-------------|
| `edge.bind` | Bind an edge between nodes |
| `edge.propagate` | Run a deterministic propagation step |

### Governance & Audit

| Tool | Description |
|------|-------------|
| `decision.check` | Governance evaluation (canonical) |
| `decision.preview` | Non-authoritative simulation |
| `audit.search` | Search governance decision history |
| `audit.export` | Export audit bundle (canonical) |
| `audit.replay` | Deterministic replay of decision history (canonical) |
| `governor.health` | Drift and coherence metrics (canonical) |
| `governance.health` | Governance subsystem health (canonical) |

Legacy aliases (`governance.evaluate`, `governor.status`, `governance.status`, `esv.audit`, `lineage.replay`) are supported but deprecated.

---

## Quick Start

```bash
# Clone and build (public_stub mode – no proprietary dependencies)
git clone https://github.com/aduboseh/iter.git
cd iter
cargo build --release

# Run governance invariant tests
cargo test --test governance_invariants

# Build the server binary
cargo build --release --bin iter-server

# Query tools list (STDIO transport)
echo '{"jsonrpc":"2.0","method":"tools/list","id":1}' | \
  cargo run --release --bin iter-server
```

Note: Examples compile in public_stub mode and demonstrate governance behavior
without proprietary substrates. See docs/ARCHITECTURE_BOUNDARY.md.

Executable Proof (Optional)
```bash
cargo run --example governance_demo
```

For additional demonstrations (stateful lineage/replay, stochastic proposal foil),
see demos/README.md
.

---

## Security & Governance Model

Iter assumes hostile inputs and untrusted clients by default.

Every outbound response is deterministic and verifiable.

### Enforcement

Fail-closed contracts (unknown enums rejected)

NaN / Inf rejection on all numeric fields

Deterministic policy evaluation order

Economic authority enforced via config and permits

### Integrity

Canonical serialization

Cryptographic checksums (SHA-256)

Append-only audit logs

Details:

`docs/SECURITY.md`

`docs/ATTACK_SURFACE.md`

`docs/GOVERNANCE.md`

`docs/contracts_v1.md`

---

## SDKs (Transport Adapters)

Iter exposes a stable MCP surface. SDKs provide ergonomic access without altering governance semantics.

Available SDKs

Rust SDK – First-class, reference implementation

TypeScript SDK – Node.js client for MCP integrations

Python SDK – Thin client for orchestration and testing

CLI – Deterministic inspection, replay, and governance queries

Invariant

SDKs do not embed policy, learning logic, or execution semantics.
All governance decisions occur inside Iter Server.

Stability

SDKs track the frozen MCP contract

Breaking changes require protocol version bumps

SDK regressions cannot alter DecisionPacket output

See sdks/
 for implementation details.

---

## Kernel Compatibility

Validated against drift-kernel v1.0.0.

---

## Testing

```text
# Governance Invariants
cargo test --test governance_invariants

# Adversarial Governance Tests
cargo test --test adversarial_governance

# Replay Contract Tests
cargo test --test replay_contract

# Golden Vector Determinism
cargo test --test golden_vectors
```

All tests pass deterministically across repeated runs.

---

## Documentation

Formal specifications and threat models are provided under /docs:

SECURITY.md

ATTACK_SURFACE.md

GOVERNANCE.md

contracts_v1.md

ARCHITECTURE_BOUNDARY.md

RELEASE.md

---

## Architectural Context

Beyond Stochastic Intelligence describes the architectural failure modes that motivate deterministic governance layers (e.g., non-determinism at the authority boundary, unverifiable reasoning chains, and authority collapse). Iter provides a concrete, executable implementation of those constraints as a deterministic governance control plane.

See papers/Beyond_Stochastic_Intelligence.pdf
 for the full position paper, and papers/README.md
 for an index.

---

## License

Iter is licensed under the Apache-2.0 License.

Proprietary substrate components are not included in this repository.

---

## Marketplace Identity

Name: Iter
Subtitle: Deterministic Governance & Audit Control Plane
Protocol: Model Context Protocol (MCP) | JSON-RPC 2.0
Status: Surface Frozen (January 2027)

Iter Server: Governance, not guesswork.
