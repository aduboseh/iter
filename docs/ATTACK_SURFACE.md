# SCG MCP Attack Surface

> Complete enumeration of blocked patterns and bypass prevention

---

## Surface Exposure

### What AI CAN Access

| Endpoint | Data Returned | Safe Because |
|----------|---------------|--------------|
| `node.create` | Node ID, belief, energy | No topology info |
| `node.mutate` | Updated node state | ESV-guarded |
| `node.query` | Single node state | No relationships |
| `edge.bind` | Edge ID only | No adjacency data |
| `edge.propagate` | Success/failure | No energy details |
| `governor.status` | Drift, coherence (summary) | Aggregated only |
| `esv.audit` | VALID/INVALID | Boolean only |
| `lineage.replay` | Checksum history | No hash chain |
| `lineage.export` | File path, checksum | No raw data |

### What AI Can NEVER Access

```
┌─────────────────────────────────────────────────────────────────┐
│                     FORBIDDEN ZONE                              │
├─────────────────────────────────────────────────────────────────┤
│  • Internal graph structure                                     │
│  • Node connection patterns                                     │
│  • Raw ethical state vectors                                    │
│  • Energy distribution matrices                                 │
│  • Merkle hash chains                                           │
│  • Governor correction deltas                                   │
│  • Meta-cognitive state                                         │
│  • Debug/stack traces                                           │
└─────────────────────────────────────────────────────────────────┘
```

---

## Forbidden Pattern Registry

### Category: DAG Topology Internals

These patterns would allow reconstruction of the internal graph structure:

| Pattern | Risk |
|---------|------|
| `dag_topology` | Full graph structure |
| `node_ids` | All node enumeration |
| `edge_weights` | Weight distribution |
| `adjacency_matrix` | Connection matrix |
| `adjacency_list` | Connection list |
| `adjacency` | Any adjacency data |
| `topology` | Structure info |
| `dag_structure` | Graph structure |
| `node_connections` | Relationship data |
| `edge_list` | All edges |
| `internal_edges` | Edge internals |
| `node_internal_state` | Full node state |
| `belief_vector` | All beliefs |
| `energy_allocation` | Energy distribution |
| `propagation_path` | Flow paths |

### Category: ESV (Ethical State Vector) Internals

These patterns would enable ethical constraint bypass:

| Pattern | Risk |
|---------|------|
| `esv_raw` | Raw ethical vector |
| `esv_matrix` | ESV structure |
| `esv_checksum_internal` | Internal checksum |
| `ethical_gradient` | Gradient data |
| `harm_potential_raw` | Harm calculations |
| `truth_confidence_internal` | Internal confidence |
| `moral_vector` | Moral state |
| `raw_tau` | Raw tau value |
| `raw_harm` | Raw harm value |
| `raw_chi` | Raw chi value |
| `ethical_potential_raw` | Potential data |

### Category: Energy System Internals

These patterns would allow energy manipulation:

| Pattern | Risk |
|---------|------|
| `energy_matrix` | Energy distribution |
| `node_energies` | Per-node energy |
| `energy_distribution` | Distribution data |
| `energy_redistribution_log` | Redistribution history |
| `governor_correction_delta` | Correction amounts |
| `thermodynamic_state` | Thermo state |
| `entropy_internal` | Entropy data |
| `energy_delta` | Energy changes |
| `node_energy_allocation` | Allocation data |
| `hamiltonian` | Energy function |
| `internal_energy` | Internal state |

### Category: Lineage Ledger Internals

These patterns would enable audit forgery:

| Pattern | Risk |
|---------|------|
| `lineage_hash_chain` | Full hash chain |
| `lineage_chain` | Chain data |
| `full_lineage` | Complete history |
| `lineage_entries` | Entry data |
| `ledger_raw` | Raw ledger |
| `cascade_hash_internal` | Cascade hashes |
| `parent_hash` | Parent references |
| `shard_id` | Shard identifiers |
| `hash_chain` | Any hash chain |
| `state_snapshots` | State history |

### Category: Elastic Governor Internals

These patterns would compromise consensus:

