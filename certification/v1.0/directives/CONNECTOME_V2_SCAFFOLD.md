# Connectome v2 Architecture Scaffold

**Phase**: Post-Substrate (Actions 4+5 from Advancement Roadmap)  
**Substrate Version**: v1.0.0-substrate (frozen)  
**Connectome Version**: v2.0.0-alpha (development)  
**Status**: SCAFFOLD — Implementation begins after SCG-PILOT-01 certification

---

## Executive Summary

The **Connectome v2** represents the second layer of SCG-MCP architecture, built atop the frozen substrate boundary. It provides advanced cognitive capabilities (attention, memory, reasoning) while maintaining complete isolation from substrate internals.

**Key Principles**:
1. **Zero Substrate Coupling**: Connectome modules interact with substrate only through defined MCP protocol
2. **Modular Design**: Each cognitive capability is an independent, testable module
3. **Pluggable Architecture**: Modules can be added/removed without substrate modification
4. **Ethical Governance**: All modules enforce Apex Directive compliance
5. **Observable Behavior**: Full telemetry integration for connectome-layer operations

---

## 1. Architectural Overview

```
┌─────────────────────────────────────────────────────────────┐
│                  CONNECTOME v2 Layer                        │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐       │
│  │  Attention   │ │   Memory     │ │  Reasoning   │       │
│  │   Module     │ │   Module     │ │   Module     │  ...  │
│  └──────────────┘ └──────────────┘ └──────────────┘       │
│         │                 │                 │               │
│         └─────────────────┴─────────────────┘               │
│                          │                                   │
│                 ┌────────▼────────┐                         │
│                 │  MCP Protocol   │                         │
│                 │   (Public API)  │                         │
│                 └────────┬────────┘                         │
└──────────────────────────┼──────────────────────────────────┘
                           │
┌──────────────────────────▼──────────────────────────────────┐
│                SUBSTRATE v1.0.0 (FROZEN)                    │
│  Core Runtime | Fault Domain | Telemetry | Lineage         │
│  Immutable — No connectome coupling allowed]               │
└─────────────────────────────────────────────────────────────┘
```

---

## 2. Directory Structure

```
scg_mcp_server/
├── src/
│   ├── scg_core.rs           FROZEN — substrate]
│   ├── types.rs              FROZEN — substrate]
│   ├── mcp_handler.rs        FROZEN — substrate]
│   ├── lib.rs                FROZEN — substrate]
│   ├── main.rs               FROZEN — substrate]
│   ├── fault/                FROZEN — substrate]
│   ├── telemetry/            FROZEN — substrate]
│   ├── lineage/              FROZEN — substrate]
│   │
│   └── connectome/           NEW — isolated layer]
│       ├── mod.rs            Connectome orchestrator]
│       ├── protocol.rs       MCP client interface]
│       ├── attention/        Attention mechanism module]
│       ├── memory/           Memory management module]
│       ├── reasoning/        Reasoning engine module]
│       └── governance/       Ethical compliance module]
│
├── tests/
│   ├── hardening_fuzz.rs           FROZEN — substrate tests]
│   ├── hardening_concurrency.rs    FROZEN — substrate tests]
│   ├── integration_validation.rs   FROZEN — substrate tests]
│   │
│   └── connectome_tests.rs         NEW — connectome tests]
│
├── SUBSTRATE_FREEZE.md       FROZEN — substrate documentation]
├── APEX_CLARIFICATIONS.md    FROZEN — substrate documentation]
├── CERTIFICATION_DOSSIER.md  NEW — pilot certification]
└── CONNECTOME_V2_SCAFFOLD.md THIS FILE — architecture guide]
```

---

## 3. Module Specifications

### 3.1 Protocol Module (`src/connectome/protocol.rs`)

**Purpose**: Provides safe, isolated interface for connectome modules to interact with substrate via MCP protocol.

