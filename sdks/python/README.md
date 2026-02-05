# Iter Python SDK

Thin async client for Iter MCP protocol with asyncio.

## Status: ✅ Contract-Compliant Async SDK

## Design Principles

- **Thin**: No business logic; pure protocol wrapper
- **Contract-driven**: Types derived from protocol specification
- **Version-aware**: Fails fast on incompatible protocol versions (supports N, N-1)
- **Telemetry-safe**: Passes trace context through, never enriches payloads

## Features

- ✅ Async asyncio-based transport
- ✅ State machine (OPEN → CLOSING → CLOSED)
- ✅ Graceful shutdown with bounded drain (5000ms)
- ✅ Backpressure enforcement (`max_inflight`)
- ✅ Request timeouts with eviction
- ✅ SIGTERM → SIGKILL process management
- ✅ Stderr ring buffer (10KB)
- ✅ Responses processed during drain (OPEN || CLOSING)

## Installation

```bash
pip install iter-sdk
```

## Usage

```python
import asyncio
from iter_sdk import IterClient

async def main() -> None:
    # Connect to an Iter server with max_inflight=1
    client = await IterClient.connect("iter-server", max_inflight=1)

    # Set trace context for distributed tracing
    client.with_trace({"trace_id": "my-trace-id", "span_id": "my-span-id"})

    # List available tools
    tools = await client.tools_list()
    print("Available tools:", tools)

    # Create a node
    node = await client.node_create(0.5, 1.0)
    print("Created node:", node)

    # Query a node
    state = await client.node_query(node.id)
    print("Node state:", state)

    # Check governor status
    status = await client.governor_status()
    print("Governor status:", status)

    # Graceful shutdown
    await client.close()

    print("Client closed successfully")
```

## Version Compatibility

This SDK supports protocol versions 1.0.0 through 1.x.x. Incompatible versions will fail fast at connection time.

```python
from iter_sdk import is_version_compatible

assert is_version_compatible("1.0.0")  # true
assert is_version_compatible("1.5.0")  # true (minor bump)
assert not is_version_compatible("2.0.0")  # false (major bump)
```

## Telemetry

The SDK propagates trace context but never enriches payloads:

```python
from iter_sdk import TraceContext, create_trace_context

trace = TraceContext(
    trace_id="abc123",
    span_id="span456",
    parent_span_id="parent789"
)

client.with_trace(trace)
# All subsequent requests will include this trace context
```

## Error Handling

```python
from iter_sdk import (
    SdkError,
    VersionMismatchError,
    ConnectionError,
    RequestError,
    BackpressureError,
    RequestTimeoutError,
    ConnectionClosedError,
)

async def example():
    client = await IterClient.connect("iter-server", max_inflight=1)
    
    try:
        node = await client.node_create(0.5, 1.0)
        print("Created node:", node)
    except VersionMismatchError as e:
        print(f"Version mismatch: {e.client} vs {e.server}")
    except ConnectionError as e:
        print(f"Connection failed: {e}")
    except RequestError as e:
        print(f"Request failed: {e.rpc_error.code} - {e.rpc_error.message}")
    except BackpressureError as e:
        print(f"Backpressure: maxInflight={e.max_inflight} exceeded")
    except RequestTimeoutError as e:
        print(f"Request timeout: {e.method} exceeded {e.timeout_ms}ms")
    except ConnectionClosedError as e:
        pending = f" (pending: {e.pending_count_at_close})" if e.pending_count_at_close else ""
        print(f"Connection closed: {e.message}{pending}")
```

## Lifecycle Contract

This SDK follows the shared lifecycle contract defined in `sdk-contract.md`.

**Key guarantees:**
- State machine: OPEN → CLOSING → CLOSED
- `send()` only allowed in OPEN state
- Responses processed during drain (OPEN || CLOSING)
- Bounded drain timeout (5s)
- Close idempotence
- Backpressure enforcement

## Testing

```bash
pip install pytest pytest-asyncio
pytest tests/
```

## License

Apache-2.0

SDKs are Apache-2.0 licensed; proprietary substrate components are not included.
