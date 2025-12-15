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

  private constructor() {}

  /** Get the current trace context */
  get traceContext(): TraceContext | null {
    return this._traceContext;
  }

  /** Connect to an Iter server process */
  static async connect(binaryPath: string): Promise<IterClient> {
    const client = new IterClient();

    client.process = spawn(binaryPath, [], {
      stdio: ["pipe", "pipe", "ignore"],
    });

    if (!client.process.stdin || !client.process.stdout) {
      throw new ConnectionError("Failed to open stdio");
    }

    client.stdin = client.process.stdin;
    client.stdout = client.process.stdout;

    // Set up line-based response reading
    client.lineReader = readline.createInterface({
      input: client.stdout,
      crlfDelay: Infinity,
    });

    client.lineReader.on("line", (line) => {
      try {
        const response: RpcResponse = JSON.parse(line);
        const pending = client.responseQueue.get(response.id as number);
        if (pending) {
          client.responseQueue.delete(response.id as number);
          pending.resolve(response);
        }
      } catch (e) {
        // Ignore malformed lines
      }
    });

    client.process.on("error", (err) => {
      for (const pending of client.responseQueue.values()) {
        pending.reject(new ConnectionError(err.message));
      }
      client.responseQueue.clear();
    });

    client.process.on("exit", () => {
      for (const pending of client.responseQueue.values()) {
        pending.reject(new ConnectionError("Process exited"));
      }
      client.responseQueue.clear();
    });

    return client;
  }

  /** Set trace context for subsequent requests */
  withTrace(trace: TraceContext): this {
    this._traceContext = trace;
    return this;
  }

  /** Send a raw JSON-RPC request */
  async send(method: string, params?: unknown): Promise<RpcResponse> {
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
      this.responseQueue.set(id, { resolve, reject });
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
  close(): void {
    if (this.lineReader) {
      this.lineReader.close();
      this.lineReader = null;
    }
    if (this.process) {
      this.process.kill();
      this.process = null;
    }
    this.stdin = null;
    this.stdout = null;
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
