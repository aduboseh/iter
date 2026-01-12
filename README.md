# Iter Server

Deterministic governance control plane for auditable decision-making, replayable reasoning paths, and policy-enforced execution.

**Determinism · Governance · MCP**

---

## What is Iter?

Iter Server is a hardened **Model Context Protocol (MCP)** server (JSON-RPC 2.0) that acts as an **authoritative governance and audit control plane** for decision systems.

Iter evaluates governance conditions, enforces policy and economic constraints, and emits **replay-sufficient DecisionPackets** that prove exactly why a decision occurred.

MCP is the transport.  
Governance, causality, and replay are the product.

**Surface freeze**  
Protocol and SDK surface are stable for 12 months (through January 2027), barring security issues.  
See `RELEASE.md` for compatibility policy.

### Designed for
- Deterministic governance evaluation  
- Replayable decision paths (without re-learning)  
- Policy-enforced reasoning and learning gates  
- Audit-ready causality with cryptographic verification  

### Iter is not
- A general-purpose agent runtime  
- An orchestration framework  
- A learning or training system  
- A low-latency execution engine  

Iter does not compute reasoning signals or perform learning.  
It governs and proves decisions produced by upstream systems.

---

## Governance Artifacts

### DecisionPacket

The primary output of Iter is a **DecisionPacket**.

A DecisionPacket is a replay-sufficient, immutable record that contains everything required to reconstruct a governance outcome without re-running learning or inference.

Each packet includes:
- System state snapshot (energy, reasoning, learning, policy)
- Capsule identity and version hashes
- Policy decisions and explicit reason codes
- Learning permissions and economic constraints
- Canonical JSON serialization with SHA-256 checksum

DecisionPackets are deterministic:
- Identical inputs and configuration produce byte-identical packets
- Checksums verify integrity across time and systems

---

## MCP Tools

The MCP surface is intentionally small. All tools are deterministic, side-effect constrained, and auditable.

### State Operations

| Tool        | Description                              |
|-------------|------------------------------------------|
| node.create | Create a node with initial values         |
| node.query  | Query node state by ID                   |
| node.mutate | Mutate node belief by delta (debug only) |

### Propagation

| Tool           | Description                |
|----------------|----------------------------|
| edge.bind      | Bind an edge between nodes |
| edge.propagate | Run a deterministic step  |

### Governance & Audit

| Tool               | Description                                  |
|--------------------|----------------------------------------------|
| governance.status  | Query governance health                      |
| governor.status    | Query drift and coherence status             |
| lineage.replay     | Replay checksum history                      |

Governance tools emit DecisionPackets when applicable.

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
echo '{"jsonrpc":"2.0","method":"tools/list","id":1}' | cargo run --release --bin iter-server
```

Note: Examples compile in public_stub mode and demonstrate governance behavior without proprietary substrates.
See `ARCHITECTURE_BOUNDARY.md` for build mode details.

---

## Deterministic Governance Demo

A reproducible demo demonstrating deterministic governance over reasoning quality, learning permissions, and economic constraints.

```bash
cargo run --example governance_demo
```

### What the demo shows

- Policy halt on low reasoning quality
- Learning frozen under scarcity conditions
- Explicit policy reason codes
- Byte-identical DecisionPackets across repeated runs

### Properties

- Canonical JSON DecisionPacket with SHA-256 checksum
- Replay without re-learning or inference
- Governance outcomes deterministic by construction

---

## Security & Governance Model

Iter assumes hostile inputs and untrusted clients by default.

Every outbound response is deterministic and verifiable.

### Enforcement

- Fail-closed contracts (unknown enums rejected)
- NaN / Inf rejection on all numeric fields
- Deterministic policy evaluation order
- Economic authority enforced via config and permits

### Integrity

- Canonical serialization
- Cryptographic checksums
- Append-only audit logs

Details:
- `docs/SECURITY.md`
- `docs/ATTACK_SURFACE.md`
- `docs/GOVERNANCE.md`
- `docs/contracts_v1.md`

---

## Kernel Compatibility

Validated against drift-kernel v1.0.0.

---

## Testing

```bash
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

## License

Iter is licensed under Apache-2.0.
Proprietary substrate components are not included in this repository.

---

## Marketplace Identity

**Name:** Iter

**Subtitle:** Deterministic Governance & Audit Control Plane