**Public API**:
```rust
pub struct McpClient {
    // MCP JSON-RPC client (communicates over stdio/HTTP)
}

impl McpClient {
    pub async fn create_node(&self, params: CreateNodeParams) -> Result<NodeId, McpError>;
    pub async fn bind_edge(&self, params: BindEdgeParams) -> Result<EdgeId, McpError>;
    pub async fn mutate_node(&self, params: MutateParams) -> Result<(), McpError>;
    pub async fn query_lineage(&self, params: LineageParams) -> Result<LineageSnapshot, McpError>;
    pub async fn get_telemetry(&self, params: TelemetryParams) -> Result<TelemetryRecord, McpError>;
    // ... all MCP tools exposed as safe async methods
}
```

**Isolation Contract**:
- ZERO direct imports from `src/scg_core.rs`, `src/types.rs`, or any substrate module
- All substrate interaction via MCP JSON-RPC protocol
- Substrate violations trigger connectome-layer alerts (not substrate quarantine)
- Independent versioning: connectome v2.x can work with substrate v1.0.x

---

### 3.2 Attention Module (`src/connectome/attention/mod.rs`)

**Purpose**: Implements selective attention mechanisms for prioritizing graph operations based on cognitive relevance.

**Capabilities**:
- **Salience Scoring**: Compute attention weights for nodes/edges based on ESV, energy, and coherence
- **Focus Management**: Maintain attention window over graph subset
- **Priority Queuing**: Order operations by attention score
- **Drift Detection**: Alert when attention diverges from ESV-optimal path

**Example Interface**:
```rust
pub struct AttentionModule {
    mcp_client: McpClient,
    attention_window: Vec<NodeId>,
    salience_threshold: f64,
}

impl AttentionModule {
    pub async fn compute_salience(&self, node_id: NodeId) -> Result<f64, AttentionError>;
    pub async fn update_focus(&mut self) -> Result<Vec<NodeId>, AttentionError>;
    pub async fn prioritize_operations(&self, ops: Vec<Operation>) -> Vec<Operation>;
}
```

**Tests**:
- Salience converges to ESV-optimal regions
- Focus window respects coherence thresholds
- Priority queue maintains substrate invariants

---

### 3.3 Memory Module (`src/connectome/memory/mod.rs`)

**Purpose**: Provides episodic and semantic memory capabilities over graph state.

**Capabilities**:
- **Episodic Storage**: Cache lineage snapshots for recall
- **Semantic Indexing**: Build concept hierarchies from graph topology
- **Recall Mechanism**: Retrieve past graph states by semantic query
- **Consolidation**: Compress old episodes into summary shards

**Example Interface**:
```rust
pub struct MemoryModule {
    mcp_client: McpClient,
    episode_cache: HashMap<EpisodeId, LineageSnapshot>,
    semantic_index: BTreeMap<ConceptId, Vec<NodeId>>,
}

impl MemoryModule {
    pub async fn store_episode(&mut self, snapshot: LineageSnapshot) -> Result<EpisodeId, MemoryError>;
    pub async fn recall_by_concept(&self, concept: &str) -> Result<Vec<NodeId>, MemoryError>;
    pub async fn consolidate_old_episodes(&mut self, threshold: Duration) -> Result<(), MemoryError>;
}
```

**Tests**:
- Episode storage preserves lineage integrity
- Recall retrieves correct snapshots
- Consolidation maintains semantic accuracy

---

### 3.4 Reasoning Module (`src/connectome/reasoning/mod.rs`)

**Purpose**: Implements high-level reasoning over graph structure.

**Capabilities**:
- **Path Planning**: Find ESV-optimal paths between nodes
- **Causal Inference**: Detect causal relationships in graph topology
- **Counterfactual Simulation**: Simulate "what if" scenarios without mutating substrate
- **Contradiction Detection**: Identify logical inconsistencies in graph state

