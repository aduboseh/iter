# SCG MCP Server Project Rules

## Table 1 — Global Invariants

### G1: Correctness Over Complexity
**Category:** Global invariant  
**Trigger:** Any non-trivial request (code, architecture, data, infra).  
**Behavior:** Select the simplest viable solution that satisfies functional, security, and compliance requirements. If two approaches are equivalent, choose the one with fewer components, dependencies, and moving parts. Document rejected "fancier" options in brief.  
**SCG Mapping:** Implementation pillar "clarity before complexity; truth before speed".  
**Notes:** Explicitly state when discarding a more complex pattern in favor of a simpler one.

### G2: Deterministic Reasoning & Replay
**Category:** Global invariant  
**Trigger:** Any plan, code generation, or multi-step reasoning chain.  
**Behavior:** Require that the proposed workflow can be replayed deterministically: same inputs → same outputs within ε ≤ 1e-10. If nondeterminism is unavoidable (e.g., external services), label those boundaries and suggest mitigation (fixed seeds, mocks, or recorded fixtures).  
**SCG Mapping:** Deterministic replay ε ≤ 1e-10; lineage ledger; replay determinism.  
**Notes:** Flag any randomness, concurrency, or external dependency and propose a deterministic harness for development and testing.

### G3: SCG Pillar Alignment Check
**Category:** Global invariant  
**Trigger:** When designing anything that touches reasoning, data, or infra.  
**Behavior:** Before committing to a solution, briefly validate against the four SCG pillars: Physics (energy/efficiency), Psychology (cognitive load), Graph Theory (acyclic, traceable structure), Philosophy/Ethics (ESV compliance). If any axis is weak, propose a simpler or safer alternative.  
**SCG Mapping:** Objective + pillars (physics, psychology, graph theory, philosophy, ethics).  
**Notes:** Quick "4-axis sanity check" step—one sentence per pillar is enough, but it must be explicit.

---

## Table 2 — Task Structuring & Cognitive Load

### T1: Thermodynamic Task Decomposition
**Category:** Task structure  
**Trigger:** Request spans >3 major steps, or >2 domains (e.g., infra + app + data), or would touch multiple repos/services.  
**Behavior:** Decompose into a DAG of subtasks: each node with clear input/output, effort estimate, and explicit dependency edges. Represent visually (outline / pseudo-Gantt) and ensure acyclicity.  
**SCG Mapping:** DAG execution model; causal coherence; acyclicity invariants.  
**Notes:** Use topological order: Phase 0 (baseline/guards), then build, then validate, then deploy. No circular dependencies between tasks.

### T2: Cognitive Energy Budget Monitoring
**Category:** Task structure  
**Trigger:** Long planning / reasoning sessions, or repeated revisions on same feature.  
**Behavior:** Track "cognitive energy" via complexity proxies (number of subtasks, cross-repo references, external systems used). If complexity is clearly growing (e.g., >7 subtasks, or three+ external systems), pause and propose refactor: either narrower scope or simpler architecture.  
**SCG Mapping:** Energy conservation; Elastic Governor minimizing drift and entropy.  
**Notes:** Explicitly say: "Complexity is rising; recommending simplification: option A (narrow scope) or B (simpler stack)."

---

## Table 3 — Code & Repo Behavior

### C1: Cognitive Codebase Context Activation
**Category:** Codebase reasoning  
**Trigger:** User is inside a directory with Cargo.toml, Dockerfile, pyproject.toml, requirements.txt, package.json, .warp-config, or SCG spec files.  
**Behavior:** Auto-load project context: core manifests, main entrypoints, and high-centrality modules. Reason across files, not in isolation. Any code change must consider dependent modules (import graph / crates).  
**SCG Mapping:** Integrative nodes; distributed coherence; deployment architecture's "Cognitive Cluster".  
**Notes:** Use static analysis (language server, tree-sitter, rust-analyzer) to build a dependency graph; prioritize highly-connected files during reasoning.

### C2: Deterministic Code Quality Gate
**Category:** Code quality  
**Trigger:** Any generated or modified code that is intended to be committed or run.  
**Behavior:** Always: (1) format with canonical tools (e.g., cargo fmt, black, prettier), (2) run lint (Ruff/ESLint/clippy), (3) run core tests or create minimal tests if none exist, (4) refuse to treat the change as "ready" if lint/test fails; instead return a fixed version or a remediation plan.  
**SCG Mapping:** Deterministic replay; energy-neutral changes; implementation discipline.  
**Notes:** Phrase results as "PASS/FAIL" and never silently ignore failures. If tests are missing, propose skeleton tests and mark the gap explicitly.

### C3: Cross-Architecture Safety (FFI & Platform)
**Category:** Code quality  
**Trigger:** Any code at FFI boundaries, pointer logic, or architecture-dependent types; or any time user mentions ARM, x86, container images, or deployment targets.  
**Behavior:** Force explicit handling of platform differences (e.g., c_char signedness, alignment, endianness). Document assumptions and, where feasible, introduce compile-time checks/tests for each target.  
**SCG Mapping:** Mathematical foundations; energy drift and determinism across environments.  
**Notes:** Always propose at least one cross-arch test path (e.g., GitHub Actions matrix: x86_64 + aarch64) when working on low-level SCG or infra code.

---

## Table 4 — Environment, Infra, and Deployment

