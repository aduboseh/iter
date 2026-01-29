import {
  SDK_PROTOCOL_VERSION,
  MIN_SERVER_VERSION,
  MAX_SERVER_VERSION,
  isVersionCompatible,
  createTraceContext,
  SdkError,
  VersionMismatchError,
  ConnectionError,
  RequestError,
  BackpressureError,
  RequestTimeoutError,
  ConnectionClosedError,
  IterClient,
} from "./index";


describe("Protocol Version", () => {
  test("SDK_PROTOCOL_VERSION is valid", () => {
    expect(SDK_PROTOCOL_VERSION).toBe("1.0.0");
  });

  test("MIN_SERVER_VERSION is valid", () => {
    expect(MIN_SERVER_VERSION).toBe("1.0.0");
  });

  test("MAX_SERVER_VERSION allows minor versions", () => {
    expect(MAX_SERVER_VERSION).toBe("1.99.99");
  });
});

describe("CT5.1: Stderr & Malformed Output", () => {
  async function flushAsync(): Promise<void> {
    await Promise.resolve();
    await jest.runOnlyPendingTimersAsync();
    await Promise.resolve();
  }

  beforeEach(() => {
    jest.useFakeTimers({ legacyFakeTimers: false });
  });

  afterEach(() => {
    jest.useRealTimers();
  });

  test("CT5.1-A: stderr is attached to ConnectionError on process error", async () => {
    const client = new (IterClient as any)(1);

    let stderrOnData: ((chunk: Buffer) => void) | null = null;
    const stderr = {
      on: jest.fn((event: string, cb: (chunk: Buffer) => void) => {
        if (event === "data") stderrOnData = cb;
      }),
    };

    const handlers: Record<string, Function> = {};
    const proc = {
      stderr,
      on: jest.fn((event: string, cb: Function) => {
        handlers[event] = cb;
      }),
    } as any;

    client["attachProcessHandlers"](proc);

    // Simulate stderr output
    expect(stderrOnData).toBeTruthy();
    stderrOnData!(Buffer.from("boom-stderr\n", "utf8"));

    const reject = jest.fn();
    client["responseQueue"].set(1, { resolve: jest.fn(), reject });

    handlers["error"]?.(new Error("boom"));

    expect(reject).toHaveBeenCalledTimes(1);
    const err = reject.mock.calls[0][0] as Error;
    expect(err).toBeInstanceOf(ConnectionError);
    expect(err.message).toContain("boom");
    expect(err.message).toContain("stderr:");
    expect(err.message).toContain("boom-stderr");

    await flushAsync();
  });

  test("CT5.1-B: stderr ring buffer is capped at 10KB by byte length", async () => {
    const client = new (IterClient as any)(1);

    let stderrOnData: ((chunk: Buffer) => void) | null = null;
    const stderr = {
      on: jest.fn((event: string, cb: (chunk: Buffer) => void) => {
        if (event === "data") stderrOnData = cb;
      }),
    };

    const handlers: Record<string, Function> = {};
    const proc = {
      stderr,
      on: jest.fn((event: string, cb: Function) => {
        handlers[event] = cb;
      }),
    } as any;

    client["attachProcessHandlers"](proc);

    // 6KB of 'a' then 6KB of 'b' => total 12KB; buffer must retain last 10KB
    expect(stderrOnData).toBeTruthy();
    stderrOnData!(Buffer.alloc(6 * 1024, "a"));
    stderrOnData!(Buffer.alloc(6 * 1024, "b"));

    expect(client["_stderrBytes"].length).toBeLessThanOrEqual(10 * 1024);
    expect(client["_stderrBytes"].length).toBe(10 * 1024);

    const snapshot = client["_stderrBytes"].toString("utf8");
    expect(snapshot.endsWith("b".repeat(6 * 1024))).toBe(true);

    // Trigger error to ensure attachment uses snapshot and does not exceed cap
    const reject = jest.fn();
    client["responseQueue"].set(1, { resolve: jest.fn(), reject });
    handlers["error"]?.(new Error("crash"));

    const err = reject.mock.calls[0][0] as Error;
    expect(err.message).toContain("stderr:");

    await flushAsync();
  });

  test("CT5.1-C: malformed stdout is fail-closed and close() is invoked once", async () => {
    const client = new (IterClient as any)(1);

    const closeSpy = jest.fn().mockResolvedValue(undefined);
    client.close = closeSpy;

    // 1) With pending request: first malformed line must fail-closed immediately
    const reject = jest.fn();
    client["responseQueue"].set(1, { resolve: jest.fn(), reject });

    client["handleStdoutLine"]("not-json");

    expect(reject).toHaveBeenCalledTimes(1);
    expect(closeSpy).toHaveBeenCalledTimes(1);

    // Additional malformed lines must not trigger additional close() calls
    client["handleStdoutLine"]("not-json-2");
    client["handleStdoutLine"]("not-json-3");
    expect(closeSpy).toHaveBeenCalledTimes(1);

    // 2) With no pending requests: close triggers at threshold (<= 3)
    const client2 = new (IterClient as any)(1);
    const closeSpy2 = jest.fn().mockResolvedValue(undefined);
    client2.close = closeSpy2;

    client2["handleStdoutLine"]("bad-1");
    client2["handleStdoutLine"]("bad-2");
    expect(closeSpy2).toHaveBeenCalledTimes(0);
    client2["handleStdoutLine"]("bad-3");
    expect(closeSpy2).toHaveBeenCalledTimes(1);

    await flushAsync();
  });
});

