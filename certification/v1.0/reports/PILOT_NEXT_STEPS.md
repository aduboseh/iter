# SCG-PILOT-01 Next Steps — Days 1-7 Execution Plan

**Directive**: SG-SCG-PILOT-ACT-03 v1.0.0  
**Status**: Day-0 preparation complete, Day-1 authorized  
**Current Phase**: Baseline collection & monitoring enhancement

---

## Priority 0 — Critical Path (Required for Day-1)

### Task 1: Upgrade Multiline JSON Parser

**Issue**: governor.status and lineage.replay responses span multiple lines in logs

**Current Behavior**:
```
{"jsonrpc":"2.0","result":{"content":{"text":"{\n  \"energy_drift\": 3.3e-11,\n  \"coherence\": 1.0,\n  \"node_count\": 34,\n  \"edge_count\": 0\n}","type":"text"}]},
```

**Required Fix**: Update `monitor-invariants.ps1` to:
1. Extract nested JSON from `result.content0].text` field
2. Handle escaped newlines (`\n`) in JSON strings
3. Parse complete multi-line structures

**Implementation**:
```powershell
# Extract JSON from MCP response wrapper
$jsonContent = $logs | Select-String -Pattern '"text":"(\{^}]+\})"' -AllMatches | 
    ForEach-Object {
        $_.Matches0].Groups1].Value -replace '\\n', '' -replace '\s+', ' '
    } | ConvertFrom-Json
```

**Done-When**:
- Monitoring script correctly extracts energy_drift, coherence, node_count
- CSV logs contain real values (not 0.0 placeholders)
- Parser handles both single-line and multi-line JSON responses

---

### Task 2: Collect 6-12 Hour Baseline Window

**Objective**: Establish Day-0 baseline with statistically valid measurements

**Duration**: 6-12 hours continuous runtime (minimum for certification)

**Measurements Required**:
- Energy drift: min/max/mean/stddev
- Coherence: stability over time
- Node/edge growth rate
- Request success rate
- Quarantine event count (must remain 0)

**Execution**:
```powershell
# Start long-running monitoring session
.\deployment\pilot\monitor-invariants.ps1 -IntervalSeconds 60 -OutputPath ".\pilot_reports\day0"
```

**Done-When**:
- 360+ data points collected (6 hours at 60s intervals)
- No restarts or quarantine events during collection
- `day0_summary_final.json` generated with actual values

---

### Task 3: Validate Time Synchronization (≤50ms)

**Directive Requirement**: ACT-02 §5 — NTP/PTP skew ≤50ms across all cluster nodes

**Current State**: Not validated (requires node-level access)

**Option A: DaemonSet Approach** (Preferred)

Create `deployment/pilot/time-sync-checker.yaml`:
```yaml
apiVersion: apps/v1
kind: DaemonSet
metadata:
  name: time-sync-checker
  namespace: scg-pilot-01
spec:
  selector:
    matchLabels:
      app: time-sync-checker
  template:
    metadata:
      labels:
        app: time-sync-checker
    spec:
      hostNetwork: true
      containers:
      - name: checker
        image: debian:bookworm-slim
        command: "/bin/sh", "-c"]
        args:
        - |
          apt-get update && apt-get install -y chrony ntpdate
          while true; do
            echo "$(date -Iseconds)] Node: $(hostname)"
            chronyc tracking || ntpdate -q time.windows.com
            sleep 300
          done
```

**Option B: Manual Validation** (If cluster SSH available)

```bash
# On each node
timedatectl status
chronyc tracking | grep "System time"
```

**Done-When**:
- All 3 cluster nodes validated
- Maximum skew documented (must be ≤50ms)
- Results appended to DEPLOYMENT_STATUS.md
- Results added to CERTIFICATION_DOSSIER.md Day-1 section

---

## Priority 1 — Required for Certification

### Task 4: Daily 24h Invariant Aggregation

**Objective**: Generate daily summaries for CERTIFICATION_DOSSIER.md

**Script**: `deployment/pilot/daily-report.ps1` (to be created)

**Inputs**:
- Previous 24h of CSV logs from monitor-invariants.ps1
- Substrate pod logs for quarantine events
- Resource usage metrics

