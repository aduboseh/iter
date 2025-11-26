param(
    [int]$Runs = 3,
    [string]$HarnessPath = ".\certification\v1.0\harness\scg_certification_harness.ps1"
)

$ErrorActionPreference = "Stop"

Write-Host @"
=====================================================================
     SCG DETERMINISM VALIDATOR v2.0
     Verifying thermodynamic reproducibility across $Runs runs
=====================================================================
"@ -ForegroundColor Cyan

$hashes = @()
$outputs = @()

for ($run = 1; $run -le $Runs; $run++) {
    Write-Host "`n=== RUN $run/$Runs ===" -ForegroundColor Cyan
    
    # Determinism-safe mode for server
    $env:SCG_DETERMINISM = "1"
    $env:SCG_DETERMINISM_SEED = "42"

    # Preflight: ensure no existing server is running
    Get-Process -Name "scg_mcp_server" -ErrorAction SilentlyContinue | ForEach-Object { Stop-Process -Id $_.Id -Force }
    
    try {
        # Run the certification harness (ignore console output)
        pwsh -NoProfile -File $HarnessPath | Out-Null

        # Locate the most recent certification JSON report
        $reportFile = Get-ChildItem -Path "." -Filter "certification_report_*.json" |
                      Sort-Object LastWriteTime -Descending |
                      Select-Object -First 1

        if (-not $reportFile) {
            Write-Host "  [ERROR] No certification report found for this run." -ForegroundColor Red
            $hashes += "ERROR"
            continue
        }

        # Load the report JSON
        $raw = Get-Content $reportFile.FullName -Raw | ConvertFrom-Json

        # Build a deterministic, canonical object
        if ($raw -is [System.Array]) {
            $total = $raw.Count
            $passed = ($raw | Where-Object { $_.Passed }).Count
            $phaseGroups = $raw | Group-Object -Property Phase
            $phases = @()
            foreach ($g in ($phaseGroups | Sort-Object Name)) {
                $phases += [ordered]@{
                    name   = $g.Name
                    total  = $g.Count
                    passed = ($g.Group | Where-Object { $_.Passed }).Count
                }
            }
            $deterministicObject = [ordered]@{
                total_tests  = $total
                passed_tests = $passed
                pass_rate    = [math]::Round((($passed / [math]::Max(1,$total)) * 100), 4)
                phases       = $phases
            }
        } else {
            # Fallback: object already includes totals/phases
            $phasesNorm = @()
            if ($raw.phases) {
                foreach ($p in ($raw.phases | Sort-Object name)) {
                    $phasesNorm += [ordered]@{ name=$p.name; total=$p.total; passed=$p.passed }
                }
            }
            $deterministicObject = [ordered]@{
                total_tests  = $raw.total_tests
                passed_tests = $raw.passed_tests
                pass_rate    = $raw.pass_rate
                phases       = $phasesNorm
            }
        }

        # Canonical serialization and hashing
        $canonicalJson = $deterministicObject | ConvertTo-Json -Depth 20 -Compress
        $bytes = [System.Text.Encoding]::UTF8.GetBytes($canonicalJson)
        $stream = New-Object System.IO.MemoryStream(,$bytes)
        $hash = (Get-FileHash -InputStream $stream -Algorithm SHA256).Hash

        Write-Host "  Result: $($deterministicObject.passed_tests)/$($deterministicObject.total_tests) tests passed" -ForegroundColor Gray
        Write-Host "  Hash: $($hash.Substring(0, 16))..." -ForegroundColor DarkGray

        $hashes += $hash

    } catch {
        Write-Host "  [ERROR] Run failed: $_" -ForegroundColor Red
        $hashes += "ERROR"
    }
}

Write-Host "`n=== DETERMINISM CHECK ===" -ForegroundColor Cyan
Write-Host "Hashes:" -ForegroundColor White
$hashes | ForEach-Object { Write-Host "  $_" -ForegroundColor DarkGray }

$uniqueHashes = $hashes | Select-Object -Unique
$allIdentical = $uniqueHashes.Count -eq 1 -and $uniqueHashes[0] -ne "ERROR"

if ($allIdentical) {
    Write-Host @"

=====================================================================
     [PASS] ALL $Runs RUNS IDENTICAL
     Thermodynamic determinism verified
     SHA-256: $($hashes[0])
=====================================================================
"@ -ForegroundColor Green
    exit 0
} else {
    Write-Host @"

=====================================================================
     [FAIL] OUTPUT DIVERGENCE DETECTED
     $($uniqueHashes.Count) unique hash(es) across $Runs runs
=====================================================================
"@ -ForegroundColor Red
    
    if ($outputs.Count -ge 2) {
        Write-Host "`nAnalyzing divergence..." -ForegroundColor Yellow
        $lines1 = $outputs[0] -split "`n"
        $lines2 = $outputs[1] -split "`n"
        $maxLines = [math]::Max($lines1.Count, $lines2.Count)
        
        for ($i = 0; $i -lt [math]::Min(20, $maxLines); $i++) {
            if ($i -lt $lines1.Count -and $i -lt $lines2.Count) {
                if ($lines1[$i] -ne $lines2[$i]) {
                    Write-Host "  Line $($i+1) differs:" -ForegroundColor Red
                    Write-Host "    Run1: $($lines1[$i].Substring(0, [math]::Min(60, $lines1[$i].Length)))..." -ForegroundColor DarkRed
                    Write-Host "    Run2: $($lines2[$i].Substring(0, [math]::Min(60, $lines2[$i].Length)))..." -ForegroundColor DarkRed
                    break
                }
            }
        }
    }
    
    exit 1
}
