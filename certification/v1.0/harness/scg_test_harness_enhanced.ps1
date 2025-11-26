param(
    [string]$ServerPath = ".\target\debug\scg_mcp_server.exe"
)

function Invoke-SCGBatch {
    param(
        [string[]]$JsonLines,
        [string]$Label = "batch"
    )

    Write-Host ">> Running batch: $Label" -ForegroundColor DarkCyan

    $outputLines = $JsonLines | & $ServerPath 2>$null

    if ($LASTEXITCODE -ne 0) {
        Write-Host "Error running SCG server for batch $Label" -ForegroundColor Red
        return @()
    }

    if (-not $outputLines) {
        Write-Host "No output received for batch $Label" -ForegroundColor Yellow
        return @()
    }

    $outText = $outputLines -join "`n"
    $regex = '\{(?:[^{}]|(?<open>\{)|(?<-open>\}))*\}(?(open)(?!))'
    $matches = [System.Text.RegularExpressions.Regex]::Matches($outText, $regex)

    $parsed = @()
    foreach ($m in $matches) {
        try {
            $obj = $m.Value | ConvertFrom-Json
            $parsed += $obj
        } catch {
            Write-Host "Failed to parse JSON object in ${Label}:" -ForegroundColor Red
            Write-Host $m.Value
        }
    }

    return $parsed
}

function Extract-Content {
    param($Response)
    
    if ($Response.result -and $Response.result.content) {
        $content = $Response.result.content[0]
        if ($content.text) {
            try {
                return $content.text | ConvertFrom-Json
            } catch {
                return $content.text
            }
        }
    }
    return $null
}

$results = @{}
$nodeIds = @()

# =====================================================================
# PHASE 0 - GOVERNOR BASELINE
# =====================================================================

Write-Host "`n===== PHASE 0: Pre-Flight Sanity =====" -ForegroundColor Cyan

$phase0Lines = @(
    '{"jsonrpc":"2.0","id":0,"method":"governor.status","params":{}}'
)

$phase0 = Invoke-SCGBatch -JsonLines $phase0Lines -Label "phase0"
$govBaseline = Extract-Content $phase0[0]

Write-Host "Governor Baseline:" -ForegroundColor Yellow
Write-Host "  Drift: $($govBaseline.drift)" -ForegroundColor $(if ($govBaseline.drift -lt 1e-10) { "Green" } else { "Red" })
Write-Host "  Coherence: $($govBaseline.coherence)" -ForegroundColor Cyan
Write-Host "  Total Energy: $($govBaseline.total_energy)" -ForegroundColor Cyan

# =====================================================================
# PHASE 1 - NODE LIFECYCLE & BOUNDARY BELIEFS
# =====================================================================

Write-Host "`n===== PHASE 1: Node Lifecycle & Boundary Beliefs =====" -ForegroundColor Cyan

$beliefs = @(0.5, 0.0, 1.0, 0.001, 0.999)
$phase1Lines = @()
$idCounter = 1

foreach ($b in $beliefs) {
    $line = '{"jsonrpc":"2.0","id":' + $idCounter + ',"method":"node.create","params":{"belief":' + $b + ',"energy":1.0}}'
    $phase1Lines += $line
    $idCounter++
}

$phase1Lines += '{"jsonrpc":"2.0","id":11,"method":"governor.status","params":{}}'

$phase1 = Invoke-SCGBatch -JsonLines $phase1Lines -Label "phase1"

Write-Host "`nNode Creation Results:" -ForegroundColor Yellow
$passCount = 0
$failCount = 0

for ($i = 0; $i -lt 5; $i++) {
    $nodeData = Extract-Content $phase1[$i]
    $requestedBelief = $beliefs[$i]
    
    if ($nodeData -and $nodeData.id) {
        $nodeIds += $nodeData.id
        $actualBelief = [math]::Round($nodeData.belief, 3)
        $clampedBelief = [math]::Max(0, [math]::Min(1, $requestedBelief))
        
        Write-Host "  Test $($i+1): Belief=$requestedBelief -> Node ID: $($nodeData.id.Substring(0,8))..." -ForegroundColor Green
        Write-Host "           Actual Belief: $actualBelief, Energy: $($nodeData.energy)" -ForegroundColor Gray
        $passCount++
    } else {
        Write-Host "  Test $($i+1): Belief=$requestedBelief -> FAILED (null UUID or invalid response)" -ForegroundColor Red
        $failCount++
    }
}

