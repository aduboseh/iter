/**
 * Iter TypeScript SDK
 *
 * Thin client for the Iter MCP protocol. This SDK provides:
 * - Type-safe request/response handling
 * - Protocol version compatibility checking
 * - Trace context propagation
 *
 * Design Principles:
 * - Thin: No business logic; pure protocol wrapper
 * - Contract-driven: Types derived from protocol specification
 * - Version-aware: Fails fast on incompatible versions
 * - Telemetry-safe: Passes trace context, never enriches payloads
 */

import { spawn, ChildProcess } from "child_process";
import { Readable, Writable } from "stream";
import * as readline from "readline";

// ============================================================================
// Protocol Version
// ============================================================================

/** SDK protocol version (must match server) */
export const SDK_PROTOCOL_VERSION = "1.0.0";

/** Minimum supported server protocol version */
export const MIN_SERVER_VERSION = "1.0.0";

/** Maximum supported server protocol version */
export const MAX_SERVER_VERSION = "1.99.99";

// ============================================================================
// Trace Context
// ============================================================================

/** Trace context for request correlation */
export interface TraceContext {
  traceId: string;
  spanId: string;
  parentSpanId?: string;
}

/** Create a new trace context */
export function createTraceContext(traceId: string): TraceContext {
  return {
    traceId,
    spanId: traceId,
  };
}

// ============================================================================
// Request/Response Types (Contract-Driven)
// ============================================================================

/** JSON-RPC 2.0 Request */
export interface RpcRequest {
  jsonrpc: "2.0";
  method: string;
  params?: unknown;
  id: number | string;
}

/** JSON-RPC 2.0 Response */
export interface RpcResponse {
  jsonrpc: "2.0";
  result?: unknown;
  error?: RpcError;
  id: number | string;
}

/** JSON-RPC 2.0 Error */
export interface RpcError {
  code: number;
  message: string;
  data?: unknown;
}

// ============================================================================
// SDK Error Types
// ============================================================================

export class SdkError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "SdkError";
  }
}

export class VersionMismatchError extends SdkError {
  constructor(
    public readonly clientVersion: string,
    public readonly serverVersion: string
  ) {
    super(
      `Version mismatch: client=${clientVersion}, server=${serverVersion}`
    );
    this.name = "VersionMismatchError";
  }
}

export class ConnectionError extends SdkError {
  constructor(message: string) {
    super(`Connection failed: ${message}`);
    this.name = "ConnectionError";
  }
}

export class RequestError extends SdkError {
  constructor(public readonly rpcError: RpcError) {
    super(`Request failed: ${rpcError.message} (${rpcError.code})`);
    this.name = "RequestError";
  }
}

export class BackpressureError extends SdkError {
  constructor(public readonly maxInflight: number) {
    super(`Backpressure: maxInflight=${maxInflight} exceeded`);
    this.name = "BackpressureError";
  }
}

export class RequestTimeoutError extends SdkError {
  constructor(
    public readonly method: string,
    public readonly timeoutMs: number
  ) {
    super(`Request timeout: ${method} exceeded ${timeoutMs}ms`);
    this.name = "RequestTimeoutError";
  }
}

export class ConnectionClosedError extends SdkError {
  constructor(
    message: string = "Connection closed",
    public readonly pendingCountAtClose?: number
  ) {
    super(message);
    this.name = "ConnectionClosedError";
  }
}


// ============================================================================
// Response Types (MCP-aligned)
// ============================================================================

export interface ToolInfo {
  name: string;
  description: string;
  inputSchema: Record<string, unknown>;
}

export interface ToolListResponse {
  tools: ToolInfo[];
}

export interface NodeState {
  id: number;
  belief: number;
  energy: number;
  esv_valid: boolean;
  stability: number;
}

export interface GovernorStatus {
  drift_ok: boolean;
  energy_drift: number;
  coherence: number;
  node_count: number;
  edge_count: number;
  healthy: boolean;
}

// ============================================================================
// Client
// ============================================================================

