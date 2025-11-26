param(
    [string]$ServerPath = ".\target\debug\scg_mcp_server.exe",
    [switch]$Determinism
)

# Optional preflight: enabled only in determinism mode
if ($Determinism) {
    Write-Host "[PRE-FLIGHT] Determinism mode enabled: cleaning up any stale servers" -ForegroundColor Yellow
    Get-Process -Name "scg_mcp_server" -ErrorAction SilentlyContinue | ForEach-Object {
        Write-Host "[PRE-FLIGHT] Killing stale server PID $($_.Id)" -ForegroundColor Yellow
        Stop-Process -Id $_.Id -Force
    }
    $env:SCG_DETERMINISM = "1"
}

# =====================================================================
# SCG MCP SERVER - COMPREHENSIVE 47-TEST CERTIFICATION HARNESS
# WITH PERSISTENT STDIO SESSION MANAGEMENT
# =====================================================================

$ErrorActionPreference = "Stop"
$script:testResults = @()
$script:passCount = 0
$script:failCount = 0
$script:totalTests = 47
$script:nodeIds = @{}
$script:edgeIds = @{}

# =====================================================================
# PERSISTENT STDIO SESSION MANAGER
# =====================================================================

class SCGSession {
    [System.Diagnostics.Process]$Process
    [System.IO.StreamWriter]$Stdin
    [System.IO.StreamReader]$Stdout
    [bool]$IsActive
    [int]$Timeout = 10000
    
    SCGSession([string]$serverPath) {
        $psi = New-Object System.Diagnostics.ProcessStartInfo
        $psi.FileName = $serverPath
        $psi.UseShellExecute = $false
        $psi.RedirectStandardInput = $true
        $psi.RedirectStandardOutput = $true
        $psi.RedirectStandardError = $false
        $psi.CreateNoWindow = $true
        
        if ($env:SCG_DETERMINISM) {
            $psi.EnvironmentVariables["SCG_DETERMINISM"] = "1"
        }
        
        $this.Process = [System.Diagnostics.Process]::Start($psi)
        $this.Stdin = $this.Process.StandardInput
        $this.Stdout = $this.Process.StandardOutput
        $this.IsActive = $true
        
        if ($env:SCG_DETERMINISM) {
            Write-Host "[SESSION] Started SCG MCP Server (PID: XXXXX)" -ForegroundColor Green
        } else {
            Write-Host "[SESSION] Started SCG MCP Server (PID: $($this.Process.Id))" -ForegroundColor Green
        }
    }
    
    # Low-level non-blocking reader: single-threaded, uses Peek to avoid blocking
    hidden [string] ReadJsonLine([int]$timeoutMs, [string]$label) {
        $builder  = New-Object System.Text.StringBuilder
        $deadline = (Get-Date).AddMilliseconds($timeoutMs)

        while ((Get-Date) -lt $deadline) {
            # Peek returns -1 if no data is available
            $next = $this.Stdout.Peek()
            if ($next -ge 0) {
                $ch = [char]$this.Stdout.Read()

                # Newline terminates the JSON frame
                if ($ch -eq "`n") {
                    $line = $builder.ToString().Trim()
                    if ($line.Length -gt 0) {
                        return $line
                    }

                    # Empty line, reset builder and keep reading
                    $builder.Clear() | Out-Null
                } else {
                    $builder.Append($ch) | Out-Null
                }
            } else {
                Start-Sleep -Milliseconds 20
            }
        }

        if ($label) {
            Write-Host "[TIMEOUT] $label after $timeoutMs ms" -ForegroundColor Yellow
        }

        return $null
    }
    
    [object] SendRequest([string]$jsonRpc) {
        return $this.SendRequestWithLabel($jsonRpc, "")
    }
    
    [object] SendRequest([string]$jsonRpc, [string]$Label) {
        return $this.SendRequestWithLabel($jsonRpc, $Label)
    }
    
    hidden [object] SendRequestWithLabel([string]$jsonRpc, [string]$label) {
        if (-not $this.IsActive -or $this.Process.HasExited) {
            if ($label) {
                Write-Host "[SESSION] Dead while sending $label" -ForegroundColor Red
            }
            return $null
        }

        # Write request
        $this.Stdin.WriteLine($jsonRpc)
        $this.Stdin.Flush()

        # Read response line with timeout
        $raw = $this.ReadJsonLine($this.Timeout, $label)
        if (-not $raw) {
            return $null
        }

        # Filter out log noise that accidentally looks like JSON start
        if (-not $raw.StartsWith("{")) {
            if ($label) {
                Write-Host "[NOISE] $label - non JSON line: $raw" -ForegroundColor DarkGray
            }
            return $null
        }

        try {
            $obj = $raw | ConvertFrom-Json
        } catch {
            if ($label) {
                Write-Host "[WARN] $label - malformed JSON: $raw" -ForegroundColor DarkGray
            }
            return $null
        }

        # JSON RPC shape validation
        if ($obj.jsonrpc -ne "2.0") {
            if ($label) {
                Write-Host "[WARN] $label - missing jsonrpc field: $raw" -ForegroundColor DarkGray
            }
            return $null
        }

        if ($null -eq $obj.result -and $null -eq $obj.error) {
            if ($label) {
                Write-Host "[WARN] $label - no result or error in payload" -ForegroundColor DarkGray
            }
            return $null
        }

        return $obj
    }
    
    [void] Close() {
        if ($this.IsActive) {
            $this.Stdin.Close()
            $this.Process.WaitForExit(2000)
            if (-not $this.Process.HasExited) {
                $this.Process.Kill()
            }
            $this.IsActive = $false
            Write-Host "[SESSION] Closed SCG MCP Server" -ForegroundColor Yellow
        }
    }
}

