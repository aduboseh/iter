# Iter Demos

## `governance_over_stochastic.ps1`

This demo demonstrates **deterministic governance verdicts over stochastic upstream proposals** using the Iter MCP server.

The purpose of the demo is to show that, for a fixed governance substrate and policy configuration, governance outcomes are a deterministic function of inputs and constraints, even when upstream proposals vary in magnitude and intent.

Stochasticity in upstream models does not compromise determinism at the governance layer.

---

## What It Proves

For a fixed governance substrate and policy configuration:

* Governance verdicts are deterministic
* Identical governance inputs produce identical verdicts
* Determinism is enforced independently of proposal source
* Determinism is verifiable via stable cryptographic checksums

---

## Determinism vs Lineage (Important Clarification)

This demo proves **deterministic governance verdicts**, not long-lived lineage accumulation.

In `governance_over_stochastic.ps1`, each JSON-RPC call spawns a fresh `iter-server` process. As a result:

* Governance verdicts are deterministic and repeatable
* Cryptographic checksums are stable across executions
* Lineage entry count is intentionally zero (`entry_count: 0`)
* Determinism is validated via checksum equality, not event accumulation

This behavior is **by design** for this demo.

---

## Proof Structure

### Inputs

Divergent proposals simulating stochastic LLM outputs.

### Process

Deterministic governance evaluation via the Iter MCP interface.

### Outputs

Governance verdicts and associated evaluation metrics.

### Evidence

Cryptographic checksum over the exported governance proof artifact, enabling independent verification of deterministic behavior across runs.

---

## How to Run

From the repository root:

```powershell
cargo build --release --bin iter-server
.\demos\governance_over_stochastic.ps1
```

If PowerShell execution policy blocks the script:

```powershell
pwsh -ExecutionPolicy Bypass -File .\demos\governance_over_stochastic.ps1
```

---

## Generated Artifacts

### Artifact Binding

The following artifacts are generated **specifically by `governance_over_stochastic.ps1`:**

* `demos/governance_proof.json`
  Deterministic governance verdict summary and checksum.

* `demos/governance_proof_log.json`
  Structured execution log capturing all evaluation phases.

These artifacts are reproducible by re-running the demo and comparing checksums.

---

## Determinism Verification

Re-run the script and compare the exported checksum values.

Identical checksums confirm deterministic governance evaluation.

---

## Extension to Live LLMs

The `$proposals` array simulates stochastic LLM outputs (for example, temperature > 0).

Live LLM routing (Claude, GPT, or local models) can replace this array without modifying governance logic.

The governance substrate evaluates whatever proposals arrive. Determinism is enforced at the governance layer, not at the proposal layer.

---

## Related Demo: Stateful Lineage Accumulation

For demonstrations of **non-empty lineage, replayable audit chains, and long-lived governance sessions**, see:

* `determinism_demo.ps1`

That demo maintains a single server session and is intended to showcase lineage accumulation behavior.
