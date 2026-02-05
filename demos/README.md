# Iter Demos

This directory contains fully reproducible demonstrations of Iter’s deterministic governance guarantees, replay properties, and enforcement boundaries.

Each demo is designed to prove a specific invariant of the Iter governance control plane. Together, they establish that governance outcomes are **deterministic, auditable, and verifiable** independent of upstream model behavior.

**The stochastic demo is included to demonstrate failure modes at the proposal layer; it is not part of Iter’s execution or trust boundary.**

---

## Available Demos

### 1. Deterministic Governance over Stochastic Proposals

**File:** `governance_over_stochastic.ps1`

This demo demonstrates deterministic governance verdicts over stochastic upstream proposals using the Iter MCP server.

The purpose of the demo is to show that, for a fixed governance substrate and policy configuration, governance outcomes are a deterministic function of inputs and constraints — even when upstream proposals vary in magnitude, intent, or origin.

**Stochasticity in upstream models does not compromise determinism at the governance layer.**

#### What It Proves

For a fixed governance substrate and policy configuration:

- Governance verdicts are deterministic
- Identical governance inputs produce identical verdicts
- Determinism is enforced independently of proposal source
- Determinism is verifiable via stable cryptographic checksums

#### Determinism vs Lineage (Important Clarification)

This demo proves deterministic governance verdicts, not long-lived lineage accumulation.

In `governance_over_stochastic.ps1`, each JSON-RPC call spawns a fresh `iter-server` process. As a result:

- Governance verdicts are deterministic and repeatable
- Cryptographic checksums are stable across executions
- Lineage entry count is intentionally zero (`entry_count: 0`)
- Determinism is validated via checksum equality, not event accumulation

This behavior is by design for this demo.

#### Proof Structure

| Phase | Description |
|------:|-------------|
| **Inputs** | Divergent proposals simulating stochastic LLM outputs |
| **Process** | Deterministic governance evaluation via the Iter MCP interface |
| **Outputs** | Governance verdicts and associated evaluation metrics |
| **Evidence** | Cryptographic checksum over exported proof artifacts, enabling independent verification across runs |

#### How to Run

From the repository root:

```powershell
cargo build --release --bin iter-server
.\demos\governance_over_stochastic.ps1
```

If PowerShell execution policy blocks the script:

```powershell
pwsh -ExecutionPolicy Bypass -File .\demos\governance_over_stochastic.ps1
```

#### Generated Artifacts

| Artifact | Description |
|---------|-------------|
| `demos/governance_proof.json` | Deterministic governance verdict summary and checksum |
| `demos/governance_proof_log.json` | Structured execution log capturing evaluation phases |

These artifacts are reproducible by re-running the demo and comparing checksums.

---

### 2. Deterministic Governance & Replay (Stateful Lineage)

**File:** `determinism_demo.ps1`

This demo demonstrates deterministic governance with stateful lineage accumulation.

It is intended to showcase:

- Non-empty lineage graphs
- Replayable audit chains
- Long-lived governance sessions
- Deterministic replay across time

Unlike `governance_over_stochastic.ps1`, this demo maintains a single server session and accumulates governance history.

#### What It Proves

- Governance lineage accumulates deterministically
- Replay is possible from checkpointed state
- Audit chains are cryptographically verifiable
- Governance outcomes remain deterministic across session lifetimes

#### How to Run

From the repository root:

```powershell
cargo build --release --bin iter-server
.\demos\determinism_demo.ps1
```

If PowerShell execution policy blocks the script:

```powershell
pwsh -ExecutionPolicy Bypass -File .\demos\determinism_demo.ps1
```

---

### 3. Rust Governance Example (Reference Path)

**File:** `examples/governance_demo.rs`

This example demonstrates deterministic governance directly via the Rust API.

It showcases:

- Policy enforcement over reasoning quality
- Learning freeze under scarcity conditions
- Canonical DecisionPacket emission
- Byte-identical outputs across repeated runs

#### How to Run

```bash
cargo run --example governance_demo
```

#### What It Proves

- Governance can be invoked programmatically without JSON-RPC overhead
- The Rust path is the reference implementation
- DecisionPackets are deterministic regardless of invocation path

---

## Requirements

### All Demos

- Rust 1.75+
- Cargo
- Linux, macOS, or Windows

### PowerShell-Based Demos

- PowerShell 7.4+
- Windows 10/11 or macOS/Linux with pwsh
- UTF-8 capable terminal

PowerShell 7 is required to ensure consistent JSON handling, numeric coercion, and deterministic text encoding across executions.

---

## Demo Comparison Matrix

| Demo | Transport | Session Model | Lineage | Primary Proof |
|------|-----------|---------------|---------|---------------|
| `governance_over_stochastic.ps1` | JSON-RPC (STDIO) | Fresh process per call | 0 (by design) | Deterministic verdicts over stochastic inputs |
| `determinism_demo.ps1` | JSON-RPC (STDIO) | Single persistent session | Accumulated | Stateful lineage + replay |
| `governance_demo.rs` | Rust API | In-process | N/A | Programmatic governance without MCP overhead |

---

## Verification Workflow

For any demo that emits checksums:

1. Run the demo once and record the checksum value
2. Run the demo again with identical inputs
3. Compare checksums

If identical, determinism is proven.

Checksum equality is the verification primitive.

No external oracles, no trusted timestamps, no probabilistic validation.

---

## Extension Points

### Replacing Stochastic Proposals with Live LLMs

The `governance_over_stochastic.ps1` demo uses a hardcoded `$proposals` array to simulate stochastic LLM outputs.

To integrate live LLM calls:

- Replace the `$proposals` array with API calls to Claude, GPT, or local models
- Pass the returned strings directly to Iter governance tools
- Keep governance evaluation inside Iter unchanged

Governance evaluation remains deterministic regardless of LLM variability. No changes to Iter are required.

### Adding Custom Policy Rules

All demos use the default governance substrate included in public_stub mode.

To test custom policy rules:

- Modify the governance substrate configuration
- Re-run the demos
- Verify that governance outcomes change deterministically based on the new rules

---

## Troubleshooting

### PowerShell Execution Policy (Windows)

If you encounter:

```
File cannot be loaded because running scripts is disabled on this system.
```

Run:

```powershell
pwsh -ExecutionPolicy Bypass -File .\demos\governance_over_stochastic.ps1
```

### PowerShell 7 Not Installed

Windows:

```powershell
winget install --id Microsoft.PowerShell --source winget
```

macOS:

```bash
brew install --cask powershell
```

### Checksum Mismatch

If checksums do not match across runs:

- Verify all inputs are identical (config files, substrate state, policy rules)
- Check for external dependencies (system time, random number generators, network calls)
- Ensure PowerShell 7.4+ (earlier versions can serialize JSON differently)

Iter’s governance layer is deterministic by construction. Checksum mismatches indicate environmental variation, not Iter non-determinism.

---

## Scope Boundary

These demos prove governance determinism and replay, not model correctness, learning performance, or inference quality.

Iter governs decisions produced by upstream systems. It does not generate them.

---

## Related Documentation

- README.md – Project overview and architecture
- docs/SECURITY.md – Threat model and attack surface
- docs/GOVERNANCE.md – Policy enforcement semantics
- docs/contracts_v1.md – MCP contract specification

---

Iter Demos: Proof, not promises.
