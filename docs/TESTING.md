# Iter Testing

> Test suite documentation and coverage analysis

---

## Quick Start

```bash
# Run all tests
cargo test

# Run MCP integration tests only
cargo test --test mcp_integration

# Run with deterministic mode
DETERMINISM=1 cargo test

# Run specific test category
cargo test boundary_tests
cargo test adversarial_tests
cargo test tool_endpoint
```

---

## Test Architecture

```
tests/
├── mcp_integration.rs          # Entry point (69 tests)
└── integration/
    ├── mod.rs                  # Module exports
    ├── common.rs               # Test utilities
    ├── boundary_tests.rs       # Sanitization tests (13)
    ├── tool_endpoint_tests.rs  # Functional tests (21)
    ├── error_handling_tests.rs # Error cases (15)
    └── adversarial_tests.rs    # Attack simulation (20)
```

---

## Test Coverage

### Summary

| Category | Tests | Description |
|----------|-------|-------------|
| Boundary | 13 | Sanitization pattern matching |
| Tool Endpoints | 21 | All MCP tools functional |
| Error Handling | 15 | Invalid inputs, edge cases |
| Adversarial | 20 | Attack simulation, bypass attempts |
| Unit | 41+ | Core logic |
| **Total** | **132** | |

---

## Category: Boundary Tests (13)

Verify the response sanitizer blocks forbidden patterns.

| Test | Pattern Tested |
|------|----------------|
| `test_dag_topology_blocked` | `dag_topology` |
| `test_adjacency_matrix_blocked` | `adjacency_matrix` |
| `test_esv_raw_blocked` | `esv_raw` |
| `test_energy_matrix_blocked` | `energy_matrix` |
| `test_lineage_hash_chain_blocked` | `lineage_hash_chain` |
| `test_internal_state_blocked` | `internal_state` |
| `test_debug_info_blocked` | `debug_info` |
| `test_safe_patterns_allowed` | `belief`, `status` |
| `test_partial_match_blocked` | Substring detection |
| `test_case_insensitive_match` | Mixed case |
| `test_contains_multiple_patterns` | Multiple patterns |
| `test_nested_pattern_detection` | Nested JSON |
| `test_empty_string_safe` | Edge case |

### Running

```bash
cargo test boundary_tests
```

---

## Category: Tool Endpoint Tests (21)

Verify each MCP tool works correctly.

### Node Operations

| Test | Tool | Validates |
|------|------|-----------|
| `test_node_create_success` | `node.create` | Basic creation |
| `test_node_create_zero_belief` | `node.create` | Edge case |
| `test_node_create_max_belief` | `node.create` | Boundary |
| `test_node_query_success` | `node.query` | Retrieval |
| `test_node_query_not_found` | `node.query` | Error path |
| `test_node_mutate_success` | `node.mutate` | Mutation |
| `test_node_mutate_esv_guard` | `node.mutate` | ESV check |

### Edge Operations

| Test | Tool | Validates |
|------|------|-----------|
| `test_edge_bind_success` | `edge.bind` | Basic binding |
| `test_edge_bind_invalid_src` | `edge.bind` | Error path |
| `test_edge_bind_invalid_dst` | `edge.bind` | Error path |
| `test_edge_propagate_success` | `edge.propagate` | Propagation |
| `test_edge_propagate_not_found` | `edge.propagate` | Error path |

### Governor & ESV

| Test | Tool | Validates |
|------|------|-----------|
| `test_governor_status` | `governor.status` | Status query |
| `test_esv_audit_valid` | `esv.audit` | Valid node |
| `test_esv_audit_invalid` | `esv.audit` | Invalid node |

### Lineage

| Test | Tool | Validates |
|------|------|-----------|
| `test_lineage_replay` | `lineage.replay` | Replay |
| `test_lineage_export` | `lineage.export` | File export |

### Protocol

| Test | Tool | Validates |
|------|------|-----------|
| `test_tools_list` | `tools/list` | Tool enumeration |
| `test_initialize` | `initialize` | MCP handshake |
| `test_tools_call` | `tools/call` | Tool dispatch |

### Running

```bash
cargo test tool_endpoint
```

---

## Category: Error Handling Tests (15)

Verify proper error responses for invalid inputs.

| Test | Scenario |
|------|----------|
| `test_invalid_json` | Malformed JSON |
| `test_missing_method` | No method field |
| `test_unknown_method` | Invalid method name |
| `test_missing_params` | Required params missing |
| `test_invalid_uuid` | Malformed UUID |
| `test_invalid_belief_type` | Wrong type |
| `test_belief_out_of_range` | Value > 1.0 |
| `test_negative_energy` | Negative energy |
| `test_node_not_found` | Missing node |
| `test_edge_not_found` | Missing edge |
| `test_esv_rejection` | ESV violation |
| `test_drift_rejection` | Drift exceeded |
| `test_null_id` | Null request ID |
| `test_empty_params` | Empty object |
| `test_extra_fields` | Unknown fields |

