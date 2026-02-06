# Demo Scope

**Classification:** External-Safe  
**Version:** 1.0  
**Last Updated:** February 2026

---

## Purpose

Iter demos demonstrate governance mechanics only. They do not contain proprietary logic or production-ready agents.

---

## What Demos Show

### 1. Deterministic Replay

**Demo:** `examples/determinism_demo.rs`

**Shows:**
- Same input → same DecisionPacket
- Same checksum across runs
- Replay verification

**Does NOT show:**
- Model inference
- Application logic
- Production deployment

---

### 2. Governance Gating

**Demo:** Policy gate evaluation

**Shows:**
- Input validation
- Policy evaluation
- Reason code emission
- ALLOW vs DENY outcomes

**Does NOT show:**
- Business-specific policies
- Customer data
- Production rules

---

### 3. Audit Trail

**Demo:** AuditEvent generation

**Shows:**
- Structured logging
- Event correlation
- Lineage reconstruction

**Does NOT show:**
- Long-term retention strategy
- Compliance reporting
- Production audit volume

---

## Absence of Agents is Intentional

**Iter is NOT an agent framework.**

Demos do not include:
- Multi-step orchestration
- Agent reasoning loops
- Conversational interfaces
- Autonomous behavior

**Why:** Agents are consumers of Iter, not part of Iter.

---

## No Proprietary Logic

**Demos use synthetic inputs.**

**Demos do NOT contain:**
- SCG internals
- Proprietary algorithms
- Customer-specific logic
- Production configurations

**What is shown:**
- External-safe interfaces
- Governance contracts
- Audit artifacts

---

## Demo Narrative Control

**Demos are scoped to prevent misinterpretation.**

| Misinterpretation | Prevention |
|-------------------|-----------|
| "Iter is an orchestrator" | Demos show single governance decisions, not workflows |
| "Iter contains AI models" | Demos use pre-generated reasoning signals |
| "Iter decides business outcomes" | Demos enforce policy, not business logic |

**Demos support architectural review, not feature evaluation.**
