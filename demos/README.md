# Iter Demos

## governance_over_stochastic.ps1

Demonstrates deterministic governance over stochastic model outputs.

### What it proves

For a fixed governance substrate and policy set, governance outcomes are a deterministic function of inputs and constraints, even when proposals vary in magnitude and intent.

### Proof structure

1. **Inputs**: Divergent proposals (simulated LLM stochasticity)
2. **Process**: Deterministic governance evaluation via MCP
3. **Outputs**: Verdicts + metrics
4. **Evidence**: Hash-chained lineage with checksum
5. **Verification**: Replay → identical checksum

### Run

```powershell
cd C:\Users\adubo\iter
cargo build --release
.\demos\governance_over_stochastic.ps1
```

### Artifacts

- `governance_proof.json` — Exported lineage (cryptographic audit trail)
- `governance_proof_log.json` — Structured proof log with all phases

### Extension to live LLM

The `$proposals` array simulates stochastic LLM outputs. Live LLM routing (Claude, GPT, local models) can replace this array without changing governance logic. The substrate evaluates whatever proposals arrive; determinism is enforced at the governance layer, not the proposal layer.

### Verification

Re-run the script and compare lineage checksums. Identical checksums confirm deterministic replay.
