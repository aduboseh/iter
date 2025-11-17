# SCG-PILOT-01 Launch Guide

**Objective**: 7-day field validation of v1.0.0-substrate  
**Target**: Zero violations across all invariants  
**Start Date**: TBD  
**Status**: READY TO LAUNCH

---

## Pre-Launch Checklist

- [ ] Substrate frozen at commit `21dd6b5`
- [ ] All 46 tests passing
- [ ] Telemetry systems operational
- [ ] Fault domains tested
- [ ] Lineage replay validated
- [ ] Monitoring infrastructure ready

---

## Launch Configuration

```powershell
# Set environment variables
$env:SCG_CLUSTER_ID = "SCG-PILOT-01"
$env:SCG_LOG_LEVEL = "info"
$env:SCG_PILOT_DURATION_DAYS = "7"
$env:SCG_RPS_TARGET = "7500"
$env:RUST_BACKTRACE = "1"

# Create telemetry directories
New-Item -ItemType Directory -Path ".\telemetry\pilot" -Force
New-Item -ItemType Directory -Path ".\snapshots\pilot" -Force
New-Item -ItemType Directory -Path ".\audit\pilot" -Force
```

---

## Launch Commands

### Option 1: Direct Execution with Logging

```powershell
# Launch with full telemetry capture
cargo run --release 2>&1 | Tee-Object -FilePath .\telemetry\pilot\scg_pilot_01_full.log

# In separate terminals, run monitors:

# Monitor 1: Telemetry violations
Get-Content .\telemetry\pilot\scg_pilot_01_full.log -Wait | Select-String -Pattern "VIOLATION|CRITICAL"

# Monitor 2: Governor corrections
Get-Content .\telemetry\pilot\scg_pilot_01_full.log -Wait | Select-String -Pattern "\[GOVERNOR_CORRECTION\]" | Tee-Object -FilePath .\audit\pilot\governor_corrections.log

# Monitor 3: Shard finalizations
Get-Content .\telemetry\pilot\scg_pilot_01_full.log -Wait | Select-String -Pattern "\[SHARD_FINALIZED\]" | Tee-Object -FilePath .\audit\pilot\shard_finalizations.log

# Monitor 4: Quarantine events (should be zero)
Get-Content .\telemetry\pilot\scg_pilot_01_full.log -Wait | Select-String -Pattern "\[QUARANTINE\]" | Tee-Object -FilePath .\audit\pilot\quarantine_events.log
```

### Option 2: Background Service with Rotation

```powershell
# Launch as background job with daily log rotation
$job = Start-Job -ScriptBlock {
    $env:SCG_CLUSTER_ID = "SCG-PILOT-01"
    cargo run --release 2>&1 | ForEach-Object {
        $timestamp = Get-Date -Format "yyyy-MM-dd"
        Add-Content -Path ".\telemetry\pilot\scg_pilot_$timestamp.log" -Value $_
    }
}

# Monitor job
Receive-Job -Job $job -Keep

# Stop after 7 days
Stop-Job -Job $job
Remove-Job -Job $job
```

---

## Success Criteria (All Must Hold for 7 Days)

### 1. Energy Conservation: ΔE ≤ 1×10⁻¹⁰

**Monitor**:
```powershell
Get-Content .\telemetry\pilot\scg_pilot_01_full.log | Select-String '"energy_drift":' | ForEach-Object {
    if ($_ -match '"energy_drift":\s*([\d.e\-+]+)') {
        $drift = [double]$matches[1]
        if ($drift -gt 1e-10) {
            Write-Host "VIOLATION: Energy drift $drift > 1e-10" -ForegroundColor Red
        }
    }
}
```

**Expected**: Zero violations

---

### 2. Replay Variance: ε ≤ 1×10⁻¹⁰

**Test Protocol**:
```powershell
# Execute replay episode in three environments
# Local
cargo test --test integration_validation test_lineage_tracking_deterministic

# Docker (requires Dockerfile)
docker run --rm scg-mcp:v1.0.0-substrate cargo test --test integration_validation test_lineage_tracking_deterministic

# Kubernetes (requires deployment manifest)
kubectl run scg-test --image=scg-mcp:v1.0.0-substrate --restart=Never --command -- cargo test --test integration_validation test_lineage_tracking_deterministic
```

**Expected**: All three environments produce identical lineage hashes

---

### 3. Coherence: C(t) ≥ 0.97

**Monitor**:
```powershell
Get-Content .\telemetry\pilot\scg_pilot_01_full.log | Select-String '"coherence":' | ForEach-Object {
    if ($_ -match '"coherence":\s*([\d.e\-+]+)') {
        $coherence = [double]$matches[1]
        if ($coherence -lt 0.97) {
            Write-Host "VIOLATION: Coherence $coherence < 0.97" -ForegroundColor Red
        }
    }
}
```

**Expected**: Zero violations

---

### 4. ESV Validation: 100% Pass Rate

**Monitor**:
```powershell
Get-Content .\telemetry\pilot\scg_pilot_01_full.log | Select-String '"esv_valid_ratio":' | ForEach-Object {
    if ($_ -match '"esv_valid_ratio":\s*([\d.e\-+]+)') {
        $ratio = [double]$matches[1]
        if ($ratio -lt 1.0) {
            Write-Host "VIOLATION: ESV ratio $ratio < 1.0" -ForegroundColor Red
        }
    }
}
```

**Expected**: Zero violations

---

### 5. Zero Quarantine Events

