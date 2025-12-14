# Iter Server

**Deterministic MCP server for governed execution, replayable decision paths, and auditable state transitions.**

[![CI](https://github.com/aduboseh/iter/actions/workflows/mcp_integration.yml/badge.svg)](https://github.com/aduboseh/iter/actions/workflows/mcp_integration.yml)
[![Governance](https://github.com/aduboseh/iter/actions/workflows/verify_rules_consistency.yml/badge.svg)](https://github.com/aduboseh/iter/actions/workflows/verify_rules_consistency.yml)
[![MCP](https://img.shields.io/badge/MCP-2024--11--05-blue)](https://modelcontextprotocol.io)

## What is Iter?

Iter Server is a hardened **Model Context Protocol (MCP)** gateway (JSON-RPC 2.0) that exposes a small, governed tool surface while enforcing a strict response sanitization boundary.

It is designed for:
- deterministic replay
- decision audit
- governed execution
- zero leakage of internal engine internals through responses

Iter is NOT:
- a general-purpose agent runtime
- an orchestration framework
- a low-latency execution engine

## MCP Tools

The MCP surface is intentionally small. All tools are deterministic, side-effect constrained, and auditable.

### State Operations

| Tool | Description |
|------|-------------|
| `node.create` | Create a node with initial values |
| `node.query` | Query node state by ID |
| `node.mutate` | Mutate node belief by delta (debug operation) |

### Propagation

| Tool | Description |
|------|-------------|
| `edge.bind` | Bind an edge between two nodes |
| `edge.propagate` | Run a deterministic step |

### Governance & Audit

| Tool | Description |
|------|-------------|
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

### Desktop Client Integration

```json
{
  "mcpServers": {
    "iter": {
      "command": "/path/to/iter-server"
    }
  }
}
```

## Security & Governance Model

**Iter assumes hostile inputs and untrusted clients by default.**

Every outbound response passes through a deterministic sanitizer backed by forbidden-pattern coverage, Unicode normalization, and zero-width character stripping.

- Threat model and defenses: [docs/SECURITY.md](docs/SECURITY.md)
- Forbidden registry overview: [docs/ATTACK_SURFACE.md](docs/ATTACK_SURFACE.md)

Governance enforcement:
- checksum verification (manifests)
- CODEOWNERS protection
- CI enforcement

Details: [docs/GOVERNANCE.md](docs/GOVERNANCE.md)

## Kernel Compatibility

Validated against [drift-kernel v1.0.0](https://github.com/aduboseh/drift-kernel/releases/tag/drift-kernel-v1.0.0).

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
