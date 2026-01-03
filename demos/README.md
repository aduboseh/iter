# Iter Demos

## `governance_over_stochastic.ps1`

Demonstrates deterministic governance over stochastic model outputs using the Iter MCP server.

---

## What It Proves

For a fixed governance substrate and policy set, governance outcomes are a deterministic function of inputs and constraints, even when proposals vary in magnitude and intent.

Stochasticity in upstream models does not compromise determinism at the governance layer.

---

## Proof Structure

1. **Inputs**  
   Divergent proposals simulating stochastic LLM outputs.

2. **Process**  
   Deterministic governance evaluation via the Iter MCP interface.

3. **Outputs**  
   Governance verdicts and associated metrics.

4. **Evidence**  
   Hash-chained lineage artifact with cryptographic checksum.

5. **Verification**  
   Lineage replay produces an identical checksum across executions.

---

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

The governance substrate evaluates whatever proposals arrive. Determinism is enforced at the governance layer, not the proposal layer.
