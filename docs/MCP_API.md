# SCG MCP Server API Reference

**Version:** 0.3.0-phase4  
**Protocol:** MCP 2024-11-05

## Overview

The SCG MCP Server provides a Model Context Protocol interface to the SCG (Self-Correcting Graph) cognitive substrate. It enables AI assistants to interact with belief networks while maintaining security boundaries and audit capabilities.

### Architecture

```
AI Assistant → MCP Protocol → scg_mcp_server → SCG Substrate
                                    │
                              Response Sanitizer
                                    │
                              Safe JSON Output
```

All responses are sanitized to prevent leakage of:
- DAG topology information
- Raw ESV (Ethical State Vector) values
- Internal energy matrices
- Lineage chain details (only checksums exposed)

## Protocol Initialization

### `initialize`

Initialize the MCP connection.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "initialize",
  "params": {},
  "id": 1
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "result": {
    "protocolVersion": "2024-11-05",
    "capabilities": {
      "tools": {},
      "resources": {},
      "prompts": {}
    },
    "serverInfo": {
      "name": "scg_mcp_server",
      "version": "0.3.0"
    }
  },
  "id": 1
}
```

### `tools/list`

List available tools.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "tools/list",
  "params": {},
  "id": 2
}
```

## Tools

### Node Operations

#### `node.create`

Create a new SCG node with initial belief and energy values.

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "belief": { "type": "number", "minimum": 0.0, "maximum": 1.0 },
    "energy": { "type": "number", "minimum": 0.0 }
  },
  "required": ["belief", "energy"]
}
```

**Side Effects:** `state_mutation`, `energy_allocation`, `lineage_append`

**Response Schema:** `McpNodeState`
```json
{
  "node_id": 0,
  "belief": 0.7,
  "energy": 100.0,
  "timestamp": "2024-01-01T00:00:00Z"
}
```

#### `node.query`

Query node state by ID.

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "node_id": { "type": "string", "description": "Node ID (numeric string)" }
  },
  "required": ["node_id"]
}
```

**Side Effects:** None

**Response Schema:** `McpNodeState`

#### `node.mutate`

Mutate node belief by delta. **DEBUG operation** - bypasses physics, use with caution.

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "node_id": { "type": "string" },
    "delta": { "type": "number" }
  },
  "required": ["node_id", "delta"]
}
```

**Side Effects:** `state_mutation`, `energy_consumption`

### Edge Operations

#### `edge.bind`

Bind an edge between two nodes.

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "src": { "type": "string", "description": "Source node ID" },
    "dst": { "type": "string", "description": "Destination node ID" },
    "weight": { "type": "number" }
  },
  "required": ["src", "dst", "weight"]
}
```

**Side Effects:** `state_mutation`, `topology_change`, `lineage_append`

**Response Schema:** `McpEdgeState`
```json
{
  "edge_id": 0,
  "src": 0,
  "dst": 1,
  "weight": 0.5,
  "timestamp": "2024-01-01T00:00:00Z"
}
```

#### `edge.propagate`

Run a simulation step (propagates beliefs along all edges).

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "edge_id": { "type": "string", "description": "Accepted for compatibility, not used" }
  },
  "required": ["edge_id"]
}
```

**Side Effects:** `state_mutation`, `energy_transfer`, `lineage_append`

### Governance Operations

#### `governor.status`

Query governor drift and coherence status.

**Input Schema:**
```json
{
  "type": "object",
  "properties": {}
}
```

**Side Effects:** None

**Response Schema:** `McpGovernorStatus`
```json
{
  "drift": 0.001,
  "coherent": true,
  "timestamp": "2024-01-01T00:00:00Z"
}
```

#### `governance.status`

Query full governance health status including checksum validity, drift, and ESV status.

**Input Schema:**
```json
{
  "type": "object",
  "properties": {}
}
```

**Side Effects:** None

#### `esv.audit`

Audit node ethical state vector.

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "node_id": { "type": "string" }
  },
  "required": ["node_id"]
}
```

**Side Effects:** `esv_validation`

### Lineage Operations

#### `lineage.replay`

Replay lineage checksum history.

**Input Schema:**
```json
{
  "type": "object",
  "properties": {}
}
```

**Side Effects:** None

**Response Schema:** Array of `McpLineageEntry`
```json
{
  "seq": 0,
  "checksum": "abc123...",
  "timestamp": "2024-01-01T00:00:00Z"
}
```

#### `lineage.export`

Export lineage log to file and return checksum.

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "path": { "type": "string", "description": "Filesystem path to write lineage JSON log" }
  },
  "required": ["path"]
}
```

**Side Effects:** `filesystem_write`

## Error Taxonomy

All errors follow a dual-code system:
- **Numeric code**: Machine-parseable (for programmatic handling)
- **String code**: Human-readable (for debugging and logs)

### Error Codes

| Code | String Code | Description |
|------|-------------|-------------|
| 1000 | `INTERNAL_ERROR` | Internal server error |
| 2000 | `BAD_REQUEST` | Invalid request parameters |
| 3000 | `INVALID_TOOL` | Unknown tool name |
| 4000 | `NOT_FOUND` | Resource not found |
| 4004 | `NODE_NOT_FOUND` | Specific node not found |
| 5000 | `INVARIANT_VIOLATION` | SCG invariant violated |

### Error Response Format

```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": 4004,
    "message": "Node not found: 999"
  },
  "id": 1
}
```

## Invariants

The SCG MCP Server guarantees the following invariants:

### Energy Conservation
Total energy in the system is conserved across all operations. Energy can only be transferred, never created or destroyed.

### Belief Bounds
All belief values are maintained in the range [0.0, 1.0].

### Lineage Integrity
All state-mutating operations are recorded in an append-only lineage log with cryptographic hash chaining.

### Governance Drift Bounds
Drift from the ethics anchor is bounded by `DRIFT_EPSILON` (typically 1e-6).

## Input Validation

All inputs are validated for well-posedness before processing:

| Parameter | Valid Range | Error on Invalid |
|-----------|-------------|------------------|
| `belief` | [0.0, 1.0] | `BAD_REQUEST` |
| `energy` | [0.0, 1e12] | `BAD_REQUEST` |
| `weight` | [-1e6, 1e6] | `BAD_REQUEST` |
| `node_id` | Valid u64 | `BAD_REQUEST` |
| Payload size | ≤ 1MB | `BAD_REQUEST` |

Special values like `NaN` and `Infinity` are rejected.

## Concurrency

The `SubstrateRuntime` is thread-safe and can be shared via `SharedSubstrateRuntime` (wraps in `Arc<Mutex<_>>`). All operations acquire the lock atomically.

**Guarantees:**
- No data races
- Linearizable operations
- Invariants hold after concurrent load

## JSON Schemas

Machine-readable JSON schemas for all types are available in the `spec/` directory:

- `spec/mcp_error.schema.json`
- `spec/mcp_node_state.schema.json`
- `spec/mcp_edge_state.schema.json`
- `spec/mcp_governor_status.schema.json`
- `spec/mcp_lineage_entry.schema.json`
- `spec/tools/mcp_tools.schema.json`

## Example Usage

See `examples/mcp_client.rs` for a complete reference client demonstrating all tools.

```bash
cargo run --example mcp_client
```

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 0.3.0-phase4 | 2024 | API contract freeze, concurrency hardening, fuzzing, observability |
| 0.2.0-phase3 | 2024 | Substrate consolidation, legacy cleanup |
| 0.1.0 | 2024 | Initial release |