**Outputs**:
```json
{
  "day": 1,
  "date": "2025-11-18",
  "energy_drift": {
    "min": "1.2e-11",
    "max": "4.5e-11",
    "mean": "3.1e-11",
    "samples": 1440
  },
  "coherence": {
    "min": "1.0",
    "mean": "1.0",
    "violations": 0
  },
  "quarantine_events": 0,
  "governor_corrections": 0,
  "replay_episodes": {
    "executed": 3,
    "variance": "0.0"
  },
  "global_ledger_hash": "TBD",
  "certification_status": "PASS"
}
```

**Automation**:
- Run daily at 00:00 UTC via Task Scheduler or cron
- Append to `pilot_reports/day{N}/daily_summary.json`
- Update CERTIFICATION_DOSSIER.md automatically

---

### Task 5: Replay Episode Automation

**Directive Requirement**: ACT-02 §6.2 — 250-cycle deterministic replay across 3 environments

**Environments**:
1. **Local**: Developer machine (Windows/PowerShell)
2. **Container**: Docker Desktop
3. **Cluster**: scg-pilot-01 namespace

**Implementation**: `deployment/pilot/replay-episode.ps1`

```powershell
param(
    int]$Cycles = 250,
    string]$Environment = "cluster"
)

# Execute replay and capture hash
$replayHash = kubectl exec -n scg-pilot-01 deployment/scg-mcp -- \
    /app/scg_mcp_server replay --cycles $Cycles --output-hash

# Compare with expected hash
$expectedHash = Get-Content ".\pilot_reports\replay_baseline_hash.txt"
$variance = Compare-Hash $replayHash $expectedHash

if ($variance -gt 1e-10) {
    Write-Error "Replay variance exceeded: $variance"
    exit 1
}
```

**Done-When**:
- Script executes 250-cycle replay successfully
- Hash output captured
- Variance computed across environments
- ε ≤ 1×10⁻¹⁰ verified

**Automation**:
Create `deployment/pilot/replay-cronjob.yaml`:
```yaml
apiVersion: batch/v1
kind: CronJob
metadata:
  name: daily-replay-validation
  namespace: scg-pilot-01
spec:
  schedule: "0 0 * * *"  # Daily at midnight UTC
  jobTemplate:
    spec:
      template:
        spec:
          containers:
          - name: replay
            image: scgpilotacr.azurecr.io/scg-mcp:v1.0.0-substrate
            command: "/app/scg_mcp_server"]
            args: "replay", "--cycles", "250", "--output-hash"]
          restartPolicy: OnFailure
```

---

### Task 6: Global Ledger Hash Validation

**Objective**: Verify |H_global − H_expected| = 0 for each day

**Process**:
1. Extract all lineage.replay checksums for 24h period
2. Concatenate in chronological order
3. Compute SHA256 of concatenated checksums
4. Compare with expected value (from initial baseline)

**Script**: `deployment/pilot/validate-ledger.ps1`

```powershell
$lineageChecksums = kubectl logs -n scg-pilot-01 -l app=scg-mcp --tail=10000 | 
    Select-String -Pattern '"checksum":\s*"(a-f0-9]+)"' -AllMatches |
    ForEach-Object { $_.Matches.Groups1].Value }

$concatenated = $lineageChecksums -join ""
$globalHash = (Get-FileHash -InputStream (System.IO.MemoryStream]::new(Text.Encoding]::UTF8.GetBytes($concatenated))) -Algorithm SHA256).Hash

Write-Output "Global Ledger Hash: $globalHash"
```

**Done-When**:
- Daily global hash computed
- Compared with Day-0 baseline
- Variance logged (must be 0 for certification)

---

## Priority 2 — Operational Excellence

### Task 7: Automated Violation Escalation

**Directive Requirement**: ACT-02 §7 — Auto-escalate violations

**Triggers**:
| Condition | Action |
|-----------|--------|
| ΔE > 5e-11 | Immediate alert |
| Coherence < 0.97 (2+ samples) | Alert + governor correction |
| ESV_valid_ratio < 1.0 | Hard quarantine + freeze |
| Missing telemetry > 3 min | Alert |
| Global hash mismatch | Critical stop |
| 2 restarts in 10 min | Restart loop alert |

**Implementation**: `deployment/pilot/violation-monitor.ps1`

```powershell
# Monitor for violations
$energyDrift = double]$invariants.energy_drift
if ($energyDrift -gt 5e-11) {
    Send-Alert -Level "IMMEDIATE" -Message "Energy drift exceeded: $energyDrift"
    Write-Log "VIOLATION: Energy drift > 5e-11"
}

# Check quarantine condition
if ($invariants.esv_valid_ratio -lt 1.0) {
    Write-Log "CRITICAL: ESV validation failed - initiating hard quarantine"
    kubectl scale deployment/scg-mcp --replicas=0 -n scg-pilot-01
    Send-Alert -Level "CRITICAL" -Message "Hard quarantine triggered"
}
```

