# MCP tools/list capture script for Phase 0
$ErrorActionPreference = "Stop"

$request = @{
    jsonrpc = "2.0"
    id = 1
    method = "tools/list"
    params = @{}
} | ConvertTo-Json -Compress

$process = Start-Process -FilePath "C:\Users\adubo\iter\target\release\iter-server.exe" `
    -NoNewWindow `
    -PassThru `
    -RedirectStandardInput "mcp-stdin.txt" `
    -RedirectStandardOutput "mcp-stdout.txt" `
    -RedirectStandardError "mcp-stderr.txt"

# Write request to stdin
Set-Content -Path "mcp-stdin.txt" -Value $request -NoNewline

# Wait for response
Start-Sleep -Seconds 3

# Kill process
Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue

# Read response
$response = Get-Content "mcp-stdout.txt" -Raw
Write-Output $response
