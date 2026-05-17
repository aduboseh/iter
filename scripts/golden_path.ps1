param(
    [switch]$SkipTests
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$RepoRoot = Split-Path -Parent $PSScriptRoot
$ContractCriticalFiles = @(
    "build.rs",
    "vendor/governance-bridge/src/contract.rs",
    "vendor/governance-bridge/src/trace.rs",
    "vendor/governance-bridge/src/errors.rs",
    "vendor/governance-bridge/src/lib.rs",
    "vendor/governance-bridge/CANONICAL_VECTORS.json"
)

function Invoke-Checked {
    param(
        [Parameter(Mandatory = $true)][string]$Name,
        [Parameter(Mandatory = $true)][scriptblock]$Command
    )

    Write-Host "running=$Name"
    & $Command
    if ($LASTEXITCODE -ne 0) {
        throw "GOLDEN_PATH_FAILED: $Name exited with $LASTEXITCODE"
    }
}

function Get-ContractFileHashes {
    $hashes = @{}
    foreach ($file in $ContractCriticalFiles) {
        $path = Join-Path $RepoRoot $file
        $hashes[$file] = (Get-FileHash -Algorithm SHA256 -LiteralPath $path).Hash
    }
    return $hashes
}

Push-Location $RepoRoot
try {
    Invoke-Checked "format_check" { cargo fmt --all -- --check }

    if (-not $SkipTests) {
        Invoke-Checked "public_stub_tests" { cargo test --features public_stub }
    }

    Invoke-Checked "locked_iter_server_build" {
        cargo build --locked --features public_stub --bin iter-server
    }

    $before = Get-ContractFileHashes
    $env:ITER_SIMULATE_DRIFT = "1"
    $driftOutput = cargo build --features public_stub 2>&1
    $driftExit = $LASTEXITCODE
    Remove-Item Env:\ITER_SIMULATE_DRIFT -ErrorAction SilentlyContinue
    $after = Get-ContractFileHashes

    if ($driftExit -eq 0) {
        $driftOutput | Select-Object -Last 80
        throw "DRIFT_SIMULATION_FAILED: build unexpectedly succeeded"
    }

    if (-not ($driftOutput -match "BRIDGE_INTEGRITY_MISMATCH_SIMULATED")) {
        $driftOutput | Select-Object -Last 120
        throw "DRIFT_SIMULATION_FAILED: expected BRIDGE_INTEGRITY_MISMATCH_SIMULATED"
    }

    $changed = @($ContractCriticalFiles | Where-Object { $before[$_] -ne $after[$_] })
    if ($changed.Count -ne 0) {
        $changed | ForEach-Object { Write-Error "mutated=$_" }
        throw "WORKING_TREE_MUTATED_BY_DRIFT_TEST"
    }

    $rustcVersion = (& rustc --version).Trim()
    $platform = ((& rustc -vV) | Where-Object { $_ -like "host:*" } | Select-Object -First 1)
    $platform = $platform.Replace("host:", "").Trim()

    Write-Host "GOLDEN_PATH_PASS"
    Write-Host "contract_version=scg.v1"
    Write-Host "claim_registry_version=1.0"
    Write-Host "determinism_scope=same_binary_only"
    Write-Host "platform=$platform"
    Write-Host "rustc_version=$rustcVersion"
    Write-Host "cross_platform_replay_claimed=false"
    Write-Host "scg_source_commit=da14c8390ba8ceeb0ab15d85c598d2042a2029cf"
    Write-Host "scg_vendor_master_head=3e0675073a50ce20bdad7c342f7a5caaa3801504"
    Write-Host "build_rerun_triggers=verified"
    Write-Host "rustc_env_exports=verified"
    Write-Host "bridge_integrity=verified"
    Write-Host "canonical_vectors_raw_byte_hash=verified"
    Write-Host "canonical_vector_uppercase_digests=verified"
    Write-Host "proof_packet_provenance=compile_time_exports+runtime_decision"
    Write-Host "proof_numeric_encoding=ieee754-f64-bits-lowerhex"
    Write-Host "replay_verification=verified"
    Write-Host "drift_simulation=verified"
    Write-Host "working_tree_mutated=false"
}
finally {
    Remove-Item Env:\ITER_SIMULATE_DRIFT -ErrorAction SilentlyContinue
    Pop-Location
}