**Example Interface**:
```rust
pub struct ReasoningModule {
    mcp_client: McpClient,
    inference_depth: usize,
}

impl ReasoningModule {
    pub async fn find_optimal_path(&self, from: NodeId, to: NodeId) -> Result<Vec<EdgeId>, ReasoningError>;
    pub async fn infer_causality(&self, cause: NodeId, effect: NodeId) -> Result<f64, ReasoningError>;
    pub async fn simulate_counterfactual(&self, intervention: Operation) -> Result<GraphState, ReasoningError>;
}
```

**Tests**:
- Path planning respects ESV constraints
- Causal inference converges to ground truth
- Counterfactuals never mutate substrate

---

### 3.5 Governance Module (`src/connectome/governance/mod.rs`)

**Purpose**: Enforces Apex Directive ethical constraints at connectome layer.

**Capabilities**:
- **Ethical Screening**: Validate operations against Apex Directive before submission
- **Bias Detection**: Monitor for discriminatory patterns in attention/memory
- **Transparency Logging**: Emit audit trail for all connectome decisions
- **Override Mechanism**: Human-in-the-loop for high-stakes operations

**Example Interface**:
```rust
pub struct GovernanceModule {
    mcp_client: McpClient,
    apex_ruleset: ApexDirective,
    audit_log: Vec<GovernanceEvent>,
}

impl GovernanceModule {
    pub async fn validate_operation(&self, op: &Operation) -> Result<bool, GovernanceError>;
    pub async fn detect_bias(&self, attention_scores: &f64]) -> Result<BiasReport, GovernanceError>;
    pub async fn request_human_approval(&self, op: &Operation) -> Result<bool, GovernanceError>;
}
```

**Tests**:
- Validation blocks unethical operations
- Bias detection triggers alerts
- Human approval integrates correctly

---

## 4. Connectome Orchestrator (`src/connectome/mod.rs`)

**Purpose**: Coordinates all connectome modules and exposes unified API.

**Interface**:
```rust
pub struct ConnectomeOrchestrator {
    mcp_client: McpClient,
    attention: AttentionModule,
    memory: MemoryModule,
    reasoning: ReasoningModule,
    governance: GovernanceModule,
}

impl ConnectomeOrchestrator {
    pub async fn new(mcp_endpoint: &str) -> Result<Self, ConnectomeError>;
    
    pub async fn process_cognitive_task(&mut self, task: CognitiveTask) -> Result<CognitiveOutput, ConnectomeError> {
        // 1. Governance: Validate task
        // 2. Attention: Focus on relevant nodes
        // 3. Memory: Recall relevant episodes
        // 4. Reasoning: Execute cognitive operation
        // 5. Memory: Store result episode
        // 6. Governance: Audit decision
    }
}
```

---

## 5. Testing Strategy

### 5.1 Unit Tests

Each module (attention, memory, reasoning, governance) has independent unit tests:
- Mock MCP client for isolation
- Test cognitive functions without substrate
- Validate module-specific invariants

### 5.2 Integration Tests (`tests/connectome_tests.rs`)

Test full connectome orchestrator against real substrate:
- Spin up substrate in test mode
- Execute cognitive tasks end-to-end
- Verify substrate invariants maintained
- Confirm connectome-layer telemetry

### 5.3 Isolation Tests

Critical tests to prove substrate independence:
- **No Direct Imports Test**: Compile fails if connectome imports substrate internals
- **Protocol-Only Test**: All substrate interaction goes through MCP client
- **Substrate Mutation Test**: Connectome cannot bypass quarantine/rollback

---

## 6. Development Roadmap

### Phase 1: Protocol Foundation (Week 1)
-  ] Implement `src/connectome/protocol.rs` with MCP client
-  ] Write protocol isolation tests
-  ] Verify zero substrate coupling

### Phase 2: Core Modules (Weeks 2-4)
-  ] Implement attention module with unit tests
-  ] Implement memory module with unit tests
-  ] Implement reasoning module with unit tests
-  ] Implement governance module with unit tests

