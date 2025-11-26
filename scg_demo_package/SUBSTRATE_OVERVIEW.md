# SCG Substrate Architecture Summary

## Core Primitives

- **Nodes:** Belief-weighted entities with values in [0.0, 1.0] range
- **Edges:** Directional relationships with confidence scores (weights)
- **Governor:** Constraint enforcement layer ensuring ESV compliance
- **Lineage:** Cryptographic operation chain with Merkle-style receipts

## Invariant Properties

- **Drift Bound:** |Δ coherence| ≤ 1e-10 per operation
- **Energy Conservation:** Cycle propagation preserves total graph energy
- **Ethical State Vector:** Rejects unsafe operations deterministically
- **Belief Clamping:** All belief values automatically clamped to [0.0, 1.0]

## MCP Interface

Standard JSON-RPC 2.0 protocol over stdin/stdout:

| Method | Description | Side Effects |
|--------|-------------|--------------|
| `governor.status` | Query invariant metrics | None |
| `node.create` | Instantiate belief-weighted entity | State mutation, lineage append |
| `node.mutate` | Modify node belief by delta | ESV validation, lineage append |
| `node.query` | Read node state | None |
| `edge.bind` | Construct topology connection | State mutation, lineage append |
| `edge.propagate` | Execute cycle-aware signal flow | Energy transfer, lineage append |
| `lineage.export` | Export cryptographic audit trail | Filesystem write |
| `lineage.replay` | Retrieve latest lineage entry | None |
| `esv.audit` | Validate node ethical compliance | ESV check |

## Response Structure

All MCP responses follow this format:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "content": [{"type": "text", "text": "<JSON payload>"}]
  },
  "id": <sequential_integer>
}
```

Error responses use code ranges:
- `1000`: ESV validation failure
- `2000`: Thermodynamic drift exceeded
- `4000`: Bad request (invalid parameters)
- `4004`: Resource not found

## Demonstration Scope

This package demonstrates substrate mechanics only:
- Domain-neutral synthetic topology
- No business logic or vertical-specific content
- Suitable for security research and validation
- All operations reversible via lineage audit