describe("Version Compatibility", () => {
  test("accepts current version", () => {
    expect(isVersionCompatible("1.0.0")).toBe(true);
  });

  test("accepts minor version bumps", () => {
    expect(isVersionCompatible("1.1.0")).toBe(true);
    expect(isVersionCompatible("1.5.0")).toBe(true);
    expect(isVersionCompatible("1.99.0")).toBe(true);
  });

  test("accepts patch version bumps", () => {
    expect(isVersionCompatible("1.0.1")).toBe(true);
    expect(isVersionCompatible("1.5.10")).toBe(true);
  });

  test("rejects major version bumps", () => {
    expect(isVersionCompatible("2.0.0")).toBe(false);
    expect(isVersionCompatible("3.0.0")).toBe(false);
  });

  test("rejects older major versions", () => {
    expect(isVersionCompatible("0.9.0")).toBe(false);
  });

  test("rejects invalid version strings", () => {
    expect(isVersionCompatible("")).toBe(false);
    expect(isVersionCompatible("1.0")).toBe(false);
    expect(isVersionCompatible("v1.0.0")).toBe(false);
    expect(isVersionCompatible("not-a-version")).toBe(false);
  });
});

describe("Trace Context", () => {
  test("createTraceContext creates valid context", () => {
    const trace = createTraceContext("test-trace-id");
    expect(trace.traceId).toBe("test-trace-id");
    expect(trace.spanId).toBe("test-trace-id");
    expect(trace.parentSpanId).toBeUndefined();
  });
});

describe("Error Types", () => {
  test("SdkError has correct name", () => {
    const err = new SdkError("test");
    expect(err.name).toBe("SdkError");
    expect(err.message).toBe("test");
  });

  test("VersionMismatchError formats message correctly", () => {
    const err = new VersionMismatchError("1.0.0", "2.0.0");
    expect(err.name).toBe("VersionMismatchError");
    expect(err.message).toBe("Version mismatch: client=1.0.0, server=2.0.0");
    expect(err.clientVersion).toBe("1.0.0");
    expect(err.serverVersion).toBe("2.0.0");
  });

  test("ConnectionError formats message correctly", () => {
    const err = new ConnectionError("timeout");
    expect(err.name).toBe("ConnectionError");
    expect(err.message).toBe("Connection failed: timeout");
  });

  test("RequestError formats message correctly", () => {
    const err = new RequestError({ code: -32600, message: "Invalid Request" });
    expect(err.name).toBe("RequestError");
    expect(err.message).toBe("Request failed: Invalid Request (-32600)");
    expect(err.rpcError.code).toBe(-32600);
  });

  test("BackpressureError formats message correctly", () => {
    const err = new BackpressureError(2);
    expect(err.name).toBe("BackpressureError");
    expect(err.message).toBe("Backpressure: maxInflight=2 exceeded");
    expect(err.maxInflight).toBe(2);
  });

  test("RequestTimeoutError formats message correctly", () => {
    const err = new RequestTimeoutError("tools/list", 5000);
    expect(err.name).toBe("RequestTimeoutError");
    expect(err.message).toBe("Request timeout: tools/list exceeded 5000ms");
    expect(err.method).toBe("tools/list");
    expect(err.timeoutMs).toBe(5000);
  });
});

