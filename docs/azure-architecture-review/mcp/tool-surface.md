# MCP Tool Surface

**Classification:** External-Safe  
**Version:** 1.0  
**Last Updated:** February 2026

---

## MCP Tools

Iter exposes a small set of tools via MCP (Model Context Protocol) over JSON-RPC 2.0.

### Node Tools

| Tool | Inputs | Outputs | Deterministic | Side Effects |
|------|--------|---------|---------------|--------------|
| `node.create` | value (f64), quality (f64) | node_id (string), checksum | Yes | None |
| `node.query` | node_id (string) | node state | Yes | None |
| `node.mutate` | node_id, delta (f64) | updated state, checksum | Yes | None |

### Edge Tools

| Tool | Inputs | Outputs | Deterministic | Side Effects |
|------|--------|---------|---------------|--------------|
| `edge.bind` | source_id, target_id, weight (f64) | edge_id, checksum | Yes | None |
| `edge.propagate` | edge_id | propagation result, checksum | Yes | None |

### Governance Tools

| Tool | Inputs | Outputs | Deterministic | Side Effects |
|------|--------|---------|---------------|--------------|
| `governor.status` | None | governance state | Yes | None |
| `governance.status` | None | policy status | Yes | None |
| `esv.audit` | tick range | audit events | Yes | None |
| `lineage.replay` | tick | DecisionPacket | Yes | None |
| `lineage.export` | tick range | DecisionPackets | Yes | None |

---

## Tool Contracts

### Deterministic Guarantee

All tools produce identical outputs for identical inputs.

**Enforcement:**
- Input validation before execution
- Canonical serialization of outputs
- Checksum verification

---

## Side-Effect Constraints

**No tool produces external side effects.**

| Prohibited | Reason |
|------------|--------|
| Network calls | Violates side-effect isolation invariant |
| File I/O | Violates side-effect isolation invariant |
| Database writes | Violates side-effect isolation invariant |
| Message queue publishes | Violates side-effect isolation invariant |

---

## Input Validation

All tools validate inputs before execution:
- Type safety
- Range bounds
- NaN/Inf detection
- Enum validity

**Invalid input → tool call rejected with error code.**
