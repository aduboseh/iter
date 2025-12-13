# Iter Server

**Deterministic decision paths & audit for Copilot (MCP / JSON-RPC 2.0).**

[![CI](https://github.com/aduboseh/iter/actions/workflows/mcp_integration.yml/badge.svg)](https://github.com/aduboseh/iter/actions/workflows/mcp_integration.yml)
[![Governance](https://github.com/aduboseh/iter/actions/workflows/verify_rules_consistency.yml/badge.svg)](https://github.com/aduboseh/iter/actions/workflows/verify_rules_consistency.yml)
[![MCP](https://img.shields.io/badge/MCP-2024--11--05-blue)](https://modelcontextprotocol.io)

## What is this?

Iter Server is a **Model Context Protocol (MCP) gateway** that exposes a small set of governed tools (create/query/mutate, bind/propagate, governance status, and audit/replay/export) while enforcing a hardened response sanitization boundary.

It is designed for:
- deterministic replay
- decision audit
- governed execution
- zero leakage of internal engine internals through responses

## MCP Tools

| Tool | Description |
|------|-------------|
| `node.create` | Create a node with initial values |
| `node.mutate` | Mutate node belief by delta (debug operation) |
| `node.query` | Query node state by ID |
| `edge.bind` | Bind an edge between two nodes |
| `edge.propagate` | Run a deterministic step |
| `governor.status` | Query drift/coherence status |
| `esv.audit` | Audit node ethical state vector |
| `lineage.replay` | Replay checksum history |
| `lineage.export` | Export lineage log |
| `governance.status` | Query governance health status |

## Quick Start

```bash
# Clone and build
git clone https://github.com/aduboseh/iter.git
cd iter
cargo build --release

# Run the determinism demo (recommended first experience)
cargo run --release --example determinism_demo

# Or run the reference client
cargo run --release --example mcp_client

# Run the server (STDIO transport)
echo '{"jsonrpc":"2.0","method":"tools/list","id":1}' | cargo run --release --bin iter-server
```

### Claude Desktop Integration

```json
{
  "mcpServers": {
    "iter": {
      "command": "/path/to/iter-server"
    }
  }
}
```

## Security

Every outbound response passes through a deterministic sanitizer backed by forbidden-pattern coverage, Unicode normalization, and zero-width character stripping.

- Threat model and defenses: `docs/SECURITY.md`
- Forbidden registry overview: `docs/ATTACK_SURFACE.md`

## Governance

- checksum verification (manifests)
- CODEOWNERS protection
- CI enforcement

Details: `docs/GOVERNANCE.md`

## Testing

```bash
cargo test                        # All tests
cargo test --test mcp_integration # Integration suite
```

Details: `docs/TESTING.md`

## Marketplace identity

Name: **Iter**

Subtitle: **Deterministic Decision Paths & Audit for Copilot**

<p align="center">
  <sub>Only SG Solutions © 2025</sub>
</p>
