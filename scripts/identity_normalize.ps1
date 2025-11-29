<#
.SYNOPSIS
    SCG Documentation Metadata Normalization - Scoped, Reversible Mutation
.DESCRIPTION
    Applies metadata normalization to SCG-authored files, creating backups and flagging prose references.
    Part of APEX DIRECTIVE v3.0 - Metadata Normalization Protocol
#>
[CmdletBinding()]
param(
    [switch]$DryRun
)

$ErrorActionPreference = "Stop"
$Root = git rev-parse --show-toplevel
$BackupDir = Join-Path $Root "target/identity/backups"
$Timestamp = Get-Date -Format "yyyyMMdd_HHmmss"
$ReviewLog = Join-Path $Root "target/identity" "review_$Timestamp.txt"

New-Item -ItemType Directory -Path $BackupDir -Force | Out-Null

$CanonAuthor = "Armonti Du-Bose-Hill"
$CanonEmail = "adubosehill@gmail.com"
$CanonOrg = "Only SG Solutions"

# Non-canonical tool references to remove from metadata blocks
$ToolPatterns = @(
    "Claude",
    "Warp Terminal",
    "Perplexity",
    "ChatGPT",
    "Gemini",
    "OpenAI",
    "Anthropic"
)

# Target SCG-authored documentation
$Targets = @()
$Targets += Get-ChildItem -Path (Join-Path $Root "docs") -Recurse -File -Include "*.md","*.yml","*.yaml" -ErrorAction SilentlyContinue
$Targets += Get-ChildItem -Path (Join-Path $Root "scg_demo_package") -Recurse -File -Include "*.md","*.yml","*.yaml" -ErrorAction SilentlyContinue
$Targets += Get-Item -Path (Join-Path $Root "README.md") -ErrorAction SilentlyContinue
$Targets += Get-Item -Path (Join-Path $Root "CHANGELOG.md") -ErrorAction SilentlyContinue
$Targets = $Targets | Where-Object { $_ -ne $null } | Sort-Object FullName

$ModifiedFiles = @()
$ReviewItems = @()

foreach ($file in $Targets) {
    # Backup original
    $BackupName = "$($file.Name).$((Get-Date).ToFileTimeUtc()).bak"
    Copy-Item -Path $file.FullName -Destination (Join-Path $BackupDir $BackupName)

    $Content = Get-Content -Path $file.FullName -Raw -ErrorAction SilentlyContinue
    if (-not $Content) { continue }

    $OriginalContent = $Content
    $InFrontmatter = $false
    $Lines = $Content -split "`n"
    $NewLines = @()
    $FrontmatterStart = -1
    $FrontmatterEnd = -1

    # Find frontmatter boundaries
    for ($i = 0; $i -lt $Lines.Count; $i++) {
        if ($Lines[$i].Trim() -eq "---") {
            if ($FrontmatterStart -eq -1) {
                $FrontmatterStart = $i
            } elseif ($FrontmatterEnd -eq -1) {
                $FrontmatterEnd = $i
                break
            }
        }
    }

    # Process lines
    for ($i = 0; $i -lt $Lines.Count; $i++) {
        $line = $Lines[$i]
        $InFrontmatter = ($FrontmatterStart -ne -1 -and $FrontmatterEnd -ne -1 -and $i -gt $FrontmatterStart -and $i -lt $FrontmatterEnd)

        if ($InFrontmatter) {
            # Normalize author/maintainer/email in frontmatter
            if ($line -match "^author:") {
                $line = "author: `"$CanonAuthor`""
            }
            if ($line -match "^maintainer:") {
                $line = "maintainer: `"$CanonAuthor`""
            }
            if ($line -match "^email:") {
                $line = "email: `"$CanonEmail`""
            }

            # Remove tool references from metadata
            foreach ($tool in $ToolPatterns) {
                $line = $line -replace "\b$tool\b", ""
            }
        } else {
            # Flag prose references for manual review (not auto-mutated)
            foreach ($tool in $ToolPatterns) {
                if ($line -match "\b$tool\b") {
                    $ReviewItems += "REVIEW: $($file.FullName):$($i+1) - contains '$tool' in prose"
                }
            }
        }

        $NewLines += $line
    }

    $NewContent = $NewLines -join "`n"

    if ($NewContent -ne $OriginalContent) {
        if (-not $DryRun) {
            Set-Content -Path $file.FullName -Value $NewContent -NoNewline -Encoding UTF8
        }
        $ModifiedFiles += $file.FullName
    }
}

# Write review log
if ($ReviewItems.Count -gt 0) {
    $ReviewItems | Set-Content -Path $ReviewLog -Encoding UTF8
    Write-Host "⚠️  Manual review required. See: $ReviewLog" -ForegroundColor Yellow
}

Write-Host "✓ Normalization complete." -ForegroundColor Green
Write-Host "  Files processed: $($Targets.Count)"
Write-Host "  Files modified: $($ModifiedFiles.Count)"
Write-Host "  Backups stored in: $BackupDir"

if ($DryRun) {
    Write-Host "  [DRY RUN - No files were actually modified]" -ForegroundColor Cyan
}
