<#
.SYNOPSIS
    SCG Metadata Normalization Commit - Lineage Integrity
.DESCRIPTION
    Generates checksums and commits changes only if validation passes.
    Part of APEX DIRECTIVE v3.0 - Metadata Normalization Protocol
#>
[CmdletBinding()]
param()

$ErrorActionPreference = "Stop"
$Root = git rev-parse --show-toplevel
$OutDir = Join-Path $Root "target/identity"
$DocsPath = Join-Path $Root "docs"

# Find latest validation log
$ValidationLogs = Get-ChildItem -Path $OutDir -Filter "validation_*.txt" -ErrorAction SilentlyContinue | Sort-Object LastWriteTime -Descending
if ($ValidationLogs.Count -eq 0) {
    Write-Host "❌ No validation log found. Run identity_validate.ps1 first." -ForegroundColor Red
    exit 1
}

$LatestLog = $ValidationLogs[0]
$LogContent = Get-Content -Path $LatestLog.FullName -Raw

if ($LogContent -match "ERROR") {
    Write-Host "❌ Validation errors present. Commit aborted." -ForegroundColor Red
    Write-Host "   Review: $($LatestLog.FullName)"
    exit 4
}

Write-Host "✓ Validation passed. Generating checksums..." -ForegroundColor Green

# Generate deterministic checksums for docs
if (Test-Path $DocsPath) {
    $ChecksumFile = Join-Path $DocsPath "CHECKSUMS.sha256"
    $Files = Get-ChildItem -Path $DocsPath -Recurse -File -Include "*.md","*.yml","*.yaml" | Sort-Object FullName
    $Checksums = @()
    foreach ($f in $Files) {
        $Hash = (Get-FileHash -Path $f.FullName -Algorithm SHA256).Hash.ToLower()
        $RelPath = $f.FullName.Replace($Root, "").TrimStart("\", "/").Replace("\", "/")
        $Checksums += "$Hash  $RelPath"
    }
    $Checksums | Set-Content -Path $ChecksumFile -Encoding UTF8
    Write-Host "✓ Checksums written to: $ChecksumFile" -ForegroundColor Green
}

# Stage and commit
git add -A
$CommitMsg = @"
chore(docs): normalize metadata to canonical SCG identity

- Unified author/maintainer fields to Armonti Du-Bose-Hill
- Removed non-canonical tool references from metadata blocks
- Preserved third-party attribution and prose context
- Generated deterministic checksums for lineage integrity

Canonical Identity:
  Author: Armonti Du-Bose-Hill
  Email: adubosehill@gmail.com
  Org: Only SG Solutions

Validation: $($LatestLog.Name)
"@

git commit -m $CommitMsg

Write-Host "✓ Commit complete." -ForegroundColor Green
