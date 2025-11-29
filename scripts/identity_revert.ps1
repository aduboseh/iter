<#
.SYNOPSIS
    SCG Metadata Normalization Rollback - Emergency Revert
.DESCRIPTION
    Atomically restores repository state from backups if validation fails.
    Part of APEX DIRECTIVE v3.0 - Metadata Normalization Protocol
#>
[CmdletBinding()]
param()

$ErrorActionPreference = "Stop"
$Root = git rev-parse --show-toplevel
$BackupDir = Join-Path $Root "target/identity/backups"

if (-not (Test-Path $BackupDir)) {
    Write-Host "ERROR: No backup directory found. Cannot revert." -ForegroundColor Red
    exit 1
}

$Backups = Get-ChildItem -Path $BackupDir -Filter "*.bak" -ErrorAction SilentlyContinue

if ($Backups.Count -eq 0) {
    Write-Host "ERROR: No backup files found in $BackupDir" -ForegroundColor Red
    exit 1
}

Write-Host "⚠️  REVERTING: Restoring files from backup..." -ForegroundColor Yellow

# Group backups by original filename (get latest backup for each file)
$BackupGroups = $Backups | Group-Object { $_.Name -replace '\.\d+\.bak$', '' }

foreach ($group in $BackupGroups) {
    $LatestBackup = $group.Group | Sort-Object LastWriteTime -Descending | Select-Object -First 1
    $OriginalName = $group.Name

    # Find original file location (search common locations)
    $PossibleLocations = @(
        (Join-Path $Root $OriginalName),
        (Join-Path $Root "docs" $OriginalName),
        (Join-Path $Root "scg_demo_package" $OriginalName)
    )

    foreach ($loc in $PossibleLocations) {
        if (Test-Path $loc) {
            Copy-Item -Path $LatestBackup.FullName -Destination $loc -Force
            Write-Host "Restored: $loc"
            break
        }
    }
}

git reset --hard HEAD
git checkout main

Write-Host "✓ Repository state reverted to pre-normalization baseline." -ForegroundColor Green