/** Iter MCP client (STDIO transport) */
export class IterClient {
  private static readonly STDERR_RING_MAX_BYTES = 10 * 1024;
  private static readonly MAX_PROTOCOL_VIOLATIONS = 3;

  private process: ChildProcess | null = null;
  private stdin: Writable | null = null;
  private stdout: Readable | null = null;
  private requestId = 0;
  private _traceContext: TraceContext | null = null;
  private responseQueue: Map<
    number,
    { resolve: (value: RpcResponse) => void; reject: (error: Error) => void }
  > = new Map();
  private lineReader: readline.Interface | null = null;
  private readonly maxInflight: number;
  private _state: "open" | "closing" | "closed" = "open";
  private _closePromise: Promise<void> | null = null;

  private _stderrBytes: Buffer = Buffer.alloc(0);
  private _protocolViolationCount = 0;
  private _circuitBreakerCloseStarted = false;

  private constructor(maxInflight: number = 1) {
    this.maxInflight = maxInflight;
  }


  /** Get the current trace context */
  get traceContext(): TraceContext | null {
    return this._traceContext;
  }

  /** Connect to an Iter server process */
  static async connect(
    binaryPath: string,
    options?: { maxInflight?: number }
  ): Promise<IterClient> {
    const client = new IterClient(options?.maxInflight ?? 1);

    client.process = spawn(binaryPath, [], {
      stdio: ["pipe", "pipe", "pipe"],
    });

    if (!client.process.stdin || !client.process.stdout) {
      throw new ConnectionError("Failed to open stdio");
    }

    client.stdin = client.process.stdin;
    client.stdout = client.process.stdout;

    client.attachProcessHandlers(client.process);

    // Set up line-based response reading
    client.lineReader = readline.createInterface({
      input: client.stdout,
      crlfDelay: Infinity,
    });

    client.lineReader.on("line", (line) => {
      client.handleStdoutLine(line);
    });

    return client;
  }

  /** Set trace context for subsequent requests */
  withTrace(trace: TraceContext): this {
    this._traceContext = trace;
    return this;
  }

  /** Send a raw JSON-RPC request */
  async send(
    method: string,
    params?: unknown,
    timeoutMs: number = 30000
  ): Promise<RpcResponse> {
    if (this._state !== "open") {
      throw new ConnectionClosedError(
        "Client is closing or closed, cannot send request"
      );
    }

    if (this.responseQueue.size >= this.maxInflight) {
      throw new BackpressureError(this.maxInflight);
    }

    if (!this.stdin) {
      throw new ConnectionError("Not connected");
    }


    this.requestId++;
    const id = this.requestId;

    const request: RpcRequest = {
      jsonrpc: "2.0",
      method,
      params,
      id,
    };

    return new Promise((resolve, reject) => {
      // Set up timeout for request eviction
      const timer = setTimeout(() => {
        this.responseQueue.delete(id);
        reject(new RequestTimeoutError(method, timeoutMs));
      }, timeoutMs);

      this.responseQueue.set(id, {
        resolve: (response) => {
          this.responseQueue.delete(id);
          clearTimeout(timer);
          resolve(response);
        },
        reject: (error) => {
          this.responseQueue.delete(id);
          clearTimeout(timer);
          reject(error);
        },
      });

      this.stdin!.write(JSON.stringify(request) + "\n");
    });
  }

  /** List available tools */
  async toolsList(): Promise<ToolInfo[]> {
    const response = await this.send("tools/list");

    if (response.error) {
      throw new RequestError(response.error);
    }

    const result = response.result as ToolListResponse;
    return result.tools;
  }

  /** Create a node */
  async nodeCreate(belief: number, energy: number): Promise<NodeState> {
    const response = await this.send("tools/call", {
      name: "node.create",
      arguments: { belief, energy },
    });

    return this.parseToolResult<NodeState>(response);
  }

  /** Query a node */
  async nodeQuery(nodeId: number): Promise<NodeState> {
    const response = await this.send("tools/call", {
      name: "node.query",
      arguments: { node_id: nodeId },
    });

    return this.parseToolResult<NodeState>(response);
  }

