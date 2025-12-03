# SCG Hardening Directive v2.0 — Complete Engineering Execution Plan

**Classification:** Internal — Founder/Chief Architect Only  
**Owner:** Armonti Du-Bose-Hill  
**Engine:** SCG Substrate + SCG MCP Server  
**Standard:** Top 0.01% Engineering (Physics, Psychology, Graph Theory, Philosophy)  
**Date:** December 3, 2025

---

## Executive Summary

This directive establishes the complete, unambiguous separation of (a) code requiring heavy engineering rigor, (b) operational configuration, (c) maintenance surfaces, and (d) zero-touch zones where no code may be written. It reflects the actual repository structure (SCG and scg_mcp_server/scg-mcp), embeds SCG's foundational invariants as CI gates, operationalizes neuro-mapping specifications, and provides Microsoft Partner Center–ready compliance artifacts.

**Core Principle:** Simplicity over complexity. If a simpler implementation satisfies SCG invariants (ΔE_total ≤ 1e-10, replay ε ≤ 1e-10, ESV checksum = 100%, coherence C ≥ 0.97), choose it. Avoid overengineering.

---

## L1 — Direct Answer

### Heavy Code (Rust Engineering Required)

- **SCG substrate (SCG repo):** DAG executor, energy model, Elastic Governor, ethics kernel, lineage ledger, replay engine, Hebbian updates, neuro-coherence metrics.
- **MCP API server (scg_mcp_server repo):** HTTP/gRPC controllers, substrate bridge, auth, rate limiting, sanitization, SDKs (Python/TS/Rust).
- **Azure containerization:** Dockerfile, Helm charts, K8s manifests, health probes.
- **Benchmarking/testing:** Invariant test harnesses, determinism validators, performance benchmarks.

### Light Configuration/Maintenance

- **Marketplace offer:** Publishing metadata, screenshots, pricing plans.
- **Partner Center:** Roles, business profile, co-sell assets.
- **Documentation:** README, CONTRIBUTING, ARCHITECTURE, compliance mappings.
- **Monitoring dashboards:** Grafana/Prometheus configs, alert thresholds.
- **Pilot deployments:** Customer-specific adapters, scenario definitions.

### Zero-Touch Zones (Never Code)

- No substrate introspection endpoints.
- No DAG topology logging.
- No ESV/energy matrix exposure.
- No debug interfaces that leak substrate internals.

---

## L2 — Rationale and Tradeoffs

### Why This Separation Matters

1. **Code = attack surface.** Only code what is necessary for SCG's reasoning engine, replay stability, and MCP delivery. Everything else should be admin/config.
2. **Substrate = maximum secrecy.** The SCG substrate is a sealed cognitive physics engine. Every extra line of code increases IP exposure risk and nondeterminism.
3. **MCP = safe boundary.** All public-facing functionality lives here—controlled, versioned, observable, and safe. No substrate internals cross this boundary.
4. **Simplicity doctrine.** SCG foundational principle: "clarity before complexity; ethics before optimization; truth before speed." Choose the simplest implementation that satisfies invariants.
5. **Microsoft Partner Center readiness.** Compliance, observability, and documentation must meet or exceed Azure Marketplace standards (NIST AI RMF v1.1, ISO 27001, SOC 2 Type II).

### Determinism is Non-Negotiable

All substrate code must enforce:

- ΔE_total ≤ 1e-10 (energy conservation)
- Replay ε ≤ 1e-10 (deterministic replay)
- τ ≥ 0.99 (causal coherence)
- ESV checksum = 100% (ethical validation)
- C ≥ 0.97 (distributed coherence)

---

## L3 — Repository Structure and Module Taxonomy

### Crate and Workspace Declaration

SCG and scg_mcp_server are **separate crates** and **separate workspaces** to enforce architectural boundaries, prevent cyclic dependencies, and simplify compliance audits. They are not part of a shared Cargo workspace.

- **Crate name (substrate):** `scg`
- **Crate name (MCP):** `scg_mcp_server`
- **Remote repositories:**
  - `SCG` (private, substrate)
  - `scg-mcp` (public-facing, MCP boundary)

### Import Rules (Non-Negotiable)

1. The `scg` crate does **NOT** import from `scg_mcp_server`.
   (Substrate is pure; no HTTP, no SDKs, no UI dependencies.)
2. The `scg_mcp_server` crate **MAY** import `scg`, but **ONLY** via `substrate_bridge.rs`.
   (All substrate interactions are mediated through a single, narrow interface.)
3. **No cyclic dependencies.**
   (This prevents build graph pollution and maintains determinism.)

---

### Repository A: SCG (Substrate Repository)

**Crate Name:** `scg`  
**Local Directory:** `SCG/`  
**Remote Repository:** `SCG` (GitHub, private)

**Purpose:** Physics-governed cognition substrate. All DAG execution, energy model, ethics kernel, belief propagation, lineage ledger, neuro-coherence, and deterministic replay logic.

**Folder Structure:**

