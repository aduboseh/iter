param(
    [string]$HarnessPath = ".\certification\v1.0\harness\scg_certification_harness.ps1"
)

$ErrorActionPreference = "Stop"

Write-Host @"
=====================================================================
     SCG HARNESS PHYSICS LINTER v1.0
     Validating thermodynamic invariant compliance
=====================================================================
"@ -ForegroundColor Cyan

if (-not (Test-Path $HarnessPath)) {
    Write-Host "[ERROR] Harness not found: $HarnessPath" -ForegroundColor Red
    exit 1
}

$content = Get-Content $HarnessPath -Raw
$errors = @()
$warnings = @()

Write-Host "`nRule 1: Energy-sum assertions (obsolete model)..." -ForegroundColor White
if ($content -match 'total_energy\s*-\s*5\.0|E_total.*5\.0|energy.*=.*5\.0') {
    $errors += "Energy-sum assertion detected (violates constant-pool thermodynamic model)"
}

Write-Host "Rule 2: Drift tolerance threshold..." -ForegroundColor White
$driftMatches = [regex]::Matches($content, 'energy_drift[^-]*-le\s+([0-9.e-]+)')
foreach ($match in $driftMatches) {
    $tol = [double]$match.Groups[1].Value
    if ($tol -gt 1e-10) {
        $errors += "Drift tolerance $tol exceeds 1e-10 thermodynamic precision limit"
    }
}
$oldDriftMatches = [regex]::Matches($content, '\.drift[^-]*-lt\s+([0-9.e-]+)')
foreach ($match in $oldDriftMatches) {
    if ($match.Value -notmatch 'energy_drift') {
        $warnings += "Using .drift instead of .energy_drift (schema inconsistency)"
    }
}

Write-Host "Rule 3: Belief bounds validation..." -ForegroundColor White
$beliefUpperChecks = [regex]::Matches($content, 'belief.*-le\s+([0-9.]+)')
foreach ($match in $beliefUpperChecks) {
    $upper = [double]$match.Groups[1].Value
    if ($upper -gt 1.0001) {
        $errors += "Belief upper bound $upper exceeds [0,1] invariant"
    }
}
$beliefLowerChecks = [regex]::Matches($content, 'belief.*-ge\s+(-?[0-9.]+)')
foreach ($match in $beliefLowerChecks) {
    $lower = [double]$match.Groups[1].Value
    if ($lower -lt -0.0001) {
        $errors += "Belief lower bound $lower violates [0,1] invariant"
    }
}

Write-Host "Rule 4: Governor normalization helper..." -ForegroundColor White
if ($content -notmatch 'function Get-GovernorResult') {
    $errors += "Missing Get-GovernorResult helper (schema resilience required)"
}

Write-Host "Rule 5: Timeout-protected STDIO parser..." -ForegroundColor White
if ($content -notmatch 'deadline.*AddMilliseconds|Timeout.*\d+') {
    $warnings += "No timeout protection detected in STDIO parser"
}

Write-Host "Rule 6: JSON-RPC 2.0 validation..." -ForegroundColor White
if ($content -notmatch 'jsonrpc.*2\.0') {
    $warnings += "No explicit JSON-RPC 2.0 response validation"
}

Write-Host "Rule 7: Edge weight bounds..." -ForegroundColor White
$weightChecks = [regex]::Matches($content, 'weight\s*=\s*([0-9.]+)')
foreach ($match in $weightChecks) {
    $weight = [double]$match.Groups[1].Value
    if ($weight -lt 0 -or $weight -gt 1) {
        $errors += "Edge weight $weight violates [0,1] normalized influence bound"
    }
}

Write-Host "`n===== LINT RESULTS =====" -ForegroundColor Cyan

if ($warnings.Count -gt 0) {
    Write-Host "`nWarnings ($($warnings.Count)):" -ForegroundColor Yellow
    $warnings | ForEach-Object { Write-Host "  - $_" -ForegroundColor Yellow }
}

if ($errors.Count -eq 0) {
    Write-Host @"

=====================================================================
     [PASS] PHYSICS INVARIANTS VALIDATED
     All thermodynamic constraints satisfied
=====================================================================
"@ -ForegroundColor Green
    exit 0
} else {
    Write-Host "`nErrors ($($errors.Count)):" -ForegroundColor Red
    $errors | ForEach-Object { Write-Host "  - $_" -ForegroundColor Red }
    
    Write-Host @"

=====================================================================
     [FAIL] $($errors.Count) PHYSICS VIOLATION(S) DETECTED
     Harness does not comply with SCG thermodynamic model
=====================================================================
"@ -ForegroundColor Red
    exit 1
}