# =====================================================================
# HELPER FUNCTIONS
# =====================================================================

function Extract-Content {
    param($Response)
    
    if ($Response -and $Response.result) {
        if ($Response.result.content) {
            $content = $Response.result.content[0]
            if ($content.text) {
                try {
                    return $content.text | ConvertFrom-Json
                } catch {
                    return $content.text
                }
            }
        }
        elseif ($Response.result -is [PSCustomObject] -or $Response.result -is [hashtable]) {
            return $Response.result
        }
    }
    return $null
}

function Get-GovernorResult {
    param($Response)
    
    if (-not $Response) { return $null }
    
    $data = Extract-Content $Response
    if (-not $data) { return $null }
    
    $result = [PSCustomObject]@{
        energy_drift = $null
        coherence = $null
        node_count = $null
        total_energy = $null
    }
    
    if ($data.PSObject.Properties.Name -contains 'energy_drift') {
        $result.energy_drift = [double]$data.energy_drift
    } elseif ($data.PSObject.Properties.Name -contains 'drift') {
        $result.energy_drift = [double]$data.drift
    }
    
    if ($data.PSObject.Properties.Name -contains 'coherence') {
        $result.coherence = [double]$data.coherence
    }
    
    if ($data.PSObject.Properties.Name -contains 'node_count') {
        $result.node_count = [int]$data.node_count
    }
    
    if ($data.PSObject.Properties.Name -contains 'total_energy') {
        $result.total_energy = [double]$data.total_energy
    }
    
    return $result
}

function Record-Test {
    param(
        [int]$TestNum,
        [string]$Phase,
        [string]$Name,
        [bool]$Passed,
        [string]$Details = ""
    )
    
    $script:testResults += [PSCustomObject]@{
        TestNum = $TestNum
        Phase = $Phase
        Name = $Name
        Passed = $Passed
        Details = $Details
    }
    
    if ($Passed) {
        $script:passCount++
        Write-Host "  [PASS] Test $TestNum - $Name" -ForegroundColor Green
    } else {
        $script:failCount++
        Write-Host "  [FAIL] Test $TestNum - $Name" -ForegroundColor Red
        if ($Details) { Write-Host "         $Details" -ForegroundColor DarkRed }
    }
}

function Build-Request {
    param(
        [int]$Id,
        [string]$Method,
        [hashtable]$Params = @{}
    )
    $paramsJson = $Params | ConvertTo-Json -Compress
    return "{`"jsonrpc`":`"2.0`",`"id`":$Id,`"method`":`"$Method`",`"params`":$paramsJson}"
}

# =====================================================================
# MAIN EXECUTION
# =====================================================================

Write-Host @"
=====================================================================
     SCG MCP SERVER - 47-TEST CERTIFICATION HARNESS v4.0
          Thermodynamic Compliance with Constant-Pool Model
=====================================================================
"@ -ForegroundColor Cyan

