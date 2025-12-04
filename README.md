# 🧠 SCG MCP Server

**The secure MCP boundary between AI assistants and the SCG cognitive substrate.**

[![CI](https://github.com/aduboseh/scg-mcp/actions/workflows/mcp_integration.yml/badge.svg)](https://github.com/aduboseh/scg-mcp/actions/workflows/mcp_integration.yml)
[![Governance](https://github.com/aduboseh/scg-mcp/actions/workflows/verify_rules_consistency.yml/badge.svg)](https://github.com/aduboseh/scg-mcp/actions/workflows/verify_rules_consistency.yml)
[![MCP](https://img.shields.io/badge/MCP-2024--11--05-blue)](https://modelcontextprotocol.io)

---

## What is this?

The SCG MCP Server is the **Model Context Protocol gateway** for the [Synthetic Cognitive Graph (SCG)](https://github.com/aduboseh/SCG). It exposes cognitive primitives (nodes, edges, propagation) while ensuring **zero leakage** of internal substrate structure, state, or ethics vectors—using a hardened, adversarially-tested sanitization boundary.

This lets AI assistants like Claude and GPT communicate with SCG without ever revealing internal topology, energy flows, or ethical kernel internals.

```
┌─────────────────────────────────────────────────────────────────┐
│                        AI Assistant                             │
└─────────────────────────────────────────────────────────────────┘
                              │ MCP Protocol (JSON-RPC 2.0)
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  SCG MCP Server                                                 │
│  ┌─────────────┐  ┌───────────────┐  ┌─────────────────────┐  │
│  │ MCP Handler │──│   Sanitizer   │──│ 60+ Forbidden Patterns│  │
│  └─────────────┘  └───────────────┘  └─────────────────────┘  │
│         │                                                       │
│         ▼                                                       │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  SCG Runtime: Nodes • Edges • Governor • Lineage          │    │
│  └─────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────┘
```

---

## Why does this exist?

SCG is powerful enough to be misused if exposed directly. This server prevents:

- **Topology reconstruction** — internal graph structure stays hidden
- **ESV bypass** — ethical state vectors are never exposed raw
- **Lineage forgery** — only checksums are visible, not hash chains
- **Unicode obfuscation attacks** — Cyrillic/Greek lookalikes normalized
- **Prompt injection via responses** — all substrate internals sanitized

> Full attack surface documentation: [`docs/ATTACK_SURFACE.md`](docs/ATTACK_SURFACE.md)

---

## MCP Tools

| Tool | Description |
|------|-------------|
| `node.create` | Create belief node |
| `node.mutate` | Adjust belief (ESV-guarded) |
| `node.query` | Query node state |
| `edge.bind` | Connect nodes |
| `edge.propagate` | Propagate belief |
| `governor.status` | Check drift/coherence |
| `esv.audit` | Audit ethical state |
| `lineage.replay` | Get checksum history |
| `lineage.export` | Export lineage log |
| `governance.status` | Health check |

---

## Quick Start

```bash
# Clone and build
git clone https://github.com/aduboseh/scg-mcp.git
cd scg-mcp
cargo build --release

# Test the server
echo '{"jsonrpc":"2.0","method":"tools/list","id":1}' | ./target/release/scg_mcp_server
```

### Claude Desktop Integration

```json
{
  "mcpServers": {
    "scg": {
      "command": "/path/to/scg_mcp_server"
    }
  }
}
```

---

## Security

Every outbound response passes through a **deterministic Response Sanitizer** backed by 60+ forbidden patterns, Unicode normalization, zero-width character stripping, and adversarial pattern matching. All internal substrate structures remain cryptographically sealed behind the MCP boundary.

> Full security architecture: [`docs/SECURITY.md`](docs/SECURITY.md)

---

## Governance

- **SHA-256 dual-manifest verification** — parity between SCG and MCP repos
- **CODEOWNERS protection** — sanitizer changes require founder approval
- **Immutable forbidden registry** — sealed at v2.0
- **CI enforcement** — 132 tests must pass

> Full governance documentation: [`docs/GOVERNANCE.md`](docs/GOVERNANCE.md)

---

## Testing

```bash
cargo test                              # All tests
cargo test --test mcp_integration       # Integration suite
SCG_DETERMINISM=1 cargo test            # Deterministic mode
```

> Full test documentation: [`docs/TESTING.md`](docs/TESTING.md)

---

## Documentation

| Document | Description |
|----------|-------------|
| [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) | System design, data flow, components |
| [`docs/SECURITY.md`](docs/SECURITY.md) | Threat model, defense layers, incident response |
| [`docs/ATTACK_SURFACE.md`](docs/ATTACK_SURFACE.md) | Complete forbidden pattern registry |
| [`docs/GOVERNANCE.md`](docs/GOVERNANCE.md) | Checksums, CODEOWNERS, CI enforcement |
| [`docs/TESTING.md`](docs/TESTING.md) | Test coverage, running tests |

---

## Releases

| Version | Description |
|---------|-------------|
| [`v0.2.0-mcp-integrity`](https://github.com/aduboseh/scg-mcp/releases/tag/v0.2.0-mcp-integrity) | MCP Hardening v2.0 — Boundary sealed |

---

## Contact

- **Research**: research@onlysgsolutions.com
- **Enterprise**: enterprise@onlysgsolutions.com
- **Security**: security@onlysgsolutions.com

---

<p align="center">
  <sub>Built with 🧠 by Only SG Solutions © 2025</sub>
</p>
