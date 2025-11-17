# SCG-PILOT-01 Time Sync Proxy Validation
# Directive: SG-SCG-PILOT-COHERENCE-01 v1.0.0 §2.2
#
# Non-privileged heartbeat-based temporal coherence check

param(
    [string]$OutputPath = ".\pilot_reports\day1"
)

Write-Host "================================================"
Write-Host "SCG-PILOT-01 Time Sync Proxy Validation"
Write-Host "Directive: COHERENCE-01 §2.2"
Write-Host "================================================"
Write-Host ""

Write-Host "Method: Kubernetes node heartbeat delta (non-privileged)" -ForegroundColor Cyan
Write-Host "Threshold: Δt_max ≤ 5 seconds"
Write-Host ""

# Extract node heartbeat times
Write-Host "Extracting node heartbeat timestamps..." -ForegroundColor Cyan
$heartbeatsRaw = kubectl get nodes -o jsonpath='{range .items[*]}{.metadata.name}{"\t"}{.status.conditions[?(@.type=="Ready")].lastHeartbeatTime}{"\n"}{end}'

if (-not $heartbeatsRaw) {
    Write-Host "ERROR: Failed to retrieve node heartbeats" -ForegroundColor Red
    exit 1
}

# Parse heartbeats
$nodes = @()
foreach ($line in ($heartbeatsRaw -split "`n" | Where-Object { $_ -ne "" })) {
    $parts = $line -split "`t"
    if ($parts.Count -eq 2) {
        $nodes += @{
            name = $parts[0]
            heartbeat = [DateTime]::Parse($parts[1])
        }
        Write-Host "  $($parts[0]): $($parts[1])"
    }
}

Write-Host ""
Write-Host "Nodes detected: $($nodes.Count)" -ForegroundColor Green
Write-Host ""

# Compute delta
$timestamps = $nodes | ForEach-Object { $_.heartbeat }
$minTime = ($timestamps | Measure-Object -Minimum).Minimum
$maxTime = ($timestamps | Measure-Object -Maximum).Maximum
$deltaSeconds = ($maxTime - $minTime).TotalSeconds

Write-Host "Temporal Coherence Analysis:" -ForegroundColor Cyan
Write-Host "  Earliest heartbeat: $minTime"
Write-Host "  Latest heartbeat:   $maxTime"
Write-Host "  Delta (Δt_max):     $deltaSeconds seconds"
Write-Host ""

# Validate threshold
$threshold = 5.0
$status = if ($deltaSeconds -le $threshold) { "PASS_PROXY" } else { "FAIL" }
$statusColor = if ($status -eq "PASS_PROXY") { "Green" } else { "Red" }

Write-Host "Threshold: ≤ $threshold seconds" -ForegroundColor Cyan
Write-Host "Status: $status" -ForegroundColor $statusColor
Write-Host ""

if ($status -eq "PASS_PROXY") {
    Write-Host "✅ Cluster heartbeat temporally coherent (proxy)" -ForegroundColor Green
    Write-Host "   Note: This is a COARSE check (5s vs canonical 50ms)" -ForegroundColor Yellow
    Write-Host "   Primary validation: Azure NTP SLA" -ForegroundColor Yellow
} else {
    Write-Host "❌ Cluster heartbeat shows temporal incoherence" -ForegroundColor Red
    Write-Host "   Delta exceeds $threshold second threshold" -ForegroundColor Red
}
Write-Host ""

# Generate output JSON
$result = @{
    method = "k8s_heartbeat_proxy"
    azure_ntp_sla = "TRUSTED"
    heartbeat_max_delta_seconds = [math]::Round($deltaSeconds, 2)
    heartbeat_threshold_seconds = $threshold
    node_count = $nodes.Count
    nodes = $nodes | ForEach-Object {
        @{
            name = $_.name
            heartbeat = $_.heartbeat.ToString("yyyy-MM-ddTHH:mm:ss.fffZ")
        }
    }
    status = if ($status -eq "PASS_PROXY") { "PASS_WITH_EXCEPTION" } else { "FAIL" }
    exception = "SCG_PILOT_TIME_SYNC_EXCEPTION_v1.0.0"
    note = "Canonical ≤50ms sync externally assured via Azure SLA; heartbeat used as coarse cross-check."
    timestamp = (Get-Date -Format "yyyy-MM-ddTHH:mm:ssZ")
}

$outputFile = Join-Path $OutputPath "time_sync.json"
$result | ConvertTo-Json -Depth 10 | Out-File -FilePath $outputFile -Encoding UTF8

Write-Host "Results saved to: $outputFile" -ForegroundColor Cyan
Write-Host ""

Write-Host "================================================"
Write-Host "Time Sync Proxy Validation Complete"
Write-Host "================================================"
Write-Host ""

return $result