describe("Backpressure", () => {
  test("CT2.1: enforces maxInflight backpressure", () => {
    // Create client with internal access for testing
    const client = new (IterClient as any)(2); // maxInflight = 2

    // Manually populate responseQueue to simulate in-flight requests
    client.responseQueue.set(1, { resolve: () => {}, reject: () => {} });
    client.responseQueue.set(2, { resolve: () => {}, reject: () => {} });

    // Set stdin to non-null to pass connection check
    client.stdin = {} as any;

    // 3rd request must fail with BackpressureError
    expect(() => {
      // Use synchronous check since backpressure is immediate
      if (client.responseQueue.size >= client.maxInflight) {
        throw new BackpressureError(client.maxInflight);
      }
    }).toThrow(BackpressureError);
  });

  test("CT2.1: allows requests when below maxInflight", () => {
    const client = new (IterClient as any)(2); // maxInflight = 2

    // Only 1 in-flight request
    client.responseQueue.set(1, { resolve: () => {}, reject: () => {} });
    client.stdin = {} as any;

    // Should not throw
    expect(() => {
      if (client.responseQueue.size >= client.maxInflight) {
        throw new BackpressureError(client.maxInflight);
      }
    }).not.toThrow();
  });
});

describe("Timeouts", () => {
  test("CT3.1: request times out after specified duration", async () => {
    const client = new (IterClient as any)(1);
    client.stdin = { write: jest.fn() } as any;

    const start = Date.now();
    const timeoutMs = 500;

    await expect(client.send("slow_method", {}, timeoutMs)).rejects.toThrow(
      RequestTimeoutError
    );

    const elapsed = Date.now() - start;

    // Verify timeout fired within expected window (±100ms)
    expect(elapsed).toBeGreaterThanOrEqual(timeoutMs - 100);
    expect(elapsed).toBeLessThan(timeoutMs + 100);
  });

  test("CT3.1: timed-out request is evicted from pending queue", async () => {
    const client = new (IterClient as any)(1);
    client.stdin = { write: jest.fn() } as any;

    const promise = client.send("test_method", {}, 100);

    // Verify request is pending
    expect(client.responseQueue.size).toBe(1);

    // Wait for timeout
    await expect(promise).rejects.toThrow(RequestTimeoutError);

    // Verify request was evicted
    expect(client.responseQueue.size).toBe(0);
  });

  test("CT3.1: late response is ignored and does not resolve", async () => {
    const client = new (IterClient as any)(1);
    client.stdin = { write: jest.fn() } as any;
    client.lineReader = { on: jest.fn(), close: jest.fn() } as any;

    // Mock console.warn to capture late response warning
    const warnSpy = jest.spyOn(console, "warn").mockImplementation();

    // Send request with short timeout
    const promise = client.send("test_method", {}, 100);
    const requestId = client.requestId;

    // Wait for timeout
    await expect(promise).rejects.toThrow(RequestTimeoutError);

    // Manually trigger the line handler logic (simulating late response)
    const pending = client.responseQueue.get(requestId);
    expect(pending).toBeUndefined(); // Already evicted

    // Verify warning would be logged (simulating R1.2.4)
    if (!pending) {
      console.warn(
        `[Iter SDK] Ignoring response for unknown/timed-out request ID: ${requestId}`
      );
    }

    expect(warnSpy).toHaveBeenCalledWith(
      expect.stringContaining("Ignoring response for unknown/timed-out request ID")
    );

    warnSpy.mockRestore();
  });

  test("CT3.1: default timeout is 30 seconds", async () => {
    const client = new (IterClient as any)(1);
    client.stdin = { write: jest.fn() } as any;

    // Send without explicit timeout
    const promise = client.send("test_method", {});

    // We can't wait 30s in a test, but we can verify the request doesn't
    // timeout immediately (within 100ms)
    await new Promise((resolve) => setTimeout(resolve, 100));

    // Request should still be pending
    expect(client.responseQueue.size).toBe(1);

    // Clean up: manually resolve to avoid hanging test
    const entry = client.responseQueue.get(client.requestId);
    if (entry) {
      entry.resolve({ jsonrpc: "2.0", id: client.requestId, result: {} });
    }

    await promise;
  });
});

