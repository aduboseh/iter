param(
    [string]$ServerPath = ".\target\debug\scg_mcp_server.exe"
)

# -------------------------------------------------------
# WARNING ABOUT STATE
# -------------------------------------------------------
# This harness assumes you will eventually move to a
# persistent STDIO session so that all requests in a
# phase share the same in-memory graph.
#
# For now, Invoke-SCG runs each group of lines through
# a single process (per phase) by design, matching the
# way you already tested manually with:
#   $lines | .\target\debug\scg_mcp_server.exe
#
# You can later swap this to a persistent child process
# if you want fully interactive, stateful multi-phase runs.
# -------------------------------------------------------

function Invoke-SCGBatch {
    param(
        [string[]]$JsonLines,
        [string]$Label = "batch"
    )

    Write-Host ">> Running batch: $Label" -ForegroundColor DarkCyan

    # Pipe all JSON-RPC lines into one server process
    $outputLines = $JsonLines | & $ServerPath 2>$null

    if ($LASTEXITCODE -ne 0) {
        Write-Host "Error running SCG server for batch $Label" -ForegroundColor Red
        return @()
    }

    if (-not $outputLines) {
        Write-Host "No output received for batch $Label" -ForegroundColor Yellow
        return @()
    }

    # Join into a single string for parsing
    $outText = $outputLines -join "`n"

    # Very simple "object splitter":
    # This finds each top-level JSON object via regex.
    $regex = '\{(?:[^{}]|(?<open>\{)|(?<-open>\}))*\}(?(open)(?!))'
    $matches = [System.Text.RegularExpressions.Regex]::Matches($outText, $regex)

    $parsed = @()
    foreach ($m in $matches) {
        try {
            $obj = $m.Value | ConvertFrom-Json
            $parsed += $obj
        } catch {
            Write-Host "Failed to parse JSON object in ${Label}:" -ForegroundColor Red
            Write-Host $m.Value
        }
    }

    return $parsed
}

# Global store
$results = @{}

# Helper to record and print
function Store-Result {
    param(
        [int]$Id,
        $Obj
    )
    $script:results[$Id] = $Obj
}

# =====================================================================
# PHASE 0 — PRE-FLIGHT SANITY (1–2 tests)
# =====================================================================

Write-Host "===== PHASE 0: Pre-Flight Sanity =====" -ForegroundColor Cyan

$phase0Lines = @(
    '{"jsonrpc":"2.0","id":0,"method":"governor.status","params":{}}'
)

$phase0 = Invoke-SCGBatch -JsonLines $phase0Lines -Label "phase0"

foreach ($obj in $phase0) {
    Store-Result -Id $obj.id -Obj $obj
}

Write-Host "Governor Baseline:" -ForegroundColor Yellow
$results[0] | Format-List

# =====================================================================
# PHASE 1 — NODE LIFECYCLE & BOUNDARY BELIEFS (approx 6–8 tests)
# =====================================================================

Write-Host "`n===== PHASE 1: Node Lifecycle =====" -ForegroundColor Cyan

$beliefs = @(0.5, 0.0, 1.0, 0.001, 0.999)
$phase1Lines = @()
$idCounter = 1

foreach ($b in $beliefs) {
    $line = '{"jsonrpc":"2.0","id":' + $idCounter + ',"method":"node.create","params":{"belief":' + $b + ',"energy":1.0}}'
    $phase1Lines += $line
    $idCounter++
}

# Governor check after creation
$phase1Lines += '{"jsonrpc":"2.0","id":11,"method":"governor.status","params":{}}'

$phase1 = Invoke-SCGBatch -JsonLines $phase1Lines -Label "phase1"

$nodeIds = @{}
foreach ($obj in $phase1) {
    Store-Result -Id $obj.id -Obj $obj
    if ($obj.id -ge 1 -and $obj.id -le 5) {
        # NOTE: For the real server, node.create returns result.content[0].text
        # containing a JSON string of the NodeState. You will likely need
        # to parse that string here instead of accessing $obj.result.id.
        Write-Host "Raw node.create response (id=$($obj.id)):" -ForegroundColor DarkGray
        $obj | Format-List
    }
}

Write-Host "Governor after node creation:" -ForegroundColor Yellow
$results[11] | Format-List

# =====================================================================
# PHASE 2 — NODE MUTATION (approx 4–6 tests)
# =====================================================================

Write-Host "`n===== PHASE 2: Node Mutation =====" -ForegroundColor Cyan

# Logical scaffold only; needs persistent session to reuse node IDs.
$phase2Lines = @()
$idCounter = 20

# Re-create a baseline node A (belief 0.5)
$phase2Lines += '{"jsonrpc":"2.0","id":' + $idCounter + ',"method":"node.create","params":{"belief":0.5,"energy":1.0}}'
$nodeCreateId = $idCounter
$idCounter++

$phase2Lines += @(
    '{"jsonrpc":"2.0","id":' + $idCounter     + ',"method":"node.mutate","params":{"node_id":"$NODE_A","delta":0.01}}',
    '{"jsonrpc":"2.0","id":' + ($idCounter+1) + ',"method":"node.mutate","params":{"node_id":"$NODE_A","delta":0.99}}',
    '{"jsonrpc":"2.0","id":' + ($idCounter+2) + ',"method":"node.mutate","params":{"node_id":"$NODE_A","delta":-0.99}}'
)

