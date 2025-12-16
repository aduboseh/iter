# MCP Relay Script
# Warp spawns this, which relays stdio to an already-running iter-server via named pipe
# 
# Usage:
# 1. In a separate terminal: .\target\release\iter-server.exe
# 2. Configure Warp to use this script as the MCP command
#
# This bypasses Warp's flaky Windows stdio process management.

$pipeName = "iter-mcp-pipe"
$pipe = $null

try {
    # Connect to the named pipe server
    $pipe = New-Object System.IO.Pipes.NamedPipeClientStream(".", $pipeName, [System.IO.Pipes.PipeDirection]::InOut)
    $pipe.Connect(5000)  # 5 second timeout
    
    $reader = New-Object System.IO.StreamReader($pipe)
    $writer = New-Object System.IO.StreamWriter($pipe)
    $writer.AutoFlush = $true
    
    $stdinReader = [Console]::In
    
    # Relay stdin to pipe, pipe to stdout
    while ($true) {
        $line = $stdinReader.ReadLine()
        if ($null -eq $line) { break }
        
        $writer.WriteLine($line)
        $response = $reader.ReadLine()
        
        if ($null -ne $response) {
            [Console]::WriteLine($response)
        }
    }
}
catch {
    [Console]::Error.WriteLine("Relay error: $_")
}
finally {
    if ($pipe) { $pipe.Dispose() }
}
