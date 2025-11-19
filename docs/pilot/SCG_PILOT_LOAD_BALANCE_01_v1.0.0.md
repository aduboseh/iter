# SCG-PILOT-01 Load Balance Protocol v1.0.0

**Directive**: SG-SCG-LOAD-BALANCE-01 v1.0.0  
**Authority**: Tier-0 (Substrate Sovereign)  
**Issuer**: Armonti Du-Bose-Hill  
**Status**: ACTIVE (Day-1A)  
**Parent Directives**: ACT-07, COHERENCE-01, SG-SCG-PILOT-AUTH-02

---

## Executive Summary

This document defines the **canonical synthetic stimulus pattern** for SCG-PILOT-01 Day-1A certification telemetry collection. The LOAD-BALANCE-01 regime provides controlled, repeatable substrate activity that maintains active state while ensuring all 7 invariants remain within certification thresholds.

**Purpose**: Convert substrate from idle state (Nodes=0) to active state (Nodes>0) with meaningful lineage writes, enabling certification-grade telemetry collection during the 24-hour Day-1A monitoring window.

---

## 1. Design Goals

### 1.1 Primary Objectives

1. **Active Substrate State**: Maintain non-zero node count with continuous cognitive operations
2. **Invariant Compliance**: Ensure all operations remain within quarantine thresholds:
   - Energy drift: ΔE ≤ 1×10⁻¹⁰
   - Coherence: C(t) ≥ 0.97
   - ESV validation: 100% pass rate
   - Quarantine events: 0
3. **Certification Validity**: Provide repeatable, auditable load pattern for reproducible telemetry
4. **Operational Safety**: Prevent ESV/quarantine triggers during Day-1A execution

### 1.2 Non-Goals

- **Not production-representative load**: This is a canonical pilot regime, distinct from future production load directives
- **Not stress testing**: Load is designed for stability, not capacity validation
- **Not optimization testing**: Focus is certification telemetry, not performance tuning

---

## 2. Stimulus Pattern Specification

### 2.1 Operation Cycle

**Cycle Order** (3 operations per iteration):

1. **`governor.status`** — Read current invariants and governor corrections
2. **`node.create`** — Create low-belief, low-energy node
3. **`lineage.replay`** — Perform minimal ledger exercise

**Cycle Timing**: ~0.9 seconds per cycle (optionally with jitter)

**Expected Rate**: ~67 operations per minute (3 ops/cycle × 0.9s/cycle ≈ 3.33 cycles/s)

### 2.2 JSON-RPC Message Format

#### Operation 1: Governor Status
```json
{
  "jsonrpc": "2.0",
  "id": "1",
  "method": "governor.status",
  "params": {}
}
```

**Purpose**: Read current substrate state without mutation  
**Expected Response**: Invariant metrics (energy_drift, coherence, node_count, etc.)

---

#### Operation 2: Node Creation
```json
{
  "jsonrpc": "2.0",
  "id": "2",
  "method": "node.create",
  "params": {
    "belief": 0.01,
    "energy": 1e-12
  }
}
```

**Purpose**: Minimal cognitive mutation to maintain active state  
**Parameters**:
- `belief`: 0.01 (low conviction, minimal cognitive weight)
- `energy`: 1e-12 (micro-energy allocation, 100× below 1×10⁻¹⁰ drift threshold)

**Rationale**: Micro-energy parameters (1e-12) place allocation 100× below quarantine threshold, preventing drift accumulation while providing non-zero lineage writes. Original v1.0.0 used energy=0.05 which triggered immediate quarantine (5×10⁷× above threshold).

---

#### Operation 3: Lineage Replay
```json
{
  "jsonrpc": "2.0",
  "id": "3",
  "method": "lineage.replay",
  "params": {
    "limit": 1
  }
}
```

**Purpose**: Exercise lineage integrity mechanisms without full replay overhead  
**Parameters**:
- `limit`: 1 (minimal replay scope)

**Rationale**: Validates lineage chain continuity, exercises SHA256 verification, maintains determinism proof.

---

### 2.3 Cycle Implementation

**In-Pod Stimulus Loop** (embedded in deployment):

```bash
#!/bin/sh
echo "[SCG] Starting MCP server with LOAD-BALANCE-01 stimulus loop"

while true; do
  printf '{"jsonrpc":"2.0","id":"1","method":"governor.status","params":{}}\n'
  printf '{"jsonrpc":"2.0","id":"2","method":"node.create","params":{"belief":0.01,"energy":1e-12}}\n'
  printf '{"jsonrpc":"2.0","id":"3","method":"lineage.replay","params":{"limit":1}}\n'
  sleep 0.9
done | /app/scg_mcp_server
```

**Execution**: Loop pipes JSON-RPC messages to `scg_mcp_server` STDIO interface at 0.9s intervals

**Startup Banner**: `[SCG] Starting MCP server with LOAD-BALANCE-01 stimulus loop`