Write-Host "NOTE: Phase 2 is currently a logical definition. Replace `$NODE_A with an actual UUID or convert this harness to a persistent STDIO session for fully automated chaining." -ForegroundColor DarkYellow

# =====================================================================
# PHASE 3 — EDGE BINDING & CYCLE TOPOLOGY (approx 6 tests)
# =====================================================================

Write-Host "`n===== PHASE 3: Edge Binding & Topology =====" -ForegroundColor Cyan

$phase3Lines = @(
    '{"jsonrpc":"2.0","id":40,"method":"edge.bind","params":{"src":"$NODE_A","dst":"$NODE_B","weight":0.2}}',
    '{"jsonrpc":"2.0","id":41,"method":"edge.bind","params":{"src":"$NODE_B","dst":"$NODE_C","weight":0.3}}',
    '{"jsonrpc":"2.0","id":42,"method":"edge.bind","params":{"src":"$NODE_A","dst":"$NODE_A","weight":0.1}}',
    '{"jsonrpc":"2.0","id":43,"method":"edge.bind","params":{"src":"$NODE_C","dst":"$NODE_A","weight":0.4}}'
)

Write-Host "NOTE: Phase 3 is defined logically; plug in actual UUIDs or migrate to an interactive harness." -ForegroundColor DarkYellow

# =====================================================================
# PHASE 4 — EDGE PROPAGATION (approx 4 tests)
# =====================================================================

Write-Host "`n===== PHASE 4: Edge Propagation =====" -ForegroundColor Cyan

$phase4Lines = @(
    '{"jsonrpc":"2.0","id":50,"method":"edge.propagate","params":{"edge_id":"$EDGE_AB"}}',
    '{"jsonrpc":"2.0","id":51,"method":"edge.propagate","params":{"edge_id":"$EDGE_BC"}}'
)

Write-Host "NOTE: Phase 4 expects known edge IDs $EDGE_AB, $EDGE_BC. Replace with real IDs or extend harness to capture IDs programmatically." -ForegroundColor DarkYellow

# =====================================================================
# PHASE 5 — ESV AUDITS (approx 3–5 tests)
# =====================================================================

Write-Host "`n===== PHASE 5: ESV Audits =====" -ForegroundColor Cyan

$phase5Lines = @(
    '{"jsonrpc":"2.0","id":60,"method":"esv.audit","params":{"id":"$NODE_A"}}',
    '{"jsonrpc":"2.0","id":61,"method":"esv.audit","params":{"id":"$NODE_B"}}'
)

Write-Host "NOTE: Phase 5 currently references NODE_A/NODE_B placeholders. Swap them for real UUIDs or wire up the interactive pipeline." -ForegroundColor DarkYellow

# =====================================================================
# PHASE 6 — LINEAGE REPLAY (approx 3 tests)
# =====================================================================

Write-Host "`n===== PHASE 6: Lineage Replay =====" -ForegroundColor Cyan

$phase6Lines = @(
    '{"jsonrpc":"2.0","id":70,"method":"lineage.replay","params":{"segment":{"start":0,"end":"latest"}}}',
    '{"jsonrpc":"2.0","id":71,"method":"lineage.replay","params":{"segment":{"start":5,"end":10}}}'
)

# =====================================================================
# PHASE 7 — LEDGER EXPORT & INTEGRITY (approx 3 tests)
# =====================================================================

Write-Host "`n===== PHASE 7: Ledger Export & Integrity =====" -ForegroundColor Cyan

$phase7Lines = @(
    '{"jsonrpc":"2.0","id":80,"method":"lineage.export","params":{}}'
)

# =====================================================================
# PHASE 8 — STRESS & ADVERSARIAL (approx 18 tests)
# =====================================================================

Write-Host "`n===== PHASE 8: Stress & Adversarial =====" -ForegroundColor Cyan

$stressLines = @()
$baseId = 100
for ($i = 0; $i -lt 100; $i++) {
    $belief = Get-Random -Minimum 0.0 -Maximum 1.0
    $stressLines += '{"jsonrpc":"2.0","id":' + ($baseId + $i) +
        ',"method":"node.create","params":{"belief":' + $belief + ',"energy":1.0}}'
}

$stressLines += '{"jsonrpc":"2.0","id":200,"method":"governor.status","params":{}}'

Write-Host "NOTE: Phase 8 stress block defined; you can uncomment and run once a persistent MCP/STDIO strategy is chosen." -ForegroundColor DarkYellow

# =====================================================================
# EXECUTIVE SUMMARY (CURRENT HARNESS STATE)
# =====================================================================

Write-Host "`n===== EXECUTIVE SUMMARY (Harness v2.0 STRUCTURE) =====" -ForegroundColor White
Write-Host "Phase 0: Executed (governor baseline captured)." -ForegroundColor Green
Write-Host "Phase 1: Executed (node lifecycle and boundary beliefs) structure in place; adjust parsing to use result.content[0].text JSON payloads." -ForegroundColor Green
Write-Host "Phases 2–8: Logically defined with placeholders ($NODE_A, $EDGE_AB, etc.)." -ForegroundColor Yellow
Write-Host "To convert to a fully automated 47-test runner:" -ForegroundColor White
Write-Host "  1) Replace placeholders with captured IDs via a persistent STDIO harness;" -ForegroundColor White
Write-Host "  2) Or generate templated requests programmatically in a single Rust/CLI test binary." -ForegroundColor White

Write-Host "`nCertification harness v2.0 structure complete." -ForegroundColor Green