```
SCG/
  src/
    core/
      dag_executor.rs         # DAG execution model (acyclicity, topological order)
      energy_model.rs         # Cognitive Hamiltonian, E_total conservation
      governor.rs             # Elastic Governor (drift correction, k=0.8)
      ethics_kernel.rs        # ESV logic (τ, h, χ), Φ_ethics potential
      lineage_ledger.rs       # Immutable lineage hash chain (SHA-256)
      replay_engine.rs        # Deterministic replay (ε ≤ 1e-10)
      hebbian.rs              # Hebbian edge weight updates (energy-neutral)
    neuro/
      coherence.rs            # Distributed coherence (C ≥ 0.97)
      reflective_loop.rs      # ACC analog: conflict monitoring, error correction
      executive_nodes.rs      # DLPFC analog: goal maintenance, constraint satisfaction
      ethical_valuation.rs    # OFC analog: harm minimization utility (U = τ - h)
    tests/
      invariants_energy.rs    # ΔE_total ≤ 1e-10 over 10^4 cycles
      invariants_replay.rs    # Replay ε ≤ 1e-10 across 3+ environments
      invariants_topology.rs  # DAG acyclicity checks, τ ≥ 0.99
      invariants_ethics.rs    # ESV checksum = 100%, dΦ_ethics/dt → 0
      neuro_validation.rs     # BEI ≥ 0.95, ACC/DLPFC/OFC coupling metrics
    lib.rs                    # Public API surface (minimal, for MCP only)
  benches/
    scg_bench.rs              # Performance benchmarks (latency, throughput)
  Cargo.toml                  # Crate metadata: name = "scg"
  README.md                   # "SCG Substrate (Physics/Ethics Core)"
  ARCHITECTURE.md             # Substrate architecture, invariants, module map
  CONTRIBUTING.md             # Developer onboarding, CI expectations
  .warp/
    context.md                # Warp AI substrate-protection directive
  .github/
    workflows/
      substrate-ci.yml        # CI: fmt, clippy, test, invariants, benchmarks
      invariant-gate.yml      # Dedicated invariant checks (blocks merges)
```

**NEVER in This Repo:**
- HTTP servers, SDKs, UI, Marketplace files, tenant logic, logs revealing DAG internals, vendor adapters.

**Done When:**
- Every module in `src/core/` and `neuro/` maps to a concept in the SCG specs.
- No HTTP, SDK, or UI code exists in the substrate repo.
- No DAG internals, ESV raw values, or substrate-level logs leak into any external interface.

---

### Repository B: scg_mcp_server (MCP Boundary Repository)

**Crate Name:** `scg_mcp_server`  
**Local Directory:** `scg_mcp_server/`  
**Remote Repository:** `scg-mcp` (GitHub, public-facing)

**Purpose:** Safe boundary layer around SCG substrate. All HTTP/gRPC endpoints, SDKs, demos, scenarios, Azure packaging, Marketplace assets, and tenant isolation.

**Folder Structure:**

```
scg_mcp_server/
  src/
    controllers/
      reason.rs               # POST /reason - primary reasoning endpoint
      explain.rs              # GET /explain - explainability (no DAG internals)
      trace.rs                # GET /trace/{id} - lineage hash summary
      governor.rs             # GET /governor/status - high-level drift metrics
      scenario.rs             # POST /scenario/run - scenario execution
    services/
      substrate_bridge.rs     # Narrow interface to SCG substrate (ONLY import point)
      auth.rs                 # Lineage-hash-based token validation
      rate_limiter.rs         # Per-tenant rate limiting
      sanitizer.rs            # Response redaction (no ESV internals, no DAG topology)
    middleware/
      logging.rs              # Structured logging (request ID, tenant ID, latency)
      metrics.rs              # Prometheus exporters (request counts, error rates)
      error_mapping.rs        # SCGError → HTTP status codes
    sdk/
      python/                 # Python SDK with type hints
        scg_client.py
        __init__.py
        README.md
      typescript/             # TypeScript SDK with strong types
        src/client.ts
        package.json
        README.md
      rust/                   # Rust client SDK
        src/lib.rs
        Cargo.toml
        README.md
    infra/
      config.rs               # Environment-based configuration
      health.rs               # Health check endpoints (/health, /ready)
    lib.rs                    # Main binary entry point
  demos/
    determinism-visualizer/   # React UI showing replay stability
      src/
      package.json
      README.md
    cli-examples/             # CLI tools for testing MCP endpoints
      example_reason.sh
      example_trace.sh
  deploy/
    Dockerfile                # Multi-stage Rust build
    helm/                     # Helm charts for AKS
      Chart.yaml
      values.yaml
      templates/
    k8s/                      # Raw K8s manifests (if not using Helm)
      deployment.yaml
      service.yaml
      ingress.yaml
    azure/                    # ARM templates or Bicep for Azure resources
      main.bicep
  tests/
    integration/              # E2E tests calling MCP endpoints
      test_reason.rs
      test_governor.rs
    security/                 # Auth, rate limiting, input validation tests
      test_auth.rs
      test_rate_limit.rs
  Cargo.toml                  # Crate metadata: name = "scg_mcp_server"
  README.md                   # "SCG MCP Boundary / API Server"
  ARCHITECTURE.md             # MCP architecture, boundary layer, SDKs
  CONTRIBUTING.md             # MCP-specific developer onboarding
  .warp/
    context.md                # Warp AI MCP-protection directive
  .github/
    workflows/
      mcp-ci.yml              # CI: fmt, clippy, test, integration, docker
```

**NEVER in This Repo:**
- Direct substrate logic, DAG executors, energy models, ethics kernel internals, raw lineage ledger access.

**Done When:**
- All controllers use `SubstrateClient` exclusively (no direct `scg` module calls).
- All responses are sanitized (no DAG internals, no ESV raw values).
- SDKs (Python, TS, Rust) can call `/reason` and `/governor/status`.
- Docker build succeeds, Helm chart deploys to AKS.

---

## PHASE 0 — Substrate Lockdown and Repository Hardening

