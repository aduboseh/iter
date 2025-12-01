# SCG Demo Package

Production-grade demonstration package for the SCG (Semantic Cognitive Graph) substrate.

## Quick Start

```bash
# Narrator demo (certified output with narration)
./demos/scg_demo_narrator.sh

# Live demo (PowerShell, persistent MCP session)
powershell -ExecutionPolicy Bypass -File ./demos/scg_demo_live.ps1
```

## Demo Modes

### Narrator Demo (`scg_demo_narrator.sh`)

- **Phases 1-7:** Certified output from actual substrate execution.
- **Phase 8:** Conceptual illustration of stability envelope behavior (design specification).
- **Purpose:** Educational - shows intended governor behavior as temperature changes.

### Live Demo (`scg_demo_live.ps1`)

- **All phases:** Strictly factual output from the live MCP runtime.
- Currently operates in deterministic mode only.
- No simulated behavior - every response comes from the running binary.

### Key Difference

The narrator demo includes Phase 8 (Stability Envelope) which illustrates how the governor
*will* respond to temperature variations. This is a design specification for the future
`governor.stability_probe` API, not current runtime behavior.

The live demo shows only what the substrate actually does today.

## Phase 8: Stability Envelope (Design Specification)

Phase 8 demonstrates the target behavior for temperature-based stability boundaries:

| Temperature | Status  | Governor Action |
|-------------|---------|-----------------|
| 0.0         | STABLE  | Accepted (fully deterministic) |
| 0.3         | STABLE  | Accepted (within envelope) |
| 0.7         | WARNING | Allowed with monitoring |
| 1.0         | REJECTED| Refused by governor |

**Note:** Temperature probing will be exposed as a future API (`governor.stability_probe`).
Phase 8 reflects the target behavior once this API is implemented.

## Files

- `demos/scg_demo_narrator.sh` - Narrated presentation with certified output
- `demos/scg_demo_live.ps1` - Live execution against MCP runtime
- `demos/scg_demo.sh` - Core demo script
- `demo_expected/` - Expected output files for verification
- `SUBSTRATE_OVERVIEW.md` - Technical substrate documentation

## Requirements

- **Narrator demo:** Bash shell
- **Live demo:** PowerShell 7+, compiled `scg_mcp_server.exe`

## Author

Armonti Du-Bose-Hill | Only SG Solutions

## License

(c) 2025 Only SG Solutions
