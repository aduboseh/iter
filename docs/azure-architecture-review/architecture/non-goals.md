# Iter Non-Goals

**Classification:** External-Safe  
**Version:** 1.0  
**Last Updated:** February 2026

---

## Purpose

This document explicitly states what Iter does NOT do. This prevents scope creep and incorrect expectations.

---

## What Iter Is NOT

### 1. NOT an Orchestrator

**Iter does not:**
- Sequence AI agent actions
- Manage workflow state machines
- Coordinate multi-step processes
- Route requests between services
- Handle retries or timeouts

**Why:**
- Orchestration involves stateful coordination
- Iter is stateless per-request
- Orchestration is the consumer's responsibility

**Correct Pattern:**
```
Orchestrator → Iter (governance check)
Orchestrator → Action (if permitted)
Orchestrator → Next Step
```

**Anti-Pattern:**
```
Iter → Action 1
Iter → Action 2
Iter → Action 3
```

---

### 2. NOT a Reasoning System

**Iter does not:**
- Generate content
- Produce reasoning signals
- Infer intent from natural language
- Interpret ambiguous inputs
- Make probabilistic decisions

**Why:**
- Reasoning is non-deterministic
- Iter enforces determinism
- Reasoning is the model's responsibility

**Correct Pattern:**
```
Model → Reasoning Signal
Consumer → Iter (validate signal)
Iter → Governance Decision
```

**Anti-Pattern:**
```
Natural Language → Iter (interpret)
Iter → Inferred Intent
```

---

### 3. NOT a Learning System

**Iter does not:**
- Train models
- Update weights
- Adapt behavior based on usage
- Perform online learning
- Store learning artifacts

**Why:**
- Learning changes behavior over time
- Iter behavior is fixed per policy version
- Learning is the model training pipeline's responsibility

**Correct Pattern:**
```
Training Pipeline → Model
Model → Reasoning Signal
Iter → Governance (using fixed policy)
```

**Anti-Pattern:**
```
Usage Data → Iter (learn)
Iter → Updated Behavior
```

---

### 4. NOT a Memory System

**Iter does not:**
- Store conversational context
- Maintain session state
- Remember past interactions
- Build user profiles
- Accumulate long-term knowledge

**Why:**
- Memory is stateful
- Iter is stateless per-request
- Memory is the application layer's responsibility

**Correct Pattern:**
```
Application → Context (from database)
Application + Context → Model
Model → Reasoning Signal
Iter → Governance (stateless)
```

**Anti-Pattern:**
```
User Input → Iter (remember)
Iter → Stateful Response
```

---

### 5. NOT a Business Logic Layer

**Iter does not:**
- Decide business outcomes
- Apply domain-specific rules (beyond policy)
- Interpret business requirements
- Make product decisions
- Optimize for business metrics

**Why:**
- Business logic belongs in the application layer
- Iter enforces governance, not business rules
- Business logic varies per customer; Iter is a platform

**Correct Pattern:**
```
Application → Business Logic
Application → Iter (governance check)
Iter → Permit/Deny
Application → Business Action (if permitted)
```

**Anti-Pattern:**
```
Generic Input → Iter
Iter → Business Decision
```

---

### 6. NOT an Ethics Engine

**Iter does not:**
- Interpret ethical principles
- Make normative judgments
- Decide what is "right" or "wrong"
- Apply moral reasoning
- Resolve ethical dilemmas

**Why:**
- Ethics require human judgment
- Iter enforces policy authored by humans
- Ethical interpretation is the policy author's responsibility

**Correct Pattern:**
```
Human → Policy (ethical constraints)
Iter → Enforce Policy
Human → Review Outcomes
```

**Anti-Pattern:**
```
Ambiguous Scenario → Iter (interpret ethics)
Iter → Ethical Judgment
```

---

### 7. NOT a Prompt Optimizer

**Iter does not:**
- Rewrite prompts
- Optimize for model performance
- A/B test prompt variations
- Suggest prompt improvements
- Inject system messages

**Why:**
- Prompt optimization is non-deterministic
- Iter receives structured inputs, not prompts
- Prompt engineering is the application's responsibility

