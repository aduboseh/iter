param(
    [switch]$Release,
    [switch]$Determinism,
    [switch]$NoBuild,
    [switch]$Lint
)

$ErrorActionPreference = "Stop"

Push-Location (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location ..

if ($Lint) {
    Write-Host "[LINT] cargo fmt --check" -ForegroundColor Cyan
    cargo fmt -- --check
    Write-Host "[LINT] cargo clippy -D warnings" -ForegroundColor Cyan
    cargo clippy -D warnings
}

if (-not $NoBuild) {
    if ($Release) {
        Write-Host "[BUILD] cargo build --release" -ForegroundColor Cyan
        cargo build --release
    } else {
        Write-Host "[BUILD] cargo build" -ForegroundColor Cyan
        cargo build
    }
}

$serverPath = if ($Release) { ".\\target\\release\\scg_mcp_server.exe" } else { ".\\target\\debug\\scg_mcp_server.exe" }

Write-Host "[CERT] Running certification harness" -ForegroundColor Cyan
$detFlag = if ($Determinism) { "-Determinism" } else { "" }
pwsh -NoProfile -File .\scg_certification_harness.ps1 -ServerPath $serverPath $detFlag

Write-Host "[DET] Running determinism validator v2.0" -ForegroundColor Cyan
$env:SCG_DETERMINISM = if ($Determinism) { "1" } else { $null }
pwsh -NoProfile -File .\scg_determinism_validator.ps1

Pop-Location