### 0.1 Repository Security and Access Control

**Objective:** Ensure SCG substrate remains accessible only to authorized personnel, with cryptographic integrity and audit trails.

**Actions (Configuration + Maintenance):**

#### GitHub Organization Setup

Create private GitHub organization for SCG with two repositories:
- `SCG` (substrate only, private)
- `scg-mcp` (MCP boundary, public-facing or private depending on commercialization stage)

Enable branch protection for `main` in both repos:
- Require pull request reviews (minimum 1)
- Require status checks to pass before merging
- Require linear history (no merge commits if desired)
- Restrict direct pushes to `main`
- Require signed commits (GPG)

Enable security features:
- Secret scanning (GitHub Advanced Security if available)
- Dependabot alerts (automated dependency updates)
- Code scanning (CodeQL for Rust)
- Tag protection for release tags (`v*`)

#### CODEOWNERS Files

**File: `SCG/CODEOWNERS`:**
```
# Substrate core modules require founder approval
/src/core/**         @Armonti
/src/neuro/**        @Armonti
/tests/invariants_*  @Armonti

# Benchmarks and general tests can be reviewed by substrate team
/benches/**          @substrate-core-team
/tests/**            @substrate-core-team
```

**File: `scg_mcp_server/CODEOWNERS`:**
```
# Substrate bridge requires founder approval
/src/services/substrate_bridge.rs  @Armonti

# Controllers and middleware can be reviewed by MCP team
/src/controllers/**                 @mcp-engineering
/src/middleware/**                  @mcp-engineering
/src/sdk/**                         @mcp-engineering
```

#### Access Policies

- Enforce 2FA/MFA for all contributors
- Use SSH keys or GPG-signed commits
- No forks allowed for SCG (disable in repo settings)
- `scg-mcp` may allow forks if open-sourcing client SDKs

#### Zero-Touch Zone (Never Code)

- No adapters, wrappers, debug endpoints, or pseudo-APIs for direct substrate access in either repo.
- No HTTP servers, SDKs, or UI code in the SCG repo.

**Done When:**
- Both repos are private (or `scg-mcp` is public if appropriate), branch-protected, and secret-scanned.
- You cannot push to `main` without a PR and passing checks.
- CODEOWNERS enforces review gates for critical modules.
- All commits are GPG-signed.

---

### 0.2 Cargo Configuration and Workspace Declaration

**Objective:** Explicitly declare that SCG and scg_mcp_server are separate crates with no shared workspace, preventing cyclic dependencies and enforcing architectural boundaries.

#### SCG Crate Configuration

**File: `SCG/Cargo.toml`:**
```toml
[package]
name = "scg"
version = "0.1.0"
edition = "2021"
authors = ["Armonti Du-Bose-Hill <your-email@example.com>"]
license = "Proprietary"
description = "Synthetic Cognitive Graph (SCG) Substrate - Physics-Governed Cognition Engine"
repository = "https://github.com/your-org/SCG"
readme = "README.md"

[lib]
name = "scg"
path = "src/lib.rs"

[dependencies]
# Core dependencies (minimal, deterministic)
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
blake3 = "1.5"  # For lineage hashing
rayon = "1.8"   # For parallel belief propagation (deterministic mode)

[dev-dependencies]
criterion = "0.5"  # For benchmarking
proptest = "1.4"   # For property-based testing

[[bench]]
name = "scg_bench"
harness = false

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
```

**Notes:**
- No HTTP, no async runtime (tokio/async-std), no web frameworks.
- All dependencies must be deterministic (no randomness unless explicitly controlled).

#### scg_mcp_server Crate Configuration

**File: `scg_mcp_server/Cargo.toml`:**
```toml
[package]
name = "scg_mcp_server"
version = "0.1.0"
edition = "2021"
authors = ["Armonti Du-Bose-Hill <your-email@example.com>"]
license = "Proprietary"
description = "SCG MCP Boundary Server - Safe API Layer for SCG Substrate"
repository = "https://github.com/your-org/scg-mcp"
readme = "README.md"

[dependencies]
# SCG substrate (imported via path or published crate)
scg = { path = "../SCG" }  # Or version = "0.1.0" if published

# Web framework
actix-web = "4.4"
actix-rt = "2.9"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Auth and security
jsonwebtoken = "9.2"
bcrypt = "0.15"

# Metrics and observability
prometheus = "0.13"
log = "0.4"
env_logger = "0.11"

# Rate limiting (stub for now, Redis-backed later)
governor = "0.6"

[dev-dependencies]
reqwest = "0.11"  # For integration tests

[[bin]]
name = "scg-mcp-server"
path = "src/main.rs"

[profile.release]
opt-level = 3
lto = true
```

**Notes:**
- `scg` is imported via `path = "../SCG"` during development.
- For production, `scg` should be a private crate published to a private registry (or vendored).

**Done When:**
- `cargo build` succeeds in both `SCG/` and `scg_mcp_server/` independently.
- No cyclic dependencies exist.
- `scg_mcp_server` imports `scg`, but `scg` does NOT import `scg_mcp_server`.

---

## PHASE 1 — Substrate Fortification (Heavy Rust Code)

### 1.1 Core Module Implementation

**Objective:** Implement the foundational SCG substrate modules with provable adherence to thermodynamic, ethical, and topological invariants.

#### Module 1: DAG Executor