### Phase 3: Orchestration (Week 5)
-  ] Implement connectome orchestrator
-  ] Write end-to-end integration tests
-  ] Performance benchmarking (target: <10ms overhead per cognitive task)

### Phase 4: Validation (Week 6)
-  ] Run connectome against certified substrate (v1.0.0-substrate)
-  ] Verify substrate invariants maintained under cognitive load
-  ] Document connectome API and examples

### Phase 5: Release (Week 7)
-  ] Tag v2.0.0-connectome
-  ] Publish connectome specification
-  ] Update README with connectome usage guide

---

## 7. Integration Example

```rust
// User application code (post-substrate certification)
use scg_mcp_server::connectome::ConnectomeOrchestrator;
use scg_mcp_server::connectome::CognitiveTask;

#tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Substrate runs as separate process (certified v1.0.0)
    // Connectome connects via MCP protocol
    let mut connectome = ConnectomeOrchestrator::new("http://localhost:3000/mcp").await?;
    
    // Define cognitive task
    let task = CognitiveTask::Reasoning {
        query: "Find causal path from node A to node B".to_string(),
        constraints: vec!"ESV ≥ 0.85".to_string()],
    };
    
    // Execute with full attention, memory, reasoning, governance
    let output = connectome.process_cognitive_task(task).await?;
    
    println!("Cognitive output: {:?}", output);
    println!("Substrate invariants: MAINTAINED (verified by governance module)");
    
    Ok(())
}
```

---

## 8. Version Compatibility Matrix

| Connectome Version | Substrate Version | Status |
|--------------------|-------------------|--------|
| v2.0.0-alpha | v1.0.0-substrate | Development |
| v2.0.0 | v1.0.0-substrate | Planned (post-pilot) |
| v2.1.x | v1.0.x-substrate | Future enhancements |
| v3.0.0 | v2.0.0-substrate | Future major revision |

**LTS Strategy**:
- Substrate v1.0.x line: 24-month support (LTS)
- Connectome v2.x line: 12-month support
- Breaking changes require major version bump

---

## 9. Security Considerations

### 9.1 Substrate Protection

- Connectome CANNOT bypass substrate quarantine
- All operations subject to ESV validation
- MCP protocol enforces rate limits
- Telemetry captures all connectome-substrate interactions

### 9.2 Connectome Vulnerabilities

- **Attention Hijacking**: Malicious input biases salience scores → Governance module blocks
- **Memory Poisoning**: Corrupted episodes injected → SHA256 validation fails
- **Reasoning Exploits**: Counterfactuals mutate substrate → Protocol layer rejects
- **Governance Bypass**: Operations skip ethical screening → Orchestrator enforces

### 9.3 Audit Trail

All connectome decisions logged to immutable audit trail:
```json
{
  "timestamp": "2025-01-15T12:34:56Z",
  "module": "reasoning",
  "operation": "find_optimal_path",
  "input": {"from": "node_A", "to": "node_B"},
  "output": {"path": "edge_1", "edge_2"], "esv": 0.92},
  "governance_approval": true,
  "substrate_invariants": "MAINTAINED"
}
```

---

## 10. Documentation Artifacts

Upon connectome v2.0.0 release:
-  ] `CONNECTOME_SPECIFICATION.md` — Formal module specifications
-  ] `CONNECTOME_API.md` — Public API documentation
-  ] `CONNECTOME_EXAMPLES.md` — Usage examples and tutorials
-  ] `CONNECTOME_SECURITY.md` — Security model and threat analysis
-  ] `CONNECTOME_CHANGELOG.md` — Version history

---

## Document Control

**Version**: 1.0.0  
**Status**: SCAFFOLD (Implementation begins after SCG-PILOT-01)  
**Last Updated**: 2025-01-15  
**Owner**: SCG Connectome Team  
**Dependencies**: Substrate v1.0.0-substrate (certified)

---

**END OF CONNECTOME v2 SCAFFOLD**

*Implementation of this architecture begins after successful SCG-PILOT-01 certification.*
