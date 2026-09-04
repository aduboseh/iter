# Iter

Iter is an auditable MCP decision layer for governed AI agents.

> **Release classification:** Release-candidate infrastructure, not Enterprise GA.
> [APEX-SCG-ITER-PROD-001](APEX_PRODUCTIZATION_V1.md), as amended by
> [APEX-PRODUCTIZATION-GAP-CLOSURE-001](APEX_PRODUCTIZATION_GAP_CLOSURE_001.md),
> is the binding productization authority. Enterprise release requires all 30
> machine-verifiable controls to pass.

It lets an agent request a decision, routes that request through SCG-backed deterministic governance, and emits a proof packet that can be replayed under the active `scg.v1` contract.

The model proposes. Iter asks. SCG decides. The bridge binds. The packet proves. Replay verifies.

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![Protocol](https://img.shields.io/badge/protocol-MCP%20%7C%20JSON--RPC%202.0-green.svg)](https://modelcontextprotocol.io/)

**Governance** · **Proof Packets** · **Replay** · **MCP**

---

## What to run

Raw server boot is not the product proof. The golden path is the product proof:

```powershell
./scripts/golden_path.ps1
```

Expected result:

```text
GOLDEN_PATH_PASS
contract_version=scg.v1
claim_registry_version=1.0
determinism_scope=same_binary_only
platform=<target_triple>
rustc_version=<version>
cross_platform_replay_claimed=false
scg_source_commit=0306feb600e12c627dc4b10963fc8f7781dc0e18
scg_vendor_master_head=b6c9a3b641291631358fcf9f8deace74d71e7615
build_rerun_triggers=verified
rustc_env_exports=verified
bridge_integrity=verified
canonical_vectors_raw_byte_hash=verified
canonical_vector_uppercase_digests=verified
proof_packet_provenance=compile_time_exports+runtime_decision
proof_numeric_encoding=ieee754-f64-bits-lowerhex
replay_verification=verified
drift_simulation=verified
working_tree_mutated=false
```

See [docs/CLAIM_BOUNDARY.md](docs/CLAIM_BOUNDARY.md) for the current claim ceiling.

## Runtime Modes

Iter Server is a hardened Model Context Protocol (MCP) server (JSON-RPC 2.0) for deterministic governance evaluation.

Iter currently exposes three public server runtime modes:

- **Demo mode** (default `iter-server` boot path): Threshold-based governance using stub graph state. Non-authoritative — `authoritative_pdp=false`, `replay_sufficient=false`, no `DecisionPacket` at the MCP edge. Suitable for protocol validation and integration testing.
- **Governed-local mode** (`--runtime-mode=governed-local`): PolicyEvaluator-based governance with typed contract envelopes. Emits `DecisionPacket` with RFC 8785 JCS checksums, governance hash binding, and ordered execution trace. This mode is replay-capable, but still runs over the local stub substrate rather than SCG.
- **Scg-backed mode** (`--runtime-mode=scg-backed`): Calls `POST /governance/evaluate` on the live SCG gateway. Requires `SCG_ENDPOINT` and `governance/governance.hash` at boot, sends `Authorization: Bearer <token>` when `SCG_AUTH_TOKEN` or `SCG_GATEWAY_AUTH_TOKEN` is configured, fails closed if required configuration is absent or malformed, enforces HTTP status, `contract_version`, replay integrity, governance hash, and SCG state-envelope checks on every response, emits governed packets on `decision.check`, and has no fallback runtime on SCG unavailability. This mode is available but is not the default.

**MCP is the transport. Default server mode remains demo/non-authoritative. SCG↔Iter seam closed — see `SKILLS/runtime-seam.md`.**

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
cargo run --features schema-gen --bin generate-schemas
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

## Governance Source

Iter does not define canonical governance. SCG does.

- Authoritative governance source: `SCG/governance/SCG_Governance_v1.0.md`
- Mirrored governance artifact in this repo: `governance/SCG_Governance_v1.0.md`
- Mirrored checksum source in this repo: `governance/governance.hash`
- Derived explanatory stub: `governance/GOVERNANCE.md`

If the mirrored governance artifact drifts from SCG, CI fails closed.

---

## Governance Artifacts

### GovernanceOutcome

The governed runtime implementation is centered on a `GovernanceOutcome` containing:
- `verdict` (ALLOW / BLOCK / REVIEW)
- `mode` (demo / governed)
- `authoritative_pdp` (true only in governed mode)
- `replay_sufficient` (true only for governed evaluate)
- `reason_codes` (namespaced: `demo.thresholds.*` or `policy.*`)
- `packet` (DecisionPacket, governed evaluate only)

Current public server status: `decision.preview` follows the active runtime mode. Default demo mode remains non-authoritative; `--runtime-mode=governed-local` performs authoritative local policy preview without emitting a packet; `--runtime-mode=scg-backed` calls the live SCG endpoint without emitting a packet.

### DecisionPacket (governed evaluate paths only)

In governed-local mode and scg-backed mode, `decision.check` emits a `DecisionPacket` — a replay-sufficient, immutable record containing everything required to reconstruct the governance outcome. Demo mode does not emit packets.

Each packet includes:
- System state snapshot (energy, reasoning, learning, policy envelopes)
- `governance_hash`: omitted on demo path, populated from local `governance/governance.hash` on governed-local path, populated from the SCG canonical hash on scg-backed path
- `execution_trace`: omitted on demo path, attached from local evaluation on governed-local path, attached from SCG evaluation on scg-backed path
- `ITER_BUILD_HASH` and `SUBSTRATE_BUILD_HASH`: real compile-time SHA-256 identifiers that trace the packet to the producing binary
- Capsule identity and version hashes
- Policy decisions and explicit reason codes
- Learning permissions and economic constraints
- Proof-critical numeric fields encoded as exact IEEE-754 lowercase hex strings (`ieee754-f64-bits-lowerhex`)
- RFC 8785 JCS canonical JSON with SHA-256 checksum

DecisionPreview is non-authoritative in demo mode. In governed-local mode and scg-backed mode it reflects authoritative policy evaluation, but it is still not replay-sufficient because no packet is emitted.
Only DecisionPacket participates in checksum, replay, and audit guarantees.

**Demo mode does not emit DecisionPackets.** Demo verdicts are non-authoritative threshold checks.

#### Determinism Guarantee

DecisionPackets are deterministic within the declared replay scope:
- Identical inputs and configuration produce byte-identical packets under the same binary, architecture, toolchain, build hash, and `scg.v1` contract
- Proof-critical numeric values are serialized as exact IEEE-754 hex strings in the packet; human-readable float displays are not the packet authority.
- Checksums use RFC 8785 JCS canonicalization
- Replay verifies policy_version and schema_version (fail-closed on mismatch)
- Cross-platform replay is not claimed. `cross_platform_replay_claimed=false` is part of the golden-path proof surface.

---

## MCP Tools (by Profile)

The MCP surface is intentionally small. All tools are deterministic, side-effect constrained, and auditable.

### Governance Profile (Server Surface)

The following tools are available when Iter is run with `--profile=governance`.

Current public server note: `--profile=governance` constrains tool exposure. Runtime behavior then depends on `--runtime-mode`: default demo stub mode, `governed-local` packet-emitting mode, or `scg-backed` fail-closed mode. SCG↔Iter seam closed — see `SKILLS/runtime-seam.md`.

#### Governance & Audit

| Tool | Description |
|------|-------------|
| decision.check | Governance decision gate. Default demo mode is non-authoritative; `--runtime-mode=governed-local` emits governed packets over the local stub substrate; `--runtime-mode=scg-backed` emits governed packets from the live SCG gateway fail-closed |
| decision.preview | Governance preview through the active runtime |
| audit.search | Search governance decision history |
| audit.export | Export DecisionPacket (canonical) |
| audit.replay | Deterministic replay of a DecisionPacket (canonical, read-only) |
| governor.health | Drift and coherence metrics |
| governance.health | Governance subsystem health |


Note: Replay is a pure function over a DecisionPacket, policy_version, and schema_version.
No server-side state mutation or historical re-execution occurs.

### Kernel-Debug Profile (Non-Production)

The following tools are available **only** when Iter is run with `--profile=kernel-debug`.

These tools are **not authoritative**, **not replay-sufficient**, and **must never be used in production**.

#### State Operations (Debug Only)
| Tool | Description |
|------|-------------|
| node.create | Create a node with initial values |
| node.query | Query node state by ID |
| node.mutate | Mutate node belief by delta (debug only) |

#### Propagation (Debug Only)
| Tool | Description |
|------|-------------|
| edge.bind | Bind an edge between nodes |
| edge.propagate | Run a deterministic propagation step |

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
echo '{"jsonrpc":"2.0","method":"tools/list","id":1}' | cargo run --release --bin iter-server
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

Production durability:

`ITER_AUDIT_LEDGER_PATH=/path/to/iter-audit.jsonl` enables a durable JSONL audit ledger for governed and `scg-backed` decisions.

`ITER_REQUIRE_AUDIT_LEDGER=1` makes the ledger mandatory. In that mode, startup fails if the ledger path is missing or the existing hash chain is invalid, and evaluation fails closed if the decision packet cannot be appended, flushed, and synced.

Ledger records are hash-chained and include the `AuditEvent` plus replay-sufficient `DecisionPacket`; this is single-writer process-local durability and integrity evidence. Immutable storage, retention, and cross-region replication remain deployment responsibilities.

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
SDKs may not bypass schema validation, replay contracts, or governance mode restrictions.

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