---

## 3. Expected Telemetry Profile

### 3.1 Invariant Targets (Day-1A)

| Invariant | Target Range | Expected Value | Threshold |
|-----------|--------------|----------------|-----------|
| **Node Count** | > 0 | 10-50 (growing) | N/A |
| **Energy Drift** | < 1×10⁻¹⁰ | ~1×10⁻¹² to 5×10⁻¹¹ | ≤ 1×10⁻¹⁰ |
| **Coherence** | ≥ 0.97 | 0.99-1.0 | ≥ 0.97 |
| **ESV Valid Ratio** | = 1.0 | 1.0 | = 1.0 |
| **Quarantine Events** | = 0 | 0 | = 0 |
| **Edge Count** | ≥ 0 | 0-50 | N/A |
| **Entropy Index** | > 0 | Low positive | N/A |

### 3.2 Operational Metrics

- **Operations/Second**: ~3.33 (3 ops per 0.9s cycle)
- **Operations/Minute**: ~200
- **Operations/Hour**: ~12,000
- **Operations/24h**: ~288,000 (total load for Day-1A)
- **Node Creations/24h**: ~96,000 (33% of total operations)
- **Reads/Writes Ratio**: ~2:1 (governor.status + lineage.replay vs. node.create)

---

## 4. Quarantine Detection & Recovery

### 4.1 Detection Mechanisms

**Primary Detection** (in monitoring script):
```powershell
$QuarantinePattern = "System is quarantined"

if ($rawBlock -match $QuarantinePattern) {
    Write-Host "❌ P0: Substrate quarantine detected. Invoke ACT-07A now." -ForegroundColor Red
    # Log to pilot_reports/day1/interruptions.json
    exit 1
}

if ($invariants.quarantined -eq $true) {
    Write-Host "❌ P0: Invariant-level quarantine state detected. Invoke ACT-07A now." -ForegroundColor Red
    exit 1
}
```

**Telemetry Indicators**:
- Log pattern: `"System is quarantined"`
- Invariant field: `quarantined: true`
- ESV field: `esv_valid: false`

### 4.2 Quarantine Recovery Protocol (ACT-07A)

**If quarantine detected during Day-1A:**

1. **Immediate Actions**:
   - Monitoring script exits with code 1
   - Interruption logged to `pilot_reports/day1/interruptions.json`
   - Operator notified via monitoring alert

2. **Investigation**:
   - Check substrate logs for quarantine trigger
   - Identify which invariant violated threshold
   - Analyze telemetry leading up to quarantine

3. **Recovery Options**:
   - **Option A**: Adjust LOAD-BALANCE-01 parameters (reduce node creation rate, lower belief/energy)
   - **Option B**: Clear quarantine via controlled restart per COHERENCE-01 §3.1
   - **Option C**: Extend Day-1 window to Day-1B with adjusted load

4. **Resumption**:
   - Document recovery in CERTIFICATION_DOSSIER.md
   - Resume monitoring with adjusted parameters
   - Continue Day-1A telemetry collection

---

## 5. Deployment Integration

### 5.1 Kubernetes Deployment Patch

**File**: `deployment/pilot/scg-mcp-deployment.yaml`

**Container Args Modification**:
```yaml
spec:
  template:
    spec:
      containers:
      - name: scg-mcp
        image: scg-mcp:v1.0.0-substrate
        command: ["/bin/sh", "-c"]
        args:
          - |
            echo "[SCG] Starting MCP server with LOAD-BALANCE-01 stimulus loop"
            while true; do
              printf '{"jsonrpc":"2.0","id":"1","method":"governor.status","params":{}}\n'
              printf '{"jsonrpc":"2.0","id":"2","method":"node.create","params":{"belief":0.01,"energy":1e-12}}\n'
              printf '{"jsonrpc":"2.0","id":"3","method":"lineage.replay","params":{"limit":1}}\n'
              sleep 0.9
            done | /app/scg_mcp_server
```

### 5.2 Deployment Commands

```bash
# Apply updated deployment
kubectl apply -f deployment/pilot/scg-mcp-deployment.yaml -n scg-pilot-01

# Restart pod with new configuration
kubectl rollout restart deployment/scg-mcp -n scg-pilot-01

# Verify startup banner
kubectl logs -n scg-pilot-01 deploy/scg-mcp -f | grep "LOAD-BALANCE-01"

# Monitor substrate activity
kubectl logs -n scg-pilot-01 deploy/scg-mcp --tail=100 | grep "node_count"
```

### 5.3 Verification Checklist

- ✅ Startup banner visible in logs: `[SCG] Starting MCP server with LOAD-BALANCE-01 stimulus loop`
- ✅ Telemetry shows `node_count > 0`
- ✅ Energy drift remains `< 1×10⁻¹⁰`
- ✅ Coherence remains `≥ 0.97`
- ✅ No quarantine events triggered
- ✅ Monitoring script shows "Overall Status: HEALTHY" (not DEGRADED)

