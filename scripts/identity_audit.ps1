<#
.SYNOPSIS
    SCG Documentation Metadata Audit - Read-Only Reconnaissance
.DESCRIPTION
    Generates deterministic baseline of all SCG-authored documentation with SHA256 fingerprints.
    Part of APEX DIRECTIVE v3.0 - Metadata Normalization Protocol
#>
[CmdletBinding()]
param()

$ErrorActionPreference = "Stop"
$Root = git rev-parse --show-toplevel
$OutDir = Join-Path $Root "target/identity"
$Timestamp = Get-Date -Format "yyyyMMdd_HHmmss"
$Report = Join-Path $OutDir "audit_$Timestamp.json"

New-Item -ItemType Directory -Path $OutDir -Force | Out-Null

# Deterministic file enumeration (sorted, SCG-authored only)
$Files = Get-ChildItem -Path $Root -Recurse -File -Include "*.md","*.yaml","*.yml","*.toml","*.json" |
    Where-Object {
        $_.FullName -notmatch "[\\/]target[\\/]" -and
        $_.FullName -notmatch "[\\/]node_modules[\\/]" -and
        $_.FullName -notmatch "[\\/]vendor[\\/]" -and
        $_.FullName -notmatch "[\\/]third_party[\\/]" -and
        $_.Name -ne "LICENSE" -and
        $_.Name -ne "NOTICE" -and
        $_.Name -ne "AUTHORS"
    } |
    Sort-Object FullName

$FileEntries = @()
foreach ($f in $Files) {
    $Hash = (Get-FileHash -Path $f.FullName -Algorithm SHA256).Hash.ToLower()
    $RelPath = $f.FullName.Replace($Root, "").TrimStart("\", "/")
    $FileEntries += @{
        path = $RelPath
        sha256 = $Hash
    }
}

$AuditData = @{
    generated = (Get-Date -Format "o")
    canonical_author = "Armonti Du-Bose-Hill"
    canonical_email = "adubosehill@gmail.com"
    canonical_org = "Only SG Solutions"
    file_count = $Files.Count
    files = $FileEntries
}

$AuditData | ConvertTo-Json -Depth 10 | Set-Content -Path $Report -Encoding UTF8

Write-Host "✓ Audit complete: $Report" -ForegroundColor Green
Write-Host "  Files scanned: $($Files.Count)"