---

### Task 8: Real-Time Dashboard (Optional)

**Objective**: Visualize invariants in real-time

**Options**:
1. **Grafana** + Prometheus (query OTEL metrics endpoint)
2. **PowerShell Console** (live updating text UI)
3. **Simple HTML** (refresh every 60s from CSV)

**Benefits**:
- Immediate visibility into substrate health
- Trend analysis for drift/coherence
- Early warning for degradation

---

## Documentation Tasks

### Update CERTIFICATION_DOSSIER.md

**Sections to populate**:
- Day 0 Summary (once baseline complete)
- Day 1-7 Daily Summaries
- Invariant violation log (should remain empty)
- Replay episode results
- Global ledger hash verification
- Final attestation

**Automation**: Daily append via cron/Task Scheduler

---

### Update DEPLOYMENT_STATUS.md

**Add**:
- Time sync validation results
- ACT-03 completion status
- Parser upgrade completion
- Baseline collection progress

---

### Create Final Report Template

**File**: `SCG_PILOT_01_FINAL_REPORT.md`

**Sections**:
1. Executive Summary
2. 7-Day Invariant Summary
3. Quarantine Event Log (expected: empty)
4. Replay Episode Results
5. Ledger Hash Verification
6. Certification Recommendation
7. Lessons Learned
8. Post-Pilot Enhancements

---

## Timeline & Milestones

| Day | Milestone | Tasks |
|-----|-----------|-------|
| **Day 0** |  Complete | Infrastructure, monitoring, baseline prep |
| **Day 1** | Parser upgrade, time sync, baseline collection | Tasks 1-3 |
| **Day 2** | Daily aggregation, first replay episode | Tasks 4-5 |
| **Day 3** | Ledger validation, violation monitoring | Tasks 6-7 |
| **Day 4-6** | Continuous monitoring, daily reports | Automation running |
| **Day 7** | Final attestation, dossier completion | Final report |
| **Day 8** | Post-pilot review, certification decision | Sign-off |

---

## Risk Registry

| Risk | Impact | Mitigation | Status |
|------|--------|----------|--------|
| Parser failure blocks baseline | HIGH | Fallback to single-line JSON only | ⏳ Active |
| Cluster node time drift > 50ms | HIGH | Deploy time-sync DaemonSet | ⏳ Pending |
| Substrate restart during baseline | MEDIUM | Extended collection window | ⏳ Monitoring |
| Telemetry gaps from log rotation | MEDIUM | Increase log retention | ⏳ Config needed |
| Replay variance exceeds ε | HIGH | Three-environment cross-validation | ⏳ Scheduled |
| Quarantine event during pilot | CRITICAL | §3.1 protocol + incident report | ⏳ Ready |

---

## Success Criteria Checklist

**Day-1 Authorization** ( Complete):
-  Infrastructure stable
-  Quarantine cleared
-  Monitoring operational
- ⏳ Baseline in progress

**7-Day Certification** (⏳ In Progress):
- ⏳ ΔE ≤ 1×10⁻¹⁰ (continuous)
- ⏳ ε ≤ 1×10⁻¹⁰ (replay)
- ⏳ C(t) ≥ 0.97 (continuous)
- ⏳ ESV_valid_ratio = 1.0 (continuous)
-  Zero quarantine events (Day 0)
- ⏳ Governor convergence verified
- ⏳ Ledger hash integrity (7 days)

---

## Contact & Escalation

**Directive Authority**: Armonti Du-Bose-Hill (Substrate Sovereign)  
**Executor**: Warp AI Terminal  
**Repository**: github.com/aduboseh/scg-mcp  
**Pilot Namespace**: scg-pilot-01  
**Cluster**: haltra-perf-aks

**Escalation Path**:
1. Technical issues → Check DEPLOYMENT_STATUS.md + logs
2. Invariant violations → Execute §7 escalation protocol
3. Directive interpretation → Review ACT-02 §1-9
4. Emergency quarantine → §3.1 controlled restart protocol

---

**Status**: Day-0 complete, Day-1 tasks documented and prioritized. Execute tasks in order for successful 7-day certification.
