# Iter MCP Tool Demo
# Demonstrates repeatable tool behavior via MCP
#
# NOTE:
# This demo illustrates observable behavior of the Iter MCP tool surface.
# It does not describe internal execution logic, governance rules, or validation criteria.

$ErrorActionPreference = "Stop"
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$serverPath = Join-Path (Split-Path -Parent $scriptDir) "target\release\iter-server.exe"

# Start long-lived server process
$psi = New-Object System.Diagnostics.ProcessStartInfo
$psi.FileName = $serverPath
$psi.Arguments = "--json-only"
$psi.UseShellExecute = $false
$psi.RedirectStandardInput = $true
$psi.RedirectStandardOutput = $true
$psi.RedirectStandardError = $true
$psi.CreateNoWindow = $true

$serverProcess = New-Object System.Diagnostics.Process
$serverProcess.StartInfo = $psi
$serverProcess.Start() | Out-Null

# Cleanup handler
$cleanup = {
    if ($serverProcess -and !$serverProcess.HasExited) {
        $serverProcess.StandardInput.Close()
        $serverProcess.WaitForExit(1000) | Out-Null
    }
}
Register-EngineEvent -SourceIdentifier PowerShell.Exiting -Action $cleanup | Out-Null

# Colors for narrative
function Write-Narrative { param($text) Write-Host "`n$text" -ForegroundColor Cyan }
function Write-Info { param($text) Write-Host "  $text" -ForegroundColor Yellow }
function Write-Result { param($text) Write-Host "  $text" -ForegroundColor Green }
function Write-Status { param($text) Write-Host "  $text" -ForegroundColor Magenta }

# Send JSON-RPC to server and parse response
function Invoke-MCP {
    param(
        [string]$Method,
        [string]$ToolName,
        [hashtable]$Arguments,
        [int]$Id
    )
    
    $request = @{
        jsonrpc = "2.0"
        method = $Method
        id = $Id
        params = @{
            name = $ToolName
            arguments = $Arguments
        }
    } | ConvertTo-Json -Depth 10 -Compress
    
    Write-Host "`n  > $ToolName" -ForegroundColor DarkGray
    
    $serverProcess.StandardInput.WriteLine($request)
    $serverProcess.StandardInput.Flush()
    
    $output = $serverProcess.StandardOutput.ReadLine()
    while ($output -and $output -notmatch '^\s*\{.*"jsonrpc"\s*:\s*"2\.0"') {
        $output = $serverProcess.StandardOutput.ReadLine()
    }
    
    if (-not $output) {
        Write-Host "ERROR: No JSON-RPC response found in server output." -ForegroundColor Red
        & $cleanup
        exit 1
    }
    
    $parsed = $output | ConvertFrom-Json
    
    if ($parsed.result.error) {
        Write-Host "ERROR: $($parsed.result.error.message)" -ForegroundColor Red
        & $cleanup
        exit 1
    }
    
    return $parsed
}

function Get-ContentText {
    param($Response)
    $textContent = $Response.result.content | Where-Object { $_.type -eq "text" } | Select-Object -First 1
    if (-not $textContent -or -not $textContent.text) {
        Write-Host "ERROR: No text content found in response" -ForegroundColor Red
        exit 1
    }
    return $textContent.text
}

# Direct method call (not tools/call)
function Invoke-MCPDirect {
    param(
        [string]$Method,
        [hashtable]$Params = @{},
        [int]$Id
    )
    
    $request = @{
        jsonrpc = "2.0"
        method = $Method
        id = $Id
        params = $Params
    } | ConvertTo-Json -Depth 10 -Compress
    
    Write-Host "`n  > $Method" -ForegroundColor DarkGray
    
    $serverProcess.StandardInput.WriteLine($request)
    $serverProcess.StandardInput.Flush()
    
    $output = $serverProcess.StandardOutput.ReadLine()
    while ($output -and $output -notmatch '^\s*\{.*"jsonrpc"\s*:\s*"2\.0"') {
        $output = $serverProcess.StandardOutput.ReadLine()
    }
    
    if (-not $output) {
        Write-Host "ERROR: No JSON-RPC response found in server output." -ForegroundColor Red
        & $cleanup
        exit 1
    }
    
    $parsed = $output | ConvertFrom-Json
    
    return $parsed
}

Write-Host @"

╔══════════════════════════════════════════════════════════════════════════════╗
║                         ITER MCP TOOL DEMO                                   ║
║                              v0.3.0 Release                                  ║
╚══════════════════════════════════════════════════════════════════════════════╝

"@ -ForegroundColor White

Write-Host "This demo exercises the iter-server tool surface." -ForegroundColor Yellow
Write-Host ""
Read-Host "Press Enter to begin"

# ═══════════════════════════════════════════════════════════════════════════════
# PHASE 1: Initialize Session
# ═══════════════════════════════════════════════════════════════════════════════

