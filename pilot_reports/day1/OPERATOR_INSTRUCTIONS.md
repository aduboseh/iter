# ACT-07 Day-1 Operator Instructions

**Directive**: SG-SCG-PILOT-ACT-07 v1.0.1  
**Status**: SECTION 1 ACTIVE (24h monitoring running)  
**T0**: 2025-11-18 18:15:05 UTC  
**Target Completion**: 2025-11-19 18:15:05 UTC

---

## Current State

### ✅ Section 1: 24-Hour Monitoring — ACTIVE

**Job Details:**
- Job ID: 3
- Job Name: SCG-Day1-Monitoring
- Status: Running
- Output: `.\pilot-monitoring\day1\20251118_181506`
- Target: ~1,440 samples (60-second intervals × 24 hours)

**Monitoring Commands:**
```powershell
# RUN THIS — Check job status
Get-Job -Name 'SCG-Day1-Monitoring'

# RUN THIS — View recent monitoring output
Receive-Job -Name 'SCG-Day1-Monitoring' -Keep | Select-Object -Last 50

# RUN THIS — Check sample count
(Import-Csv '.\pilot-monitoring\day1\20251118_181506\invariants.csv').Count

# RUN THIS — View CSV files
Get-ChildItem '.\pilot-monitoring\day1\20251118_181506' -Filter *.csv
```

---

## Monitoring Period (Next 24 Hours)

### Periodic Checks (Every 2-4 hours recommended)

**1. Verify job is still running:**
```powershell
# RUN THIS
$job = Get-Job -Name 'SCG-Day1-Monitoring'
if ($job.State -eq 'Running') {
    Write-Host "✅ Monitoring active" -ForegroundColor Green
} else {
    Write-Host "⚠️  Job state: $($job.State)" -ForegroundColor Yellow
}
```

**2. Check sample accumulation:**
```powershell
# RUN THIS
$sampleCount = (Import-Csv '.\pilot-monitoring\day1\20251118_181506\invariants.csv').Count
$hoursElapsed = ((Get-Date) - (Get-Date "2025-11-18T18:15:05Z")).TotalHours
$expectedSamples = [int]($hoursElapsed * 60)
Write-Host "Samples collected: $sampleCount / ~$expectedSamples expected" -ForegroundColor Cyan
```

**3. Verify substrate health:**
```powershell
# RUN THIS
kubectl get pod -n scg-pilot-01 -l app=scg-mcp -o wide
```

**4. Check for invariant violations:**
```powershell
# RUN THIS
Receive-Job -Name 'SCG-Day1-Monitoring' -Keep | Select-String -Pattern "⚠️|❌|VIOLATION|QUARANTINE" | Select-Object -Last 10
```

---

## Risk Controls (ACT-07 §6)

### If Job Stops/Fails

**Diagnosis:**
```powershell
# RUN THIS
Get-Job -Name 'SCG-Day1-Monitoring' | Format-List *
Receive-Job -Name 'SCG-Day1-Monitoring' -Keep | Select-Object -Last 100
```

**Recovery Protocol:**
1. Document interruption time
2. Calculate samples collected before failure
3. Restart job:
```powershell
# RUN THIS — If job needs restart
Remove-Job -Name 'SCG-Day1-Monitoring' -Force
$jobScript = {
    param($namespace, $interval, $outputPath)
    Set-Location "C:\Users\adubo\scg_mcp_server"
    .\deployment\pilot\monitor-invariants.ps1 -Namespace $namespace -IntervalSeconds $interval -OutputPath $outputPath
}
Start-Job -ScriptBlock $jobScript -ArgumentList "scg-pilot-01", 60, ".\pilot-monitoring\day1" -Name "SCG-Day1-Monitoring"
```
4. Log interruption to `pilot_reports/day1/interruptions.json`:
```powershell
# RUN THIS — Document interruption
@{
    timestamp = (Get-Date -Format "yyyy-MM-ddTHH:mm:ssZ")
    event = "Job interruption"
    samples_before_failure = (Import-Csv '.\pilot-monitoring\day1\20251118_181506\invariants.csv').Count
    restart_time = (Get-Date -Format "yyyy-MM-ddTHH:mm:ssZ")
    action = "Job restarted, Day-1 window extended if needed"
} | ConvertTo-Json | Out-File -FilePath "pilot_reports/day1/interruptions.json" -Encoding UTF8 -Force
```

### If Substrate Pods Restart

**Detection:**
```powershell
# RUN THIS
kubectl get pod -n scg-pilot-01 -l app=scg-mcp -o json | ConvertFrom-Json | `
    ForEach-Object { $_.items[0].status.containerStatuses[0].restartCount }