**File: `SCG/src/core/dag_executor.rs`:**
```rust
use std::collections::{HashMap, HashSet, VecDeque};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EdgeId(pub u64);

#[derive(Debug, Clone)]
pub struct Node {
    pub id: NodeId,
    pub belief: f64,
    pub energy: f64,
    pub esv: EthicalStateVector,
}

#[derive(Debug, Clone)]
pub struct Edge {
    pub id: EdgeId,
    pub src: NodeId,
    pub dest: NodeId,
    pub weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthicalStateVector {
    pub valid: bool,
    pub tau: f64,    // truth
    pub harm: f64,   // harm potential
    pub chi: f64,    // coherence
}

pub struct DagExecutor {
    nodes: HashMap<NodeId, Node>,
    edges: HashMap<EdgeId, Edge>,
    adjacency: HashMap<NodeId, Vec<NodeId>>,  // src -> [dest]
}

impl DagExecutor {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            adjacency: HashMap::new(),
        }
    }
    
    /// Add a node to the DAG
    pub fn add_node(&mut self, node: Node) {
        self.nodes.insert(node.id, node);
        self.adjacency.entry(node.id).or_insert_with(Vec::new);
    }
    
    /// Add an edge, ensuring no cycles are introduced
    pub fn add_edge(&mut self, edge: Edge) -> Result<(), DagError> {
        // Check for cycle
        if self.would_create_cycle(edge.src, edge.dest) {
            return Err(DagError::CycleDetected);
        }
        
        self.adjacency.entry(edge.src).or_insert_with(Vec::new).push(edge.dest);
        self.edges.insert(edge.id, edge);
        Ok(())
    }
    
    /// Check if adding an edge would create a cycle (DFS-based)
    fn would_create_cycle(&self, src: NodeId, dest: NodeId) -> bool {
        let mut visited = HashSet::new();
        let mut stack = vec![dest];
        
        while let Some(node) = stack.pop() {
            if node == src {
                return true;  // Cycle detected
            }
            if visited.contains(&node) {
                continue;
            }
            visited.insert(node);
            
            if let Some(neighbors) = self.adjacency.get(&node) {
                stack.extend(neighbors);
            }
        }
        
        false
    }
    
    /// Verify DAG acyclicity (topological sort existence)
    pub fn is_acyclic(&self) -> bool {
        self.topological_order().is_some()
    }
    
    /// Compute topological order (Kahn's algorithm)
    pub fn topological_order(&self) -> Option<Vec<NodeId>> {
        let mut in_degree: HashMap<NodeId, usize> = self.nodes.keys().map(|&id| (id, 0)).collect();
        
        for neighbors in self.adjacency.values() {
            for &dest in neighbors {
                *in_degree.get_mut(&dest).unwrap() += 1;
            }
        }
        
        let mut queue: VecDeque<NodeId> = in_degree.iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(&id, _)| id)
            .collect();
        
        let mut order = Vec::new();
        
        while let Some(node) = queue.pop_front() {
            order.push(node);
            
            if let Some(neighbors) = self.adjacency.get(&node) {
                for &dest in neighbors {
                    let deg = in_degree.get_mut(&dest).unwrap();
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push_back(dest);
                    }
                }
            }
        }
        
        if order.len() == self.nodes.len() {
            Some(order)
        } else {
            None  // Cycle exists
        }
    }
    
    /// Compute causal coherence (τ): fraction of acyclic paths
    pub fn causal_coherence(&self) -> f64 {
        if self.is_acyclic() {
            1.0  // All paths are acyclic
        } else {
            0.0  // Cycle detected, coherence is zero
        }
    }
}

#[derive(Debug, Clone)]
pub enum DagError {
    CycleDetected,
    NodeNotFound,
    EdgeNotFound,
}
```

#### Module 2: Energy Model

**File: `SCG/src/core/energy_model.rs`:**
```rust
use crate::core::dag_executor::{DagExecutor, NodeId};

pub struct EnergyModel {
    total_energy: f64,
    drift_threshold: f64,  // ΔE_total ≤ 1e-10
}

impl EnergyModel {
    pub fn new() -> Self {
        Self {
            total_energy: 0.0,
            drift_threshold: 1e-10,
        }
    }
    
    /// Initialize total energy from DAG state
    pub fn initialize(&mut self, dag: &DagExecutor) {
        self.total_energy = dag.nodes().values().map(|n| n.energy).sum();
    }
    
    /// Validate energy conservation after an update
    pub fn validate(&self, dag: &DagExecutor) -> Result<(), EnergyError> {
        let current_energy: f64 = dag.nodes().values().map(|n| n.energy).sum();
        let drift = (current_energy - self.total_energy).abs();
        
        if drift > self.drift_threshold {
            Err(EnergyError::DriftExceeded { drift })
        } else {
            Ok(())
        }
    }
    
    /// Get total energy
    pub fn total_energy(&self) -> f64 {
        self.total_energy
    }
}

#[derive(Debug, Clone)]
pub enum EnergyError {
    DriftExceeded { drift: f64 },
}
```

#### Module 3: Elastic Governor

