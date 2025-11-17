# SCG-PILOT-01 Console Hygiene Protocol

**Document ID**: CONSOLE_HYGIENE  
**Directive**: SG-SCG-PILOT-COHERENCE-01 v1.0.0 §5  
**Purpose**: Eliminate console friction caused by pasting prose/formatting into shell environments

---

## Problem Statement (Gap D)

**Console Errors from Formatted Text**:
- Directives often include bullets (•), checkmarks (✅), section markers (===), headers (##)
- PowerShell/bash interpret these as unknown commands
- Result: `The term '===' is not recognized as a cmdlet`, `unknown command: ✅`

**Root Cause**:
- Formatted documentation pasted directly into terminal
- No clear boundary between "documentation to read" and "commands to run"

---

## Hygiene Protocol

### Rule 1: Fenced Code Blocks Only

**Only fenced code blocks are executable**:

````markdown
```powershell
# RUN THIS
kubectl apply -f deployment/pilot/time-sync-checker.yaml
```
````

**Everything else is documentation** (read-only, never pasted to console).

---

### Rule 2: Explicit Execution Markers

All runnable commands MUST include `# RUN THIS` comment:

✅ **Good**:
```powershell
# RUN THIS
.\deployment\pilot\monitor-invariants.ps1 -Namespace scg-pilot-01
```

❌ **Bad** (ambiguous):
```powershell
.\deployment\pilot\monitor-invariants.ps1 -Namespace scg-pilot-01
```

---

### Rule 3: No Prose in Code Blocks

Code blocks contain ONLY:
- Shell commands
- Code comments (starting with `#` or `//`)
- Actual code

❌ **Never include**:
- Checkmarks: ✅ ❌ ⏳
- Bullets: •, -, *
- Section headers: ===, ##, ---
- Status indicators: [COMPLETE], (P0)

---

### Rule 4: Multi-Line Commands Use Line Continuation

**PowerShell**:
```powershell
# RUN THIS
kubectl exec -n scg-pilot-01 scg-mcp-pod `
  -- cat /var/scg/lineage/ledger.bin `
  > pilot_reports/day1/ledger_export.bin
```

**Bash**:
```bash
# RUN THIS
kubectl exec -n scg-pilot-01 scg-mcp-pod \
  -- cat /var/scg/lineage/ledger.bin \
  > pilot_reports/day1/ledger_export.bin
```

---

### Rule 5: Document Structure Separation

Directives MUST separate:

**Documentation Section** (prose, tables, bullets):
```markdown
## Section 2 — Time Sync Validation

Warp SHALL:
- Deploy DaemonSet
- Validate skew ≤ 50ms
- Generate time_sync.json
```

**Execution Section** (code only):
````markdown
```powershell
# RUN THIS
kubectl apply -f deployment/pilot/time-sync-checker.yaml
```
````

---

## Common Violations (Avoid These)

### Violation 1: Inline Status Markers

❌ **Bad**:
```
✅ PASS — Infrastructure deployed
❌ FAIL — Time sync blocked
```

✅ **Good** (in code context):
```powershell
# RUN THIS
if ($status -eq "PASS") {
    Write-Host "PASS - Infrastructure deployed" -ForegroundColor Green
}
```

---

### Violation 2: Section Dividers in Scripts

❌ **Bad**:
```
================================================
SCG-PILOT-01 Monitoring
================================================
```

✅ **Good**:
```powershell
# RUN THIS
Write-Host "================================================"
Write-Host "SCG-PILOT-01 Monitoring"
Write-Host "================================================"
```

---

### Violation 3: Mixed Prose and Commands

❌ **Bad**:
```
To deploy time sync, run:
kubectl apply -f time-sync-checker.yaml

Then verify with:
kubectl get pods -n scg-pilot-01
```

✅ **Good**:
```powershell
# RUN THIS
kubectl apply -f time-sync-checker.yaml

# RUN THIS
kubectl get pods -n scg-pilot-01
```

---

## Directive Author Guidelines

When writing directives:

1. **Separate "explain" from "execute"**
   - Prose sections: Explain what and why
   - Code sections: Show how (executable only)

2. **Use consistent markers**
   - `# RUN THIS` for all executable commands
   - Language tags on fenced blocks: ```powershell, ```bash

3. **Test before publishing**
   - Copy code block to actual shell
   - Verify it runs without syntax errors
   - No emoji, no formatting, no prose

4. **Provide context in comments**
   ```powershell
   # RUN THIS - Deploy time sync DaemonSet
   kubectl apply -f deployment/pilot/time-sync-checker.yaml
   
   # RUN THIS - Verify pods scheduled
   kubectl get pods -n scg-pilot-01 -l app=time-sync-checker
   ```

---

## Executor (Warp AI) Guidelines

When executing directives:

1. **Identify executable blocks**
   - Look for fenced code with `# RUN THIS`
   - Ignore all prose, tables, bullets

2. **Extract clean commands**
   - Remove markdown fence markers (```)
   - Remove `# RUN THIS` comments before execution
   - Preserve all other logic

3. **Report syntax errors**
   - If command fails due to unexpected token
   - Check if prose leaked into code block
   - Request corrected directive

4. **Never guess**
   - If unsure whether text is executable, ask
   - Better to clarify than execute invalid commands

---

## Example: Correct Directive Format

### Documentation (Read-Only)

**Objective**: Deploy time sync validation infrastructure

**Requirements**:
- Kubernetes cluster access
- scg-pilot-01 namespace exists
- kubectl configured

---

### Execution (Run This)

```powershell
# RUN THIS - Deploy DaemonSet
kubectl apply -f deployment/pilot/time-sync-checker.yaml -n scg-pilot-01

# RUN THIS - Wait for pods
Start-Sleep -Seconds 10

# RUN THIS - Verify deployment
kubectl get daemonset time-sync-checker -n scg-pilot-01

# RUN THIS - Check pod status
kubectl get pods -n scg-pilot-01 -l app=time-sync-checker
```

---

## Benefits

**Before Hygiene Protocol**:
```
The term '✅' is not recognized as a cmdlet
The term '===' is not recognized as a cmdlet
P0: The term 'P0' is not recognized as a cmdlet
```

**After Hygiene Protocol**:
```powershell
# Clean execution, no noise
kubectl apply -f time-sync-checker.yaml
daemonset.apps/time-sync-checker created
```

---

## Summary

**Golden Rule**: If it's not in a fenced code block with `# RUN THIS`, don't paste it to console.

**For Directive Authors**: Separate documentation from execution.  
**For Executors**: Only run fenced code blocks.  
**For Everyone**: Test commands in actual shells before committing.

---

**END OF HYGIENE PROTOCOL**

*Console hygiene eliminates 90% of "unknown cmdlet" noise and keeps focus on actual system behavior rather than formatting artifacts.*
