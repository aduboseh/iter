# SCG MCP Server Security

> Hardened boundary between AI assistants and SCG internals

---

## Threat Model

### Adversary Capabilities

We assume an adversary who:
- Controls the AI assistant's queries
- Can craft arbitrary JSON-RPC requests
- Has knowledge of SCG's public documentation
- May attempt prompt injection via responses
- May use Unicode obfuscation techniques

### Assets Protected

| Asset | Risk if Exposed |
|-------|-----------------|
| DAG topology | Model reconstruction, gaming |
| ESV internals | Ethical constraint bypass |
| Energy distribution | Resource manipulation |
| Lineage hash chain | Audit forgery |
| Internal state | Full substrate compromise |

---

## Security Architecture

### The Sanitization Boundary

Every response passes through a hardened sanitizer:

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

---

## Defense Layers

### Layer 1: Input Validation

All incoming requests are validated:
- JSON-RPC 2.0 structure
- Method whitelist
- Parameter type checking
- UUID format validation

```rust
// Invalid method → rejected
"method": "internal.dump" // ❌ Unknown method

// Invalid params → rejected
"params": {"belief": "not_a_number"} // ❌ Type error
```

### Layer 2: ESV Guard

Pre-operation safety check:
- Validates proposed state changes
- Rejects operations exceeding ethical thresholds
- Returns deterministic error codes

```rust
if new_belief.abs() > threshold {
    return Err(ScgError::EsvFailed); // Code 1000
}
```

### Layer 3: Drift Guard

Energy conservation enforcement:
- Checks thermodynamic drift before mutations
- Ensures system stability
- Rejects destabilizing operations

```rust
if !runtime.energy_drift_ok() {
    return Err(ScgError::DriftExceeded); // Code 2000
}
```

### Layer 4: Response Sanitization

The critical boundary. See [ATTACK_SURFACE.md](./ATTACK_SURFACE.md) for full pattern list.

**Sanitization Pipeline:**

1. **Unicode Normalization**
   - Strip zero-width characters (U+200B, U+200C, U+200D, U+FEFF, U+00AD)
   - Normalize Cyrillic lookalikes (а→a, е→e, о→o)
   - Normalize Greek lookalikes (α→a, ο→o)

2. **Pattern Matching**
   - Check against 60+ forbidden patterns
   - Case-insensitive matching
   - Substring detection

3. **Field Redaction**
   - Remove forbidden fields from JSON
   - Replace sensitive values with placeholders
   - Log violations (without exposing to AI)

---

## Blocked Attack Vectors

| Attack | Technique | Defense |
|--------|-----------|---------|
| **Topology Reconstruction** | Query for `dag_topology`, `adjacency_matrix` | Pattern blocked |
| **ESV Bypass** | Request `esv_raw`, `ethical_gradient` | Pattern blocked |
| **Energy Gaming** | Access `energy_matrix`, `energy_distribution` | Pattern blocked |
| **Lineage Forgery** | Request `lineage_hash_chain`, `parent_hash` | Only checksums exposed |
| **Unicode Obfuscation** | Use `dаg_topology` (Cyrillic а) | Normalized to ASCII |
| **Zero-Width Injection** | Use `dag\u200B_topology` | Zero-width stripped |
| **Prompt Injection** | Embed instructions in response | All internals sanitized |
| **Timing Attack** | Measure response latency | Deterministic execution |

---

## Error Handling

Errors are sanitized before returning to AI:

| Code | Message | Meaning |
|------|---------|---------|
| 1000 | `ESV_VALIDATION_FAILED` | Ethical constraint violation |
| 2000 | `THERMODYNAMIC_DRIFT_EXCEEDED` | Energy conservation failure |
| 4000 | `BAD_REQUEST: ...` | Invalid input |
| 4004 | `NOT_FOUND: ...` | Resource doesn't exist |
| -32700 | `Parse error` | Invalid JSON |

**Never exposed:**
- Stack traces
- Internal state dumps
- File paths
- Memory addresses

---

## Immutable Security Components

### Forbidden Pattern Registry

Location: `src/services/sanitizer/forbidden.rs`

```
╔══════════════════════════════════════════════════════════════════════════╗
║  IMMUTABLE REGISTRY — DO NOT MODIFY WITHOUT FOUNDER-LEVEL OVERRIDE       ║
║  Version: 2.0.0 | Sealed: 2025-12-03 | Authority: SCG Governor           ║
║  Any modification requires CODEOWNERS approval and audit trail entry.    ║
╚══════════════════════════════════════════════════════════════════════════╝
```

Protected by:
- CODEOWNERS (`@aduboseh` required for changes)
- CI validation
- Governance checksum

---

## Security Testing

### Adversarial Test Suite

20 tests simulating attack scenarios:

- `test_unicode_bypass_cyrillic` - Cyrillic lookalike injection
- `test_unicode_bypass_zero_width` - Zero-width character injection
- `test_prompt_injection_in_params` - Malicious parameter content
- `test_nested_forbidden_patterns` - Deeply nested JSON attacks
- `test_base64_encoded_patterns` - Encoding bypass attempts
- `test_case_variation_bypass` - Mixed case attacks
- `test_substring_patterns` - Partial pattern matching
- ... and more

### Running Security Tests

```bash
cargo test adversarial
cargo test boundary
```

---

## Incident Response

### If a Pattern Bypass is Discovered

1. **Immediate:** Add pattern to `FORBIDDEN_PATTERNS`
2. **CI:** Ensure new test covers the bypass
3. **Audit:** Review lineage for past exposures
4. **Release:** Tag new version with security fix

### Reporting Security Issues

Email: security@onlysgsolutions.com

**Do not** open public issues for security vulnerabilities.

---

## Compliance

- **Determinism:** All operations produce identical results given identical inputs
- **Auditability:** Complete lineage chain with SHA-256 proofs
- **Isolation:** No substrate internals exposed via MCP

---

## See Also

- [ATTACK_SURFACE.md](./ATTACK_SURFACE.md) - Complete forbidden pattern list
- [ARCHITECTURE.md](./ARCHITECTURE.md) - System design
- [GOVERNANCE.md](./GOVERNANCE.md) - Change control
