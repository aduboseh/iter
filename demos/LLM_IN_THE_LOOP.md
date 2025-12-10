# SCG LLM-in-the-Loop Demo for Warp

## Overview

This demo shows an LLM reasoning **through** SCG's cognitive physics engine, where:
- LLM proposes → SCG vets → LLM adapts

The goal is to demonstrate that an LLM can only act rationally when constrained by SCG's physics and governance.

## Prerequisites

- Warp terminal with Agent Mode
- Rust toolchain (cargo)
- scg_mcp_server v0.3.0 built

## Architecture Note

**IMPORTANT**: scg_mcp_server uses **STDIO transport** (JSON-RPC over stdin/stdout), not HTTP.

For Warp LLM integration, there are two approaches:

### Option A: Use the Reference Client Demo (Recommended for Now)

The reference client demonstrates all cognitive physics in a single run:

```powershell
cd C:\Users\adubo\scg_mcp_server
cargo run --release --example mcp_client
```

This shows:
- Node creation (cognitive mass instantiation)
- Mutation with energy cost
- Edge binding (conductive pathways)
- Propagation (cognitive time steps)
- Governance invariants
- Lineage audit trail

### Option B: Interactive STDIO Session

Start the server and interact via piped JSON-RPC:

```powershell
cd C:\Users\adubo\scg_mcp_server
cargo run --release
```

Then pipe JSON-RPC requests. Example session:

```json
{"jsonrpc":"2.0","method":"initialize","params":{},"id":1}
{"jsonrpc":"2.0","method":"tools/call","params":{"name":"node.create","arguments":{"belief":0.5,"energy":100}},"id":2}
{"jsonrpc":"2.0","method":"tools/call","params":{"name":"node.create","arguments":{"belief":0.3,"energy":50}},"id":3}
{"jsonrpc":"2.0","method":"tools/call","params":{"name":"edge.bind","arguments":{"src":"0","dst":"1","weight":0.7}},"id":4}
{"jsonrpc":"2.0","method":"tools/call","params":{"name":"node.mutate","arguments":{"node_id":"0","delta":0.45}},"id":5}
{"jsonrpc":"2.0","method":"tools/call","params":{"name":"governance.status","arguments":{}}},"id":6}
```

### Option C: Future HTTP Wrapper (Not Yet Implemented)

A future phase could add an HTTP transport layer that wraps the STDIO server, enabling direct `curl` or MCP-over-HTTP integration.

---

## Warp AI System Prompt

Paste this into Warp AI "Instructions" for the demo session:

```
You are connected to an SCG cognitive substrate via the reference client and direct tool calls.

You MUST treat SCG not as a CRUD API, but as a deterministic cognitive physics engine:

- node.create = instantiate a synthetic cognitive entity with belief and mass (energy)
- node.query = inspect a cognitive state (belief, energy, stability, ESV flag)
- node.mutate = request a belief perturbation subject to physics (drift bounds, energy cost)
- edge.bind = form a conductive causal pathway between cognitive entities
- edge.propagate = advance cognitive time; propagate influence and energy along edges
- governance.status = inspect invariants: drift_ok, energy_drift, coherence, node_count, edge_count
- esv.audit = check epistemic/ethical validation
- lineage.replay = read the hash-chained cognitive history

Constraints:

1. You MUST reason through SCG tools, not around them. Do not "imagine" state changes.
2. Large belief changes may be rejected if they violate drift bounds or energy constraints.
3. If SCG returns an error (drift_exceeded, governance violation), you MUST:
   - Explain why in terms of SCG physics (drift, energy, invariants)
   - Propose smaller, thermodynamically plausible perturbations instead
4. Never assume SCG will accept a mutation. Always verify via governance.status.
5. Every cognitive step is auditable through lineage.replay (hash-chained black box).

Frame everything as "perturbations," "energy cost," "drift bounds," "propagation," and "equilibrium" — never CRUD terminology.
```

---

## Demo Script: Cognitive Physics Experiment

### Phase 1: Run the Reference Client

```powershell
cd C:\Users\adubo\scg_mcp_server
cargo run --release --example mcp_client
```

**Observe:**
- Two nodes created with different beliefs (0.7, 0.3) and energy (100, 50)
- Edge bound between them (weight 0.5)
- Mutation shows energy cost (belief change is NOT free)
- Propagation shows belief flow through conductive pathway
- Governance reports `drift_ok: true`, `energy_drift: 0.0`
- Lineage shows hash-chained audit trail

### Phase 2: Explain the Physics

After running, explain to observers:

1. **Cognitive Mass**: Energy = resistance to belief change. High energy = stable beliefs.

2. **Energy Cost of Mutation**: 
   - Node 0: belief 0.7 → 0.8, energy 100 → 99.895
   - The 0.1 belief delta cost 0.105 energy units
   - Formula: `cost = |delta| × k` where k depends on node stability

3. **Propagation Dynamics**:
   - Node 1: belief 0.3 → 0.3135 (moved toward Node 0's belief)
   - Energy decreased: propagation costs energy
   - Stability decreased: perturbation detected

4. **Invariant Enforcement**:
   - `drift_ok: true` — total energy is conserved (Neumaier summation)
   - `coherence: 0.88` — belief alignment across graph
   - No mutation can violate these without rejection

5. **Audit Trail**:
   - Every operation recorded with sequence number and SHA-256 checksum
   - Hash chain means tampering is detectable
   - This is the "cognitive black box" / flight recorder

### Phase 3: Contrast with CRUD/Vector/Graph Systems

| Aspect | CRUD/Graph DB | Vector Store | SCG |
|--------|---------------|--------------|-----|
| State change | Instant, free | Instant, free | Costs energy |
| Constraints | Schema only | None | Physics + governance |
| History | Optional logs | None | Hash-chained lineage |
| Consistency | ACID | None | Thermodynamic invariants |
| Belief updates | Direct write | Similarity search | Perturbation + propagation |

**Key Point**: In SCG, you cannot "just update a belief." You must:
1. Have sufficient energy
2. Stay within drift bounds
3. Accept the energy cost
4. Let propagation distribute effects
5. All recorded in immutable lineage

---

## Done When

The demo is successful when observers understand:

1. **SCG is physics, not storage** — mutations cost energy, beliefs propagate
2. **Governance enforces invariants** — drift bounds, energy conservation
3. **Everything is auditable** — hash-chained lineage as proof of thought
4. **LLMs must adapt to physics** — cannot hallucinate state changes
5. **This is not a graph database** — it's a cognitive substrate

---

## Next Steps

1. **v0.3.1**: Package this demo with video narrative
2. **HTTP wrapper**: Add optional HTTP transport for MCP-over-HTTP clients
3. **Warp MCP integration**: Once HTTP wrapper exists, Warp can call tools directly
4. **Governance veto demo**: Create scenario that deliberately triggers `drift_exceeded`