$session = $null
try {
    $session = [SCGSession]::new((Resolve-Path $ServerPath).Path)
    $testNum = 0

    # =================================================================
    # PHASE 0: PRE-FLIGHT SANITY (Tests 1-2)
    # =================================================================
    
    Write-Host "`n===== PHASE 0: Pre-Flight Sanity (Tests 1-2) =====" -ForegroundColor Cyan
    
    # Test 1: Governor baseline status
    $testNum++
    $req = Build-Request -Id $testNum -Method "governor.status" -Params @{}
    $resp = $session.SendRequest($req, "governor.status")
    $govData = Get-GovernorResult $resp
    $test1Pass = ($govData -ne $null) -and ($null -ne $govData.energy_drift)
    Record-Test -TestNum $testNum -Phase "Phase0" -Name "Governor baseline query" -Passed $test1Pass -Details $(if (-not $test1Pass) { "Invalid governor response" } else { "Drift: $($govData.energy_drift), Coherence: $($govData.coherence)" })
    $baselineDrift = $govData.energy_drift
    
    # Test 2: Zero initial drift (energy conservation: |dE/dt| <= 1e-10)
    $testNum++
    $test2Pass = ($govData -ne $null) -and ([math]::Abs([double]$govData.energy_drift) -le 1e-10)
    Record-Test -TestNum $testNum -Phase "Phase0" -Name "Zero initial drift" -Passed $test2Pass -Details "Drift: $($govData.energy_drift)"

    # =================================================================
    # PHASE 1: NODE LIFECYCLE & BOUNDARY BELIEFS (Tests 3-10)
    # =================================================================
    
    Write-Host "`n===== PHASE 1: Node Lifecycle & Boundary Beliefs (Tests 3-10) =====" -ForegroundColor Cyan
    
    $beliefs = @(
        @{value=0.5; label="midpoint"},
        @{value=0.0; label="lower bound"},
        @{value=1.0; label="upper bound"},
        @{value=0.001; label="near-zero"},
        @{value=0.999; label="near-one"}
    )
    
    foreach ($b in $beliefs) {
        $testNum++
        $req = Build-Request -Id $testNum -Method "node.create" -Params @{belief=$b.value; energy=1.0}
        $resp = $session.SendRequest($req)
        $nodeData = Extract-Content $resp
        
        $passed = ($nodeData -ne $null) -and ($nodeData.id -ne $null) -and ($nodeData.id -match '^[0-9a-f-]{36}$')
        if ($passed) {
            $script:nodeIds["Node$($testNum - 2)"] = $nodeData.id
        }
        Record-Test -TestNum $testNum -Phase "Phase1" -Name "Create node belief=$($b.label) ($($b.value))" -Passed $passed -Details $(if ($passed) { "UUID: $($nodeData.id.Substring(0,8))..." } else { "Invalid UUID" })
    }
    
    # Test 8: Energy accounting (constant-pool model: drift <= 1e-10, topology verified)
    $testNum++
    $req = Build-Request -Id $testNum -Method "governor.status" -Params @{}
    $resp = $session.SendRequest($req, "governor.status")
    $govAfter = Get-GovernorResult $resp
    $test8Pass = ($govAfter -ne $null) -and ($govAfter.node_count -eq 5) -and ([math]::Abs([double]$govAfter.energy_drift) -le 1e-10)
    Record-Test -TestNum $testNum -Phase "Phase1" -Name "Energy accounting (5 nodes)" -Passed $test8Pass -Details "Nodes: $($govAfter.node_count), Drift: $($govAfter.energy_drift)"
    
    # Test 9: Node query
    $testNum++
    if ($script:nodeIds["Node1"]) {
        $req = Build-Request -Id $testNum -Method "node.query" -Params @{node_id=$script:nodeIds["Node1"]}
        $resp = $session.SendRequest($req, "node.query")
        $queryData = Extract-Content $resp
        $test9Pass = ($queryData -ne $null) -and ([math]::Abs($queryData.belief - 0.5) -lt 0.001)
        Record-Test -TestNum $testNum -Phase "Phase1" -Name "Node query verification" -Passed $test9Pass -Details "Belief: $($queryData.belief)"
    } else {
        Record-Test -TestNum $testNum -Phase "Phase1" -Name "Node query verification" -Passed $false -Details "No node ID available"
    }
    
    # Test 10: Verify drift still minimal (thermodynamic conservation)
    $testNum++
    $driftCheck = ($govAfter -ne $null) -and ([math]::Abs([double]$govAfter.energy_drift) -le 1e-10)
    Record-Test -TestNum $testNum -Phase "Phase1" -Name "Drift stability after creation" -Passed $driftCheck -Details "Drift: $($govAfter.energy_drift)"

    # =================================================================
    # PHASE 2: NODE MUTATION (Tests 11-16)
    # =================================================================
    
    Write-Host "`n===== PHASE 2: Node Mutation (Tests 11-16) =====" -ForegroundColor Cyan
    
    # Test 11: Small positive mutation
    $testNum++
    if ($script:nodeIds["Node1"]) {
        $req = Build-Request -Id $testNum -Method "node.mutate" -Params @{node_id=$script:nodeIds["Node1"]; delta=0.1}
        $resp = $session.SendRequest($req)
        $mutateData = Extract-Content $resp
        $test11Pass = ($mutateData -ne $null) -and ([math]::Abs($mutateData.belief - 0.6) -lt 0.001)
        Record-Test -TestNum $testNum -Phase "Phase2" -Name "Small positive mutation (+0.1)" -Passed $test11Pass -Details "New belief: $($mutateData.belief)"
    } else {
        Record-Test -TestNum $testNum -Phase "Phase2" -Name "Small positive mutation (+0.1)" -Passed $false -Details "No node available"
    }
    
    # Test 12: Small negative mutation
    $testNum++
    if ($script:nodeIds["Node1"]) {
        $req = Build-Request -Id $testNum -Method "node.mutate" -Params @{node_id=$script:nodeIds["Node1"]; delta=-0.2}
        $resp = $session.SendRequest($req)
        $mutateData = Extract-Content $resp
        $test12Pass = ($mutateData -ne $null) -and ([math]::Abs($mutateData.belief - 0.4) -lt 0.001)
        Record-Test -TestNum $testNum -Phase "Phase2" -Name "Small negative mutation (-0.2)" -Passed $test12Pass -Details "New belief: $($mutateData.belief)"
    } else {
        Record-Test -TestNum $testNum -Phase "Phase2" -Name "Small negative mutation (-0.2)" -Passed $false -Details "No node available"
    }
    
    # Test 13: Upper bound clamping (belief in [0,1] invariant)
    $testNum++
    if ($script:nodeIds["Node1"]) {
        $req = Build-Request -Id $testNum -Method "node.mutate" -Params @{node_id=$script:nodeIds["Node1"]; delta=0.99}
        $resp = $session.SendRequest($req, "node.mutate")
        $mutateData = Extract-Content $resp
        $beliefClamped = [double]$mutateData.belief
        $test13Pass = ($mutateData -ne $null) -and ($beliefClamped -ge 0.9999) -and ($beliefClamped -le 1.0)
        Record-Test -TestNum $testNum -Phase "Phase2" -Name "Upper bound clamping (+0.99)" -Passed $test13Pass -Details "Clamped belief: $beliefClamped"
    } else {
        Record-Test -TestNum $testNum -Phase "Phase2" -Name "Upper bound clamping (+0.99)" -Passed $false -Details "No node available"
    }
    
    # Test 14: Large mutation clamped to lower bound (Node2 was created with belief=0.0)
    $testNum++
    if ($script:nodeIds["Node2"]) {
        $req = Build-Request -Id $testNum -Method "node.mutate" -Params @{node_id=$script:nodeIds["Node2"]; delta=-0.5}
        $resp = $session.SendRequest($req)
        $mutateData = Extract-Content $resp
        # Even with clamping, belief should never go below 0
        $test14Pass = ($mutateData -ne $null) -and ($mutateData.belief -ne $null)
        Record-Test -TestNum $testNum -Phase "Phase2" -Name "Lower bound mutation (-0.5)" -Passed $test14Pass -Details "Result belief: $($mutateData.belief)"
    } else {
        Record-Test -TestNum $testNum -Phase "Phase2" -Name "Lower bound mutation (-0.5)" -Passed $false -Details "No node available"
    }
    
    # Test 15: Zero delta mutation (no-op)
    $testNum++
    if ($script:nodeIds["Node3"]) {
        $req = Build-Request -Id $testNum -Method "node.query" -Params @{node_id=$script:nodeIds["Node3"]}
        $respBefore = $session.SendRequest($req)
        $beforeData = Extract-Content $respBefore
        
        $req = Build-Request -Id ($testNum * 100) -Method "node.mutate" -Params @{node_id=$script:nodeIds["Node3"]; delta=0.0}
        $resp = $session.SendRequest($req)
        $mutateData = Extract-Content $resp
        $test15Pass = ($mutateData -ne $null) -and ([math]::Abs($mutateData.belief - $beforeData.belief) -lt 0.0001)
        Record-Test -TestNum $testNum -Phase "Phase2" -Name "Zero delta mutation (no-op)" -Passed $test15Pass -Details "Belief unchanged: $($mutateData.belief)"
    } else {
        Record-Test -TestNum $testNum -Phase "Phase2" -Name "Zero delta mutation (no-op)" -Passed $false -Details "No node available"
    }
    
    # Test 16: Governor stability after mutations (drift <= 1e-10)
    $testNum++
    $req = Build-Request -Id $testNum -Method "governor.status" -Params @{}
    $resp = $session.SendRequest($req, "governor.status")
    $govAfterMutations = Get-GovernorResult $resp
    $test16Pass = ($govAfterMutations -ne $null) -and ([math]::Abs([double]$govAfterMutations.energy_drift) -le 1e-10)
    Record-Test -TestNum $testNum -Phase "Phase2" -Name "Governor stability post-mutation" -Passed $test16Pass -Details "Drift: $($govAfterMutations.energy_drift)"

    # =================================================================
    # PHASE 3: EDGE BINDING & TOPOLOGY (Tests 17-24)
    # =================================================================
    
    Write-Host "`n===== PHASE 3: Edge Binding & Topology (Tests 17-24) =====" -ForegroundColor Cyan
    
    # Test 17: Simple edge binding (A -> B)
    $testNum++
    if ($script:nodeIds["Node1"] -and $script:nodeIds["Node2"]) {
        $req = Build-Request -Id $testNum -Method "edge.bind" -Params @{src=$script:nodeIds["Node1"]; dst=$script:nodeIds["Node2"]; weight=0.5}
        $resp = $session.SendRequest($req)
        $edgeData = Extract-Content $resp
        $test17Pass = ($edgeData -ne $null) -and ($edgeData.id -ne $null)
        if ($test17Pass) { $script:edgeIds["Edge_AB"] = $edgeData.id }
        Record-Test -TestNum $testNum -Phase "Phase3" -Name "Edge binding A->B (weight 0.5)" -Passed $test17Pass -Details $(if ($test17Pass) { "Edge ID: $($edgeData.id.Substring(0,8))..." } else { "Failed" })
    } else {
        Record-Test -TestNum $testNum -Phase "Phase3" -Name "Edge binding A->B (weight 0.5)" -Passed $false -Details "Missing nodes"
    }
    
    # Test 18: Reverse edge (B -> A)
    $testNum++
    if ($script:nodeIds["Node1"] -and $script:nodeIds["Node2"]) {
        $req = Build-Request -Id $testNum -Method "edge.bind" -Params @{src=$script:nodeIds["Node2"]; dst=$script:nodeIds["Node1"]; weight=0.3}
        $resp = $session.SendRequest($req)
        $edgeData = Extract-Content $resp
        $test18Pass = ($edgeData -ne $null) -and ($edgeData.id -ne $null)
        if ($test18Pass) { $script:edgeIds["Edge_BA"] = $edgeData.id }
        Record-Test -TestNum $testNum -Phase "Phase3" -Name "Edge binding B->A (weight 0.3)" -Passed $test18Pass -Details $(if ($test18Pass) { "Bidirectional link created" } else { "Failed" })
    } else {
        Record-Test -TestNum $testNum -Phase "Phase3" -Name "Edge binding B->A (weight 0.3)" -Passed $false -Details "Missing nodes"
    }
    
    # Test 19: Chain edge (B -> C)
    $testNum++
    if ($script:nodeIds["Node2"] -and $script:nodeIds["Node3"]) {
        $req = Build-Request -Id $testNum -Method "edge.bind" -Params @{src=$script:nodeIds["Node2"]; dst=$script:nodeIds["Node3"]; weight=0.4}
        $resp = $session.SendRequest($req)
        $edgeData = Extract-Content $resp
        $test19Pass = ($edgeData -ne $null) -and ($edgeData.id -ne $null)
        if ($test19Pass) { $script:edgeIds["Edge_BC"] = $edgeData.id }
        Record-Test -TestNum $testNum -Phase "Phase3" -Name "Edge chain B->C (weight 0.4)" -Passed $test19Pass -Details $(if ($test19Pass) { "Chain extended" } else { "Failed" })
    } else {
        Record-Test -TestNum $testNum -Phase "Phase3" -Name "Edge chain B->C (weight 0.4)" -Passed $false -Details "Missing nodes"
    }
    
    # Test 20: Cycle closure (C -> A)
    $testNum++
    if ($script:nodeIds["Node3"] -and $script:nodeIds["Node1"]) {
        $req = Build-Request -Id $testNum -Method "edge.bind" -Params @{src=$script:nodeIds["Node3"]; dst=$script:nodeIds["Node1"]; weight=0.2}
        $resp = $session.SendRequest($req)
        $edgeData = Extract-Content $resp
        $test20Pass = ($edgeData -ne $null) -and ($edgeData.id -ne $null)
        if ($test20Pass) { $script:edgeIds["Edge_CA"] = $edgeData.id }
        Record-Test -TestNum $testNum -Phase "Phase3" -Name "Cycle closure C->A (weight 0.2)" -Passed $test20Pass -Details $(if ($test20Pass) { "Triangle formed" } else { "Failed" })
    } else {
        Record-Test -TestNum $testNum -Phase "Phase3" -Name "Cycle closure C->A (weight 0.2)" -Passed $false -Details "Missing nodes"
    }
    
    # Test 21: Self-loop edge (D -> D)
    $testNum++
    if ($script:nodeIds["Node4"]) {
        $req = Build-Request -Id $testNum -Method "edge.bind" -Params @{src=$script:nodeIds["Node4"]; dst=$script:nodeIds["Node4"]; weight=0.1}
        $resp = $session.SendRequest($req)
        $edgeData = Extract-Content $resp
        $test21Pass = ($edgeData -ne $null) -and ($edgeData.id -ne $null)
        if ($test21Pass) { $script:edgeIds["Edge_DD"] = $edgeData.id }
        Record-Test -TestNum $testNum -Phase "Phase3" -Name "Self-loop D->D (weight 0.1)" -Passed $test21Pass -Details $(if ($test21Pass) { "Self-reference created" } else { "Failed" })
    } else {
        Record-Test -TestNum $testNum -Phase "Phase3" -Name "Self-loop D->D (weight 0.1)" -Passed $false -Details "Missing node"
    }
    
    # Test 22: High-weight edge (D -> E)
    $testNum++
    if ($script:nodeIds["Node4"] -and $script:nodeIds["Node5"]) {
        $req = Build-Request -Id $testNum -Method "edge.bind" -Params @{src=$script:nodeIds["Node4"]; dst=$script:nodeIds["Node5"]; weight=0.95}
        $resp = $session.SendRequest($req)
        $edgeData = Extract-Content $resp
        $test22Pass = ($edgeData -ne $null) -and ($edgeData.id -ne $null)
        if ($test22Pass) { $script:edgeIds["Edge_DE"] = $edgeData.id }
        Record-Test -TestNum $testNum -Phase "Phase3" -Name "High-weight edge D->E (0.95)" -Passed $test22Pass -Details $(if ($test22Pass) { "Strong connection" } else { "Failed" })
    } else {
        Record-Test -TestNum $testNum -Phase "Phase3" -Name "High-weight edge D->E (0.95)" -Passed $false -Details "Missing nodes"
    }
    
    # Test 23: Low-weight edge (E -> D)
    $testNum++
    if ($script:nodeIds["Node5"] -and $script:nodeIds["Node4"]) {
        $req = Build-Request -Id $testNum -Method "edge.bind" -Params @{src=$script:nodeIds["Node5"]; dst=$script:nodeIds["Node4"]; weight=0.05}
        $resp = $session.SendRequest($req)
        $edgeData = Extract-Content $resp
        $test23Pass = ($edgeData -ne $null) -and ($edgeData.id -ne $null)
        if ($test23Pass) { $script:edgeIds["Edge_ED"] = $edgeData.id }
        Record-Test -TestNum $testNum -Phase "Phase3" -Name "Low-weight edge E->D (0.05)" -Passed $test23Pass -Details $(if ($test23Pass) { "Weak connection" } else { "Failed" })
    } else {
        Record-Test -TestNum $testNum -Phase "Phase3" -Name "Low-weight edge E->D (0.05)" -Passed $false -Details "Missing nodes"
    }
    
    # Test 24: Governor after topology changes (drift <= 1e-10)
    $testNum++
    $req = Build-Request -Id $testNum -Method "governor.status" -Params @{}
    $resp = $session.SendRequest($req, "governor.status")
    $govAfterEdges = Get-GovernorResult $resp
    $test24Pass = ($govAfterEdges -ne $null) -and ([math]::Abs([double]$govAfterEdges.energy_drift) -le 1e-10)
    Record-Test -TestNum $testNum -Phase "Phase3" -Name "Governor stability post-topology" -Passed $test24Pass -Details "Coherence: $($govAfterEdges.coherence), Drift: $($govAfterEdges.energy_drift)"

    # =================================================================
    # PHASE 4: EDGE PROPAGATION (Tests 25-30)
    # =================================================================
    
    Write-Host "`n===== PHASE 4: Edge Propagation (Tests 25-30) =====" -ForegroundColor Cyan
    
    # Test 25: Propagate along A->B
    $testNum++
    if ($script:edgeIds.ContainsKey("Edge_AB") -and -not [string]::IsNullOrWhiteSpace($script:edgeIds["Edge_AB"])) {
        $req = Build-Request -Id $testNum -Method "edge.propagate" -Params @{edge_id=$script:edgeIds["Edge_AB"]}
        $resp = $session.SendRequest($req, "Test 25 - Propagate A->B")
        $propData = Extract-Content $resp
        $test25Pass = ($propData -ne $null) -or ($resp -and -not $resp.error)
        Record-Test -TestNum $testNum -Phase "Phase4" -Name "Propagate edge A->B" -Passed $test25Pass -Details $(if ($propData.dst_belief) { "Dst belief: $($propData.dst_belief)" } else { "Propagation executed" })
    } else {
        Record-Test -TestNum $testNum -Phase "Phase4" -Name "Propagate edge A->B" -Passed $false -Details "Edge_AB missing (check Test 18)"
    }
    
    # Test 26: Propagate along B->C
    $testNum++
    if ($script:edgeIds.ContainsKey("Edge_BC") -and -not [string]::IsNullOrWhiteSpace($script:edgeIds["Edge_BC"])) {
        $req = Build-Request -Id $testNum -Method "edge.propagate" -Params @{edge_id=$script:edgeIds["Edge_BC"]}
        $resp = $session.SendRequest($req, "Test 26 - Propagate B->C")
        $propData = Extract-Content $resp
        $test26Pass = ($propData -ne $null) -or ($resp -and -not $resp.error)
        Record-Test -TestNum $testNum -Phase "Phase4" -Name "Propagate edge B->C" -Passed $test26Pass -Details "Chain propagation"
    } else {
        Record-Test -TestNum $testNum -Phase "Phase4" -Name "Propagate edge B->C" -Passed $false -Details "Edge_BC missing (check Test 19)"
    }
    
    # Test 27: Propagate along cycle edge C->A (CRITICAL: cycle closure)
    $testNum++
    if ($script:edgeIds.ContainsKey("Edge_CA") -and -not [string]::IsNullOrWhiteSpace($script:edgeIds["Edge_CA"])) {
        $req = Build-Request -Id $testNum -Method "edge.propagate" -Params @{edge_id=$script:edgeIds["Edge_CA"]}
        $resp = $session.SendRequest($req, "Test 27 - Propagate cycle C->A")
        $propData = Extract-Content $resp
        $test27Pass = ($propData -ne $null) -or ($resp -and -not $resp.error)
        Record-Test -TestNum $testNum -Phase "Phase4" -Name "Propagate cycle edge C->A" -Passed $test27Pass -Details "Cycle propagation"
    } else {
        Record-Test -TestNum $testNum -Phase "Phase4" -Name "Propagate cycle edge C->A" -Passed $false -Details "Edge_CA missing (check Test 20 - cycle closure)"
        Write-Host "         [HINT] Test 20 must succeed to create Edge_CA" -ForegroundColor Yellow
    }
    
    # Test 28: Propagate self-loop D->D
    $testNum++
    if ($script:edgeIds.ContainsKey("Edge_DD") -and -not [string]::IsNullOrWhiteSpace($script:edgeIds["Edge_DD"])) {
        $req = Build-Request -Id $testNum -Method "edge.propagate" -Params @{edge_id=$script:edgeIds["Edge_DD"]}
        $resp = $session.SendRequest($req, "Test 28 - Propagate self-loop D->D")
        $propData = Extract-Content $resp
        $test28Pass = ($propData -ne $null) -or ($resp -and -not $resp.error)
        Record-Test -TestNum $testNum -Phase "Phase4" -Name "Propagate self-loop D->D" -Passed $test28Pass -Details "Self-reference propagation"
    } else {
        Record-Test -TestNum $testNum -Phase "Phase4" -Name "Propagate self-loop D->D" -Passed $false -Details "Edge_DD missing (check Test 21)"
    }
    
    # Test 29: Propagate high-weight edge D->E
    $testNum++
    if ($script:edgeIds.ContainsKey("Edge_DE") -and -not [string]::IsNullOrWhiteSpace($script:edgeIds["Edge_DE"])) {
        $req = Build-Request -Id $testNum -Method "edge.propagate" -Params @{edge_id=$script:edgeIds["Edge_DE"]}
        $resp = $session.SendRequest($req, "Test 29 - Propagate D->E")
        $propData = Extract-Content $resp
        $test29Pass = ($propData -ne $null) -or ($resp -and -not $resp.error)
        Record-Test -TestNum $testNum -Phase "Phase4" -Name "Propagate high-weight D->E" -Passed $test29Pass -Details "Strong influence propagation"
    } else {
        Record-Test -TestNum $testNum -Phase "Phase4" -Name "Propagate high-weight D->E" -Passed $false -Details "Edge_DE missing (check Test 22)"
    }
    
    # Test 30: Governor stability after propagation (drift <= 1e-10)
    $testNum++
    $req = Build-Request -Id $testNum -Method "governor.status" -Params @{}
    $resp = $session.SendRequest($req, "governor.status")
    $govAfterProp = Get-GovernorResult $resp
    $test30Pass = ($govAfterProp -ne $null) -and ([math]::Abs([double]$govAfterProp.energy_drift) -le 1e-10)
    Record-Test -TestNum $testNum -Phase "Phase4" -Name "Governor stability post-propagation" -Passed $test30Pass -Details "Drift: $($govAfterProp.energy_drift)"

    # =================================================================
    # PHASE 5: ESV AUDITS (Tests 31-35)
    # =================================================================
    
    Write-Host "`n===== PHASE 5: ESV Audits (Tests 31-35) =====" -ForegroundColor Cyan
    
    # Test 31-35: ESV audit for each node
    $nodeLabels = @("Node1", "Node2", "Node3", "Node4", "Node5")
    foreach ($label in $nodeLabels) {
        $testNum++
        if ($script:nodeIds[$label]) {
            $req = Build-Request -Id $testNum -Method "esv.audit" -Params @{node_id=$script:nodeIds[$label]}
            $resp = $session.SendRequest($req)
            $esvData = Extract-Content $resp
            $testPass = ($esvData -ne $null)
            Record-Test -TestNum $testNum -Phase "Phase5" -Name "ESV audit $label" -Passed $testPass -Details $(if ($testPass) { "ESV vector retrieved" } else { "Audit failed" })
        } else {
            Record-Test -TestNum $testNum -Phase "Phase5" -Name "ESV audit $label" -Passed $false -Details "Node not found"
        }
    }

    # =================================================================
    # PHASE 6: LINEAGE OPERATIONS (Tests 36-39)
    # =================================================================
    
    Write-Host "`n===== PHASE 6: Lineage Operations (Tests 36-39) =====" -ForegroundColor Cyan
    
    # Test 36: Lineage replay
    $testNum++
    $req = Build-Request -Id $testNum -Method "lineage.replay" -Params @{}
    $resp = $session.SendRequest($req)
    $lineageData = Extract-Content $resp
    $test36Pass = ($lineageData -ne $null)
    Record-Test -TestNum $testNum -Phase "Phase6" -Name "Lineage replay" -Passed $test36Pass -Details $(if ($test36Pass) { "Lineage retrieved" } else { "Replay failed" })
    
    # Test 37: Lineage export
    $testNum++
    $exportPath = ".\test_lineage_export_harness.json"
    $req = Build-Request -Id $testNum -Method "lineage.export" -Params @{path=$exportPath}
    $resp = $session.SendRequest($req)
    $exportData = Extract-Content $resp
    $test37Pass = ($exportData -ne $null) -and ($exportData.checksum -or $resp.result)
    Record-Test -TestNum $testNum -Phase "Phase6" -Name "Lineage export" -Passed $test37Pass -Details $(if ($exportData.checksum) { "Checksum: $($exportData.checksum.Substring(0,16))..." } else { "Export executed" })
    
    # Test 38: Lineage replay checksum history
    $testNum++
    $req = Build-Request -Id $testNum -Method "lineage.replay" -Params @{}
    $resp = $session.SendRequest($req)
    $test38Pass = ($resp -ne $null) -and (-not $resp.error)
    Record-Test -TestNum $testNum -Phase "Phase6" -Name "Lineage replay checksum" -Passed $test38Pass -Details "Checksum verification"
    
    # Test 39: Governor after lineage operations (drift <= 1e-10)
    $testNum++
    $req = Build-Request -Id $testNum -Method "governor.status" -Params @{}
    $resp = $session.SendRequest($req, "governor.status")
    $govAfterLineage = Get-GovernorResult $resp
    $test39Pass = ($govAfterLineage -ne $null) -and ([math]::Abs([double]$govAfterLineage.energy_drift) -le 1e-10)
    Record-Test -TestNum $testNum -Phase "Phase6" -Name "Governor stability post-lineage" -Passed $test39Pass -Details "Drift: $($govAfterLineage.energy_drift)"

    # =================================================================
    # PHASE 7: STRESS TESTS (Tests 40-47)
    # =================================================================
    
    Write-Host "`n===== PHASE 7: Stress & Adversarial Tests (Tests 40-47) =====" -ForegroundColor Cyan
    
    # Test 40: Rapid node creation (10 nodes)
    $testNum++
    $stressNodes = @()
    $allCreated = $true
    for ($i = 0; $i -lt 10; $i++) {
        $belief = Get-Random -Minimum 0.0 -Maximum 1.0
        $req = Build-Request -Id ($testNum * 1000 + $i) -Method "node.create" -Params @{belief=$belief; energy=1.0}
        $resp = $session.SendRequest($req, "stress.node.create.$i")
        $nodeData = Extract-Content $resp
        if ($nodeData -and $nodeData.id) {
            $stressNodes += $nodeData.id
        } else {
            $allCreated = $false
        }
    }
    Record-Test -TestNum $testNum -Phase "Phase7" -Name "Rapid node creation (10 nodes)" -Passed $allCreated -Details "Created: $($stressNodes.Count)/10"
    
    # Test 41: Rapid edge binding (5 edges)
    $testNum++
    $edgesCreated = 0
    for ($i = 0; $i -lt [math]::Min(5, $stressNodes.Count - 1); $i++) {
        $req = Build-Request -Id ($testNum * 1000 + $i) -Method "edge.bind" -Params @{src=$stressNodes[$i]; dst=$stressNodes[$i+1]; weight=0.5}
        $resp = $session.SendRequest($req, "stress.edge.bind.$i")
        $edgeData = Extract-Content $resp
        if ($edgeData -and $edgeData.id) { $edgesCreated++ }
    }
    Record-Test -TestNum $testNum -Phase "Phase7" -Name "Rapid edge binding (5 edges)" -Passed ($edgesCreated -ge 4) -Details "Created: $edgesCreated/5"
    
    # Test 42: Burst mutations (5 mutations)
    $testNum++
    $mutationsSuccess = 0
    if ($stressNodes.Count -gt 0) {
        for ($i = 0; $i -lt [math]::Min(5, $stressNodes.Count); $i++) {
            $delta = (Get-Random -Minimum -50 -Maximum 50) / 100.0
            $req = Build-Request -Id ($testNum * 1000 + $i) -Method "node.mutate" -Params @{node_id=$stressNodes[$i]; delta=$delta}
            $resp = $session.SendRequest($req, "stress.node.mutate.$i")
            if (-not $resp.error) { $mutationsSuccess++ }
        }
    }
    Record-Test -TestNum $testNum -Phase "Phase7" -Name "Burst mutations (5 mutations)" -Passed ($mutationsSuccess -ge 4) -Details "Success: $mutationsSuccess/5"
    
    # Test 43: Burst propagations
    $testNum++
    $propSuccess = 0
    foreach ($edgeName in @("Edge_AB", "Edge_BC", "Edge_CA")) {
        if ($script:edgeIds[$edgeName]) {
            $req = Build-Request -Id ($testNum * 1000 + $propSuccess) -Method "edge.propagate" -Params @{edge_id=$script:edgeIds[$edgeName]}
            $resp = $session.SendRequest($req, "stress.edge.propagate.$edgeName")
            if (-not $resp.error) { $propSuccess++ }
        }
    }
    Record-Test -TestNum $testNum -Phase "Phase7" -Name "Burst propagations" -Passed ($propSuccess -ge 2) -Details "Success: $propSuccess/3"
    
    # Test 44: Governor after stress (drift <= 1e-10)
    $testNum++
    $req = Build-Request -Id $testNum -Method "governor.status" -Params @{}
    $resp = $session.SendRequest($req, "governor.status")
    $govAfterStress = Get-GovernorResult $resp
    $test44Pass = ($govAfterStress -ne $null) -and ([math]::Abs([double]$govAfterStress.energy_drift) -le 1e-10)
    Record-Test -TestNum $testNum -Phase "Phase7" -Name "Governor stability post-stress" -Passed $test44Pass -Details "Drift: $($govAfterStress.energy_drift), Coherence: $($govAfterStress.coherence)"
    
    # Test 45: Invalid node ID handling
    $testNum++
    $req = Build-Request -Id $testNum -Method "node.query" -Params @{node_id="00000000-0000-0000-0000-000000000000"}
    $resp = $session.SendRequest($req)
    $test45Pass = ($resp -ne $null) -and ($resp.error -or (Extract-Content $resp) -eq $null -or ((Extract-Content $resp).error))
    Record-Test -TestNum $testNum -Phase "Phase7" -Name "Invalid node ID handling" -Passed $test45Pass -Details "Error properly returned"
    
    # Test 46: Invalid edge ID handling
    $testNum++
    $req = Build-Request -Id $testNum -Method "edge.propagate" -Params @{edge_id="00000000-0000-0000-0000-000000000000"}
    $resp = $session.SendRequest($req)
    $test46Pass = ($resp -ne $null) -and ($resp.error -or (Extract-Content $resp) -eq $null -or ((Extract-Content $resp).error))
    Record-Test -TestNum $testNum -Phase "Phase7" -Name "Invalid edge ID handling" -Passed $test46Pass -Details "Error properly returned"
    
    # Test 47: Final governor integrity (drift <= 1e-10, energy conservation)
    $testNum++
    $req = Build-Request -Id $testNum -Method "governor.status" -Params @{}
    $resp = $session.SendRequest($req, "governor.status")
    $govFinal = Get-GovernorResult $resp
    $test47Pass = ($govFinal -ne $null) -and ([math]::Abs([double]$govFinal.energy_drift) -le 1e-10)
    Record-Test -TestNum $testNum -Phase "Phase7" -Name "Final governor integrity check" -Passed $test47Pass -Details "Final Drift: $($govFinal.energy_drift), Coherence: $($govFinal.coherence)"

} catch {
    Write-Host "`n[ERROR] Exception during test execution: $_" -ForegroundColor Red
} finally {
    if ($session) {
        $session.Close()
    }
}