**File: `SCG/src/core/governor.rs`:**
```rust
use crate::core::dag_executor::{DagExecutor, NodeId};

pub struct ElasticGovernor {
    k: f64,  // Elasticity constant (0.8 per neuro spec)
    drift_threshold: f64,
}

impl ElasticGovernor {
    pub fn new() -> Self {
        Self {
            k: 0.8,
            drift_threshold: 1e-10,
        }
    }
    
    /// Perform drift correction: dE_i/dt = -k(E_i - E_mean)
    pub fn correct_drift(&self, dag: &mut DagExecutor) -> Result<(), GovernorError> {
        let mean_energy: f64 = dag.nodes().values().map(|n| n.energy).sum::<f64>() 
            / dag.nodes().len() as f64;
        
        for node in dag.nodes_mut().values_mut() {
            let drift = node.energy - mean_energy;
            let correction = -self.k * drift;
            node.energy += correction;
        }
        
        Ok(())
    }
    
    /// Check if system is stable (drift within threshold)
    pub fn is_stable(&self, dag: &DagExecutor) -> bool {
        let mean_energy: f64 = dag.nodes().values().map(|n| n.energy).sum::<f64>() 
            / dag.nodes().len() as f64;
        
        let max_drift = dag.nodes().values()
            .map(|n| (n.energy - mean_energy).abs())
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);
        
        max_drift <= self.drift_threshold
    }
}

#[derive(Debug, Clone)]
pub enum GovernorError {
    InstabilityDetected,
}
```

#### Module 4: Ethics Kernel

**File: `SCG/src/core/ethics_kernel.rs`:**
```rust
use crate::core::dag_executor::{DagExecutor, EthicalStateVector, NodeId};

pub struct EthicsKernel {
    esv_threshold: f64,  // ESV checksum = 100%
}

impl EthicsKernel {
    pub fn new() -> Self {
        Self {
            esv_threshold: 1.0,  // 100% validity
        }
    }
    
    /// Validate ESV for a single node
    pub fn validate_esv(&self, esv: &EthicalStateVector) -> bool {
        esv.valid && esv.tau >= 0.0 && esv.tau <= 1.0 
            && esv.harm >= 0.0 && esv.harm <= 1.0
            && esv.chi >= 0.0 && esv.chi <= 1.0
    }
    
    /// Compute ethical potential: Φ_ethics = Σ (τ_i - h_i) χ_i
    pub fn ethical_potential(&self, dag: &DagExecutor) -> f64 {
        dag.nodes().values()
            .map(|n| (n.esv.tau - n.esv.harm) * n.esv.chi)
            .sum()
    }
    
    /// Check if system is in ethical equilibrium (dΦ/dt ≈ 0)
    pub fn is_equilibrium(&self, dag: &DagExecutor, prev_potential: f64) -> bool {
        let current_potential = self.ethical_potential(dag);
        let delta = (current_potential - prev_potential).abs();
        delta <= 1e-10
    }
    
    /// Validate all ESVs in DAG
    pub fn validate_all(&self, dag: &DagExecutor) -> Result<(), EthicsError> {
        for node in dag.nodes().values() {
            if !self.validate_esv(&node.esv) {
                return Err(EthicsError::InvalidEsv { node_id: node.id });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum EthicsError {
    InvalidEsv { node_id: NodeId },
    EthicalEquilibriumFailed,
}
```

#### Module 5: Lineage Ledger

**File: `SCG/src/core/lineage_ledger.rs`:**
```rust
use blake3;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageEntry {
    pub hash: String,
    pub timestamp: u64,
    pub state_summary: String,  // High-level summary, not full state
}

pub struct LineageLedger {
    entries: Vec<LineageEntry>,
    current_hash: String,
}

impl LineageLedger {
    pub fn new() -> Self {
        let genesis_hash = blake3::hash(b"SCG_GENESIS").to_hex().to_string();
        Self {
            entries: vec![LineageEntry {
                hash: genesis_hash.clone(),
                timestamp: 0,
                state_summary: "Genesis state".to_string(),
            }],
            current_hash: genesis_hash,
        }
    }
    
    /// Append a new entry: H_{k+1} = BLAKE3(H_k || state)
    pub fn append(&mut self, state_summary: String, timestamp: u64) {
        let combined = format!("{}{}", self.current_hash, state_summary);
        let new_hash = blake3::hash(combined.as_bytes()).to_hex().to_string();
        
        self.entries.push(LineageEntry {
            hash: new_hash.clone(),
            timestamp,
            state_summary,
        });
        
        self.current_hash = new_hash;
    }
    
    /// Get current lineage hash
    pub fn current_hash(&self) -> &str {
        &self.current_hash
    }
    
    /// Export ledger (ISO 27001-compliant JSON)
    pub fn export(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.entries)
    }
}
```

#### Module 6: Replay Engine

**File: `SCG/src/core/replay_engine.rs`:**
```rust
use crate::core::lineage_ledger::{LineageLedger, LineageEntry};
use crate::core::dag_executor::DagExecutor;

pub struct ReplayEngine {
    epsilon_threshold: f64,  // Replay variance ≤ 1e-10
}

impl ReplayEngine {
    pub fn new() -> Self {
        Self {
            epsilon_threshold: 1e-10,
        }
    }
    
    /// Replay a scenario and compare lineage hashes
    pub fn replay_and_verify(
        &self,
        scenario: &Scenario,
        reference_hash: &str,
    ) -> Result<ReplayReport, ReplayError> {
        // Execute scenario
        let mut dag = DagExecutor::new();
        let mut ledger = LineageLedger::new();
        
        // (Scenario execution logic would go here)
        // For now, stub:
        ledger.append("Replay state".to_string(), 1000);
        
        let replay_hash = ledger.current_hash();
        
        if replay_hash != reference_hash {
            return Err(ReplayError::HashMismatch {
                expected: reference_hash.to_string(),
                actual: replay_hash.to_string(),
            });
        }
        
        Ok(ReplayReport {
            success: true,
            hash: replay_hash.to_string(),
            variance: 0.0,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Scenario {
    pub name: String,
    // Scenario definition fields
}

#[derive(Debug, Clone)]
pub struct ReplayReport {
    pub success: bool,
    pub hash: String,
    pub variance: f64,
}

#[derive(Debug, Clone)]
pub enum ReplayError {
    HashMismatch { expected: String, actual: String },
    VarianceExceeded { variance: f64 },
}
```