### Running

```bash
cargo test error_handling
```

---

## Category: Adversarial Tests (20)

Simulate attack scenarios to verify security boundary.

### Unicode Bypass Attempts

| Test | Attack |
|------|--------|
| `test_unicode_bypass_cyrillic_a` | Cyrillic 'а' in pattern |
| `test_unicode_bypass_cyrillic_e` | Cyrillic 'е' in pattern |
| `test_unicode_bypass_cyrillic_o` | Cyrillic 'о' in pattern |
| `test_unicode_bypass_greek_alpha` | Greek 'α' in pattern |
| `test_unicode_bypass_greek_omicron` | Greek 'ο' in pattern |
| `test_unicode_bypass_zero_width` | Zero-width space |
| `test_unicode_bypass_soft_hyphen` | Soft hyphen |
| `test_unicode_bypass_bom` | BOM character |

### Injection Attempts

| Test | Attack |
|------|--------|
| `test_prompt_injection_in_belief` | Instructions in number |
| `test_prompt_injection_in_id` | Instructions in UUID |
| `test_sql_injection_pattern` | SQL in params |
| `test_command_injection_pattern` | Shell in params |

### Pattern Evasion

| Test | Attack |
|------|--------|
| `test_base64_encoded_pattern` | Encoded pattern |
| `test_url_encoded_pattern` | URL encoding |
| `test_nested_json_pattern` | Deeply nested |
| `test_array_pattern_injection` | Array payload |
| `test_mixed_case_evasion` | Random casing |
| `test_whitespace_padding` | Space padding |
| `test_underscore_variation` | `dag__topology` |
| `test_concatenation_attack` | Split pattern |

### Running

```bash
cargo test adversarial
```

---

## Unit Tests (41+)

Located in `src/` modules with `#[cfg(test)]`.

### By Module

| Module | Tests | Coverage |
|--------|-------|----------|
| `sanitizer/forbidden.rs` | 8 | Pattern matching |
| `sanitizer/response.rs` | 6 | Response sanitization |
| `governance.rs` | 5 | Validation logic |
| `substrate_runtime.rs` | 10 | Runtime operations |
| `types.rs` | 4 | Serialization |
| `lineage/` | 8 | Hash chain |

### Running

```bash
cargo test --lib
```

---

## Deterministic Mode

For reproducible test runs:

```bash
# Enable deterministic timestamps
export DETERMINISM=1
export TIMESTAMP_MODE=deterministic

# Run tests
cargo test
```

### Verifying Determinism

```bash
# Run twice, compare checksums
cargo test 2>&1 | sha256sum
cargo test 2>&1 | sha256sum
# Should produce identical hashes
```

---

## CI Integration

### Workflow: `mcp_integration.yml`

```yaml
- name: Run MCP integration tests
  run: cargo test --test mcp_integration --release -- --test-threads=1

- name: Run unit tests for sanitizer
  run: cargo test services::sanitizer --release

- name: Verify zero sanitization violations
  run: |
    cargo test --test mcp_integration --release 2>&1 | tee test_output.log
    if grep -i "forbidden pattern" test_output.log; then
      echo "::error::Sanitization violations detected"
      exit 1
    fi
```

---

## Snapshot Testing

### Golden Files

Location: `tests/snapshots/`

| File | Purpose |
|------|---------|
| `response_baseline.json` | Expected tool responses |

### Updating Snapshots

```bash
# Generate new baseline
cargo test --test mcp_integration -- --ignored generate_baseline

# Manually review and commit
```

---

## Writing New Tests

### Test Utilities

```rust
use crate::integration::common::{TestResponse, make_rpc_request};

#[test]
fn test_example() {
    let req = make_rpc_request("node.create", json!({
        "belief": 0.5,
        "energy": 1.0
    }));
    
    let resp = TestResponse::from_request(req);
    assert!(resp.is_success());
    assert!(!resp.contains_forbidden_patterns());
}
```

### Test Location Guidelines

| Type | Location |
|------|----------|
| New tool test | `tool_endpoint_tests.rs` |
| New error case | `error_handling_tests.rs` |
| New attack vector | `adversarial_tests.rs` |
| New forbidden pattern | `boundary_tests.rs` |
| Unit test | Same file as code |

---

## See Also

- [SECURITY.md](./SECURITY.md) - Security testing context
- [ATTACK_SURFACE.md](./ATTACK_SURFACE.md) - Patterns being tested


