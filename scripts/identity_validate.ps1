<#
.SYNOPSIS
    SCG Metadata Normalization Validation - Structural & Semantic Integrity
.DESCRIPTION
    Verifies that normalization preserved structural integrity and did not introduce errors.
    Part of APEX DIRECTIVE v3.0 - Metadata Normalization Protocol
#>
[CmdletBinding()]
param()

$ErrorActionPreference = "Stop"
$Root = git rev-parse --show-toplevel
$OutDir = Join-Path $Root "target/identity"
$Timestamp = Get-Date -Format "yyyyMMdd_HHmmss"
$Log = Join-Path $OutDir "validation_$Timestamp.txt"

$CanonAuthor = "Armonti Du-Bose-Hill"
$Errors = @()
$Warnings = @()

"=== SCG Metadata Normalization Validation ===" | Out-File -FilePath $Log -Encoding UTF8
"Generated: $(Get-Date -Format 'o')" | Out-File -FilePath $Log -Append -Encoding UTF8
"" | Out-File -FilePath $Log -Append -Encoding UTF8

# 1. Check for residual non-canonical references in metadata blocks
"[1] Metadata Block Validation" | Out-File -FilePath $Log -Append -Encoding UTF8
$ToolPatterns = @("Claude", "Perplexity", "ChatGPT", "Gemini", "OpenAI", "Anthropic")
$DocsPath = Join-Path $Root "docs"

if (Test-Path $DocsPath) {
    $DocsFiles = Get-ChildItem -Path $DocsPath -Recurse -File -Include "*.md","*.yml","*.yaml"
    foreach ($file in $DocsFiles) {
        $Content = Get-Content -Path $file.FullName -Raw -ErrorAction SilentlyContinue
        if (-not $Content) { continue }

        # Check frontmatter for tool references
        if ($Content -match "(?s)^---(.+?)---") {
            $Frontmatter = $Matches[1]
            foreach ($tool in $ToolPatterns) {
                if ($Frontmatter -match "\b$tool\b") {
                    $Errors += "ERROR: Non-canonical reference '$tool' found in frontmatter: $($file.FullName)"
                }
            }
        }
    }
}

if ($Errors.Count -eq 0) {
    "✓ No non-canonical metadata references found." | Out-File -FilePath $Log -Append -Encoding UTF8
} else {
    $Errors | Out-File -FilePath $Log -Append -Encoding UTF8
}

# 2. YAML syntax validation (basic check)
"" | Out-File -FilePath $Log -Append -Encoding UTF8
"[2] Syntax Integrity Check" | Out-File -FilePath $Log -Append -Encoding UTF8
$YamlFiles = Get-ChildItem -Path $Root -Recurse -File -Include "*.yml","*.yaml" -ErrorAction SilentlyContinue |
    Where-Object { $_.FullName -notmatch "[\\/]target[\\/]" }
$SyntaxErrors = 0

foreach ($yaml in $YamlFiles) {
    $Content = Get-Content -Path $yaml.FullName -Raw -ErrorAction SilentlyContinue
    # Basic YAML validation (check for obvious syntax issues)
    if ($Content -match "^\t") {
        "WARNING: Tab indentation in YAML: $($yaml.FullName)" | Out-File -FilePath $Log -Append -Encoding UTF8
        $SyntaxErrors++
    }
}

if ($SyntaxErrors -eq 0) {
    "✓ Basic YAML syntax check passed." | Out-File -FilePath $Log -Append -Encoding UTF8
}

# 3. Authorship consistency check
"" | Out-File -FilePath $Log -Append -Encoding UTF8
"[3] Authorship Consistency" | Out-File -FilePath $Log -Append -Encoding UTF8
$AuthorIssues = @()

if (Test-Path $DocsPath) {
    $MdFiles = Get-ChildItem -Path $DocsPath -Recurse -File -Include "*.md"
    foreach ($file in $MdFiles) {
        $Content = Get-Content -Path $file.FullName -Raw -ErrorAction SilentlyContinue
        if ($Content -match "(?s)^---(.+?)---") {
            $Frontmatter = $Matches[1]
            if ($Frontmatter -match "author:" -and $Frontmatter -notmatch "author:.*$CanonAuthor") {
                $AuthorIssues += "Non-canonical author in: $($file.FullName)"
            }
            if ($Frontmatter -match "maintainer:" -and $Frontmatter -notmatch "maintainer:.*$CanonAuthor") {
                $AuthorIssues += "Non-canonical maintainer in: $($file.FullName)"
            }
        }
    }
}

if ($AuthorIssues.Count -eq 0) {
    "✓ All author/maintainer fields normalized (or none present)." | Out-File -FilePath $Log -Append -Encoding UTF8
} else {
    $AuthorIssues | ForEach-Object { "ERROR: $_" } | Out-File -FilePath $Log -Append -Encoding UTF8
    $Errors += $AuthorIssues
}

# 4. Hash drift analysis
"" | Out-File -FilePath $Log -Append -Encoding UTF8
"[4] Hash Drift Analysis" | Out-File -FilePath $Log -Append -Encoding UTF8
$AuditFiles = Get-ChildItem -Path $OutDir -Filter "audit_*.json" -ErrorAction SilentlyContinue | Sort-Object LastWriteTime -Descending
if ($AuditFiles.Count -gt 0) {
    $LatestAudit = $AuditFiles[0]
    "Baseline: $($LatestAudit.Name)" | Out-File -FilePath $Log -Append -Encoding UTF8
    $AuditData = Get-Content -Path $LatestAudit.FullName -Raw | ConvertFrom-Json
    $DriftCount = 0
    foreach ($entry in $AuditData.files) {
        $FullPath = Join-Path $Root $entry.path
        if (Test-Path $FullPath) {
            $CurrentHash = (Get-FileHash -Path $FullPath -Algorithm SHA256).Hash.ToLower()
            if ($CurrentHash -ne $entry.sha256) {
                "MODIFIED: $($entry.path)" | Out-File -FilePath $Log -Append -Encoding UTF8
                $DriftCount++
            }
        }
    }
    "Files with hash drift: $DriftCount" | Out-File -FilePath $Log -Append -Encoding UTF8
} else {
    "No audit baseline found." | Out-File -FilePath $Log -Append -Encoding UTF8
}

"" | Out-File -FilePath $Log -Append -Encoding UTF8
"=== Validation Summary ===" | Out-File -FilePath $Log -Append -Encoding UTF8
"Errors: $($Errors.Count)" | Out-File -FilePath $Log -Append -Encoding UTF8

Write-Host "✓ Validation complete. Review: $Log" -ForegroundColor Green

if ($Errors.Count -gt 0) {
    Write-Host "❌ Validation failed with $($Errors.Count) error(s)." -ForegroundColor Red
    exit 4
}

Write-Host "✓ All validation checks passed." -ForegroundColor Green