**Correct Pattern:**
```
Application → Optimized Prompt
Model → Structured Output
Iter → Governance (on structured output)
```

**Anti-Pattern:**
```
Raw Prompt → Iter (optimize)
Iter → Rewritten Prompt
```

---

### 8. NOT a Model Hosting Platform

**Iter does not:**
- Host foundation models
- Serve inference endpoints
- Manage GPU allocation
- Optimize inference latency
- Provide model APIs

**Why:**
- Model hosting is a separate concern
- Iter consumes model outputs, does not produce them
- Model hosting is Azure AI, OpenAI, etc.'s responsibility

**Correct Pattern:**
```
Model Hosting Platform → Inference
Application → Iter (governance on output)
```

**Anti-Pattern:**
```
Application → Iter (host model)
Iter → Inference
```

---

### 9. NOT a Data Pipeline

**Iter does not:**
- Ingest training data
- Transform datasets
- Store embeddings
- Manage vector databases
- Build search indexes

**Why:**
- Data pipelines are stateful and high-volume
- Iter is request-scoped and low-latency
- Data engineering is a separate infrastructure concern

**Correct Pattern:**
```
Data Pipeline → Vector DB
Application → Vector Search
Application → Iter (governance on retrieval)
```

**Anti-Pattern:**
```
Raw Data → Iter (ingest)
Iter → Indexed Data
```

---

### 10. NOT a Monitoring System

**Iter does not:**
- Aggregate metrics across requests
- Build dashboards
- Trigger alerts
- Analyze trends
- Generate reports

**Why:**
- Monitoring is a cross-cutting concern
- Iter emits telemetry; monitoring systems consume it
- Monitoring is Azure Monitor, Grafana, etc.'s responsibility

**Correct Pattern:**
```
Iter → Structured Logs
Monitoring System → Aggregate + Alert
```

**Anti-Pattern:**
```
Monitoring Query → Iter (analyze trends)
Iter → Dashboards
```

---

## Why Non-Goals Matter

### Prevents Roadmap Poisoning

**Without explicit non-goals:**
- Features creep into scope
- Boundaries blur
- Determinism is compromised

**With explicit non-goals:**
- Scope is defensible
- Features are rejected with clear rationale
- Architecture stays coherent

### Clarifies Integration Patterns

**External reviewers can:**
- Understand where Iter fits
- Identify integration points
- Avoid misuse patterns

### Supports Compliance

**Auditors can verify:**
- Iter does not make decisions it claims not to make
- Boundaries are enforced
- Responsibilities are clear

---

## Non-Goals Verification

### Claim

Iter does not perform any activity listed in this document.

### Verification Method

1. Review codebase for prohibited capabilities
2. Analyze audit logs for prohibited operations
3. Inspect telemetry for prohibited side effects
4. Review external dependencies (no model hosting libraries, etc.)

### Continuous Verification

- Code review checklist includes non-goals verification
- Architectural reviews flag scope expansion
- Integration tests verify boundary enforcement

---

## Non-Goals Evolution

### Adding a Non-Goal

**Required when:**
- External confusion about scope occurs
- Integration anti-patterns emerge
- Roadmap proposals threaten boundaries

**Process:**
1. Document prohibited capability
2. Explain why it is a non-goal
3. Provide correct integration pattern
4. Update this document

### Removing a Non-Goal

**NOT PERMITTED without explicit architectural review.**

Removing a non-goal expands scope and may compromise determinism.

---

## Summary

| Non-Goal | Iter Does NOT | Responsible System |
|----------|---------------|-------------------|
| Orchestration | Sequence actions | Workflow systems |
| Reasoning | Generate content | AI models |
| Learning | Adapt behavior | Training pipelines |
| Memory | Store context | Application layer |
| Business Logic | Decide outcomes | Application layer |
| Ethics | Interpret principles | Policy authors, humans |
| Prompt Optimization | Rewrite prompts | Application layer |
| Model Hosting | Serve inference | Azure AI, OpenAI, etc. |
| Data Pipeline | Ingest data | Data engineering |
| Monitoring | Aggregate metrics | Azure Monitor, Grafana, etc. |

**Iter is a governance control plane. Nothing more, nothing less.**
