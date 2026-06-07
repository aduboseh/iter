# Iter TypeScript SDK

Thin client SDK for the Iter MCP protocol (Node.js).

## Design Principles

- **Thin**: No business logic; pure protocol wrapper
- **Contract-driven**: Types derived from protocol specification
- **Version-aware**: Fails fast on incompatible protocol versions (supports N, N-1)
- **Telemetry-safe**: Passes trace context through, never enriches payloads

## Installation

```bash
npm install @iter/sdk
```

## Usage

```typescript
import { IterClient, createTraceContext } from "@iter/sdk";

async function main() {
  // Connect to an Iter server
  const client = await IterClient.connect("iter-server");

  // Set trace context for distributed tracing
  client.withTrace(createTraceContext("my-trace-id"));

  // List available tools
  const tools = await client.toolsList();
  console.log("Available tools:", tools);

  // Create a node
  const node = await client.nodeCreate(0.5, 1.0);
  console.log("Created node:", node);

  // Query a node
  const state = await client.nodeQuery(node.id);
  console.log("Node state:", state);

  // Register the governed resource before decision checks/previews.
  const resourceHash = "sha256:8e51aaaa299f88b416976abd2a25a7d3a0db01b61b105066013f43a077408e25";
  await client.registerResource({
    resource_path: "docs/README.md",
    expected_hash: resourceHash,
  });

  // Preview a governance decision without mutating lineage.
  const preview = await client.decisionPreview({
    proposal_id: "proposal-1",
    state_snapshot_hash: resourceHash,
    requested_action: "write to docs/README.md",
    constraints: { scope: "docs/README.md" },
  });
  console.log("Preview verdict:", preview.verdict);

  // Search governance history
  const history = await client.auditSearch({ limit: 10 });
  console.log("Audit results:", history.results.length);

  // Check governor status
  const status = await client.governorHealth();
  console.log("Governor:", status);

  // Clean up
  client.close();
}

main().catch(console.error);
```

## Version Compatibility

This SDK supports protocol versions 1.0.0 through 1.x.x. Incompatible versions will fail fast at connection time.

```typescript
import { isVersionCompatible } from "@iter/sdk";

console.log(isVersionCompatible("1.0.0")); // true
console.log(isVersionCompatible("1.5.0")); // true (minor bump)
console.log(isVersionCompatible("2.0.0")); // false (major bump)
```

## Telemetry

The SDK propagates trace context but never enriches payloads:

```typescript
import { TraceContext, createTraceContext } from "@iter/sdk";

const trace: TraceContext = {
  traceId: "abc123",
  spanId: "span456",
  parentSpanId: "parent789",
};

client.withTrace(trace);
// All subsequent requests will include this trace context
```

## Error Handling

```typescript
import {
  SdkError,
  VersionMismatchError,
  ConnectionError,
  RequestError,
} from "@iter/sdk";

try {
  const node = await client.nodeCreate(0.5, 1.0);
} catch (e) {
  if (e instanceof VersionMismatchError) {
    console.error(`Version mismatch: ${e.clientVersion} vs ${e.serverVersion}`);
  } else if (e instanceof ConnectionError) {
    console.error(`Connection failed: ${e.message}`);
  } else if (e instanceof RequestError) {
    console.error(`Request failed: ${e.rpcError.code} - ${e.rpcError.message}`);
  }
}
```

## License

Apache-2.0

SDKs are Apache-2.0 licensed; proprietary substrate components are not included.