Write-Narrative "═══ STEP 1: INITIALIZE ═══"
Write-Info "Initializing server session..."

$init = Invoke-MCPDirect -Method "initialize" -Id 0
Write-Result "Protocol: $($init.result.protocolVersion)"
Write-Result "Server: $($init.result.serverInfo.name) v$($init.result.serverInfo.version)"
Write-Status "Initialized"

Read-Host "`nPress Enter to create nodes"

# ═══════════════════════════════════════════════════════════════════════════════
# PHASE 2: Create Nodes
# ═══════════════════════════════════════════════════════════════════════════════

Write-Narrative "═══ STEP 2: CREATE NODES ═══"
Write-Info "Creating Node 0..."

$node0 = Invoke-MCP -Method "tools/call" -ToolName "node.create" -Arguments @{belief=0.5; energy=100.0} -Id 1
 
$content0 = Get-ContentText $node0 | ConvertFrom-Json
Write-Result "Node 0 instantiated:"
Write-Result "  ID: $($content0.id)"
Write-Result "  Belief: $($content0.belief)"
Write-Result "  Energy: $($content0.energy)"
Write-Result "  Compliance: $($content0.esv_valid)"
 
Read-Host "`nPress Enter to create a second node"
 
# Create Node 1
Write-Info "Creating Node 1..."
 
$node1 = Invoke-MCP -Method "tools/call" -ToolName "node.create" -Arguments @{belief=0.2; energy=30.0} -Id 2
 
$content1 = Get-ContentText $node1 | ConvertFrom-Json
Write-Result "Node 1 instantiated:"
Write-Result "  ID: $($content1.id)"
Write-Result "  Belief: $($content1.belief)"
Write-Result "  Energy: $($content1.energy)"
 
Read-Host "`nPress Enter to bind edge"
 
# ═══════════════════════════════════════════════════════════════════════════════════════
# PHASE 3: Bind Edge
# ═══════════════════════════════════════════════════════════════════════════════════
 
Write-Narrative "═══ STEP 3: BIND EDGE ═══"
Write-Info "Binding edge $($content0.id)→$($content1.id)..."
 
$edge = Invoke-MCP -Method "tools/call" -ToolName "edge.bind" -Arguments @{src="$($content0.id)"; dst="$($content1.id)"; weight=0.8} -Id 3

$edgeContent = Get-ContentText $edge | ConvertFrom-Json
Write-Result "Edge bound:"
Write-Result "  Source: Node $($edgeContent.src) → Target: Node $($edgeContent.dst)"
Write-Result "  Weight: $($edgeContent.weight)"

Read-Host "`nPress Enter to submit mutation request"

# ═══════════════════════════════════════════════════════════════════════════════
# PHASE 4: Mutation Request
# ═══════════════════════════════════════════════════════════════════════════════

Write-Narrative "═══ STEP 4: MUTATE NODE ═══"
Write-Host ""
Write-Host "  ╔════════════════════════════════════════════════════════════════╗" -ForegroundColor Red
Write-Host "  ║  This step applies a mutation request via the public tool API. ║" -ForegroundColor Red
Write-Host "  ║  The server may accept or reject the request.                  ║" -ForegroundColor Red
Write-Host "  ║  Inspect the response for the observed outcome.                ║" -ForegroundColor Red
Write-Host "  ╚════════════════════════════════════════════════════════════════╝" -ForegroundColor Red
Write-Host ""
Write-Info "Submitting mutation request..."

# Submit mutation request and observe result
$mutateResp = Invoke-MCP -Method "tools/call" -ToolName "node.mutate" -Arguments @{node_id="$($content0.id)"; delta=0.49} -Id 4

$mutateContent = Get-ContentText $mutateResp | ConvertFrom-Json
Write-Result "Mutation processed:"
Write-Result "  New Belief: $($mutateContent.belief)"
Write-Result "  Energy: $($mutateContent.energy)"

$gov1 = Invoke-MCP -Method "tools/call" -ToolName "governance.status" -Arguments @{} -Id 5
$govContent1 = Get-ContentText $gov1 | ConvertFrom-Json
Write-Status "Governance status captured"

Read-Host "`nPress Enter to run propagation steps"

# ═══════════════════════════════════════════════════════════════════════════════
# PHASE 5: Run Propagation Steps
# ═══════════════════════════════════════════════════════════════════════════════

Write-Narrative "═══ STEP 5: RUN STEPS ═══"
Write-Info "Running two steps..."

Write-Host "`n  --- TICK 1 ---" -ForegroundColor White
$prop1 = Invoke-MCP -Method "tools/call" -ToolName "edge.propagate" -Arguments @{edge_id="$($edgeContent.id)"} -Id 6
Write-Result (Get-ContentText $prop1)