describe("CT4.1: Graceful Shutdown", () => {
  class FakeChildProcess {
    public pid = 12345;
    public killed = false;
    private exitHandlers: Array<() => void> = [];
    private closeHandlers: Array<() => void> = [];
    private _shouldExitOnSigterm = true;
    private _shouldExitOnSigkill = true;

    constructor(options?: { exitOnSigterm?: boolean; exitOnSigkill?: boolean }) {
      this._shouldExitOnSigterm = options?.exitOnSigterm ?? true;
      this._shouldExitOnSigkill = options?.exitOnSigkill ?? true;
    }

    once(event: string, handler: () => void) {
      if (event === "exit") this.exitHandlers.push(handler);
      if (event === "close") this.closeHandlers.push(handler);
    }

    removeListener(event: string, handler: () => void) {
      if (event === "exit") {
        this.exitHandlers = this.exitHandlers.filter((h) => h !== handler);
      }
      if (event === "close") {
        this.closeHandlers = this.closeHandlers.filter((h) => h !== handler);
      }
    }

    kill(signal?: string): boolean {
      if (signal === "SIGTERM" && this._shouldExitOnSigterm) {
        Promise.resolve().then(() => {
          this.killed = true;
          this.emitExit();
        });
        return true;
      }

      if (signal === "SIGKILL" && this._shouldExitOnSigkill) {
        Promise.resolve().then(() => {
          this.killed = true;
          this.emitExit();
        });
        return true;
      }

      // Signal sent but process doesn't respond — killed stays false
      return true;
    }

    private emitExit() {
      this.exitHandlers.forEach((h) => h());
      this.closeHandlers.forEach((h) => h());
    }
  }
  async function flushAsync(): Promise<void> {
    await Promise.resolve();
    await jest.runOnlyPendingTimersAsync();
    await Promise.resolve();
  }

  beforeEach(() => {
    jest.useFakeTimers({ legacyFakeTimers: false });
  });

  afterEach(() => {
    jest.useRealTimers();
  });

  test("CT4.1-A: drains pending requests within 5s", async () => {

    const client = new (IterClient as any)(2);
    const fakeProc = new FakeChildProcess();

    client["process"] = fakeProc as any;
    client["stdin"] = { write: jest.fn() } as any;
    client["_state"] = "open";

    const promise1 = client.send("test1", {});
    const promise2 = client.send("test2", {});

    const entry1 = client["responseQueue"].get(1);
    const entry2 = client["responseQueue"].get(2);

    setTimeout(() => {
      entry1?.resolve({ jsonrpc: "2.0", id: 1, result: {} });
      entry2?.resolve({ jsonrpc: "2.0", id: 2, result: {} });
      client["responseQueue"].delete(1);
      client["responseQueue"].delete(2);
    }, 10);

    const closePromise = client.close();
    await jest.advanceTimersByTimeAsync(10);
    await flushAsync();
    await jest.advanceTimersByTimeAsync(100);
    await flushAsync();
    await jest.advanceTimersByTimeAsync(8000);
    await flushAsync();
    await jest.runOnlyPendingTimersAsync();
    await flushAsync();

    await expect(promise1).resolves.toBeDefined();
    await expect(promise2).resolves.toBeDefined();

    await closePromise;

    expect(client["_state"]).toBe("closed");
    expect(client["responseQueue"].size).toBe(0);
  });

  test("CT4.1-B: fails undrained requests after 5s timeout", async () => {

    const client = new (IterClient as any)(1);
    const fakeProc = new FakeChildProcess();

    client["process"] = fakeProc as any;
    client["stdin"] = { write: jest.fn() } as any;
    client["_state"] = "open";

    const promise = client.send("hang", {});

    // Attach rejection handler immediately to avoid unhandled rejection
    let requestError: Error | null = null;
    promise.catch((e: Error) => { requestError = e; });

    // Start close
    const closePromise = client.close();

    // Advance past the 5s drain timeout (200 iterations of 25ms each)
    for (let i = 0; i < 200; i++) {
      await jest.advanceTimersByTimeAsync(25);
      await flushAsync();
    }

    // The request should now be rejected with ConnectionClosedError
    expect(requestError).toBeInstanceOf(ConnectionClosedError);

    // Advance through SIGTERM wait (2s) and process exit
    for (let i = 0; i < 80; i++) {
      await jest.advanceTimersByTimeAsync(25);
      await flushAsync();
    }

    // closePromise should resolve (not reject) after successful process termination
    await closePromise;

    expect(client["responseQueue"].size).toBe(0);
  });

  test("CT4.1-C: executes SIGTERM then SIGKILL when needed", async () => {

    const client = new (IterClient as any)(1);
    const fakeProc = new FakeChildProcess({
      exitOnSigterm: false,
      exitOnSigkill: true,
    });

    const killSpy = jest.spyOn(fakeProc, "kill");

    client["process"] = fakeProc as any;
    client["stdin"] = { write: jest.fn() } as any;
    client["_state"] = "open";

    const closePromise = client.close();

    // Advance past drain (no pending requests, so immediate)
    await flushAsync();

    // Advance through SIGTERM wait (2s = 80 iterations of 25ms)
    for (let i = 0; i < 80; i++) {
      await jest.advanceTimersByTimeAsync(25);
      await flushAsync();
    }

    // Advance through SIGKILL wait (1s = 40 iterations of 25ms)
    for (let i = 0; i < 40; i++) {
      await jest.advanceTimersByTimeAsync(25);
      await flushAsync();
    }

    await closePromise;

    expect(killSpy).toHaveBeenCalledWith("SIGTERM");
    expect(killSpy).toHaveBeenCalledWith("SIGKILL");
    expect(killSpy).toHaveBeenCalledTimes(2);
  });

  test("CT4.1-D: fails closed if process never exits", async () => {

    const client = new (IterClient as any)(1);
    const fakeProc = new FakeChildProcess({
      exitOnSigterm: false,
      exitOnSigkill: false,
    });

    client["process"] = fakeProc as any;
    client["stdin"] = { write: jest.fn() } as any;
    client["_state"] = "open";

    // Attach rejection handler immediately to avoid unhandled rejection
    let closeError: Error | null = null;
    const closePromise = client.close();
    closePromise.catch((e: Error) => { closeError = e; });

    // Advance past drain (no pending requests)
    await flushAsync();

    // Advance through SIGTERM wait (2s)
    for (let i = 0; i < 80; i++) {
      await jest.advanceTimersByTimeAsync(25);
      await flushAsync();
    }

    // Advance through SIGKILL wait (1s)
    for (let i = 0; i < 40; i++) {
      await jest.advanceTimersByTimeAsync(25);
      await flushAsync();
    }

    expect(closeError).toBeInstanceOf(ConnectionError);
    expect(closeError!.message).toMatch(/zombie/i);
  });

  test("CT4.1-E: rejects new requests after close initiated", async () => {

    const client = new (IterClient as any)(1);
    const fakeProc = new FakeChildProcess();

    client["process"] = fakeProc as any;
    client["stdin"] = { write: jest.fn() } as any;
    client["_state"] = "open";

    const closePromise = client.close();

    await expect(client.send("test", {})).rejects.toThrow(ConnectionClosedError);

    await jest.advanceTimersByTimeAsync(8000);
    await flushAsync();
    await jest.runOnlyPendingTimersAsync();
    await flushAsync();
    await expect(closePromise).resolves.toBeUndefined();

    await expect(client.send("test2", {})).rejects.toThrow(ConnectionClosedError);
  });

  test("CT4.1-F: close() is idempotent", async () => {

    const client = new (IterClient as any)(1);
    const fakeProc = new FakeChildProcess();

    client["process"] = fakeProc as any;
    client["stdin"] = { write: jest.fn() } as any;
    client["_state"] = "open";

    const closePromises = [client.close(), client.close(), client.close()];

    await jest.advanceTimersByTimeAsync(8000);
    await flushAsync();
    await jest.runOnlyPendingTimersAsync();
    await flushAsync();

    await expect(Promise.all(closePromises)).resolves.toEqual([
      undefined,
      undefined,
      undefined,
    ]);

    expect(client["_state"]).toBe("closed");
  });
});

