# 🧠 SCG MCP Server

**The secure MCP interface to the SCG cognitive substrate**

[![CI](https://github.com/aduboseh/scg-mcp/actions/workflows/mcp_integration.yml/badge.svg)](https://github.com/aduboseh/scg-mcp/actions/workflows/mcp_integration.yml)
[![Governance](https://github.com/aduboseh/scg-mcp/actions/workflows/verify_rules_consistency.yml/badge.svg)](https://github.com/aduboseh/scg-mcp/actions/workflows/verify_rules_consistency.yml)
[![MCP](https://img.shields.io/badge/MCP-2024--11--05-blue)](https://modelcontextprotocol.io)
[![License](https://img.shields.io/badge/license-proprietary-red)]()

---

## What is this?

This repository provides a **Model Context Protocol (MCP)** server that exposes the [SCG cognitive substrate](https://github.com/aduboseh/SCG) to AI assistants like Claude, GPT, and other MCP-compatible clients.

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
│                      SCG MCP Server                             │
│                                                                 │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
│  │   MCP       │  │  Response   │  │   Forbidden Pattern     │  │
│  │  Handler    │──│  Sanitizer  │──│   Registry (60+ rules)  │  │
│  └─────────────┘  └─────────────┘  └─────────────────────────┘  │
│         │                                                       │
│         │ Safe, sanitized operations                            │
│         ▼                                                       │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │              SCG Runtime (scg-connectome)               │    │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────────┐    │    │
│  │  │  Nodes  │ │  Edges  │ │Governor │ │   Lineage   │    │    │
│  │  │ (belief)│ │(weights)│ │  (ESV)  │ │  (SHA-256)  │    │    │
│  │  └─────────┘ └─────────┘ └─────────┘ └─────────────┘    │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

**Key insight**: The MCP server acts as a *security boundary* between AI assistants and SCG internals. It prevents substrate reconstruction attacks while still exposing useful cognitive primitives.

---

## Why does this exist?

SCG (Substrate Cognitive Graph) is a deterministic reasoning engine with cryptographic auditability. But exposing it directly to AI models is dangerous—an adversary could:

- 🔴 Reconstruct the internal topology to game the system
- 🔴 Bypass ethical constraints (ESV) by manipulating raw state
- 🔴 Forge lineage records to hide malicious operations

This MCP server solves that by providing a **hardened boundary**:

```
┌─────────────────────────────────────────────────────────────────┐
│                     WHAT AI SEES                                │
├─────────────────────────────────────────────────────────────────┤
│  ✅ node.create     → Create belief nodes                       │
│  ✅ node.mutate     → Adjust beliefs (ESV-guarded)              │
│  ✅ edge.bind       → Connect nodes                             │
│  ✅ governor.status → Check system health                       │
│  ✅ lineage.replay  → Audit trail (summaries only)              │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                  WHAT AI NEVER SEES                             │
├─────────────────────────────────────────────────────────────────┤
│  ❌ dag_topology        → No internal graph structure           │
│  ❌ adjacency_matrix    → No connection patterns                │
│  ❌ esv_raw             → No raw ethical state vectors          │
│  ❌ energy_matrix       → No energy distribution details        │
│  ❌ lineage_hash_chain  → No raw merkle chain access            │
│  ❌ ... 60+ more patterns blocked                               │
└─────────────────────────────────────────────────────────────────┘
```

---

## Available MCP Tools

| Tool | Description | Side Effects |
|------|-------------|--------------|
| `node.create` | Create a new belief node with initial belief and energy | state_mutation, lineage_append |
| `node.mutate` | Adjust a node's belief value (ESV-guarded) | state_mutation, esv_validation |
| `node.query` | Query current state of a node | none |
| `edge.bind` | Create weighted connection between nodes | topology_change, lineage_append |
| `edge.propagate` | Propagate belief along an edge | energy_transfer, lineage_append |
| `governor.status` | Check drift and coherence status | none |
| `esv.audit` | Audit a node's ethical state vector | esv_validation |
| `lineage.replay` | Get lineage checksum history | none |
| `lineage.export` | Export lineage log to file | filesystem_write |
| `governance.status` | Full governance health check | none |

### Example: Create and Connect Nodes

```json
// 1. Create first node
{"jsonrpc":"2.0","method":"node.create","params":{"belief":0.7,"energy":1.0},"id":1}
// Response: {"result":{"id":"550e8400-...","belief":0.7,"energy":1.0}}

// 2. Create second node  
{"jsonrpc":"2.0","method":"node.create","params":{"belief":0.3,"energy":1.0},"id":2}

// 3. Bind them
{"jsonrpc":"2.0","method":"edge.bind","params":{"src":"<node1>","dst":"<node2>","weight":0.5},"id":3}

// 4. Propagate belief
{"jsonrpc":"2.0","method":"edge.propagate","params":{"edge_id":"<edge>"},"id":4}

// 5. Check system health
{"jsonrpc":"2.0","method":"governor.status","params":{},"id":5}
```

---

## Security Architecture

### The Sanitization Boundary

Every response passes through a hardened sanitizer before reaching the AI:

```
              Request from AI
                    │
                    ▼
           ┌───────────────┐
           │  MCP Handler  │
           └───────────────┘
                    │
                    ▼
           ┌───────────────┐
           │  SCG Runtime  │  ← Substrate operations happen here
           └───────────────┘
                    │
                    ▼
           ┌───────────────────────────────────────┐
           │         RESPONSE SANITIZER            │
           │  ┌─────────────────────────────────┐  │
           │  │  Forbidden Pattern Registry     │  │
           │  │  • 60+ blocked patterns         │  │
           │  │  • Unicode normalization        │  │
           │  │  • Zero-width char stripping    │  │
           │  │  • Cyrillic/Greek lookalike     │  │
           │  │    detection                    │  │
           │  └─────────────────────────────────┘  │
           └───────────────────────────────────────┘
                    │
                    ▼
            Sanitized Response
                    │
                    ▼
               AI Assistant
```

### Blocked Attack Vectors

| Attack | How it's blocked |
|--------|------------------|
| Topology reconstruction | `dag_topology`, `adjacency_*` patterns blocked |
| ESV bypass | `esv_raw`, `ethical_gradient` never exposed |
| Energy gaming | `energy_matrix`, `energy_distribution` blocked |
| Lineage forgery | Only checksums exposed, not hash chains |
| Unicode obfuscation | Zero-width chars stripped, lookalikes normalized |
| Prompt injection via response | All substrate internals sanitized |

---

## Quick Start

### Prerequisites
- Rust 1.70+
- Access to [SCG](https://github.com/aduboseh/SCG) repo (private)

### Build
```bash
git clone https://github.com/aduboseh/scg-mcp.git
cd scg-mcp
cargo build --release
```

### Run (STDIO mode)
```bash
echo '{"jsonrpc":"2.0","method":"tools/list","id":1}' | ./target/release/scg_mcp_server
```

### Configure with Claude Desktop

Add to `claude_desktop_config.json`:
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

## Project Structure

```
scg_mcp_server/
├── src/
│   ├── main.rs                 # STDIO server entry point
│   ├── mcp_handler.rs          # JSON-RPC method dispatch
│   ├── scg_core.rs             # SCG runtime wrapper
│   ├── governance.rs           # Governance validation
│   ├── lineage/                # Lineage tracking
│   └── services/
│       └── sanitizer/          # 🔒 MCP Boundary
│           ├── forbidden.rs    # Pattern registry (IMMUTABLE)
│           ├── response.rs     # Response sanitizer
│           └── mod.rs
├── tests/
│   ├── mcp_integration.rs      # 69 integration tests
│   └── integration/
│       ├── boundary_tests.rs   # Sanitization tests
│       ├── adversarial_tests.rs# Attack simulation
│       └── ...
├── governance/
│   └── SCG_Governance_v1.0.md  # Governance manifest
├── .github/
│   ├── workflows/              # CI pipelines
│   └── CODEOWNERS              # Protected paths
└── Cargo.toml
```

---

## Governance & Integrity

This repo enforces strict governance:

- **Dual-checksum verification**: Governance manifest matches SCG repo
- **CODEOWNERS protection**: Sanitizer changes require founder approval
- **Immutable pattern registry**: `forbidden.rs` is frozen at v2.0
- **CI enforcement**: All PRs must pass 132 tests

### Governance Flow

```
┌──────────────┐     ┌──────────────┐
│   SCG Repo   │     │  MCP Server  │
│              │     │              │
│ governance/  │────▶│ governance/  │
│ SCG_Gov_v1.0 │     │ SCG_Gov_v1.0 │
└──────────────┘     └──────────────┘
       │                    │
       │    SHA-256 match   │
       └────────┬───────────┘
                │
                ▼
       ┌────────────────┐
       │ CI Verification│
       │  (weekly cron) │
       └────────────────┘
```

---

## Testing

```bash
# Run all tests
cargo test

# Run MCP integration tests only
cargo test --test mcp_integration

# Run with deterministic mode
SCG_DETERMINISM=1 cargo test
```

### Test Coverage

| Category | Tests | Description |
|----------|-------|--------------|
| Boundary | 13 | Sanitization pattern matching |
| Tool Endpoints | 21 | All MCP tools functional |
| Error Handling | 15 | Invalid inputs, edge cases |
| Adversarial | 20 | Attack simulation, bypass attempts |
| Unit | 41+ | Core logic |

---

## Releases

| Version | Description |
|---------|-------------|
| [`v0.2.0-mcp-integrity`](https://github.com/aduboseh/scg-mcp/releases/tag/v0.2.0-mcp-integrity) | MCP Hardening v2.0 - Boundary sealed |
| `v0.1.0` | Initial MCP server |

---

## Related

- [SCG Core](https://github.com/aduboseh/SCG) - The cognitive substrate
- [Model Context Protocol](https://modelcontextprotocol.io) - MCP specification

---

## Contact

- **Research**: research@onlysgsolutions.com
- **Enterprise**: enterprise@onlysgsolutions.com  
- **Security**: security@onlysgsolutions.com

---

<p align="center">
  <sub>Built with 🧠 by Only SG Solutions</sub><br>
  <sub>© 2025 All Rights Reserved</sub>
</p>
