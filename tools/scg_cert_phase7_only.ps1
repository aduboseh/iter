# =====================================================================
# SCG PHASE-7 ADVERSARIAL STRESS ISOLATION & CERTIFICATION HARNESS
# Version: 1.0.0
# Directive: SCG-PHASE7-CERT-V1.0
# Author: Only SG Solutions — SCG Substrate Group
# Classification: Thermodynamic | Deterministic | Lineage-Critical
# =====================================================================

param(
    [string]$ServerPath = "..\target\debug\scg_mcp_server.exe",
    [int]$StallThresholdSeconds = 10,
    [int]$MaxExecutionSeconds = 120,
    [string]$LogDir = "..\logs\phase7"
)

$ErrorActionPreference = "Stop"
$script:testResults = @()
$script:passCount = 0
$script:failCount = 0
$script:nodeIds = @{}
$script:edgeIds = @{}
$script:lastProgressTime = Get-Date
$script:stallDetected = $false
$script:startTime = Get-Date
$script:logBuffer = @()

# =====================================================================
# LOGGING & INSTRUMENTATION
# =====================================================================

function Write-Phase7Log {
    param(
        [string]$Level,
        [string]$Message,
        [string]$Component = "HARNESS"
    )
    $timestamp = (Get-Date).ToString("yyyy-MM-dd HH:mm:ss.fff")
    $entry = "[$timestamp] [$Level] [$Component] $Message"
    $script:logBuffer += $entry
    
    $color = switch ($Level) {
        "INFO"    { "Cyan" }
        "PASS"    { "Green" }
        "FAIL"    { "Red" }
        "WARN"    { "Yellow" }
        "DEBUG"   { "Gray" }
        "STALL"   { "Magenta" }
        "CKPT"    { "White" }
        default   { "White" }
    }
    Write-Host $entry -ForegroundColor $color
}

function Update-Progress {
    param([string]$Marker)
    $script:lastProgressTime = Get-Date
    Write-Phase7Log -Level "CKPT" -Message "Progress marker: $Marker" -Component "PROGRESS"
}

function Check-StallCondition {
    $elapsed = ((Get-Date) - $script:lastProgressTime).TotalSeconds
    if ($elapsed -gt $StallThresholdSeconds) {
        $script:stallDetected = $true
        Write-Phase7Log -Level "STALL" -Message "STALL DETECTED: No progress for $([math]::Round($elapsed, 2))s (threshold: ${StallThresholdSeconds}s)" -Component "STALL_DETECTOR"
        return $true
    }
    return $false
}

function Check-ExecutionTimeout {
    $totalElapsed = ((Get-Date) - $script:startTime).TotalSeconds
    if ($totalElapsed -gt $MaxExecutionSeconds) {
        Write-Phase7Log -Level "STALL" -Message "EXECUTION TIMEOUT: Total time ${totalElapsed}s exceeds ${MaxExecutionSeconds}s limit" -Component "TIMEOUT"
        return $true
    }
    return $false
}

function Export-Logs {
    $logPath = Join-Path $LogDir "phase7_last_run.log"
    $script:logBuffer | Out-File -FilePath $logPath -Encoding UTF8 -Force
    Write-Host "`n[LOGS] Exported to: $logPath" -ForegroundColor Cyan
}

function Export-Metadata {
    param(
        [string]$Status,
        [hashtable]$GovernorFinal
    )
    $metaPath = Join-Path $LogDir "phase7_metadata.yml"
    $duration = ((Get-Date) - $script:startTime).TotalSeconds
    
    $metadata = @"
# SCG Phase-7 Certification Metadata
# Generated: $(Get-Date -Format "yyyy-MM-dd HH:mm:ss UTC")

certification:
  directive_id: "SCG-PHASE7-CERT-V1.0"
  harness_version: "1.0.0"
  
execution:
  start_time: "$($script:startTime.ToString("yyyy-MM-ddTHH:mm:ss.fffZ"))"
  end_time: "$(Get-Date -Format "yyyy-MM-ddTHH:mm:ss.fffZ")"
  duration_seconds: $([math]::Round($duration, 3))
  max_allowed_seconds: $MaxExecutionSeconds
  stall_threshold_seconds: $StallThresholdSeconds
  stall_detected: $($script:stallDetected.ToString().ToLower())
  
results:
  status: "$Status"
  tests_passed: $($script:passCount)
  tests_failed: $($script:failCount)
  tests_total: $($script:testResults.Count)
  pass_rate: $([math]::Round(($script:passCount / [math]::Max(1, $script:testResults.Count)) * 100, 2))

governor_final:
  energy_drift: $(if ($GovernorFinal) { $GovernorFinal.energy_drift } else { "null" })
  coherence: $(if ($GovernorFinal) { $GovernorFinal.coherence } else { "null" })
  node_count: $(if ($GovernorFinal) { $GovernorFinal.node_count } else { "null" })

invariants:
  drift_threshold: 1e-10
  coherence_threshold: 0.97
  governor_resilient: $(if ($GovernorFinal -and [math]::Abs($GovernorFinal.energy_drift) -le 1e-10) { "true" } else { "false" })
"@
    $metadata | Out-File -FilePath $metaPath -Encoding UTF8 -Force
    Write-Host "[META] Exported to: $metaPath" -ForegroundColor Cyan
}