# Query both nodes after propagation
$q0 = Invoke-MCP -Method "tools/call" -ToolName "node.query" -Arguments @{node_id="$($content0.id)"} -Id 7
$q1 = Invoke-MCP -Method "tools/call" -ToolName "node.query" -Arguments @{node_id="$($content1.id)"} -Id 8
$qc0 = Get-ContentText $q0 | ConvertFrom-Json
$qc1 = Get-ContentText $q1 | ConvertFrom-Json
Write-Result "  Node 0: belief=$($qc0.belief), energy=$($qc0.energy)"
Write-Result "  Node 1: belief=$($qc1.belief), energy=$($qc1.energy)"
Write-Status "Step complete"

Write-Host "`n  --- TICK 2 ---" -ForegroundColor White
$prop2 = Invoke-MCP -Method "tools/call" -ToolName "edge.propagate" -Arguments @{edge_id="$($edgeContent.id)"} -Id 9
Write-Result (Get-ContentText $prop2)

$q0b = Invoke-MCP -Method "tools/call" -ToolName "node.query" -Arguments @{node_id="$($content0.id)"} -Id 10
$q1b = Invoke-MCP -Method "tools/call" -ToolName "node.query" -Arguments @{node_id="$($content1.id)"} -Id 11
$qc0b = Get-ContentText $q0b | ConvertFrom-Json
$qc1b = Get-ContentText $q1b | ConvertFrom-Json
Write-Result "  Node 0: belief=$($qc0b.belief), energy=$($qc0b.energy)"
Write-Result "  Node 1: belief=$($qc1b.belief), energy=$($qc1b.energy)"
Write-Status "Step complete"

Read-Host "`nPress Enter to examine the audit trail"

# ═══════════════════════════════════════════════════════════════════════════════
# PHASE 6: Lineage Audit
# ═══════════════════════════════════════════════════════════════════════════════

Write-Narrative "═══ STEP 6: AUDIT SUMMARY ═══"
Write-Info "Fetching audit summary..."

$lineage = Invoke-MCP -Method "tools/call" -ToolName "lineage.replay" -Arguments @{} -Id 12
$lineageContent = Get-ContentText $lineage | ConvertFrom-Json

Write-Result "Lineage entries:"
foreach ($entry in $lineageContent | Select-Object -First 5) {
    $entryLine = "  seq=$($entry.sequence) `| op=$($entry.operation)"
    Write-Result $entryLine
}
Write-Status "Audit summary captured"

Read-Host "`nPress Enter for final status"

# ═══════════════════════════════════════════════════════════════════════════════
# PHASE 7: FINAL STATUS
# ═══════════════════════════════════════════════════════════════════════════════

Write-Narrative "═══ STEP 7: FINAL STATUS ═══"
Write-Info "Fetching final status..."

$finalGov = Invoke-MCP -Method "tools/call" -ToolName "governance.status" -Arguments @{} -Id 13
$finalContent = Get-ContentText $finalGov | ConvertFrom-Json

Write-Host ""
Write-Host "  ╔════════════════════════════════════════════════════════════════╗" -ForegroundColor Green
Write-Host "  ║                     SYSTEM VITAL SIGNS                         ║" -ForegroundColor Green
Write-Host "  ╠════════════════════════════════════════════════════════════════╣" -ForegroundColor Green
Write-Host "  ║  Drift OK:      $(($finalContent.drift_ok ?? $false).ToString().PadRight(43))║" -ForegroundColor Green
Write-Host "  ║  Energy Drift:  $(($finalContent.energy_drift ?? 0.0).ToString().PadRight(43))║" -ForegroundColor Green
Write-Host "  ║  Coherence:     $(($finalContent.coherence ?? 0.0).ToString().PadRight(43))║" -ForegroundColor Green
Write-Host "  ║  Node Count:    $(($finalContent.node_count ?? 0).ToString().PadRight(43))║" -ForegroundColor Green
Write-Host "  ║  Edge Count:    $(($finalContent.edge_count ?? 0).ToString().PadRight(43))║" -ForegroundColor Green
Write-Host "  ║  Healthy:       $(($finalContent.healthy ?? $false).ToString().PadRight(43))║" -ForegroundColor Green
Write-Host "  ╚════════════════════════════════════════════════════════════════╝" -ForegroundColor Green

Write-Status "Completed"

& $cleanup

Write-Host @"

╔══════════════════════════════════════════════════════════════════════════════╗
║                            DEMO COMPLETE                                     ║
╠══════════════════════════════════════════════════════════════════════════════╣
║  Demo complete.                                                              ║
║                                                                              ║
║  This script demonstrates repeatable tool behavior and audit/status surfaces ║
║  via MCP.                                                                    ║
╚══════════════════════════════════════════════════════════════════════════════╝

"@ -ForegroundColor Cyan
