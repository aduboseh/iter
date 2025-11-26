# SCG MCP Server - Pilot Deployment Guide

## Overview

This directory contains configuration for deploying SCG-MCP server into real-world MCP client environments for field trials and validation.

## Quick Start

### 1. Build the Server

```powershell
cargo build --release
```

### 2. Configure Your MCP Client

Copy the server configuration to your MCP client's config directory:

**For Warp:**
```powershell
# Create config directory if it doesn't exist
New-Item -ItemType Directory -Path "$env:USERPROFILE\.warp\mcp" -Force

# Copy configuration
Copy-Item .\deployment\pilot\mcp_client_config.json "$env:USERPROFILE\.warp\mcp\scg-mcp.json"
```

**For Cursor:**
```powershell
New-Item -ItemType Directory -Path "$env:APPDATA\Cursor\User\globalStorage\mcp" -Force
Copy-Item .\deployment\pilot\mcp_client_config.json "$env:APPDATA\Cursor\User\globalStorage\mcp\scg-mcp.json"
```

**For VSCode (Cline):**
```powershell
New-Item -ItemType Directory -Path "$env:APPDATA\Code\User\globalStorage\mcp" -Force
Copy-Item .\deployment\pilot\mcp_client_config.json "$env:APPDATA\Code\User\globalStorage\mcp\scg-mcp.json"
```

### 3. Start Monitoring

Open a separate terminal to monitor telemetry:

```powershell
# Watch telemetry output
Get-Content -Path .\telemetry\scg_pilot.jsonl -Wait
```

### 4. Verify Operation

Launch your MCP client (Warp, Cursor, etc.) and verify the server is running:

```
# In your MCP client, list available tools:
tools/list

# Expected output should include SCG tools:
- node.create
- node.mutate
- edge.bind
- governor.status
# etc.
```

## Monitoring During Pilot

### Real-Time Telemetry

Telemetry records are emitted to stderr and to `telemetry/scg_pilot.jsonl`. Each record contains:

```json
{
  "timestamp": "2025-11-16T21:00:00Z",
  "cluster_id": "SCG-PILOT-01",
  "energy_drift": 9.4e-11,
  "coherence": 0.974,
  "esv_valid_ratio": 1.0,
  "entropy_index": 0.0002,
  "node_count": 42,
  "edge_count": 108
}
```

### Watch for Violations

Monitor stderr for alerts:

```
TELEMETRY] VIOLATION DETECTED: EnergyDrift { current: 1.2e-9, threshold: 1e-10 }
QUARANTINE] ===== ENTERING QUARANTINE MODE =====
```

### Lineage Snapshots

Snapshots are automatically created every 500 operations in `./snapshots/`:

```powershell
# List snapshots
Get-ChildItem .\snapshots\

# Validate a snapshot
cargo run -- validate-snapshot .\snapshots\snapshot_20251116_210000.json
```

## Success Criteria

The pilot is successful if:

1. **Zero energy drift violations** over 7 days (drift ≤ 1e-10)
2. **Zero lineage mismatches** across replays
3. **Zero ESV violations** (esv_valid_ratio = 1.0 continuously)
4. **Coherence maintained** (C(t) ≥ 0.97) under realistic load
5. **No catastrophic failures** requiring quarantine mode
6. **Uptime ≥ 99.9%** across all client environments

## Troubleshooting

### Server Won't Start

Check logs for initialization errors:
```powershell
cargo run 2>&1 | Select-String -Pattern "ERROR|PANIC"
```

### Drift Violations

If energy drift exceeds threshold:
1. Check telemetry for pattern (gradual vs sudden)
2. Review lineage log for problematic operations
3. Inspect fault trace in quarantine logs
4. Roll back to last valid checkpoint

### Lineage Mismatch

If replay produces different hash:
1. Export lineage: `lineage.export path=./debug_lineage.json`
2. Replay in isolated environment
3. Compare checksums at each operation
4. File issue with exact reproduction steps

### Quarantine Mode

If system enters quarantine:
1. Export audit report: `cargo run -- export-quarantine-report`
2. Review fault trace ID in logs
3. Validate rollback to last checkpoint
4. Determine root cause before clearing quarantine
5. Clear with manual approval token

## Configuration Reference

See `scg_mcp_pilot.yml` for detailed configuration options:

- **Transport**: stdio (MCP protocol)
- **Thresholds**: Energy drift, coherence, ESV ratio
- **Telemetry**: Emission interval, export targets
- **Fault Handling**: Rollback frequency, quarantine triggers
- **Resources**: Graph size limits, memory limits

## Deployment Checklist

-  ] Rust toolchain installed (1.70+)
-  ] Server builds successfully (`cargo build --release`)
-  ] MCP client configured with correct path
-  ] Telemetry directory created (`./telemetry/`)
-  ] Snapshots directory created (`./snapshots/`)
-  ] Monitoring dashboard prepared (optional)
-  ] Alert webhook configured (optional)
-  ] Team notified of pilot start date
-  ] Baseline telemetry captured for comparison

## Support

For issues during pilot:
- Review logs in `./telemetry/` and stderr
- Consult `scg_mcp_pilot.yml` for threshold configuration
- Check GitHub issues for known problems
- Contact: Your contact info]

## After Pilot

Upon successful completion:
1. Export final telemetry summary
2. Generate lineage audit report
3. Document any observed failure modes
4. Prepare substrate hardening recommendations
5. Plan connectomics v2 integration (if substrate is stable)
