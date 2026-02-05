# MCP Forbidden Behaviors

**Classification:** External-Safe  
**Version:** 1.0  
**Last Updated:** February 2026

---

## Hard Red Lines

These behaviors are architecturally forbidden in MCP tools.

### 1. No Business Logic in Tools

**Forbidden:**
- Domain-specific decision-making
- Customer-specific rules
- Application logic

**Why:** Tools are platform primitives, not application features.

---

### 2. No Stateful Inference

**Forbidden:**
- Session state
- Accumulated context
- Conversational memory

**Why:** Iter is stateless per-request.

---

### 3. No Hidden Side Effects

**Forbidden:**
- Network calls
- File writes
- Database mutations
- External process spawns

**Why:** Violates side-effect isolation invariant.

---

### 4. No Adaptive Behavior

**Forbidden:**
- Learning from usage
- Behavior modification over time
- Online optimization

**Why:** Violates determinism invariant.

---

## Enforcement

**Build-time:** Type system prevents prohibited operations.

**Runtime:** Audit log analysis detects violations.

**Test-time:** Integration tests verify constraints.

**Violation = architectural failure.**
