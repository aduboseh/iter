# Iter Server Architecture

> Full technical breakdown of the MCP boundary layer

---

## System Overview

The Iter Server sits between AI assistants and the Iter governed execution, acting as a **hardened security boundary**.

```
┌─────────────────────────────────────────────────────────────────┐
│                        AI Assistant                             │
│                   (Claude, GPT, etc.)                           │
└─────────────────────────────────────────────────────────────────┘
                              │
                              │ MCP Protocol (JSON-RPC 2.0)
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                                                                 │
│                      Iter Server                             │
│                                                                 │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
│  │   MCP       │  │  Response   │  │   Forbidden Pattern     │  │
│  │  Handler    │──│  Sanitizer  │──│   Registry (60+ rules)  │  │
│  └─────────────┘  └─────────────┘  └─────────────────────────┘  │
│         │                                                       │
│         │ Safe, sanitized operations                            │
│         ▼                                                       │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │              Iter Runtime (Iter-connectome)                   ││
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────────┐        ││
│  │  │  Nodes  │ │  Edges  │ │Governor │ │   Lineage   │        ││
│  │  │ (belief)│ │(weights)│ │  (ESV)  │ │  (SHA-256)  │        ││
│  │  └─────────┘ └─────────┘ └─────────┘ └─────────────┘        ││
│  └─────────────────────────────────────────────────────────────┘│
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## Component Breakdown

### 1. MCP Handler (`src/mcp_handler.rs`)

The JSON-RPC 2.0 dispatcher. Routes incoming requests to appropriate substrate operations.

**Responsibilities:**
- Parse JSON-RPC requests
- Validate method names and parameters
- Dispatch to Iter runtime
- Format MCP-compliant responses

**Supported Methods:**
- `initialize` - MCP protocol handshake
- `tools/list` - Enumerate available tools
- `tools/call` - Execute a tool
- Direct tool methods (`node.create`, etc.)

### 2. Iter Runtime (`src/substrate_runtime.rs`)

Wrapper around `Iter-connectome` that manages substrate state.

**Core Operations:**
- `node_create(belief, energy)` → Node
- `node_mutate(id, delta)` → Node
- `node_query(id)` → Option<Node>
- `edge_bind(src, dst, weight)` → Edge
- `edge_propagate(id)` → Result
- `governor_status()` → GovernorStatus
- `esv_audit(id)` → bool

### 3. Response Sanitizer (`src/services/sanitizer/`)

The critical security boundary. Ensures no substrate internals leak to AI.

```
              Raw Response from Iter Runtime
                         │
                         ▼
              ┌─────────────────────┐
              │  Unicode Normalize  │  ← Strip zero-width chars
              └─────────────────────┘
                         │
                         ▼
              ┌─────────────────────┐
              │  Pattern Matching   │  ← Check 60+ forbidden patterns
              └─────────────────────┘
                         │
                         ▼
              ┌─────────────────────┐
              │  Field Redaction    │  ← Remove/replace forbidden fields
              └─────────────────────┘
                         │
                         ▼
              Sanitized Response to AI
```

### 4. Governance Validator (`src/governance.rs`)

Validates runtime state against governance constraints.

**Checks:**
- Drift ≤ 1e-10
- ESV compliance
- Checksum integrity

### 5. Lineage Tracker (`src/lineage/`)

Maintains cryptographic audit trail of all operations.

**Features:**
- SHA-256 hash chain
- Episode-based receipts
- Export to JSON

---

## Data Flow

### Request Flow (AI → Substrate)

```
AI Assistant
    │
    │ {"jsonrpc":"2.0","method":"node.create","params":{...},"id":1}
    ▼
┌─────────────────┐
│   STDIO Input   │
└─────────────────┘
    │
    ▼
┌─────────────────┐
│  JSON Parser    │  ← Validate JSON-RPC structure
└─────────────────┘
    │
    ▼
┌─────────────────┐
│  MCP Handler    │  ← Route to appropriate method
└─────────────────┘
    │
    ▼
┌─────────────────┐
│  ESV Guard      │  ← Pre-operation safety check
└─────────────────┘
    │
    ▼
┌─────────────────┐
│  Drift Guard    │  ← Energy conservation check
└─────────────────┘
    │
    ▼
┌─────────────────┐
│  Iter Runtime    │  ← Execute operation
└─────────────────┘
    │
    ▼
┌─────────────────┐
│  Lineage Append │  ← Record operation
└─────────────────┘
```

### Response Flow (Substrate → AI)

```
Iter Runtime Result
    │
    ▼
┌─────────────────┐
│  Sanitizer      │  ← Remove forbidden patterns
└─────────────────┘
    │
    ▼
┌─────────────────┐
│  MCP Formatter  │  ← Wrap in content array
└─────────────────┘
    │
    ▼
┌─────────────────┐
│  JSON Serialize │
└─────────────────┘
    │
    ▼
┌─────────────────┐
│  STDIO Output   │
└─────────────────┘
    │
    ▼
AI Assistant
```

---

## Substrate Layer

### Nodes

Belief-weighted cognitive entities.

| Field | Type | Range | Description |
|-------|------|-------|-------------|
| `id` | UUID | - | Unique identifier |
| `belief` | f64 | [0.0, 1.0] | Confidence value |
| `energy` | f64 | [0.0, ∞) | Allocated energy |

### Edges

Weighted connections between nodes.

| Field | Type | Range | Description |
|-------|------|-------|-------------|
| `id` | UUID | - | Unique identifier |
| `src` | UUID | - | Source node |
| `dst` | UUID | - | Destination node |
| `weight` | f64 | [0.0, 1.0] | Connection strength |

### Governor (ESV)

Ethical State Vector enforcement.

- **Drift Threshold:** 1e-10
- **Rejection Code:** -32001
- **Validation:** Pre-operation and post-operation

### Lineage

Merkle-style audit chain.

```
Operation 1 ─────┐
                 │
Operation 2 ─────┼───▶ SHA-256 ───▶ Episode Hash
                 │
Operation 3 ─────┘
```

---

## Module Structure

```
src/
├── main.rs                 # Entry point, STDIO server loop
├── lib.rs                  # Library exports for testing
├── mcp_handler.rs          # JSON-RPC dispatcher
├── substrate_runtime.rs     # Iter runtime facade
├── governance.rs           # Governance validation
├── types.rs                # RPC request/response types
├── fault.rs                # Error handling
├── telemetry.rs            # Observability
├── lineage/
│   ├── mod.rs
│   └── snapshot.rs         # Lineage snapshots
└── services/
    └── sanitizer/
        ├── mod.rs          # Module exports
        ├── forbidden.rs    # Pattern registry (IMMUTABLE)
        └── response.rs     # Response sanitization
```

---

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `tokio` | 1.x | Async runtime |
| `serde` | 1.x | Serialization |
| `serde_json` | 1.x | JSON handling |
| `uuid` | 1.x | UUID generation |
| `sha2` | 0.10 | SHA-256 hashing |
| `parking_lot` | 0.12 | Synchronization |
| `Iter-connectome` | local | Iter substrate |

---

## See Also

- [SECURITY.md](./SECURITY.md) - Security architecture details
- [ATTACK_SURFACE.md](./ATTACK_SURFACE.md) - Blocked patterns
- [GOVERNANCE.md](./GOVERNANCE.md) - Governance system