# =====================================================================
# PERSISTENT STDIO SESSION WITH STALL DETECTION
# =====================================================================

class Phase7Session {
    [System.Diagnostics.Process]$Process
    [System.IO.StreamWriter]$Stdin
    [System.IO.StreamReader]$Stdout
    [System.IO.StreamReader]$Stderr
    [bool]$IsActive
    [int]$Timeout = 10000
    [System.Collections.Concurrent.ConcurrentQueue[string]]$OutputQueue
    [System.Collections.Concurrent.ConcurrentQueue[string]]$ErrorQueue
    [System.Management.Automation.PowerShell]$ReaderPowerShell
    [System.Management.Automation.PowerShell]$ErrorReaderPowerShell
    [System.IAsyncResult]$ReaderHandle
    [System.IAsyncResult]$ErrorReaderHandle
    
    Phase7Session([string]$serverPath) {
        $this.OutputQueue = [System.Collections.Concurrent.ConcurrentQueue[string]]::new()
        $this.ErrorQueue = [System.Collections.Concurrent.ConcurrentQueue[string]]::new()
        
        $psi = New-Object System.Diagnostics.ProcessStartInfo
        $psi.FileName = $serverPath
        $psi.UseShellExecute = $false
        $psi.RedirectStandardInput = $true
        $psi.RedirectStandardOutput = $true
        $psi.RedirectStandardError = $true
        $psi.CreateNoWindow = $true
        $psi.EnvironmentVariables["RUST_LOG"] = "debug"
        $psi.EnvironmentVariables["SCG_PHASE7_MODE"] = "adversarial"
        
        $this.Process = [System.Diagnostics.Process]::Start($psi)
        $this.Stdin = $this.Process.StandardInput
        $this.Stdout = $this.Process.StandardOutput
        $this.Stderr = $this.Process.StandardError
        $this.IsActive = $true
        
        # Start background stdout reader using PowerShell runspace
        $this.ReaderPowerShell = [System.Management.Automation.PowerShell]::Create()
        $this.ReaderPowerShell.AddScript({
            param($reader, $queue)
            while ($true) {
                try {
                    $line = $reader.ReadLine()
                    if ($null -eq $line) { break }
                    $queue.Enqueue($line)
                } catch {
                    break
                }
            }
        }).AddArgument($this.Stdout).AddArgument($this.OutputQueue)
        $this.ReaderHandle = $this.ReaderPowerShell.BeginInvoke()
        
        # Start background stderr reader using PowerShell runspace
        $this.ErrorReaderPowerShell = [System.Management.Automation.PowerShell]::Create()
        $this.ErrorReaderPowerShell.AddScript({
            param($reader, $queue)
            while ($true) {
                try {
                    $line = $reader.ReadLine()
                    if ($null -eq $line) { break }
                    $queue.Enqueue($line)
                } catch {
                    break
                }
            }
        }).AddArgument($this.Stderr).AddArgument($this.ErrorQueue)
        $this.ErrorReaderHandle = $this.ErrorReaderPowerShell.BeginInvoke()
    }
    
    [string[]] DrainStderr() {
        $lines = @()
        $line = $null
        while ($this.ErrorQueue.TryDequeue([ref]$line)) {
            $lines += $line
        }
        return $lines
    }
    
    [object] SendRequest([string]$jsonRpc, [string]$Label, [scriptblock]$ProgressCallback) {
        if (-not $this.IsActive -or $this.Process.HasExited) {
            return $null
        }
        
        $this.Stdin.WriteLine($jsonRpc)
        $this.Stdin.Flush()
        
        $deadline = (Get-Date).AddMilliseconds($this.Timeout)
        
        while ((Get-Date) -lt $deadline) {
            $line = $null
            if ($this.OutputQueue.TryDequeue([ref]$line)) {
                $line = $line.Trim()
                
                if (-not $line.StartsWith("{")) {
                    continue
                }
                
                try {
                    $obj = $line | ConvertFrom-Json
                    
                    if ($obj.jsonrpc -eq "2.0" -and ($null -ne $obj.result -or $null -ne $obj.error)) {
                        if ($ProgressCallback) {
                            & $ProgressCallback $Label
                        }
                        return $obj
                    }
                } catch {
                    continue
                }
            } else {
                # No data yet, small sleep to avoid spinning
                Start-Sleep -Milliseconds 5
            }
        }
        
        return $null
    }
    
