# Iter Demos

## governance_over_stochastic.ps1

This demo demonstrates deterministic governance over stochastic model outputs using the Iter MCP server.

The purpose of the demo is to show that, for a fixed governance substrate and policy set, governance outcomes are a deterministic function of inputs and constraints, even when upstream proposals vary in magnitude and intent.

Stochasticity in upstream models does not compromise determinism at the governance layer.

## What It Proves

For a fixed governance substrate and policy configuration:

- Governance verdicts are deterministic
- Identical governance inputs produce identical outputs
- Lineage artifacts can be replayed to reproduce results exactly
- Determinism is enforced independently of proposal source

## Proof Structure

Inputs  
Divergent proposals simulating stochastic LLM outputs.

Process  
Deterministic governance evaluation via the Iter MCP interface.

Outputs  
Governance verdicts and associated evaluation metrics.

Evidence  
Hash-chained lineage artifact with a cryptographic checksum.

Verification  
Lineage replay produces an identical checksum across executions.

## How to Run

```powershell
cd C:\Users\adubo\iter
cargo build --release
.\demos\governance_over_stochastic.ps1
Generated Artifacts
demos/governance_proof.json
Exported lineage artifact representing the full governance execution trace.

demos/governance_proof_log.json
Structured proof log capturing all phases of the evaluation.

Determinism Verification
Re-run the script and compare the exported lineage checksums.
Identical checksums confirm bit-exact deterministic replay.

Extension to Live LLMs
The $proposals array simulates stochastic LLM outputs (e.g., temperature > 0).

Live LLM routing (Claude, GPT, or local models) can replace this array without modifying governance logic.

The governance substrate evaluates whatever proposals arrive. Determinism is enforced at the governance layer, not at the proposal layer.