```

**Action:**
- If restarts > 0: Document in interruptions.json
- Verify telemetry continuity in monitoring output
- Check for gaps in CSV data
- May need to extend Day-1 window

### If Telemetry Shows Nodes=0 (Idle State)

**Verification:**
```powershell
# RUN THIS — Check actual substrate state
kubectl logs -n scg-pilot-01 deployment/scg-mcp --tail=20 | Select-String -Pattern "node_count"
```

**Expected**: Pod logs should show `node_count: 102` (active state)

**If truly idle**:
- Restart substrate pod to restore synthetic load
- Document downtime in interruptions.json
- Extend Day-1 window to compensate

---

## After 24 Hours (T+24h: 2025-11-19 18:15:05 UTC)

### Pre-Aggregation Validation

**1. Verify sample count:**
```powershell
# RUN THIS
$sampleCount = (Import-Csv '.\pilot-monitoring\day1\20251118_181506\invariants.csv').Count
Write-Host "Total samples: $sampleCount" -ForegroundColor Cyan
Write-Host "Target: ~1,440" -ForegroundColor Gray
if ($sampleCount -ge 1400) {
    Write-Host "✅ Sample count sufficient for Day-1 certification" -ForegroundColor Green
} else {
    Write-Host "⚠️  Sample count below target - review interruptions" -ForegroundColor Yellow
}
```

**2. Check for P0 violations:**
```powershell
# RUN THIS
$violations = Receive-Job -Name 'SCG-Day1-Monitoring' -Keep | Select-String -Pattern "❌|QUARANTINE"
if ($violations) {
    Write-Host "⚠️  P0 violations detected - review before proceeding" -ForegroundColor Red
    $violations | Select-Object -Last 20
} else {
    Write-Host "✅ No P0 violations - ready for aggregation" -ForegroundColor Green
}
```

**3. Stop monitoring job:**
```powershell
# RUN THIS
Stop-Job -Name 'SCG-Day1-Monitoring'
Receive-Job -Name 'SCG-Day1-Monitoring' | Out-File -FilePath ".\pilot-monitoring\day1\monitoring_full_output.log" -Encoding UTF8
Remove-Job -Name 'SCG-Day1-Monitoring'
Write-Host "✅ Monitoring job stopped and logged"
```

### Execute ACT-07 Sections 2-5

**Once validation passes, resume directive execution:**

```powershell
# RUN THIS — Section 2: Aggregate telemetry
.\deployment\pilot\aggregate-day1.ps1 `
  -MonitoringPath ".\pilot-monitoring\day1" `
  -OutputJson "pilot_reports/day1/day1_summary.json"
```

Then notify Warp AI to continue with:
- **Section 3**: Update CERTIFICATION_DOSSIER.md with final metrics
- **Section 4**: Commit and tag DAY1_COMPLETE
- **Section 5**: Update STATUS.md with closure

---

## Quick Reference

### Job Control
```powershell
# Check status
Get-Job -Name 'SCG-Day1-Monitoring'

# View output
Receive-Job -Name 'SCG-Day1-Monitoring' -Keep | Select-Object -Last 50

# Stop job
Stop-Job -Name 'SCG-Day1-Monitoring'
Remove-Job -Name 'SCG-Day1-Monitoring'
```

### Sample Tracking
```powershell
# Count samples
(Import-Csv '.\pilot-monitoring\day1\20251118_181506\invariants.csv').Count

# View latest samples
Import-Csv '.\pilot-monitoring\day1\20251118_181506\invariants.csv' | Select-Object -Last 10
```

### Substrate Health
```powershell
# Pod status
kubectl get pod -n scg-pilot-01 -l app=scg-mcp

# Pod logs
kubectl logs -n scg-pilot-01 deployment/scg-mcp --tail=50 | Select-String -Pattern "node_count|energy_drift"
```

---

## Contact & Escalation

**P0 Violations**: Pause monitoring, document in interruptions.json, notify SCG Ops  
**Infrastructure Issues**: Check AKS cluster health, verify network connectivity  
**Governance Questions**: Refer to COHERENCE-01 exception protocols

---

**Tracking Files:**
- `pilot_reports/day1/T0_record.json` (timing and job details)
- `pilot_reports/day1/day1_execution_plan.json` (full execution plan)
- `pilot_reports/day1/interruptions.json` (create if interruptions occur)

**Directive Reference:** SG-SCG-PILOT-ACT-07 v1.0.1  
**Governance:** COHERENCE-01, ACT-06, SG-SCG-PILOT-AUTH-02
