# Iter MCP Tool Demo
# Demonstrates repeatable tool behavior via MCP
#
# NOTE:
# This demo illustrates observable behavior of the Iter MCP tool surface.
# It does not describe internal execution logic, governance rules, or validation criteria.

$ErrorActionPreference = "Stop"
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$serverPath = Join-Path (Split-Path -Parent $scriptDir) "target\release\iter-server.exe"

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
    
    $result = $request | & $serverPath 2>$null | Select-Object -First 1
    $parsed = $result | ConvertFrom-Json
    
    return $parsed
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
    
    $result = $request | & $serverPath 2>$null | Select-Object -First 1
    $parsed = $result | ConvertFrom-Json
    
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

$content0 = ($node0.result.content | Where-Object { $_.type -eq "text" }).text | ConvertFrom-Json
Write-Result "Node 0 instantiated:"
Write-Result "  ID: $($content0.id)"
Write-Result "  Belief: $($content0.belief)"
Write-Result "  Energy: $($content0.energy)"
Write-Result "  Compliance: $($content0.esv_valid)"

Read-Host "`nPress Enter to create a second node"

# Create Node 1
Write-Info "Creating Node 1..."

$node1 = Invoke-MCP -Method "tools/call" -ToolName "node.create" -Arguments @{belief=0.2; energy=30.0} -Id 2

$content1 = ($node1.result.content | Where-Object { $_.type -eq "text" }).text | ConvertFrom-Json
Write-Result "Node 1 instantiated:"
Write-Result "  ID: $($content1.id)"
Write-Result "  Belief: $($content1.belief)"
Write-Result "  Energy: $($content1.energy)"

Read-Host "`nPress Enter to bind edge"

# ═══════════════════════════════════════════════════════════════════════════════
# PHASE 3: Bind Edge
# ═══════════════════════════════════════════════════════════════════════════════

Write-Narrative "═══ STEP 3: BIND EDGE ═══"
Write-Info "Binding edge 0→1..."

$edge = Invoke-MCP -Method "tools/call" -ToolName "edge.bind" -Arguments @{src="0"; dst="1"; weight=0.8} -Id 3

$edgeContent = ($edge.result.content | Where-Object { $_.type -eq "text" }).text | ConvertFrom-Json
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
$mutateResp = Invoke-MCP -Method "tools/call" -ToolName "node.mutate" -Arguments @{node_id="0"; delta=0.49} -Id 4

$mutateContent = ($mutateResp.result.content | Where-Object { $_.type -eq "text" }).text | ConvertFrom-Json
Write-Result "Mutation processed:"
Write-Result "  New Belief: $($mutateContent.belief)"
Write-Result "  Energy: $($mutateContent.energy)"

$gov1 = Invoke-MCP -Method "tools/call" -ToolName "governance.status" -Arguments @{} -Id 5
$govContent1 = ($gov1.result.content | Where-Object { $_.type -eq "text" }).text | ConvertFrom-Json
Write-Status "Governance status captured"

Read-Host "`nPress Enter to run propagation steps"

# ═══════════════════════════════════════════════════════════════════════════════
# PHASE 5: Run Propagation Steps
# ═══════════════════════════════════════════════════════════════════════════════

Write-Narrative "═══ STEP 5: RUN STEPS ═══"
Write-Info "Running two steps..."

Write-Host "`n  --- TICK 1 ---" -ForegroundColor White
$prop1 = Invoke-MCP -Method "tools/call" -ToolName "edge.propagate" -Arguments @{edge_id="0"} -Id 6
Write-Result ($prop1.result.content | Where-Object { $_.type -eq "text" }).text

# Query both nodes after propagation
$q0 = Invoke-MCP -Method "tools/call" -ToolName "node.query" -Arguments @{node_id="0"} -Id 7
$q1 = Invoke-MCP -Method "tools/call" -ToolName "node.query" -Arguments @{node_id="1"} -Id 8
$qc0 = ($q0.result.content | Where-Object { $_.type -eq "text" }).text | ConvertFrom-Json
$qc1 = ($q1.result.content | Where-Object { $_.type -eq "text" }).text | ConvertFrom-Json
Write-Result "  Node 0: belief=$($qc0.belief), energy=$($qc0.energy)"
Write-Result "  Node 1: belief=$($qc1.belief), energy=$($qc1.energy)"
Write-Status "Step complete"

Write-Host "`n  --- TICK 2 ---" -ForegroundColor White
$prop2 = Invoke-MCP -Method "tools/call" -ToolName "edge.propagate" -Arguments @{edge_id="0"} -Id 9
Write-Result ($prop2.result.content | Where-Object { $_.type -eq "text" }).text

$q0b = Invoke-MCP -Method "tools/call" -ToolName "node.query" -Arguments @{node_id="0"} -Id 10
$q1b = Invoke-MCP -Method "tools/call" -ToolName "node.query" -Arguments @{node_id="1"} -Id 11
$qc0b = ($q0b.result.content | Where-Object { $_.type -eq "text" }).text | ConvertFrom-Json
$qc1b = ($q1b.result.content | Where-Object { $_.type -eq "text" }).text | ConvertFrom-Json
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
$lineageContent = ($lineage.result.content | Where-Object { $_.type -eq "text" }).text | ConvertFrom-Json

Write-Result "Lineage entries:"
foreach ($entry in $lineageContent | Select-Object -First 5) {
    Write-Result "  seq=$($entry.sequence) | op=$($entry.operation) | checksum=$($entry.checksum.Substring(0,16))..."
}
Write-Status "Audit summary captured"

Read-Host "`nPress Enter for final status"

# ═══════════════════════════════════════════════════════════════════════════════
# PHASE 7: FINAL STATUS
# ═══════════════════════════════════════════════════════════════════════════════

Write-Narrative "═══ STEP 7: FINAL STATUS ═══"
Write-Info "Fetching final status..."

$finalGov = Invoke-MCP -Method "tools/call" -ToolName "governance.status" -Arguments @{} -Id 13
$finalContent = ($finalGov.result.content | Where-Object { $_.type -eq "text" }).text | ConvertFrom-Json

Write-Host ""
Write-Host "  ╔════════════════════════════════════════════════════════════════╗" -ForegroundColor Green
Write-Host "  ║                     SYSTEM VITAL SIGNS                         ║" -ForegroundColor Green
Write-Host "  ╠════════════════════════════════════════════════════════════════╣" -ForegroundColor Green
Write-Host "  ║  Drift OK:      $($finalContent.drift_ok.ToString().PadRight(43))║" -ForegroundColor Green
Write-Host "  ║  Energy Drift:  $($finalContent.energy_drift.ToString().PadRight(43))║" -ForegroundColor Green
Write-Host "  ║  Coherence:     $($finalContent.coherence.ToString().PadRight(43))║" -ForegroundColor Green
Write-Host "  ║  Node Count:    $($finalContent.node_count.ToString().PadRight(43))║" -ForegroundColor Green
Write-Host "  ║  Edge Count:    $($finalContent.edge_count.ToString().PadRight(43))║" -ForegroundColor Green
Write-Host "  ║  Healthy:       $($finalContent.healthy.ToString().PadRight(43))║" -ForegroundColor Green
Write-Host "  ╚════════════════════════════════════════════════════════════════╝" -ForegroundColor Green

Write-Status "Completed"

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
