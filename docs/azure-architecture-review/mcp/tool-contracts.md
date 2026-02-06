# MCP Tool Contracts

**Classification:** External-Safe  
**Version:** 1.0  
**Last Updated:** February 2026

---

## Tool Immutability

**Once a tool is published, its contract is immutable.**

| Change | Breaking | Permitted |
|--------|----------|-----------|
| Add optional parameter | No | Yes |
| Remove parameter | Yes | No |
| Change parameter type | Yes | No |
| Change output schema | Yes | No |
| Rename tool | Yes | No |

---

## Backward Compatibility

### Minor Version Changes (1.x.0)

- Add new tool (non-breaking)
- Add optional parameter (non-breaking)
- Add new enum value (breaking for old readers)

### Major Version Changes (x.0.0)

- Remove tool
- Change tool contract
- Change required parameters

---

## Contract Enforcement

**Location:** MCP protocol layer (pre-execution)

**Mechanism:**
- JSON schema validation
- Type checking
- Enum validation

**Failure:** Reject with JSON-RPC error code -32602 (Invalid params)
