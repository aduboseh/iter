# SCG Demo Package - Operational Runbook

## Prerequisites

- POSIX shell (bash 4.0+)
- `jq` command-line tool (1.6+)
- `bc` calculator
- `sha256sum` utility
- SCG MCP server binary in PATH or at `target/release/scg_mcp_server`

## Step 1: Environment Setup

```bash
# Set deterministic environment
export SCG_TIMESTAMP_MODE=deterministic
export SCG_DETERMINISM=1
export LC_ALL=C
export LANG=C
export TZ=UTC

# Verify dependencies
command -v jq || echo "ERROR: jq not found"
command -v bc || echo "ERROR: bc not found"
command -v sha256sum || echo "ERROR: sha256sum not found"
```

## Step 2: Execute Demo

```bash
cd scg_mcp_server
chmod +x demos/scg_demo.sh
./demos/scg_demo.sh
```

**Expected Runtime:** 30-60 seconds for both runs.

**Expected Output:**
```
[SCG-DEMO] SCG Substrate Demo v1.0 (Production Edition)
[SCG-DEMO] Determinism mode: enabled
...
[SCG-DEMO] DETERMINISM VERIFIED
[SCG-DEMO] All invariant artifacts match across runs
```

## Step 3: Verify Checksums

```bash
# View checksums from both runs
cat demo_runs/run_1/demo_output/07_checksums.txt
cat demo_runs/run_2/demo_output/07_checksums.txt

# Compare (should produce no output)
diff demo_runs/run_1/demo_output/07_checksums.txt \
     demo_runs/run_2/demo_output/07_checksums.txt
```

**Success Criteria:** `diff` outputs nothing (files identical).

---

## Troubleshooting

### Issue: MCP server fails to start
**Symptoms:** "Server binary not found or not executable"
**Fix:**
```bash
# Build release binary
cargo build --release

# Verify executable
chmod +x target/release/scg_mcp_server
ls -la target/release/scg_mcp_server*
```

### Issue: Checksums mismatch between runs
**Symptoms:** DETERMINISM FAILURE message
**Check:**
```bash
# Verify environment is set
echo $SCG_DETERMINISM  # Should be "1"
echo $LC_ALL           # Should be "C"

# Check for timestamp patterns in logs
grep -E "[0-9]{4}-[0-9]{2}-[0-9]{2}" demo_runs/run_1/demo_output/*.log
```

### Issue: jq not found
**Fix (Ubuntu/Debian):**
```bash
sudo apt-get update && sudo apt-get install -y jq
```
**Fix (macOS):**
```bash
brew install jq
```
**Fix (Windows/Git Bash):**
```bash
# Download from https://stedolan.github.io/jq/download/
```

### Issue: Server timeout (>5 seconds)
**Symptoms:** "MCP server unresponsive after 5 seconds"
**Check:**
```bash
# Look for stale processes
ps aux | grep scg_mcp_server
pkill -f scg_mcp_server

# Check server stderr
cat demo_runs/run_1/demo_output/server_stderr.log
```

### Issue: Named pipe errors
**Symptoms:** "mkfifo: cannot create fifo"
**Fix:**
```bash
# Clean up stale pipes
rm -f demo_runs/*/demo_output/.mcp_*
```

---

## Cleanup

```bash
# Remove run artifacts
rm -rf demo_runs/

# Kill any background servers
pkill -f scg_mcp_server || true
```

---

## Quick Validation Commands

```bash
# Count invariant files (should be 7 per run)
ls demo_runs/run_1/demo_output/*.{log,json,txt} 2>/dev/null | wc -l

# Verify JSON validity
for f in demo_runs/run_1/demo_output/*.{log,json}; do
  jq empty "$f" 2>/dev/null || echo "Invalid JSON: $f"
done

# Check for prohibited keywords
grep -ri "haltra\|nodetic\|patient\|vehicle" demos/ demo_expected/
```