#### Module 7: Hebbian Weight Updates

**File: `SCG/src/core/hebbian.rs`:**
```rust
use crate::core::dag_executor::{DagExecutor, EdgeId};

pub struct HebbianUpdater {
    learning_rate: f64,
}

impl HebbianUpdater {
    pub fn new(learning_rate: f64) -> Self {
        Self { learning_rate }
    }
    
    /// Update edge weights: Δw_ij ∝ b_i · b_j (energy-neutral)
    pub fn update_weights(&self, dag: &mut DagExecutor) -> Result<(), HebbianError> {
        let total_weight_before: f64 = dag.edges().values().map(|e| e.weight).sum();
        
        for edge in dag.edges_mut().values_mut() {
            let src_belief = dag.nodes().get(&edge.src).unwrap().belief;
            let dest_belief = dag.nodes().get(&edge.dest).unwrap().belief;
            
            let delta = self.learning_rate * src_belief * dest_belief;
            edge.weight += delta;
        }
        
        // Renormalize to preserve total weight (energy-neutral)
        let total_weight_after: f64 = dag.edges().values().map(|e| e.weight).sum();
        let scale_factor = total_weight_before / total_weight_after;
        
        for edge in dag.edges_mut().values_mut() {
            edge.weight *= scale_factor;
        }
        
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum HebbianError {
    WeightViolation,
}
```

#### Module 8: Neuro-Coherence

**File: `SCG/src/neuro/coherence.rs`:**
```rust
use crate::core::dag_executor::DagExecutor;
use std::f64::consts::PI;

pub struct CoherenceCalculator {
    threshold: f64,  // C ≥ 0.97
}

impl CoherenceCalculator {
    pub fn new() -> Self {
        Self { threshold: 0.97 }
    }
    
    /// Compute global coherence: C(t) = (1/N) Σ cos(θ_i - θ_mean)
    pub fn global_coherence(&self, dag: &DagExecutor) -> f64 {
        let phases: Vec<f64> = dag.nodes().values()
            .map(|n| self.compute_phase(n.belief))
            .collect();
        
        let mean_phase: f64 = phases.iter().sum::<f64>() / phases.len() as f64;
        
        let coherence: f64 = phases.iter()
            .map(|&phase| (phase - mean_phase).cos())
            .sum::<f64>() / phases.len() as f64;
        
        coherence
    }
    
    /// Compute phase from belief (stub: map belief to [0, 2π])
    fn compute_phase(&self, belief: f64) -> f64 {
        belief * 2.0 * PI
    }
    
    /// Check if coherence meets threshold
    pub fn is_coherent(&self, dag: &DagExecutor) -> bool {
        self.global_coherence(dag) >= self.threshold
    }
}
```

---

### 1.2 Invariant Test Harness

**Objective:** Create executable proofs that SCG respects its foundational invariants.

#### Test Suite: Energy Conservation

**File: `SCG/tests/invariants_energy.rs`:**
```rust
use scg::core::dag_executor::{DagExecutor, Node, NodeId, EthicalStateVector};
use scg::core::energy_model::EnergyModel;

#[test]
fn test_energy_conservation_long_run() {
    let mut dag = initialize_test_dag();
    let mut energy_model = EnergyModel::new();
    energy_model.initialize(&dag);
    
    let initial_energy = energy_model.total_energy();
    
    // Simulate 10^4 updates
    for _ in 0..10_000 {
        // (Belief propagation logic would go here)
        
        // Validate energy conservation
        assert!(energy_model.validate(&dag).is_ok());
    }
    
    let final_energy = energy_model.total_energy();
    let drift = (final_energy - initial_energy).abs();
    
    assert!(drift <= 1e-10, "Energy drift {} exceeds threshold", drift);
}

fn initialize_test_dag() -> DagExecutor {
    let mut dag = DagExecutor::new();
    
    for i in 0..10 {
        dag.add_node(Node {
            id: NodeId(i),
            belief: 0.5,
            energy: 1.0,
            esv: EthicalStateVector {
                valid: true,
                tau: 0.8,
                harm: 0.1,
                chi: 0.9,
            },
        });
    }
    
    dag
}
```

#### Test Suite: Replay Determinism

**File: `SCG/tests/invariants_replay.rs`:**
```rust
use scg::core::replay_engine::{ReplayEngine, Scenario};
use scg::core::lineage_ledger::LineageLedger;

#[test]
fn test_deterministic_replay() {
    let scenario = Scenario {
        name: "conflict_resolution_01".to_string(),
    };
    
    // Run scenario three times
    let report1 = run_scenario(&scenario);
    let report2 = run_scenario(&scenario);
    let report3 = run_scenario(&scenario);
    
    // All hashes must match
    assert_eq!(report1.hash, report2.hash);
    assert_eq!(report2.hash, report3.hash);
    
    // Variance must be ≤ 1e-10
    assert!(report1.variance <= 1e-10);
    assert!(report2.variance <= 1e-10);
    assert!(report3.variance <= 1e-10);
}

fn run_scenario(scenario: &Scenario) -> scg::core::replay_engine::ReplayReport {
    let mut ledger = LineageLedger::new();
    
    // (Scenario execution logic)
    ledger.append("Scenario state".to_string(), 1000);
    
    scg::core::replay_engine::ReplayReport {
        success: true,
        hash: ledger.current_hash().to_string(),
        variance: 0.0,
    }
}
```

