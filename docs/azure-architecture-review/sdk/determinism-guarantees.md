# SDK Determinism Guarantees

**Classification:** External-Safe  
**Version:** 1.0  
**Last Updated:** February 2026

---

## SDK Principle

**SDKs submit structure, not intent.**

### What SDKs Do

- Serialize requests to JSON-RPC 2.0
- Deserialize responses from server
- Validate inputs locally (type safety)
- Propagate TraceContext

### What SDKs Do NOT Do

- Infer intent from natural language
- Generate content
- Make probabilistic decisions
- Cache decisions
- Alter server outputs

---

## Determinism Parity

**Guarantee:** SDKs preserve server-side determinism.

**Enforcement:**
- SDKs do not introduce randomness
- SDKs do not modify server responses
- SDKs use canonical serialization

---

## Replay is Checksum-Based

**Replay uses DecisionPacket checksum, not SDK state.**

**SDK Role in Replay:**
1. Submit replay request to server
2. Receive replayed DecisionPacket
3. Verify checksum match
4. Report success/failure

**SDKs do NOT:**
- Store replay state
- Modify replay logic
- Cache replay results

---

## SDK Responsibilities

| Responsibility | Owned By |
|----------------|----------|
| Protocol serialization | SDK |
| Checksum verification | Server |
| Determinism enforcement | Server |
| Replay logic | Server |

**SDKs are thin clients. Server is authoritative.**
