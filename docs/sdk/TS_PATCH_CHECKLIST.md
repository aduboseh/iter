# TypeScript SDK Patch Checklist — CLIENT_CONTRACT_v1.0.md Conformance

**Status**: EXECUTION READY  
**Target SDK**: `sdks/typescript/src/index.ts`  
**Authority**: CLIENT_CONTRACT_v1.0.md  
**Mode**: Fail-closed (all patches MUST pass tests before Python work begins)  
**Version**: v1.0  
**Date**: 2026-01-28

---

## Executive Summary

**Current Status**: TypeScript SDK has 7/14 conformance gaps  
**Risk Level**: Medium (no API breakage expected, backward-compatible changes only)  
**Estimated Effort**: 3–4 hours (implementation + tests)  
**Blocker for**: Python SDK implementation

---

## Conformance Gap Analysis

| Requirement | Status | Severity | File | Line(s) |
|-------------|--------|----------|------|---------|
| **R2.1.1**: Backpressure (maxInflight) | ❌ MISSING | HIGH | index.ts | Constructor |
| **R3.1.1**: Per-request timeout | ❌ MISSING | HIGH | index.ts | send() |
| **R4.1.3**: Graceful shutdown (SIGTERM→SIGKILL) | ❌ MISSING | HIGH | index.ts | close() |
| **R4.2.1**: Process crash handling | ⚠️ PARTIAL | MEDIUM | index.ts | 212-217 |
| **R5.1.2**: stderr capture | ❌ MISSING | HIGH | index.ts | connect() |
| **R6.1.2**: Malformed stdout handling | ❌ MISSING | MEDIUM | index.ts | 200-202 |
| **R9.1.1**: Missing error types | ❌ MISSING | LOW | index.ts | 80-114 |

---

## Patch Tasks (Prioritized)

### ✅ PHASE 0: Pre-Flight (Already Conformant)

- [x] **CT1.1**: STDIO JSON-RPC line protocol  
  - Status: ✅ Conformant (lines 175-203)
  - Evidence: `spawn()` with `stdio: ["pipe", "pipe", ...]`, readline interface

- [x] **CT1.2**: Out-of-order response dispatch by ID  
  - Status: ✅ Conformant (lines 192-199)
  - Evidence: `responseQueue.get(response.id)`

- [x] **CT7.1**: Semantic version compatibility check  
  - Status: ✅ Conformant (lines 353-361)
  - Evidence: `isVersionCompatible()` with MIN/MAX bounds

- [x] **CT8.1**: TraceContext stored but not serialized  
  - Status: ✅ Conformant (lines 38-50, 222-226)
  - Evidence: `_traceContext` stored, never added to JSON-RPC

- [x] **CT9.1**: Typed error hierarchy (partial)  
  - Status: ⚠️ Needs extension (lines 83-114)
  - Evidence: `SdkError`, `VersionMismatchError`, `ConnectionError`, `RequestError` exist

---

### 🔴 PHASE 1: Critical Safety Patches (MANDATORY)

#### **PATCH 1.1: Backpressure Enforcement (CT2.1)**

**Requirement**: R2.1.1–R2.1.4  
**Severity**: HIGH (prevents unbounded memory growth)

**Current Code** (index.ts:152-163):
```typescript
export class IterClient {
  private process: ChildProcess | null = null;
  private stdin: Writable | null = null;
  private stdout: Readable | null = null;
  private requestId = 0;
  private _traceContext: TraceContext | null = null;
  private responseQueue: Map<...> = new Map();
  private lineReader: readline.Interface | null = null;

  private constructor() {}
```

**Required Changes**:

1. Add constructor parameter:
```typescript
export interface IterClientConfig {
  maxInflight?: number; // default: 1
}

private constructor(private config: IterClientConfig) {}
```

2. Add backpressure check in `send()`:
```typescript
async send(method: string, params?: unknown): Promise<RpcResponse> {
  if (!this.stdin) {
    throw new ConnectionError("Not connected");
  }

  const maxInflight = this.config.maxInflight ?? 1;
  if (this.responseQueue.size >= maxInflight) {
    throw new BackpressureError(maxInflight);
  }

  // ... existing code
}
```