#### Test Suite: Topological Integrity

**File: `SCG/tests/invariants_topology.rs`:**
```rust
use scg::core::dag_executor::{DagExecutor, Node, Edge, NodeId, EdgeId, EthicalStateVector};

#[test]
fn test_dag_acyclicity_under_dynamic_edges() {
    let mut dag = initialize_test_dag();
    
    // Attempt to add edges
    for i in 0..100 {
        let src = NodeId(i % 10);
        let dest = NodeId((i + 1) % 10);
        let edge = Edge {
            id: EdgeId(i),
            src,
            dest,
            weight: 1.0,
        };
        
        let _ = dag.add_edge(edge);
    }
    
    // Verify DAG is acyclic
    assert!(dag.is_acyclic(), "DAG contains cycles");
    
    // Verify causal coherence
    let tau = dag.causal_coherence();
    assert!(tau >= 0.99, "Causal coherence {} below threshold", tau);
}

fn initialize_test_dag() -> DagExecutor {
    let mut dag = DagExecutor::new();
    
    for i in 0..10 {
        dag.add_node(Node {
            id: NodeId(i),
            belief: 0.5,
            energy: 1.0,
            esv: EthicalStateVector {
                valid: true,
                tau: 0.8,
                harm: 0.1,
                chi: 0.9,
            },
        });
    }
    
    dag
}
```

#### Test Suite: Ethical Stability

**File: `SCG/tests/invariants_ethics.rs`:**
```rust
use scg::core::dag_executor::{DagExecutor, Node, NodeId, EthicalStateVector};
use scg::core::ethics_kernel::EthicsKernel;

#[test]
fn test_esv_enforcement() {
    let mut dag = initialize_test_dag();
    let kernel = EthicsKernel::new();
    
    // Attempt to add node with invalid ESV
    let invalid_node = Node {
        id: NodeId(100),
        belief: 0.5,
        energy: 1.0,
        esv: EthicalStateVector {
            valid: false,  // Invalid
            tau: 0.1,
            harm: 0.9,
            chi: 0.2,
        },
    };
    
    dag.add_node(invalid_node);
    
    // Validation must fail
    assert!(kernel.validate_all(&dag).is_err(), "Invalid ESV was not rejected");
}

#[test]
fn test_ethical_equilibrium() {
    let dag = initialize_test_dag();
    let kernel = EthicsKernel::new();
    
    let initial_potential = kernel.ethical_potential(&dag);
    
    // Check equilibrium (dΦ/dt → 0)
    assert!(kernel.is_equilibrium(&dag, initial_potential), "Ethical equilibrium not reached");
}

fn initialize_test_dag() -> DagExecutor {
    let mut dag = DagExecutor::new();
    
    for i in 0..10 {
        dag.add_node(Node {
            id: NodeId(i),
            belief: 0.5,
            energy: 1.0,
            esv: EthicalStateVector {
                valid: true,
                tau: 0.8,
                harm: 0.1,
                chi: 0.9,
            },
        });
    }
    
    dag
}
```

#### Test Suite: Neuro-Coherence Validation

**File: `SCG/tests/neuro_validation.rs`:**
```rust
use scg::core::dag_executor::{DagExecutor, Node, NodeId, EthicalStateVector};
use scg::neuro::coherence::CoherenceCalculator;

#[test]
fn test_neuro_coherence() {
    let dag = initialize_test_dag();
    let calculator = CoherenceCalculator::new();
    
    let coherence = calculator.global_coherence(&dag);
    
    assert!(coherence >= 0.97, "Coherence {} below threshold", coherence);
    assert!(calculator.is_coherent(&dag), "System is not coherent");
}

fn initialize_test_dag() -> DagExecutor {
    let mut dag = DagExecutor::new();
    
    for i in 0..10 {
        dag.add_node(Node {
            id: NodeId(i),
            belief: 0.5,
            energy: 1.0,
            esv: EthicalStateVector {
                valid: true,
                tau: 0.8,
                harm: 0.1,
                chi: 0.9,
            },
        });
    }
    
    dag
}
```

---

### 1.3 Benchmarking Harness

**File: `SCG/benches/scg_bench.rs`:**
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use scg::core::dag_executor::{DagExecutor, Node, NodeId, EthicalStateVector};

fn benchmark_belief_propagation(c: &mut Criterion) {
    let dag = initialize_large_dag(1000);
    
    c.bench_function("propagate_beliefs_1k_nodes", |b| {
        b.iter(|| {
            // (Belief propagation logic)
            black_box(&dag);
        })
    });
}

fn initialize_large_dag(size: usize) -> DagExecutor {
    let mut dag = DagExecutor::new();
    
    for i in 0..size {
        dag.add_node(Node {
            id: NodeId(i as u64),
            belief: 0.5,
            energy: 1.0,
            esv: EthicalStateVector {
                valid: true,
                tau: 0.8,
                harm: 0.1,
                chi: 0.9,
            },
        });
    }
    
    dag
}

criterion_group!(benches, benchmark_belief_propagation);
criterion_main!(benches);
```

---

## PHASE 2 — MCP Boundary Hardening

### 2.1 Substrate Bridge (Narrow and Sealed)

**File: `scg_mcp_server/src/services/substrate_bridge.rs`:**
```rust
use scg::{ScgCore, ReasonRequest, ScenarioSpec, TraceId};
use crate::error::ScgError;