  /** Get governor status */
  async governorStatus(): Promise<GovernorStatus> {
    const response = await this.send("tools/call", {
      name: "governor.status",
      arguments: {},
    });

    return this.parseToolResult<GovernorStatus>(response);
  }

  /** Close the connection */
  async close(): Promise<void> {
    if (this._closePromise) {
      return this._closePromise;
    }

    this._closePromise = this.performClose();
    return this._closePromise;
  }

  private async waitForDrain(timeoutMs: number): Promise<void> {
    const intervalMs = 25;
    const maxIterations = Math.ceil(timeoutMs / intervalMs);
    let iterations = 0;

    while (this.responseQueue.size > 0 && iterations < maxIterations) {
      await new Promise((resolve) => setTimeout(resolve, intervalMs));
      iterations++;
    }
  }

  private async waitForExit(timeoutMs: number): Promise<boolean> {
    if (!this.process) return true;

    return new Promise<boolean>((resolve) => {
      const proc = this.process!;
      let done = false;

      const finish = (exitObserved: boolean) => {
        if (done) return;
        done = true;
        proc.removeListener("exit", onExit);
        proc.removeListener("close", onExit);
        resolve(exitObserved);
      };

      const onExit = () => finish(true);

      proc.once("exit", onExit);
      proc.once("close", onExit);
      setTimeout(() => finish(false), timeoutMs);
    });
  }

  private async performClose(): Promise<void> {
    this._state = "closing";

    await this.waitForDrain(5000);

    const pendingCount = this.responseQueue.size;
    if (pendingCount > 0) {
      const error = new ConnectionClosedError(
        "Client closed during drain, request did not complete in time",
        pendingCount
      );
      for (const pending of this.responseQueue.values()) {
        pending.reject(error);
      }
      this.responseQueue.clear();
    }

    if (this.lineReader) {
      this.lineReader.close();
      this.lineReader = null;
    }

    if (this.process && !this.process.killed) {
      this.process.kill("SIGTERM");

      const exitedAfterSigterm = await this.waitForExit(2000);

      if (!exitedAfterSigterm && this.process && !this.process.killed) {
        this.process.kill("SIGKILL");

        const exitedAfterSigkill = await this.waitForExit(1000);

        if (!exitedAfterSigkill) {
          throw this.makeConnectionErrorWithStderr(
            "Failed to reap process after SIGKILL (zombie detected)"
          );
        }
      }
    }

    this.process = null;
    this.stdin = null;
    this.stdout = null;
    this._state = "closed";
  }


  private appendStderrBytes(chunk: Buffer) {
    if (chunk.length === 0) return;

    const max = IterClient.STDERR_RING_MAX_BYTES;
    if (chunk.length >= max) {
      this._stderrBytes = chunk.subarray(chunk.length - max);
      return;
    }

    const combined = Buffer.concat([this._stderrBytes, chunk]);
    if (combined.length <= max) {
      this._stderrBytes = combined;
      return;
    }

    this._stderrBytes = combined.subarray(combined.length - max);
  }

  private getStderrSnapshotText(): string {
    if (this._stderrBytes.length === 0) return "";

    return this._stderrBytes
      .toString("utf8")
      .replace(/\r\n/g, "\n")
      .replace(/\r/g, "\n");
  }

  private makeConnectionErrorWithStderr(message: string): ConnectionError {
    const stderr = this.getStderrSnapshotText();
    if (!stderr) return new ConnectionError(message);

    return new ConnectionError(`${message}\nstderr:\n${stderr}`);
  }