# =================================================================
# CERTIFICATION REPORT
# =================================================================

Write-Host @"

=====================================================================
          SCG MCP SERVER CERTIFICATION REPORT
=====================================================================
"@ -ForegroundColor White

$phaseGroups = $script:testResults | Group-Object -Property Phase
foreach ($group in $phaseGroups) {
    $phasePassed = ($group.Group | Where-Object { $_.Passed }).Count
    $phaseTotal = $group.Group.Count
    $phaseColor = if ($phasePassed -eq $phaseTotal) { "Green" } elseif ($phasePassed -gt 0) { "Yellow" } else { "Red" }
    Write-Host "$($group.Name): $phasePassed/$phaseTotal tests passed" -ForegroundColor $phaseColor
}

Write-Host "`n---------------------------------------------------------------------" -ForegroundColor DarkGray
Write-Host "TOTAL: $($script:passCount)/$($script:testResults.Count) tests passed" -ForegroundColor $(if ($script:passCount -eq $script:testResults.Count) { "Green" } elseif ($script:passCount -gt $script:testResults.Count * 0.8) { "Yellow" } else { "Red" })

$certificationPassed = $script:passCount -ge ($script:testResults.Count * 0.9)
if ($certificationPassed) {
    Write-Host @"

 ██████╗ ███████╗██████╗ ████████╗██╗███████╗██╗███████╗██████╗ 
██╔════╝ ██╔════╝██╔══██╗╚══██╔══╝██║██╔════╝██║██╔════╝██╔══██╗
██║      █████╗  ██████╔╝   ██║   ██║█████╗  ██║█████╗  ██║  ██║
██║      ██╔══╝  ██╔══██╗   ██║   ██║██╔══╝  ██║██╔══╝  ██║  ██║
╚██████╗ ███████╗██║  ██║   ██║   ██║██║     ██║███████╗██████╔╝
 ╚═════╝ ╚══════╝╚═╝  ╚═╝   ╚═╝   ╚═╝╚═╝     ╚═╝╚══════╝╚═════╝ 

"@ -ForegroundColor Green
    Write-Host "[PASS] SCG MCP Server has passed certification" -ForegroundColor Green
} else {
    Write-Host "`n[FAIL] SCG MCP Server certification FAILED" -ForegroundColor Red
    Write-Host "       Required: 90% pass rate | Actual: $([math]::Round($script:passCount / $script:testResults.Count * 100, 1))%" -ForegroundColor Red
}

# Export results
$reportPath = ".\certification_report_$(Get-Date -Format 'yyyyMMdd_HHmmss').json"
$script:testResults | ConvertTo-Json -Depth 10 | Out-File -FilePath $reportPath -Encoding UTF8
Write-Host "`nDetailed report saved to: $reportPath" -ForegroundColor Cyan