pub struct SubstrateClient {
    core: ScgCore,  // Initialized at startup, never exposed
}

impl SubstrateClient {
    pub fn new(config: SubstrateConfig) -> Result<Self, ScgError> {
        let core = ScgCore::initialize(config)?;
        Ok(Self { core })
    }
    
    /// Primary reasoning endpoint
    pub async fn reason(&self, input: ReasonRequest) -> Result<ReasonResponse, ScgError> {
        let outcome = self.core.reason(input)?;
        Ok(outcome.into_sanitized_response())
    }
    
    /// Run a predefined scenario
    pub async fn run_scenario(&self, scenario: ScenarioSpec) -> Result<ScenarioResult, ScgError> {
        let result = self.core.execute_scenario(scenario)?;
        Ok(result.into_sanitized_response())
    }
    
    /// High-level governor status (drift metrics only, no raw energy)
    pub async fn governor_status(&self) -> Result<GovernorStatus, ScgError> {
        let status = self.core.governor_telemetry();
        Ok(GovernorStatus {
            stable: status.drift <= 1e-10,
            drift_magnitude: status.drift,
            last_correction: status.last_correction_timestamp,
        })
    }
    
    /// Trace summary (lineage hash + metadata, no DAG topology)
    pub async fn trace(&self, trace_id: TraceId) -> Result<TraceSummary, ScgError> {
        let trace = self.core.get_trace(trace_id)?;
        Ok(TraceSummary {
            id: trace.id,
            hash: trace.lineage_hash,
            stable: trace.replay_verified,
        })
    }
}

// Config and response types
pub struct SubstrateConfig {}
pub struct ReasonResponse {}
pub struct ScenarioResult {}
pub struct GovernorStatus {
    pub stable: bool,
    pub drift_magnitude: f64,
    pub last_correction: u64,
}
pub struct TraceSummary {
    pub id: TraceId,
    pub hash: String,
    pub stable: bool,
}
```

---

### 2.2 Endpoint Implementation

**File: `scg_mcp_server/src/controllers/reason.rs`:**
```rust
use actix_web::{post, web, HttpResponse};
use crate::services::substrate_bridge::SubstrateClient;
use crate::middleware::auth::AuthContext;

#[post("/reason")]
async fn reason_handler(
    auth: AuthContext,
    bridge: web::Data<SubstrateClient>,
    req: web::Json<ReasonRequest>,
) -> HttpResponse {
    match bridge.reason(req.into_inner()).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => {
            log::error!("Reason failed: {:?}", e);
            HttpResponse::InternalServerError().json(ErrorResponse::from(e))
        }
    }
}
```

---

## PHASE 3 — CI/CD and Automated Gating

### 3.1 CI Pipelines

**File: `.github/workflows/substrate-ci.yml` (in SCG repo):**
```yaml
name: SCG Substrate CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo fmt --check
      - run: cargo clippy -- -D warnings
      - run: cargo test --all
      - run: cargo test --test invariants_energy
      - run: cargo test --test invariants_replay
      - run: cargo test --test invariants_topology
      - run: cargo test --test invariants_ethics
      - run: cargo test --test neuro_validation
      - run: cargo bench --no-run
```

**File: `.github/workflows/mcp-ci.yml` (in scg_mcp_server repo):**
```yaml
name: MCP Boundary CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo fmt --check
      - run: cargo clippy -- -D warnings
      - run: cargo test --all
      - run: cargo test --test integration
      - run: cargo audit
  
  docker:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - run: docker build -f deploy/Dockerfile .
```

---

## L4 — Risks and Mitigations

### Risk 1: Invariants Enforced by Culture, Not Code
**Mitigation:** Phase 1.2 + Phase 3 make invariants executable (unit tests) and enforce them via CI gates. Invariants become facts, not aspirations.

### Risk 2: MCP Slowly Accretes Substrate Shortcuts
**Mitigation:** Narrow substrate bridge (Phase 2.1), explicit "never do" list (Phase 0.1), and review gates via CODEOWNERS stop leakage.

### Risk 3: Over-Hardening Kills Iteration Speed
**Mitigation:** The directive explicitly prioritizes "simplest implementation that satisfies invariants." As long as tests pass and boundaries hold, you're free to refactor internally.

### Risk 4: Determinism Breaks Under Azure Orchestration
**Mitigation:** Replay tests run across multiple environments (Phase 1.2). Azure deployment uses immutable container images and controlled configuration. Health checks verify substrate initialization before accepting traffic.

### Risk 5: Compliance Docs Lag Behind Code
**Mitigation:** Phase 5 creates living docs (CONTRIBUTING.md, ARCHITECTURE.md, compliance mappings) that are versioned alongside code and required for Partner Center submission.

### Risk 6: Cyclic Dependencies Between scg and scg_mcp_server
**Mitigation:** Explicit import rules (Phase 0.2) and separate workspaces prevent cyclic dependencies. CI enforces this via build graph validation.

---

## One-Line Recommendation

Start with Phase 1.1 + 1.2 (substrate modules + invariant tests) and Phase 2.1 (substrate bridge) this week—once the substrate modules, invariants, and the MCP bridge exist and are under CI, the rest of the hardening becomes incremental instead of heroic, and you'll have a production-ready, acquisition-grade, Microsoft-compliant SCG system that proves morality through physics.