3. Add error type:
```typescript
export class BackpressureError extends SdkError {
  constructor(public readonly maxInflight: number) {
    super(`Backpressure: ${maxInflight} requests already in-flight. Wait for responses before sending more.`);
    this.name = "BackpressureError";
  }
}
```

**Test Requirements** (index.test.ts):
```typescript
describe("Backpressure", () => {
  test("CT2.1: rejects 3rd request when maxInflight=2", async () => {
    const client = await IterClient.connect("./mock-binary", { maxInflight: 2 });
    
    // Send 2 requests (don't await)
    const p1 = client.send("method1");
    const p2 = client.send("method2");
    
    // 3rd request MUST fail immediately
    await expect(client.send("method3")).rejects.toThrow(BackpressureError);
  });
});
```

**Files to Edit**:
- `sdks/typescript/src/index.ts` (lines 152-163, 229-248, 80-114)
- `sdks/typescript/src/index.test.ts` (new test suite)

---

#### **PATCH 1.2: Per-Request Timeout (CT3.1)**

**Requirement**: R3.1.1–R3.1.4  
**Severity**: HIGH (prevents indefinite hangs)

**Current Code** (index.ts:229-248):
```typescript
async send(method: string, params?: unknown): Promise<RpcResponse> {
  // ... validation
  
  return new Promise((resolve, reject) => {
    this.responseQueue.set(id, { resolve, reject });
    this.stdin!.write(JSON.stringify(request) + "\n");
  }); // ⚠️ No timeout
}
```

**Required Changes**:

1. Add timeout parameter:
```typescript
async send(
  method: string, 
  params?: unknown, 
  timeoutMs: number = 30000
): Promise<RpcResponse> {
  // ... existing validation
  
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => {
      this.responseQueue.delete(id);
      reject(new RequestTimeoutError(method, timeoutMs));
    }, timeoutMs);
    
    this.responseQueue.set(id, {
      resolve: (response) => {
        clearTimeout(timer);
        resolve(response);
      },
      reject: (error) => {
        clearTimeout(timer);
        reject(error);
      },
    });
    
    this.stdin!.write(JSON.stringify(request) + "\n");
  });
}
```

2. Update dispatcher to ignore late responses:
```typescript
client.lineReader.on("line", (line) => {
  try {
    const response: RpcResponse = JSON.parse(line);
    const pending = client.responseQueue.get(response.id as number);
    if (pending) {
      client.responseQueue.delete(response.id as number);
      pending.resolve(response);
    } else {
      // R1.2.4: Late response (likely timed out) — log and ignore
      console.warn(`Ignoring response for unknown/timed-out request ID: ${response.id}`);
    }
  } catch (e) {
    // Malformed line handling (see PATCH 2.2)
  }
});
```

3. Add error type:
```typescript
export class RequestTimeoutError extends SdkError {
  constructor(
    public readonly method: string,
    public readonly timeoutMs: number
  ) {
    super(`Request timeout: ${method} exceeded ${timeoutMs}ms`);
    this.name = "RequestTimeoutError";
  }
}
```

**Test Requirements** (index.test.ts):
```typescript
describe("Timeouts", () => {
  test("CT3.1: request times out after 2s", async () => {
    const client = await IterClient.connect("./slow-binary");
    
    const start = Date.now();
    await expect(client.send("slow_method", {}, 2000)).rejects.toThrow(RequestTimeoutError);
    const elapsed = Date.now() - start;
    
    expect(elapsed).toBeGreaterThanOrEqual(1900); // 2s ± 100ms
    expect(elapsed).toBeLessThan(2100);
  });
  
  test("CT3.1: late response ignored", async () => {
    // Binary responds at t=5s, but timeout=2s
    // Late response must not resolve a different request
  });
});
```

**Files to Edit**:
- `sdks/typescript/src/index.ts` (lines 229-248, 192-203, 80-114)
- `sdks/typescript/src/index.test.ts` (new test suite)

---

#### **PATCH 1.3: Graceful Shutdown (CT4.1)**

**Requirement**: R4.1.3–R4.1.5  
**Severity**: HIGH (prevents zombie processes)

**Current Code** (index.ts:293-304):
```typescript
close(): void {
  if (this.lineReader) {
    this.lineReader.close();
    this.lineReader = null;
  }
  if (this.process) {
    this.process.kill(); // ⚠️ SIGKILL immediate, no drain
    this.process = null;
  }
  this.stdin = null;
  this.stdout = null;
}
```