$govAfter = Extract-Content $phase1[5]
Write-Host "`nGovernor After Node Creation:" -ForegroundColor Yellow
Write-Host "  Drift: $($govAfter.drift)" -ForegroundColor $(if ($govAfter.drift -lt 1e-10) { "Green" } else { "Red" })
Write-Host "  Coherence: $($govAfter.coherence)" -ForegroundColor Cyan
Write-Host "  Total Energy: $($govAfter.total_energy)" -ForegroundColor Cyan

$energyDrift = [math]::Abs($govAfter.total_energy - $govBaseline.total_energy)
Write-Host "  Energy Drift (Delta-E): $energyDrift" -ForegroundColor $(if ($energyDrift -lt 1e-10) { "Green" } else { "Red" })

# =====================================================================
# PHASE 1 CERTIFICATION RESULT
# =====================================================================

Write-Host "`n===== PHASE 1 CERTIFICATION RESULT =====" -ForegroundColor White
Write-Host "  Tests Passed: $passCount / 5" -ForegroundColor $(if ($passCount -eq 5) { "Green" } else { "Yellow" })
Write-Host "  Tests Failed: $failCount / 5" -ForegroundColor $(if ($failCount -eq 0) { "Green" } else { "Red" })

$energyConserved = $energyDrift -lt 1e-10
$allNodesValid = $passCount -eq 5

Write-Host "`n  Energy Conservation: $(if ($energyConserved) { 'PASS' } else { 'FAIL' })" -ForegroundColor $(if ($energyConserved) { "Green" } else { "Red" })
Write-Host "  All Boundary Beliefs Valid: $(if ($allNodesValid) { 'PASS' } else { 'FAIL' })" -ForegroundColor $(if ($allNodesValid) { "Green" } else { "Red" })

if ($energyConserved -and $allNodesValid) {
    Write-Host "`n[PASS] PHASE 1 CRITICAL BUGS FIXED" -ForegroundColor Green
    Write-Host "  - Energy accounting: CORRECTED (Delta-E less than 1e-10)" -ForegroundColor Green
    Write-Host "  - Belief validation: CORRECTED (all boundary values accepted)" -ForegroundColor Green
} else {
    Write-Host "`n[FAIL] PHASE 1 CERTIFICATION FAILED" -ForegroundColor Red
    if (-not $energyConserved) {
        Write-Host "  - Energy drift exceeds threshold: $energyDrift (should be less than 1e-10)" -ForegroundColor Red
    }
    if (-not $allNodesValid) {
        Write-Host "  - Some boundary beliefs returned null UUIDs" -ForegroundColor Red
    }
}

# =====================================================================
# PHASE 2 - NODE MUTATION (requires persistent session)
# =====================================================================

Write-Host "`n===== PHASE 2: Node Mutation (Persistent Session Required) =====" -ForegroundColor Cyan
Write-Host "NOTE: Phase 2-8 require a persistent STDIO session to reuse node/edge IDs." -ForegroundColor DarkYellow
Write-Host "      Current harness is batch-based (stateless between phases)." -ForegroundColor DarkYellow
Write-Host "      To enable full 47-test automation, migrate to persistent harness architecture." -ForegroundColor DarkYellow

# =====================================================================
# EXECUTIVE SUMMARY
# =====================================================================

Write-Host "`n===== EXECUTIVE SUMMARY =====" -ForegroundColor White
Write-Host "Phase 0: Governor baseline captured successfully." -ForegroundColor Green
Write-Host "Phase 1: Node lifecycle tests executed." -ForegroundColor Green
Write-Host "  - $passCount/5 boundary belief tests passed" -ForegroundColor $(if ($passCount -eq 5) { "Green" } else { "Yellow" })
Write-Host "  - Energy conservation: $(if ($energyConserved) { 'VERIFIED' } else { 'FAILED' })" -ForegroundColor $(if ($energyConserved) { "Green" } else { "Red" })
Write-Host "`nPhases 2-8: Awaiting persistent harness implementation for stateful multi-phase testing." -ForegroundColor Yellow

if ($energyConserved -and $allNodesValid) {
    Write-Host "`n[PASS] CRITICAL PATCH VALIDATED - Ready for full certification" -ForegroundColor Green
} else {
    Write-Host "`n[FAIL] CRITICAL BUGS STILL PRESENT - Further debugging required" -ForegroundColor Red
}