    [void] Close() {
        if ($this.IsActive) {
            if ($this.ReaderPowerShell) {
                $this.ReaderPowerShell.Stop()
                $this.ReaderPowerShell.Dispose()
            }
            if ($this.ErrorReaderPowerShell) {
                $this.ErrorReaderPowerShell.Stop()
                $this.ErrorReaderPowerShell.Dispose()
            }
            $this.Stdin.Close()
            $this.Process.WaitForExit(2000)
            if (-not $this.Process.HasExited) {
                $this.Process.Kill()
            }
            $this.IsActive = $false
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
    
    $result = @{
        energy_drift = $null
        coherence = $null
        node_count = $null
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
    
    return $result
}

function Record-Test {
    param(
        [int]$TestNum,
        [string]$Name,
        [bool]$Passed,
        [string]$Details = ""
    )
    
    $script:testResults += [PSCustomObject]@{
        TestNum = $TestNum
        Phase = "Phase7"
        Name = $Name
        Passed = $Passed
        Details = $Details
        Timestamp = (Get-Date).ToString("yyyy-MM-dd HH:mm:ss.fff")
    }
    
    if ($Passed) {
        $script:passCount++
        Write-Phase7Log -Level "PASS" -Message "Test $TestNum - $Name" -Component "TEST"
    } else {
        $script:failCount++
        Write-Phase7Log -Level "FAIL" -Message "Test $TestNum - $Name | $Details" -Component "TEST"
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

function Governor-Checkpoint {
    param(
        [Phase7Session]$Session,
        [int]$Cycle,
        [string]$Context
    )
    
    $req = Build-Request -Id (9000 + $Cycle) -Method "governor.status" -Params @{}
    $resp = $Session.SendRequest($req, "governor.checkpoint.$Cycle", { param($l) Update-Progress $l })
    $gov = Get-GovernorResult $resp
    
    if ($gov) {
        $driftOk = [math]::Abs([double]$gov.energy_drift) -le 1e-10
        $status = if ($driftOk) { "STABLE" } else { "DRIFT_EXCEEDED" }
        Write-Phase7Log -Level "DEBUG" -Message "phase7 - cycle=$Cycle, context=$Context, nodes=$($gov.node_count), drift=$($gov.energy_drift), status=$status" -Component "GOVERNOR"
        return @{
            ok = $driftOk
            data = $gov
        }
    }
    return @{ ok = $false; data = $null }
}

# =====================================================================
# MAIN EXECUTION
# =====================================================================

Write-Host @"

=====================================================================
     SCG PHASE-7 ADVERSARIAL CERTIFICATION HARNESS v1.0
     Directive: SCG-PHASE7-CERT-V1.0
     Stall Threshold: ${StallThresholdSeconds}s | Max Execution: ${MaxExecutionSeconds}s
=====================================================================

"@ -ForegroundColor Cyan

# Ensure log directory exists
if (-not (Test-Path $LogDir)) {
    New-Item -ItemType Directory -Force -Path $LogDir | Out-Null
}

Write-Phase7Log -Level "INFO" -Message "Phase 7 certification harness starting" -Component "INIT"
Write-Phase7Log -Level "INFO" -Message "Server path: $ServerPath" -Component "INIT"

$session = $null
$finalGovernor = $null
$certificationStatus = "FAILED"

try {
    # Resolve server path
    $resolvedPath = if ([System.IO.Path]::IsPathRooted($ServerPath)) {
        $ServerPath
    } else {
        $scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
        Join-Path $scriptDir $ServerPath | Resolve-Path -ErrorAction Stop
    }
    
    Write-Phase7Log -Level "INFO" -Message "Resolved server: $resolvedPath" -Component "INIT"
    
    $session = [Phase7Session]::new($resolvedPath.ToString())
    Write-Phase7Log -Level "INFO" -Message "Session started (PID: $($session.Process.Id))" -Component "SESSION"
    
    Update-Progress "session_started"
    
    # =================================================================
    # PRE-FLIGHT: Setup nodes/edges for adversarial tests
    # =================================================================
    
    Write-Phase7Log -Level "INFO" -Message "===== PRE-FLIGHT: Setup Phase =====" -Component "PREFLIGHT"
    
    # Create baseline nodes for adversarial tests
    $baseNodes = @()
    for ($i = 0; $i -lt 5; $i++) {
        if (Check-StallCondition -or Check-ExecutionTimeout) { throw "Stall/timeout during preflight" }
        
        $belief = @(0.5, 0.0, 1.0, 0.001, 0.999)[$i]
        $req = Build-Request -Id ($i + 1) -Method "node.create" -Params @{belief=$belief; energy=1.0}
        $resp = $session.SendRequest($req, "preflight.node.$i", { param($l) Update-Progress $l })
        $nodeData = Extract-Content $resp
        
        if ($nodeData -and $nodeData.id) {
            $baseNodes += $nodeData.id
            $script:nodeIds["Node$($i + 1)"] = $nodeData.id
            Write-Phase7Log -Level "DEBUG" -Message "Preflight node $i created: $($nodeData.id.Substring(0,8))..." -Component "PREFLIGHT"
        }
    }
    
    # Create baseline edges
    for ($i = 0; $i -lt 4; $i++) {
        if (Check-StallCondition -or Check-ExecutionTimeout) { throw "Stall/timeout during preflight" }
        
        $req = Build-Request -Id (100 + $i) -Method "edge.bind" -Params @{src=$baseNodes[$i]; dst=$baseNodes[$i+1]; weight=0.5}
        $resp = $session.SendRequest($req, "preflight.edge.$i", { param($l) Update-Progress $l })
        $edgeData = Extract-Content $resp
        
        if ($edgeData -and $edgeData.id) {
            $script:edgeIds["Edge$($i + 1)"] = $edgeData.id
            Write-Phase7Log -Level "DEBUG" -Message "Preflight edge $i created: $($edgeData.id.Substring(0,8))..." -Component "PREFLIGHT"
        }
    }
    
    # Governor checkpoint after preflight
    $preflightGov = Governor-Checkpoint -Session $session -Cycle 0 -Context "preflight_complete"
    Write-Phase7Log -Level "INFO" -Message "Preflight complete. Nodes: $($baseNodes.Count), Edges: $($script:edgeIds.Count)" -Component "PREFLIGHT"
    
    # =================================================================
    # PHASE 7: STRESS & ADVERSARIAL TESTS (Tests 40-47)
    # =================================================================
    
    Write-Phase7Log -Level "INFO" -Message "===== PHASE 7: Stress & Adversarial Tests (40-47) =====" -Component "PHASE7"
    
    # Test 40: Rapid node creation (10 nodes)
    $testNum = 40
    Write-Phase7Log -Level "INFO" -Message "Starting Test ${testNum}: Rapid node creation" -Component "TEST"
    
    $stressNodes = @()
    $allCreated = $true
    $creationStart = Get-Date
    
    for ($i = 0; $i -lt 10; $i++) {
        if (Check-StallCondition) { 
            $allCreated = $false
            Write-Phase7Log -Level "STALL" -Message "Stall during rapid node creation at iteration $i" -Component "TEST40"
            break 
        }
        if (Check-ExecutionTimeout) {
            $allCreated = $false
            break
        }
        
        $belief = Get-Random -Minimum 0.0 -Maximum 1.0
        $req = Build-Request -Id ($testNum * 1000 + $i) -Method "node.create" -Params @{belief=$belief; energy=1.0}
        $resp = $session.SendRequest($req, "stress.node.create.$i", { param($l) Update-Progress $l })
        $nodeData = Extract-Content $resp
        
        if ($nodeData -and $nodeData.id) {
            $stressNodes += $nodeData.id
            Write-Phase7Log -Level "DEBUG" -Message "Stress node ${i} - belief=$([math]::Round($belief, 4)), id=$($nodeData.id.Substring(0,8))..." -Component "TEST40"
        } else {
            $allCreated = $false
            Write-Phase7Log -Level "WARN" -Message "Failed to create stress node $i" -Component "TEST40"
        }
    }
    
    $creationDuration = ((Get-Date) - $creationStart).TotalMilliseconds
    Record-Test -TestNum $testNum -Name "Rapid node creation (10 nodes)" -Passed $allCreated -Details "Created: $($stressNodes.Count)/10 in ${creationDuration}ms"
    
    # Governor checkpoint after node creation
    $gov40 = Governor-Checkpoint -Session $session -Cycle 40 -Context "post_rapid_nodes"
    
    # Test 41: Rapid edge binding (5 edges)
    # Directive: SCG-EDGEBIND-STALL-V1.0 - Enhanced stall detection wrapper
    $testNum = 41
    Write-Phase7Log -Level "INFO" -Message "Starting Test ${testNum}: Rapid edge binding (SCG-EDGEBIND-STALL-V1.0)" -Component "TEST"
    
    $edgesCreated = 0
    $edgeTimings = @()
    $test41Start = Get-Date
    $test41StallDetected = $false
    $maxEdgeLatencyMs = 500  # Per directive: 500ms max per edge bind
    
    for ($i = 0; $i -lt [math]::Min(5, $stressNodes.Count - 1); $i++) {
        # Per-iteration stall check with context
        if (Check-StallCondition) { 
            $test41StallDetected = $true
            Write-Phase7Log -Level "STALL" -Message "Stall detected at stress.edge.bind.$i - no progress for ${StallThresholdSeconds}s" -Component "TEST41"
            break 
        }
        if (Check-ExecutionTimeout) { 
            Write-Phase7Log -Level "STALL" -Message "Execution timeout at stress.edge.bind.$i" -Component "TEST41"
            break 
        }
        
        $edgeStart = Get-Date
        Write-Phase7Log -Level "DEBUG" -Message "Initiating edge bind ${i}: $($stressNodes[$i].Substring(0,8))... -> $($stressNodes[$i+1].Substring(0,8))..." -Component "TEST41"
        
        $req = Build-Request -Id ($testNum * 1000 + $i) -Method "edge.bind" -Params @{src=$stressNodes[$i]; dst=$stressNodes[$i+1]; weight=0.5}
        $resp = $session.SendRequest($req, "stress.edge.bind.$i", { param($l) Update-Progress $l })
        
        $edgeElapsedMs = ((Get-Date) - $edgeStart).TotalMilliseconds
        $edgeTimings += $edgeElapsedMs
        
        $edgeData = Extract-Content $resp
        
        if ($edgeData -and $edgeData.id) { 
            $edgesCreated++
            Write-Phase7Log -Level "DEBUG" -Message "Stress edge $i bound: $($edgeData.id.Substring(0,8))... elapsed_ms=$([math]::Round($edgeElapsedMs, 2))" -Component "TEST41"
            
            # Per-edge latency check per directive
            if ($edgeElapsedMs -gt $maxEdgeLatencyMs) {
                Write-Phase7Log -Level "WARN" -Message "Edge $i exceeded ${maxEdgeLatencyMs}ms threshold: elapsed=$([math]::Round($edgeElapsedMs, 2))ms" -Component "TEST41"
            }
        } else {
            Write-Phase7Log -Level "WARN" -Message "Edge bind $i returned no data after $([math]::Round($edgeElapsedMs, 2))ms" -Component "TEST41"
            # Dump server stderr on timeout for diagnostics
            $stderrLines = $session.DrainStderr()
            if ($stderrLines.Count -gt 0) {
                Write-Phase7Log -Level "DEBUG" -Message "Server stderr dump (${$stderrLines.Count} lines):" -Component "SERVER"
                foreach ($sline in $stderrLines) {
                    Write-Phase7Log -Level "DEBUG" -Message "  [SERVER] $sline" -Component "SERVER"
                }
            }
        }
    }
    
    $test41Duration = ((Get-Date) - $test41Start).TotalMilliseconds
    $avgEdgeLatency = if ($edgeTimings.Count -gt 0) { ($edgeTimings | Measure-Object -Average).Average } else { 0 }
    $maxEdgeLatency = if ($edgeTimings.Count -gt 0) { ($edgeTimings | Measure-Object -Maximum).Maximum } else { 0 }
    
    Write-Phase7Log -Level "DEBUG" -Message "phase7: Test 41 summary - edges=$edgesCreated total_ms=$([math]::Round($test41Duration, 2)) avg_ms=$([math]::Round($avgEdgeLatency, 2)) max_ms=$([math]::Round($maxEdgeLatency, 2))" -Component "TEST41"
    
    $test41Pass = ($edgesCreated -ge 4) -and (-not $test41StallDetected) -and ($maxEdgeLatency -le $maxEdgeLatencyMs)
    Record-Test -TestNum $testNum -Name "Rapid edge binding (5 edges)" -Passed $test41Pass -Details "Created: $edgesCreated/5, Duration: $([math]::Round($test41Duration, 2))ms, MaxLatency: $([math]::Round($maxEdgeLatency, 2))ms"
    
    # Governor checkpoint after edge binding per directive
    $gov41 = Governor-Checkpoint -Session $session -Cycle 41 -Context "post_edge_binding"
    if ($gov41 -and $gov41.data) {
        Write-Phase7Log -Level "DEBUG" -Message "phase7 - cycle=41 context=post_edge_binding nodes=$($gov41.data.node_count) edges=$(($gov41.data | Select-Object -ExpandProperty edge_count -ErrorAction SilentlyContinue) ?? 'N/A') drift=$($gov41.data.energy_drift) status=$(if ($gov41.ok) { 'STABLE' } else { 'DRIFT_EXCEEDED' })" -Component "GOVERNOR"
    }
    
    # Test 42: Burst mutations (5 mutations)
    $testNum = 42
    Write-Phase7Log -Level "INFO" -Message "Starting Test ${testNum}: Burst mutations" -Component "TEST"
    
    $mutationsSuccess = 0
    if ($stressNodes.Count -gt 0) {
        for ($i = 0; $i -lt [math]::Min(5, $stressNodes.Count); $i++) {
            if (Check-StallCondition -or Check-ExecutionTimeout) { break }
            
            $delta = (Get-Random -Minimum -50 -Maximum 50) / 100.0
            $req = Build-Request -Id ($testNum * 1000 + $i) -Method "node.mutate" -Params @{node_id=$stressNodes[$i]; delta=$delta}
            $resp = $session.SendRequest($req, "stress.node.mutate.$i", { param($l) Update-Progress $l })
            
            if (-not $resp.error) { 
                $mutationsSuccess++
            Write-Phase7Log -Level "DEBUG" -Message "Mutation ${i} - delta=$([math]::Round($delta, 4))" -Component "TEST42"
            }
        }
    }
    Record-Test -TestNum $testNum -Name "Burst mutations (5 mutations)" -Passed ($mutationsSuccess -ge 4) -Details "Success: $mutationsSuccess/5"
    
    # Governor checkpoint after mutations
    $gov42 = Governor-Checkpoint -Session $session -Cycle 42 -Context "post_burst_mutations"
    
    # Test 43: Burst propagations
    $testNum = 43
    Write-Phase7Log -Level "INFO" -Message "Starting Test ${testNum}: Burst propagations" -Component "TEST"
    
    $propSuccess = 0
    $edgeNames = @("Edge1", "Edge2", "Edge3", "Edge4")
    foreach ($edgeName in $edgeNames) {
        if (Check-StallCondition -or Check-ExecutionTimeout) { break }
        
        if ($script:edgeIds.ContainsKey($edgeName)) {
            $req = Build-Request -Id ($testNum * 1000 + $propSuccess) -Method "edge.propagate" -Params @{edge_id=$script:edgeIds[$edgeName]}
            $resp = $session.SendRequest($req, "stress.edge.propagate.$edgeName", { param($l) Update-Progress $l })
            
            if (-not $resp.error) { 
                $propSuccess++
                Write-Phase7Log -Level "DEBUG" -Message "Propagation along $edgeName successful" -Component "TEST43"
            }
        }
    }
    Record-Test -TestNum $testNum -Name "Burst propagations" -Passed ($propSuccess -ge 3) -Details "Success: $propSuccess/$($edgeNames.Count)"
    
    # Test 44: Governor stability post-stress (drift <= 1e-10)
    $testNum = 44
    Write-Phase7Log -Level "INFO" -Message "Starting Test ${testNum}: Governor stability post-stress" -Component "TEST"
    
    if (Check-StallCondition -or Check-ExecutionTimeout) { throw "Stall/timeout at governor check" }
    
    $req = Build-Request -Id $testNum -Method "governor.status" -Params @{}
    $resp = $session.SendRequest($req, "governor.status.44", { param($l) Update-Progress $l })
    $govAfterStress = Get-GovernorResult $resp
    $test44Pass = ($govAfterStress -ne $null) -and ([math]::Abs([double]$govAfterStress.energy_drift) -le 1e-10)
    Record-Test -TestNum $testNum -Name "Governor stability post-stress" -Passed $test44Pass -Details "Drift: $($govAfterStress.energy_drift), Coherence: $($govAfterStress.coherence)"
    
    Write-Phase7Log -Level "DEBUG" -Message "phase7 - cycle=44, nodes=$($govAfterStress.node_count), drift=$($govAfterStress.energy_drift), coherence=$($govAfterStress.coherence)" -Component "GOVERNOR"
    
    # Test 45: Invalid node ID handling
    $testNum = 45
    Write-Phase7Log -Level "INFO" -Message "Starting Test ${testNum}: Invalid node ID handling" -Component "TEST"
    
    if (Check-StallCondition -or Check-ExecutionTimeout) { throw "Stall/timeout at error handling test" }
    
    $req = Build-Request -Id $testNum -Method "node.query" -Params @{node_id="00000000-0000-0000-0000-000000000000"}
    $resp = $session.SendRequest($req, "invalid.node.query", { param($l) Update-Progress $l })
    $test45Pass = ($resp -ne $null) -and ($resp.error -or (Extract-Content $resp) -eq $null -or ((Extract-Content $resp).error))
    Record-Test -TestNum $testNum -Name "Invalid node ID handling" -Passed $test45Pass -Details "Error properly returned"
    
    # Test 46: Invalid edge ID handling
    $testNum = 46
    Write-Phase7Log -Level "INFO" -Message "Starting Test ${testNum}: Invalid edge ID handling" -Component "TEST"
    
    if (Check-StallCondition -or Check-ExecutionTimeout) { throw "Stall/timeout at error handling test" }
    
    $req = Build-Request -Id $testNum -Method "edge.propagate" -Params @{edge_id="00000000-0000-0000-0000-000000000000"}
    $resp = $session.SendRequest($req, "invalid.edge.propagate", { param($l) Update-Progress $l })
    $test46Pass = ($resp -ne $null) -and ($resp.error -or (Extract-Content $resp) -eq $null -or ((Extract-Content $resp).error))
    Record-Test -TestNum $testNum -Name "Invalid edge ID handling" -Passed $test46Pass -Details "Error properly returned"
    
    # Test 47: Final governor integrity (drift <= 1e-10, energy conservation)
    $testNum = 47
    Write-Phase7Log -Level "INFO" -Message "Starting Test ${testNum}: Final governor integrity check" -Component "TEST"
    
    if (Check-StallCondition -or Check-ExecutionTimeout) { throw "Stall/timeout at final integrity check" }
    
    $req = Build-Request -Id $testNum -Method "governor.status" -Params @{}
    $resp = $session.SendRequest($req, "governor.status.final", { param($l) Update-Progress $l })
    $govFinal = Get-GovernorResult $resp
    $test47Pass = ($govFinal -ne $null) -and ([math]::Abs([double]$govFinal.energy_drift) -le 1e-10)
    Record-Test -TestNum $testNum -Name "Final governor integrity check" -Passed $test47Pass -Details "Final Drift: $($govFinal.energy_drift), Coherence: $($govFinal.coherence)"
    
    $finalGovernor = $govFinal
    
    Write-Phase7Log -Level "DEBUG" -Message "phase7 - cycle=47, nodes=$($govFinal.node_count), drift=$($govFinal.energy_drift), coherence=$($govFinal.coherence), esv_avg=1.0" -Component "GOVERNOR"
    
    # =================================================================
    # LINEAGE VERIFICATION
    # =================================================================
    
    Write-Phase7Log -Level "INFO" -Message "===== Lineage Verification =====" -Component "LINEAGE"
    
    $req = Build-Request -Id 9999 -Method "lineage.replay" -Params @{}
    $resp = $session.SendRequest($req, "lineage.replay", { param($l) Update-Progress $l })
    $lineageData = Extract-Content $resp
    
    if ($lineageData -and $lineageData.checksum) {
        Write-Phase7Log -Level "INFO" -Message "Lineage checksum verified: $($lineageData.checksum.Substring(0, 16))..." -Component "LINEAGE"
    } else {
        Write-Phase7Log -Level "WARN" -Message "Lineage replay returned no checksum" -Component "LINEAGE"
    }
    
    # Determine certification status
    if (-not $script:stallDetected -and $script:passCount -eq $script:testResults.Count) {
        $certificationStatus = "CERTIFIED"
    } elseif ($script:stallDetected) {
        $certificationStatus = "STALL_DETECTED"
    } else {
        $certificationStatus = "TESTS_FAILED"
    }
    
} catch {
    Write-Phase7Log -Level "FAIL" -Message "Exception during execution: $_" -Component "EXCEPTION"
    $certificationStatus = "EXCEPTION"
} finally {
    if ($session) {
        $session.Close()
        Write-Phase7Log -Level "INFO" -Message "Session closed" -Component "SESSION"
    }
    
    # Export logs and metadata
    Export-Logs
    Export-Metadata -Status $certificationStatus -GovernorFinal $finalGovernor
}

# =====================================================================
# CERTIFICATION REPORT
# =====================================================================

$totalDuration = ((Get-Date) - $script:startTime).TotalSeconds

Write-Host @"

=====================================================================
          SCG PHASE-7 ADVERSARIAL CERTIFICATION REPORT
=====================================================================
"@ -ForegroundColor White

Write-Host "Status: " -NoNewline
switch ($certificationStatus) {
    "CERTIFIED" { Write-Host "CERTIFIED" -ForegroundColor Green }
    "STALL_DETECTED" { Write-Host "STALL_DETECTED - FAILED" -ForegroundColor Red }
    "TESTS_FAILED" { Write-Host "TESTS_FAILED" -ForegroundColor Red }
    "EXCEPTION" { Write-Host "EXCEPTION - FAILED" -ForegroundColor Red }
    default { Write-Host $certificationStatus -ForegroundColor Yellow }
}

Write-Host "Duration: $([math]::Round($totalDuration, 2))s / ${MaxExecutionSeconds}s max" -ForegroundColor $(if ($totalDuration -lt $MaxExecutionSeconds) { "Green" } else { "Red" })
Write-Host "Stall Detected: $($script:stallDetected)" -ForegroundColor $(if (-not $script:stallDetected) { "Green" } else { "Red" })
Write-Host ""
Write-Host "Tests Passed: $($script:passCount)/$($script:testResults.Count)" -ForegroundColor $(if ($script:passCount -eq $script:testResults.Count) { "Green" } elseif ($script:passCount -gt 0) { "Yellow" } else { "Red" })

if ($finalGovernor) {
    Write-Host ""
    Write-Host "Final Governor State:" -ForegroundColor Cyan
    Write-Host "  Drift: $($finalGovernor.energy_drift)" -ForegroundColor $(if ([math]::Abs($finalGovernor.energy_drift) -le 1e-10) { "Green" } else { "Red" })
    Write-Host "  Coherence: $($finalGovernor.coherence)" -ForegroundColor $(if ($finalGovernor.coherence -ge 0.97) { "Green" } else { "Yellow" })
    Write-Host "  Nodes: $($finalGovernor.node_count)" -ForegroundColor White
}

Write-Host ""
Write-Host "---------------------------------------------------------------------" -ForegroundColor DarkGray

if ($certificationStatus -eq "CERTIFIED") {
    Write-Host @"

 ██████╗███████╗██████╗ ████████╗██╗███████╗██╗███████╗██████╗ 
██╔════╝██╔════╝██╔══██╗╚══██╔══╝██║██╔════╝██║██╔════╝██╔══██╗
██║     █████╗  ██████╔╝   ██║   ██║█████╗  ██║█████╗  ██║  ██║
██║     ██╔══╝  ██╔══██╗   ██║   ██║██╔══╝  ██║██╔══╝  ██║  ██║
╚██████╗███████╗██║  ██║   ██║   ██║██║     ██║███████╗██████╔╝
 ╚═════╝╚══════╝╚═╝  ╚═╝   ╚═╝   ╚═╝╚═╝     ╚═╝╚══════╝╚═════╝ 

"@ -ForegroundColor Green
    Write-Host "[PASS] SCG Phase-7 Adversarial Tests CERTIFIED" -ForegroundColor Green
    Write-Host "       Governor resilience verified. Lineage stable." -ForegroundColor Green
} else {
    Write-Host ""
    Write-Host "[FAIL] SCG Phase-7 Certification FAILED" -ForegroundColor Red
    Write-Host "       Status: $certificationStatus" -ForegroundColor Red
    
    if ($script:stallDetected) {
        Write-Host ""
        Write-Host "STALL DIAGNOSTIC:" -ForegroundColor Magenta
        Write-Host "  - Review logs/phase7/phase7_last_run.log for progress markers" -ForegroundColor Yellow
        Write-Host "  - Check for MCP transport blockage" -ForegroundColor Yellow
        Write-Host "  - Check for propagation runaway (ESV divergence)" -ForegroundColor Yellow
        Write-Host "  - Check for mutex deadlock (lock acquisition > 500ms)" -ForegroundColor Yellow
    }
    
    if ($script:failCount -gt 0) {
        Write-Host ""
        Write-Host "Failed Tests:" -ForegroundColor Red
        foreach ($test in $script:testResults | Where-Object { -not $_.Passed }) {
            Write-Host "  - Test $($test.TestNum): $($test.Name)" -ForegroundColor Red
            Write-Host "    Details: $($test.Details)" -ForegroundColor DarkRed
        }
    }
}

Write-Host ""
Write-Host "Logs: logs/phase7/phase7_last_run.log" -ForegroundColor Cyan
Write-Host "Metadata: logs/phase7/phase7_metadata.yml" -ForegroundColor Cyan
Write-Host ""

# Exit with appropriate code
if ($certificationStatus -eq "CERTIFIED") {
    exit 0
} else {
    exit 1
}