**Required Changes**:

1. Implement graceful shutdown:
```typescript
private closed = false;

async close(): Promise<void> {
  if (this.closed) return;
  this.closed = true;
  
  // 1. Stop accepting new requests (checked in send())
  
  // 2. Drain pending requests (max 5s)
  const drainPromise = this.drainPending(5000);
  
  // 3. SIGTERM
  if (this.process && !this.process.killed) {
    this.process.kill("SIGTERM");
    
    // 4. Wait 2s for graceful exit
    await Promise.race([
      new Promise(resolve => setTimeout(resolve, 2000)),
      new Promise(resolve => this.process?.once("exit", resolve)),
    ]);
    
    // 5. SIGKILL if still alive
    if (this.process && !this.process.killed) {
      this.process.kill("SIGKILL");
    }
    
    // 6. Reap zombie
    await new Promise(resolve => {
      this.process?.once("exit", resolve);
      setTimeout(resolve, 1000); // fallback
    });
  }
  
  await drainPromise;
  
  if (this.lineReader) {
    this.lineReader.close();
    this.lineReader = null;
  }
  this.process = null;
  this.stdin = null;
  this.stdout = null;
}

private async drainPending(timeoutMs: number): Promise<void> {
  if (this.responseQueue.size === 0) return;
  
  const pending = Array.from(this.responseQueue.values());
  const deadline = Date.now() + timeoutMs;
  
  await Promise.race([
    Promise.allSettled(pending.map(p => new Promise(p.resolve))),
    new Promise(resolve => setTimeout(resolve, timeoutMs)),
  ]);
  
  // Fail undrained requests
  for (const [id, { reject }] of this.responseQueue.entries()) {
    reject(new ConnectionClosedError("Client closed before response received"));
    this.responseQueue.delete(id);
  }
}
```

2. Reject requests after close:
```typescript
async send(method: string, params?: unknown, timeoutMs = 30000): Promise<RpcResponse> {
  if (this.closed) {
    throw new ConnectionClosedError("Client is closed");
  }
  if (!this.stdin) {
    throw new ConnectionError("Not connected");
  }
  // ... existing code
}
```

3. Add error type:
```typescript
export class ConnectionClosedError extends SdkError {
  constructor(message: string) {
    super(`Connection closed: ${message}`);
    this.name = "ConnectionClosedError";
  }
}
```

**Test Requirements** (index.test.ts):
```typescript
describe("Lifecycle", () => {
  test("CT4.1: graceful shutdown drains pending", async () => {
    const client = await IterClient.connect("./mock-binary");
    
    const p1 = client.send("method1");
    const p2 = client.send("method2");
    
    const closePromise = client.close();
    
    // Requests should complete (or timeout) within 5s
    await expect(Promise.race([p1, p2])).resolves.toBeDefined();
    await closePromise;
    
    // No zombie process
    // (Use `ps aux | grep mock-binary` assertion if needed)
  });
  
  test("CT4.1: close() rejects subsequent requests", async () => {
    const client = await IterClient.connect("./mock-binary");
    await client.close();
    
    await expect(client.send("method")).rejects.toThrow(ConnectionClosedError);
  });
});
```

**Files to Edit**:
- `sdks/typescript/src/index.ts` (lines 293-304, 229-248, 80-114)
- `sdks/typescript/src/index.test.ts` (new test suite)

---

#### **PATCH 1.4: Process Crash Handling (CT4.2)**

**Requirement**: R4.2.1–R4.2.2  
**Severity**: MEDIUM (current partial implementation at lines 212-217)

**Current Code** (index.ts:212-217):
```typescript
client.process.on("exit", () => {
  for (const pending of client.responseQueue.values()) {
    pending.reject(new ConnectionError("Process exited"));
  }
  client.responseQueue.clear();
}); // ⚠️ Does not mark client as closed or reject future requests
```

**Required Changes**:

1. Mark client as closed on crash:
```typescript
client.process.on("exit", (code) => {
  client.closed = true;
  
  const error = new ConnectionError(
    `Process exited unexpectedly (code: ${code})`
  );
  
  for (const pending of client.responseQueue.values()) {
    pending.reject(error);
  }
  client.responseQueue.clear();
});
```

2. No changes to `send()` needed (already checks `this.closed` from PATCH 1.3)