| Pattern | Risk |
|---------|------|
| `governor_quorum_state` | Quorum data |
| `consensus_votes` | Voting data |
| `drift_correction_vector` | Correction vectors |
| `node_energy_deltas` | Energy deltas |
| `quorum_members` | Member list |

### Category: Meta-Cognitive Layer

These patterns would expose self-referential state:

| Pattern | Risk |
|---------|------|
| `reflective_state` | Reflection data |
| `coherence_raw` | Raw coherence |
| `meta_cognitive_variance` | Variance data |
| `self_referential_state` | Self-reference |

### Category: Implementation/Debug

These patterns would leak implementation details:

| Pattern | Risk |
|---------|------|
| `internal_state` | Any internal state |
| `debug_info` | Debug data |
| `substrate_state` | Substrate internals |
| `raw_state` | Raw state dump |
| `stack_trace` | Error traces |
| `backtrace` | Call stack |
| `panic_message` | Panic info |
| `internal_error` | Error internals |

---

## Bypass Prevention

### Unicode Normalization

Attackers may attempt to bypass pattern matching using lookalike characters:

| Attack | Example | Defense |
|--------|---------|---------|
| Cyrillic 'а' | `dаg_topology` | Normalized to `dag_topology` |
| Cyrillic 'е' | `еsv_raw` | Normalized to `esv_raw` |
| Cyrillic 'о' | `tоpology` | Normalized to `topology` |
| Cyrillic 'р' | `рarent_hash` | Normalized to `parent_hash` |
| Cyrillic 'с' | `сhain` | Normalized to `chain` |
| Greek 'α' | `αdjacency` | Normalized to `adjacency` |
| Greek 'ο' | `tοpology` | Normalized to `topology` |
| Turkish 'ı' | `ınternal` | Normalized to `internal` |

### Zero-Width Character Stripping

Invisible characters are removed before pattern matching:

| Character | Unicode | Name |
|-----------|---------|------|
| ​ | U+200B | Zero-width space |
| ‌ | U+200C | Zero-width non-joiner |
| ‍ | U+200D | Zero-width joiner |
| ﻿ | U+FEFF | BOM / Zero-width no-break space |
| ­ | U+00AD | Soft hyphen |

**Example Attack:**
```
"dag​_topology"  →  Stripped to  →  "dag_topology"  →  BLOCKED
    ^
    Zero-width space (invisible)
```

### Case Normalization

All patterns are matched case-insensitively:

```
"DAG_TOPOLOGY"     → BLOCKED
"Dag_Topology"     → BLOCKED
"dAg_ToPOloGY"     → BLOCKED
```

---

## Sensitive Patterns (Contextual)

These patterns are allowed in aggregated form but forbidden in raw form:

| Pattern | Allowed | Forbidden |
|---------|---------|-----------|
| `energy` | `energy_summary` | `energy_matrix` |
| `coherence` | `coherence_index` | `coherence_raw` |
| `drift` | `drift_status` | `drift_correction_vector` |
| `checksum` | `validation_status` | `esv_checksum_internal` |
| `hash` | `integrity_verified` | `lineage_hash_chain` |

---

## Testing Coverage

### Boundary Tests (13)

Verify each forbidden pattern is blocked:

```bash
cargo test boundary_tests
```

### Adversarial Tests (20)

Simulate bypass attempts:

```bash
cargo test adversarial_tests
```

### Test Examples

```rust
#[test]
fn test_unicode_bypass_cyrillic() {
    // Cyrillic 'а' looks like Latin 'a'
    let obfuscated = "dаg_topology"; // Uses Cyrillic а
    assert!(is_forbidden(obfuscated));
}

#[test]
fn test_unicode_bypass_zero_width() {
    let obfuscated = "dag\u{200B}_topology"; // Zero-width space
    assert!(is_forbidden(obfuscated));
}
```

---

## Adding New Patterns

When a new attack vector is discovered:

1. Add pattern to `FORBIDDEN_PATTERNS` in `src/services/sanitizer/forbidden.rs`
2. Add test case to `tests/integration/boundary_tests.rs`
3. Submit PR (requires `@aduboseh` approval via CODEOWNERS)
4. Update this document

---

## See Also

- [SECURITY.md](./SECURITY.md) - Security architecture
- [ARCHITECTURE.md](./ARCHITECTURE.md) - System design
