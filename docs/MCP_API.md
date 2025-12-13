# Iter Server API Reference

**Protocol:** MCP 2024-11-05

---

## Overview

Iter Server provides an MCP (JSON-RPC 2.0) interface that exposes a small set of tools for governed execution and audit.

This document describes **what** the interface provides. Internal mechanisms and enforcement details are intentionally not described here.

---

## Protocol

### `initialize`

Initialize an MCP connection.

### `tools/list`

List available tools.

### `tools/call`

Call a tool by name with arguments.

---

## Tools (names)

Node:
- `node.create`
- `node.query`
- `node.mutate`

Edge:
- `edge.bind`
- `edge.propagate`

Governance / audit:
- `governor.status`
- `governance.status`
- `esv.audit`
- `lineage.replay`
- `lineage.export`

---

## Schemas

Machine-readable JSON schemas are available in `spec/`.

---

## Examples

- `examples/mcp_client.rs`
- `examples/determinism_demo.rs`