  private attachProcessHandlers(proc: ChildProcess) {
    if (proc.stderr) {
      proc.stderr.on("data", (chunk: Buffer) => {
        const buf = Buffer.isBuffer(chunk) ? chunk : Buffer.from(String(chunk));
        this.appendStderrBytes(buf);
      });
    }

    proc.on("error", (err) => {
      const error = this.makeConnectionErrorWithStderr(err.message);
      for (const pending of this.responseQueue.values()) {
        pending.reject(error);
      }
      this.responseQueue.clear();
    });

    proc.on("exit", (code: number | null, signal: NodeJS.Signals | null) => {
      const nonZeroExit = typeof code === "number" && code !== 0;
      const msg = nonZeroExit
        ? `Process exited with code ${code}${signal ? ` signal ${signal}` : ""}`
        : `Process exited${signal ? ` signal ${signal}` : ""}`;

      const error = nonZeroExit
        ? this.makeConnectionErrorWithStderr(msg)
        : new ConnectionError(msg);

      for (const pending of this.responseQueue.values()) {
        pending.reject(error);
      }
      this.responseQueue.clear();
    });
  }

  private failClosedProtocolViolation(reason: string) {
    if (this._state === "closed") return;

    const error = new ConnectionError(reason);
    for (const pending of this.responseQueue.values()) {
      pending.reject(error);
    }
    this.responseQueue.clear();

    if (!this._circuitBreakerCloseStarted) {
      this._circuitBreakerCloseStarted = true;
      void this.close().catch(() => {});
    }
  }

  private handleStdoutLine(line: string) {
    if (this._state !== "open") return;

    let parsed: unknown;
    try {
      parsed = JSON.parse(line);
    } catch {
      this._protocolViolationCount++;

      // Not attributable to a request ID. If any request is in-flight, fail-closed immediately.
      if (this.responseQueue.size > 0) {
        this.failClosedProtocolViolation("Protocol violation: malformed stdout");
        return;
      }

      if (this._protocolViolationCount >= IterClient.MAX_PROTOCOL_VIOLATIONS) {
        this.failClosedProtocolViolation("Protocol violation: malformed stdout");
      }
      return;
    }

    const response = parsed as Partial<RpcResponse> & { id?: unknown };
    const id = response.id;
    const hasAttributableId = typeof id === "number" || typeof id === "string";

    if (!hasAttributableId) {
      this._protocolViolationCount++;
      if (this._protocolViolationCount >= IterClient.MAX_PROTOCOL_VIOLATIONS) {
        this.failClosedProtocolViolation("Protocol violation: invalid JSON-RPC");
      }
      return;
    }

    const pending = this.responseQueue.get(id as number);
    if (!pending) {
      console.warn(
        `[Iter SDK] Ignoring response for unknown/timed-out request ID: ${id}`
      );
      return;
    }

    if (response.jsonrpc !== "2.0") {
      this.responseQueue.delete(id as number);
      pending.reject(
        new RequestError({ code: -32700, message: "Malformed JSON-RPC response" })
      );
      return;
    }

    pending.resolve(response as RpcResponse);
  }

  private parseToolResult<T>(response: RpcResponse): T {
    if (response.error) {
      throw new RequestError(response.error);
    }

    const result = response.result as { content?: { text?: string }[] };
    const text = result?.content?.[0]?.text;

    if (!text) {
      throw new RequestError({
        code: -1,
        message: "Invalid tool response format",
      });
    }

    return JSON.parse(text) as T;
  }
}

// ============================================================================
// Version Checking
// ============================================================================

/** Parse a semver version string */
function parseVersion(version: string): [number, number, number] | null {
  const parts = version.split(".");
  if (parts.length !== 3) return null;

  const nums = parts.map((p) => parseInt(p, 10));
  if (nums.some((n) => isNaN(n))) return null;

  return nums as [number, number, number];
}

/** Compare two version tuples */
function compareVersions(
  a: [number, number, number],
  b: [number, number, number]
): number {
  for (let i = 0; i < 3; i++) {
    if (a[i] < b[i]) return -1;
    if (a[i] > b[i]) return 1;
  }
  return 0;
}

/** Check if a server version is compatible with this SDK */
export function isVersionCompatible(serverVersion: string): boolean {
  const server = parseVersion(serverVersion);
  if (!server) return false;

  const min = parseVersion(MIN_SERVER_VERSION)!;
  const max = parseVersion(MAX_SERVER_VERSION)!;

  return compareVersions(server, min) >= 0 && compareVersions(server, max) <= 0;
}