**Monitor**:
```powershell
$quarantine_count = (Get-Content .\audit\pilot\quarantine_events.log | Measure-Object).Count
if ($quarantine_count -gt 0) {
    Write-Host "FAILURE: $quarantine_count quarantine events detected" -ForegroundColor Red
} else {
    Write-Host "SUCCESS: Zero quarantine events" -ForegroundColor Green
}
```

**Expected**: Zero events

---

### 6. Governor Convergence: post_delta < pre_delta

**Monitor**:
```powershell
Get-Content .\audit\pilot\governor_corrections.log | ForEach-Object {
    $json = $_ | ConvertFrom-Json -ErrorAction SilentlyContinue
    if ($json) {
        if ($json.post_delta -ge $json.pre_delta) {
            Write-Host "WARNING: Governor correction failed to converge" -ForegroundColor Yellow
            Write-Host "  Pre: $($json.pre_delta), Post: $($json.post_delta)"
        }
    }
}
```

**Expected**: All corrections converge (post < pre)

---

### 7. Shard Global Hash Integrity

**Monitor**:
```powershell
# Daily shard reconstruction validation
Get-Content .\audit\pilot\shard_finalizations.log | Select-String "Shard.*hash:" | ForEach-Object {
    Write-Host $_ -ForegroundColor Cyan
}
```

**Expected**: All shard hashes computed successfully, no reconstruction failures

---

## Daily Health Report

Run this script daily to generate a health report:

```powershell
# daily_health_check.ps1
param(
    [int]$Day
)

$report = @"
SCG-PILOT-01 Daily Health Report - Day $Day
Generated: $(Get-Date)

=== Energy Conservation ===
"@

$drift_violations = Get-Content .\telemetry\pilot\scg_pilot_*.log | Select-String '"energy_drift":' | Where-Object {
    $_ -match '"energy_drift":\s*([\d.e\-+]+)' -and [double]$matches[1] -gt 1e-10
}
$report += "`nViolations: $($drift_violations.Count)"

$report += "`n`n=== Coherence ===`n"
$coherence_violations = Get-Content .\telemetry\pilot\scg_pilot_*.log | Select-String '"coherence":' | Where-Object {
    $_ -match '"coherence":\s*([\d.e\-+]+)' -and [double]$matches[1] -lt 0.97
}
$report += "Violations: $($coherence_violations.Count)"

$report += "`n`n=== ESV Validation ===`n"
$esv_violations = Get-Content .\telemetry\pilot\scg_pilot_*.log | Select-String '"esv_valid_ratio":' | Where-Object {
    $_ -match '"esv_valid_ratio":\s*([\d.e\-+]+)' -and [double]$matches[1] -lt 1.0
}
$report += "Violations: $($esv_violations.Count)"

$report += "`n`n=== Quarantine Events ===`n"
$quarantine_count = (Get-Content .\audit\pilot\quarantine_events.log -ErrorAction SilentlyContinue | Measure-Object).Count
$report += "Events: $quarantine_count"

$report += "`n`n=== Governor Corrections ===`n"
$corrections = (Get-Content .\audit\pilot\governor_corrections.log -ErrorAction SilentlyContinue | Measure-Object).Count
$report += "Attempts: $corrections"

$report += "`n`n=== Shard Finalizations ===`n"
$shards = (Get-Content .\audit\pilot\shard_finalizations.log -ErrorAction SilentlyContinue | Measure-Object).Count
$report += "Shards Finalized: $shards"

$report += "`n`n=== Status ===`n"
if ($drift_violations.Count -eq 0 -and $coherence_violations.Count -eq 0 -and $esv_violations.Count -eq 0 -and $quarantine_count -eq 0) {
    $report += "✅ ALL INVARIANTS HOLDING"
} else {
    $report += "❌ VIOLATIONS DETECTED - INVESTIGATION REQUIRED"
}

Write-Output $report
$report | Out-File ".\audit\pilot\day_${Day}_health_report.txt"
```

---

## Failure Response Protocol

### Micro-Drift Detection (5e-11 < drift < 1e-10)

1. Capture telemetry snapshot
2. Review governor correction logs
3. Check for sustained drift vs. transient spike
4. If sustained, increase snapshot frequency to N=125

### ESV Violation

1. Immediate capture: lineage snapshot, node states, telemetry
2. Full lineage forensics
3. Identify violating operation
4. Root cause analysis required before continuing

### Quarantine Event

1. System is automatically in read-only mode
2. Export quarantine audit report
3. Review fault trace ID
4. Validate rollback to last checkpoint
5. Determine root cause
6. Clear with manual approval token only after remediation

---

## Post-Pilot Actions

After successful 7-day run:

1. **Generate Final Report**
   ```powershell
   .\scripts\generate_pilot_final_report.ps1
   ```

2. **Build Certification Dossier**
   - Collect all telemetry
   - Export shard reconstructions
   - Package replay proofs
   - Generate cryptographic signatures

3. **Tag Release**
   ```powershell
   git tag -a v1.0.0-substrate-certified -m "SCG-PILOT-01 validation complete"
   git push origin v1.0.0-substrate-certified
   ```

4. **Archive Pilot Data**
   ```powershell
   Compress-Archive -Path .\telemetry\pilot,.\audit\pilot,.\snapshots\pilot -DestinationPath scg_pilot_01_archive.zip
   ```

---

## Contact and Escalation

**Pilot Lead**: Haltra AI Team  
**Escalation**: SCG Operations  
**Compliance**: SCG Space Specifications

For critical issues during pilot, halt execution and escalate immediately.

---

**Status**: READY TO LAUNCH  
**Substrate Version**: v1.0.0-substrate  
**Commit**: `21dd6b5`