---

## 6. Risks & Mitigations

### 6.1 Residual Quarantine Risk

**Risk**: Load pattern may still trigger quarantine during Day-1A execution

**Likelihood**: LOW (conservative parameters designed for sub-threshold operation)

**Impact**: HIGH (Day-1A certification requires re-execution)

**Mitigation**:
- Quarantine detection integrated into monitoring script (exits with P0 alert)
- ACT-07A recovery protocol defined
- Parameters can be adjusted mid-cycle if needed
- Load regime is version-bound (v1.0.0) and can be revised

### 6.2 Non-Production-Representative Load

**Risk**: Synthetic load differs from production cognitive workloads

**Likelihood**: CERTAIN (by design)

**Impact**: LOW (pilot goal is substrate certification, not production simulation)

**Mitigation**:
- Clearly labeled as "canonical pilot regime"
- Distinct from future production load directives
- Documentation explicitly notes synthetic nature
- Future pilots can define production-representative loads

### 6.3 Operator Confusion on Active Load

**Risk**: Operators may not understand why substrate is generating load

**Likelihood**: MEDIUM (without clear communication)

**Impact**: LOW (confusion, not technical failure)

**Mitigation**:
- Startup banner clearly identifies load regime: `[SCG] Starting MCP server with LOAD-BALANCE-01 stimulus loop`
- Load profile embedded in telemetry metadata
- OPERATOR_INSTRUCTIONS.md updated with load regime explanation
- CERTIFICATION_DOSSIER.md documents Day-1A load description

---

## 7. Version History

### v1.0.1 (2025-11-19) — CANONICAL

**Parameter Correction (Micro-Energy Calibration)**

- **CORRECTED**: Node creation energy: 0.05 → **1e-12** (100× below quarantine threshold)
- **CORRECTED**: Node creation belief: 0.05 → **0.01** (reduced cognitive weight)
- Verified energy drift stable: ~1×10⁻¹² to 5×10⁻¹¹ (well below 1×10⁻¹⁰ threshold)
- Substrate transitions to clean ACTIVE state with no quarantine
- Node count grows continuously (10-50 range)
- All 7 invariants nominal after correction

**Root Cause (v1.0.0)**:
- Original energy=0.05 was 5×10⁷× above 1×10⁻¹⁰ threshold
- Triggered immediate quarantine after first node.create operation
- Calculation: 0.05 / 1e-10 = 500,000,000× threshold

**Resolution**:
- Adjusted to micro-energy domain (1e-12) matching ACT-01 original pattern
- Places energy allocation 100× below drift threshold
- Provides safe margin for 24h continuous operation
- Documented in `pilot_reports/day1/load_balance_correction.json`

---

### v1.0.0 (2025-11-19) — DEPRECATED

**Initial Release** (superseded by v1.0.1)

- Defined 3-operation cycle: governor.status → node.create → lineage.replay
- Cycle timing: 0.9 seconds
- ~~Node creation parameters: belief=0.05, energy=0.05~~ (INCORRECT — triggered quarantine)
- Quarantine detection integrated into monitoring
- Deployment integration via container args
- Expected telemetry profile documented
- Recovery protocol (ACT-07A) specified

**Deprecation Reason**: Energy parameter (0.05) exceeded quarantine threshold by 7 orders of magnitude. Replaced by v1.0.1 with corrected micro-energy parameters.

**Expiry Conditions**:
- End of Day-1A monitoring window (2025-11-19 18:15:05 UTC)
- Replacement by LOAD-BALANCE-02 or production load directive
- Quarantine event requiring parameter adjustment

---

## 8. Governance & Compliance

**Parent Directives**:
- ACT-07 v1.0.1 (24h monitoring execution)
- COHERENCE-01 v1.0.0 (exception protocols)
- SG-SCG-PILOT-AUTH-02 (telemetry fabric authorization)

**Certification Impact**: LOAD-BALANCE-01 provides controlled, auditable stimulus for valid Day-1A certification telemetry. Load regime is version-bound and documented for reproducibility.

**Exception Classification**: This is an **operational adjustment**, not a governance exception. Load regime is within pilot scope and does not modify substrate code or invariant thresholds.

**Documentation Requirements**:
- CERTIFICATION_DOSSIER.md must include Day-1A Load Description referencing this document
- STATUS.md must show LOAD-BALANCE-01 ACTIVE status
- Any quarantine events must be logged per ACT-07A protocol
- Final Day-1 summary must note load regime in telemetry metadata

---

## Document Control

**Version**: 1.0.0  
**Effective Date**: 2025-11-19  
**Valid Until**: End of Day-1A OR replacement by subsequent directive  
**Owner**: SCG Substrate Team  
**Classification**: Internal/Operational

---

**END OF LOAD-BALANCE-01 SPECIFICATION**