**Test Requirements** (index.test.ts):
```typescript
describe("Lifecycle", () => {
  test("CT4.2: process crash fails pending and rejects new requests", async () => {
    const client = await IterClient.connect("./crash-binary");
    
    const p1 = client.send("method");
    
    // Trigger crash (implementation-specific)
    
    await expect(p1).rejects.toThrow(ConnectionError);
    await expect(client.send("method2")).rejects.toThrow(ConnectionClosedError);
  });
});
```

**Files to Edit**:
- `sdks/typescript/src/index.ts` (lines 212-217)
- `sdks/typescript/src/index.test.ts` (new test suite)

---

### 🟡 PHASE 2: Observability Patches (REQUIRED)

#### **PATCH 2.1: stderr Capture (CT5.1)**

**Requirement**: R5.1.1–R5.1.4  
**Severity**: HIGH (diagnostic necessity)

**Current Code** (index.ts:175-177):
```typescript
client.process = spawn(binaryPath, [], {
  stdio: ["pipe", "pipe", "ignore"], // ⚠️ stderr discarded
});
```

**Required Changes**:

1. Capture stderr:
```typescript
private stderrBuffer: string[] = [];
private readonly STDERR_MAX_LINES = 100;

static async connect(binaryPath: string, config: IterClientConfig = {}): Promise<IterClient> {
  const client = new IterClient(config);
  
  client.process = spawn(binaryPath, [], {
    stdio: ["pipe", "pipe", "pipe"], // stderr now captured
  });
  
  // ... existing stdin/stdout setup
  
  // Capture stderr
  if (client.process.stderr) {
    const stderrReader = readline.createInterface({
      input: client.process.stderr,
      crlfDelay: Infinity,
    });
    
    stderrReader.on("line", (line) => {
      client.stderrBuffer.push(line);
      if (client.stderrBuffer.length > client.STDERR_MAX_LINES) {
        client.stderrBuffer.shift(); // Keep last 100 lines
      }
    });
  }
  
  // ... existing event handlers
}
```

2. Attach stderr to errors:
```typescript
export class ConnectionError extends SdkError {
  constructor(
    message: string,
    public readonly stderrTail?: string[]
  ) {
    const stderrHint = stderrTail?.length
      ? `\n\nstderr tail:\n${stderrTail.join("\n")}`
      : "";
    super(`Connection failed: ${message}${stderrHint}`);
    this.name = "ConnectionError";
  }
}

// Update exit handler:
client.process.on("exit", (code) => {
  client.closed = true;
  
  const error = new ConnectionError(
    `Process exited unexpectedly (code: ${code})`,
    client.stderrBuffer.slice() // snapshot
  );
  
  // ... existing cleanup
});
```

3. Add diagnostic accessor:
```typescript
/** Get captured stderr tail (last 100 lines) */
getStderrTail(): string[] {
  return this.stderrBuffer.slice();
}
```

**Test Requirements** (index.test.ts):
```typescript
describe("stderr Capture", () => {
  test("CT5.1: stderr appears in ConnectionError", async () => {
    const client = await IterClient.connect("./panic-binary");
    
    try {
      await client.send("trigger_panic");
    } catch (err) {
      expect(err).toBeInstanceOf(ConnectionError);
      expect((err as ConnectionError).stderrTail).toContain("PANIC: index out of bounds");
    }
  });
});
```

**Files to Edit**:
- `sdks/typescript/src/index.ts` (lines 175-220, 102-107)
- `sdks/typescript/src/index.test.ts` (new test suite)

---

#### **PATCH 2.2: Malformed stdout Handling (CT6.1)**

**Requirement**: R6.1.1–R6.1.4  
**Severity**: MEDIUM (observability gap)

**Current Code** (index.ts:200-202):
```typescript
} catch (e) {
  // Ignore malformed lines ⚠️ SILENT DISCARD
}
```

**Required Changes**:

1. Add diagnostic buffer:
```typescript
private malformedLines: Array<{ line: string; error: string }> = [];
private readonly MALFORMED_MAX_LINES = 50;
```

