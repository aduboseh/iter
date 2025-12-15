# Iter Rust SDK

Thin client SDK for the Iter MCP protocol.

## Design Principles

- **Thin**: No business logic; pure protocol wrapper
- **Contract-driven**: Types derived from protocol specification
- **Version-aware**: Fails fast on incompatible protocol versions (supports N, N-1)
- **Telemetry-safe**: Passes trace context through, never enriches payloads

## Installation

```toml
[dependencies]
iter-sdk = "0.1"
```

## Usage

```rust
use iter_sdk::{IterClient, TraceContext};

fn main() -> iter_sdk::Result<()> {
    // Connect to an Iter server
    let mut client = IterClient::connect("iter-server")?;
    
    // Set trace context for distributed tracing
    client.with_trace(TraceContext::new("my-trace-id"));
    
    // List available tools
    let tools = client.tools_list()?;
    println!("Available tools: {:?}", tools);
    
    // Create a node
    let node = client.node_create(0.5, 1.0)?;
    println!("Created node: {:?}", node);
    
    // Query a node
    let state = client.node_query(node.id)?;
    println!("Node state: {:?}", state);
    
    // Check governor status
    let status = client.governor_status()?;
    println!("Governor: {:?}", status);
    
    Ok(())
}
```

## Version Compatibility

This SDK supports protocol versions 1.0.0 through 1.x.x. Incompatible versions will fail fast at connection time.

```rust
use iter_sdk::is_version_compatible;

assert!(is_version_compatible("1.0.0"));  // OK
assert!(is_version_compatible("1.5.0"));  // OK (minor bump)
assert!(!is_version_compatible("2.0.0")); // Rejected (major bump)
```

## Telemetry

The SDK propagates trace context but never enriches payloads:

```rust
use iter_sdk::TraceContext;

let trace = TraceContext {
    trace_id: "abc123".to_string(),
    span_id: "span456".to_string(),
    parent_span_id: Some("parent789".to_string()),
};

client.with_trace(trace);
// All subsequent requests will include this trace context
```

## License

MIT
