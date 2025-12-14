# Release Notes: v0.3.0-phase4 (INTERNAL)

**INTERNAL — NOT FOR PUBLIC DISTRIBUTION**

**Release Date:** December 2024  
**Tag:** `v0.3.0-phase4`

## Overview

Phase 4 transforms the SCG MCP Server from "internally correct" to "externally stable" with a frozen API contract, comprehensive testing, observability hooks, and documentation suitable for external integrators.

## Highlights

### API Contract Freeze
- **Dual Error Code System**: All `McpError` variants now expose both numeric (`code()`) and string (`code_string()`) codes for machine and human consumption
- **JSON Schemas**: Machine-readable schemas in `spec/` for all response types and tools
- **Contract Tests**: 17 tests ensuring serialization roundtrips and schema shape compliance

### Concurrency Hardening
- **8 Concurrent Tests**: Real stress tests with N=32/64 threads exercising node/edge operations
- **Thread-Safe Runtime**: `SharedSubstrateRuntime` wraps runtime in `Arc<Mutex<_>>`
- **Invariant Verification**: Confirms invariants hold under concurrent load

### Property-Based Testing (Fuzzing)
- **Proptest Integration**: 8 property tests with bounded graph (64 nodes, 128 edges)
- **Random Operation Sequences**: 20-50 operations per sequence
- **Panic-Free Guarantee**: Validates no crashes under arbitrary valid inputs

### Observability & Security
- **Metrics Module** (`src/metrics.rs`): Atomic counters for requests, success/failure rates, invariant violations
- **Input Validation Module** (`src/validation.rs`): Validates belief [0,1], energy ≥0, weight bounds, payload sizes
- **CallerContext Placeholder** (`src/caller_context.rs`): Future auth/RBAC integration point

### Documentation & Examples
- **MCP_API.md**: Comprehensive API reference with all tools, schemas, error taxonomy, invariants
- **Reference Client**: `examples/mcp_client.rs` demonstrating all 10 tools

## New Files

```
src/
├── caller_context.rs    # Auth context placeholder
├── metrics.rs           # Request metrics abstraction
└── validation.rs        # Input validation functions

spec/
├── mcp_error.schema.json
├── mcp_node_state.schema.json
├── mcp_edge_state.schema.json
├── mcp_governor_status.schema.json
├── mcp_lineage_entry.schema.json
└── tools/
    └── mcp_tools.schema.json

tests/
├── contract_tests.rs           # API contract tests
├── hardening_concurrency.rs    # Concurrent operation tests
└── mcp_fuzz_scenarios.rs       # Property-based fuzzing

examples/
└── mcp_client.rs               # Reference client

docs/
└── MCP_API.md                  # API documentation
```

## Test Coverage

| Category | Tests |
|----------|-------|
| Unit tests (lib) | 39 |
| Unit tests (main) | 22 |
| Contract tests | 17 |
| MCP integration | 69 |
| Concurrency | 8 |
| Fuzzing | 8 |
| Governance | 4 |
| Lineage | 5 |
| Fuzz hardening | 7 |
| **Total** | **179** |

## Error Code Reference

| Code | String | Description |
|------|--------|-------------|
| 1000 | INTERNAL_ERROR | Internal server error |
| 2000 | BAD_REQUEST | Invalid request parameters |
| 3000 | INVALID_TOOL | Unknown tool name |
| 4000 | NOT_FOUND | Resource not found |
| 4004 | NODE_NOT_FOUND | Node not found by ID |
| 5000 | INVARIANT_VIOLATION | SCG invariant violated |

## Breaking Changes

None. This release maintains backward compatibility with v0.2.0.

## Dependencies

- Added `proptest = "1.4"` (dev-dependency)

## Upgrade Path

No migration required. Drop-in replacement for v0.2.0.

## Validation Commands

```bash
# Run all tests
cargo test

# Check lints
cargo clippy --all-targets -- -D warnings

# Run reference client
cargo run --example mcp_client
```


