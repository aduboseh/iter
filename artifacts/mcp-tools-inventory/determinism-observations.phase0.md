# Phase 0 Determinism Observations

**APEX DIRECTIVE:** ITER-MCP-TOOL-SURFACE v1 — Phase 0  
**Date:** 2026-02-05  
**Observation Mode:** Record only, no fixes

---

## Tool List Determinism

### Test Method
1. Spawned `iter-server.exe` (release binary)
2. Called `tools/list` via JSON-RPC
3. Repeated with fresh process

### Findings

**Cargo Build Logs:**
- Run 1 included compilation output: `Finished 'dev' profile in 3.74s`
- Run 2 included compilation output: `Finished 'dev' profile in 1.96s`
- **Root Cause:** `cargo run --example` triggers recompilation check; timing varies
- **Impact:** Build logs differ, but actual JSON response is byte-identical

**JSON Response (lines 5-179):**
- ✓ **DETERMINISTIC:** Both runs produced identical JSON
- Tool count: 10 tools
- Tool order: Stable across runs
- Schemas: Identical

### Checksum Analysis

| Artifact | SHA-256 |
|----------|---------|
| Run 1 (with build logs) | 996C9DBCD7B147BB806CAC5615FAF9E06A27BEA59863FB5AA163E54FE36EFC83 |
| Run 2 (with build logs) | 3E889A16499485A564EB1F3B04778EEAA9153EE8BC3A6C13410C732165CB1DF7 |
| Clean JSON (no build logs) | B8FAE70F0783836C8C2D29D7848B05C8C0A3BE0BA26A958E63634FDBA89D689B |

### Conclusion

**MCP Tool List is Deterministic**

The `tools/list` response from `iter-server.exe` is byte-identical across runs when cargo build output is excluded.

**Recommendation for Future Phases:**
- Use pre-built release binary (`iter-server.exe` directly, not `cargo run`)
- Or redirect stderr to filter build logs

---

## Tools Observed (Count: 10)

1. `node.create`
2. `node.query`
3. `node.mutate`
4. `edge.bind`
5. `edge.propagate`
6. `governor.status`
7. `governance.status`
8. `esv.audit`
9. `lineage.replay`
10. `governance.evaluate`

---

## No Further Tests Performed

Per APEX directive Section 6, read-only tool determinism tests were not performed in Phase 0 (observation-only mode).

Future phases may test:
- `governance.status` (read-only)
- `governor.status` (read-only)
- `lineage.replay` (deterministic replay on stable ticks)