### E1: Fail-Proof Environment Validation & Remediation
**Category:** Environment  
**Trigger:** Before suggesting or executing commands involving: Docker, Kubernetes, Azure, databases, queues, or external APIs (OpenAI, Azure AI, NVIDIA, etc.).  
**Behavior:** 1) Enumerate required env vars, credentials, and tools. 2) Ask user which are present vs missing (or specify checks/scripts). 3) For missing pieces, generate deterministic remediation steps (e.g., sample .env, az login commands), with clear rollback or "no-side-effects" options.  
**SCG Mapping:** Deployment architecture's fault tolerance; governor + kernel coordination; energy-neutral recovery.  
**Notes:** Never output a kubectl/az/docker command as "final" without first naming the prerequisites and failure modes.

### E2: Production-Ready by Default (SCG Thresholds)
**Category:** Environment / Quality  
**Trigger:** Any workflow labeled deploy, migrate, release, or "this goes to production / demo".  
**Behavior:** Treat everything as production-grade: enforce clear rollback path, logging, and observability. Map the plan back to SCG metrics: (a) deterministic behavior, (b) traceability, (c) safe failure. If a shortcut is proposed (e.g., skipping tests), explicitly label it as non-SCG compliant.  
**SCG Mapping:** Deployment architecture validation; uptime, determinism, ESV checks; ethics before optimization.  
**Notes:** Make a tiny "production checklist" every time: config, secrets, monitoring, rollback. Keep it short but concrete.

### E3: Physics-Based Model/Tool Selection
**Category:** Environment / Tooling  
**Trigger:** Any decision between model providers (Azure OpenAI vs NVIDIA vs local), runtimes, or infra components.  
**Behavior:** Choose models/tools based on measurable criteria: latency/throughput, cost, determinism, data locality, compliance requirements. Document the tradeoffs and pick the option with best balance for the stated goal; default to Azure OpenAI only when metrics are within a narrow band and operationally simpler.  
**SCG Mapping:** Physics pillar (energy, cost, efficiency); implementation discipline; deployment constraints.  
**Notes:** Converts the old "user prefers Azure" rule into a physics/ethics decision rule, not a preference rule.

---

## Table 5 — Ethics, Lineage, and Governance

### X1: Ethical Kernel Constraint Validation
**Category:** Ethics  
**Trigger:** Any logic that touches user data, healthcare, finance, risk scoring, or decisions affecting human outcomes; any plan that automates actions.  
**Behavior:** Perform a lightweight ESV check: Is this accurate (truth), safe (harm), and consistent (coherence) with your stated use-case and legal frameworks (HIPAA, SOC2, NIST)? If not, either refuse or provide a safer alternative.  
**SCG Mapping:** Ethical State Vector, Φ_ethics; ethical autonomy; reflective feedback loops.  
**Notes:** Explicitly say "ESV check: PASS/FAIL" and highlight any potential harms, bias amplification, or compliance risks.

### X2: Deterministic Lineage Ledger Commit (Conceptual)
**Category:** Ethics / Traceability  
**Trigger:** Any "final" artifact: code snippets, scripts, directives, or architecture diagrams intended for re-use.  
**Behavior:** Treat each artifact as if a lineage ledger exists: provide a brief header with purpose, version/date, and assumptions. Where a repo uses real SCG ledger tooling, also suggest or auto-generate SHA256 and update the ledger manifest.  
**SCG Mapping:** Lineage Ledger; cascading checksums; replay determinism.  
**Notes:** Always emit structured headers that a ledger script can later consume.

### X3: Consensus-Based Conflict Resolution
**Category:** Governance  
**Trigger:** When multiple valid solutions exist or tradeoffs are non-obvious (e.g., monolith vs services, Rust vs Python for a component).  
**Behavior:** Present at least two options and compare on: determinism, complexity, cost/energy, compliance surface, and long-term maintainability. If scores are close, recommend the simpler one and record rationale.  
**SCG Mapping:** Governor consensus; multi-objective optimization; ethics kernel + physics tradeoff.  
**Notes:** Prevents silent bias toward "cool tech" and keeps future-you able to reconstruct why a path was chosen.

---

## Table 6 — Meta-Stability & Self-Audit

### M1: Self-Referential Integrity Check
**Category:** Meta / Stability  
**Trigger:** Before major repo refactors, SCG MCP/ substrate changes, or a tagged release / external demo.  
**Behavior:** Run a mental or scripted audit: (1) Do key invariants still hold (determinism, energy neutrality, ESV constraints)? (2) Are there any parts of the design that you cannot currently explain? (3) Are any rules redundant or conflicting? If yes to any, propose targeted cleanups.  
**SCG Mapping:** Reflective feedback loop; distributed coherence; validation criteria.  
**Notes:** Manifests as "pause and sanity-check your own plan" before committing to a big move.

### M2: Rule Drift & Redundancy Guard
**Category:** Meta / Governance  
**Trigger:** When adding, editing, or deleting Warp rules themselves.  
**Behavior:** Check every new/changed rule against this table: if it duplicates another rule's effect, narrow it or merge it. If it contradicts an invariant (G1–G3), it must be revised or dropped. Document any intentional override explicitly.  
**SCG Mapping:** Spec index, governance, and "no redundant entropy" principle.  
**Notes:** Keeps the Warp rule system itself from becoming the next source of complexity and drift.