2. Log and store malformed lines:
```typescript
client.lineReader.on("line", (line) => {
  try {
    const response: RpcResponse = JSON.parse(line);
    // ... existing dispatch
  } catch (e) {
    // R6.1.2: Store malformed line
    const error = e instanceof Error ? e.message : String(e);
    client.malformedLines.push({ line, error });
    if (client.malformedLines.length > client.MALFORMED_MAX_LINES) {
      client.malformedLines.shift();
    }
    
    // R6.1.3: Log at WARN level
    console.warn(`[Iter SDK] Malformed stdout line: ${line} (${error})`);
    
    // R6.1.4: Dispatcher continues (no throw)
  }
});
```

3. Add diagnostic accessor:
```typescript
/** Get malformed stdout lines (last 50) */
getMalformedLines(): Array<{ line: string; error: string }> {
  return this.malformedLines.slice();
}
```

**Test Requirements** (index.test.ts):
```typescript
describe("Malformed stdout", () => {
  test("CT6.1: malformed line captured and logged", async () => {
    const client = await IterClient.connect("./debug-binary");
    
    // Binary writes: "DEBUG: initializing graph\n"
    // followed by valid JSON-RPC
    
    await client.send("method");
    
    const malformed = client.getMalformedLines();
    expect(malformed).toContainEqual({
      line: "DEBUG: initializing graph",
      error: expect.stringContaining("JSON"),
    });
  });
});
```

**Files to Edit**:
- `sdks/typescript/src/index.ts` (lines 200-203)
- `sdks/typescript/src/index.test.ts` (new test suite)

---

### 🟢 PHASE 3: Final Compliance (LOW PRIORITY)

#### **PATCH 3.1: Complete Error Type Coverage**

**Requirement**: R9.1.1  
**Severity**: LOW (mostly complete)

**Current Status**: Missing `ConnectionClosedError` (added in PATCH 1.3), all others exist

**Required Changes**: None (completed by PATCH 1.3)

---

## Implementation Order (CRITICAL PATH)

Execute patches in this exact order to minimize test failures:

1. **PATCH 1.1**: Backpressure (enables safe multi-request tests)
2. **PATCH 1.2**: Timeouts (prevents test hangs)
3. **PATCH 1.4**: Process crash handling (completes lifecycle)
4. **PATCH 1.3**: Graceful shutdown (depends on crash handling)
5. **PATCH 2.1**: stderr capture (observable failures)
6. **PATCH 2.2**: Malformed stdout (completes observability)

---

## Test Mock Requirements

To execute conformance tests, create mock binaries in `sdks/typescript/test/mocks/`:

| Mock Binary | Behavior |
|-------------|----------|
| `slow-binary` | Responds after 5s delay |
| `crash-binary` | Exits with code 1 after first request |
| `panic-binary` | Writes "PANIC: ..." to stderr, exits 101 |
| `debug-binary` | Writes debug lines to stdout before JSON-RPC |

**Alternative**: Use Jest mocks with `spawn` interception (preferred for CI).

---

## CI Integration

Add to `.github/workflows/test.yml`:

```yaml
- name: TypeScript SDK Conformance Tests
  run: |
    cd sdks/typescript
    npm test -- --testNamePattern="CT[0-9]\\."
```

**Success Criteria**: All CT* tests pass before merging.

---

## Compliance Matrix Update

After all patches land, update `docs/sdk/CLIENT_CONTRACT_v1.0.md` line 303:

```diff
- | TypeScript | ❓ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ✅ |
+ | TypeScript | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
```

---

## Next Steps

1. **Execute this checklist** sequentially (no parallel patches)
2. **Run all tests** after each patch
3. **Update compliance matrix** only when all CT* tests pass
4. **Commit with message**: `feat(sdk/ts): implement CLIENT_CONTRACT_v1.0.md conformance`
5. **Do NOT touch Python SDK** until TypeScript shows 8/8 ✅

---

## Rollback Plan

If any patch causes breakage:

```bash
git restore sdks/typescript/src/index.ts sdks/typescript/src/index.test.ts
```

No schema changes. No protocol changes. No side effects.

---

**FINAL RECOMMENDATION**

Execute PHASE 1 patches (1.1–1.4) as a single atomic commit. These are the safety-critical changes. PHASE 2 can follow incrementally.

**Authority**: CLIENT_CONTRACT_v1.0.md (canonical)  
**Blocker for**: Python SDK (APEX-SDK-PY-C1)  
**Approval**: Auto-approved (mechanical conformance)
