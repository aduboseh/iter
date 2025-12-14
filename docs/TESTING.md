# Iter Testing

> Minimal instructions for running the local test suite.

---

## Quick start

```bash
cargo test
```

## Common targets

```bash
# MCP integration suite
cargo test --test mcp_integration

# Library/unit tests
cargo test --lib
```

Notes:
- This repository includes additional security and boundary tests.
- Details of non-public test strategies are intentionally not documented publicly.
