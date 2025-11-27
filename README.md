<div align="center">

# 🧠 SCG Substrate
### Deterministic Cognitive Engine with MCP Interface

[![Version](https://img.shields.io/badge/version-0.1.0-blue.svg)](https://github.com/OnlySGSolutions/scg_mcp_server/releases)
[![License](https://img.shields.io/badge/license-Proprietary-red.svg)](LICENSE)
[![MCP](https://img.shields.io/badge/MCP-2.0-green.svg)](https://modelcontextprotocol.io)
[![Certification](https://img.shields.io/badge/certified-cryptographic_determinism-gold.svg)](docs/RUN_CERTIFICATION.md)

**The deterministic reasoning substrate behind the SCG ecosystem**

[Quick Start](#-quick-start) • [Documentation](#-documentation) • [Demo Package](#-certified-demo-package) • [Architecture](#-architecture) • [Examples](#-usage-examples)

</div>

---

## 🎯 What is SCG?

The **SCG Substrate** is a constraint-enforced, safety-critical cognitive engine designed for deterministic reasoning.  
This repository exposes SCG through a **Model Context Protocol (MCP)** interface with full JSON-RPC 2.0 compliance.

Every operation—node creation, propagation, constraint rejection, lineage export—is **cryptographically verifiable and invariant-controlled**.

SCG's purpose:  
**Provide mathematically-governed reasoning primitives where stability, explainability, and auditability are mandatory.**

---

## 🔬 Key Differentiators

| Property | Traditional Graphs | SCG Substrate |
|----------|-------------------|---------------|
| State Model | CRUD | Belief-weighted coherence-tracked state |
| Cycle Handling | Undefined/manual | Cycle-aware propagation with energy conservation |
| Safety | Application-level | Substrate-level Ethical State Vector (ESV) |
| Auditability | Logs/timestamps | Merkle-style lineage receipts |
| Determinism | Best-effort | Cryptographically proven (dual-run SHA-256) |

---

## 👥 Who Uses SCG?

- **Security researchers** evaluating deterministic behavior and drift bounds  
- **Enterprise engineers** needing predictable reasoning under regulatory constraints  
- **AI safety researchers** analyzing invariant-governed cognitive systems  
- **Compliance teams** requiring complete audit chains  

---

## 🚫 What SCG Is NOT

- ❌ Not a domain system (no healthcare, mobility, finance logic)  
- ❌ Not an LLM, RAG, vector DB, agent, or chatbot  
- ❌ Not a wrapper around a foundation model  
- ❌ Not a knowledge graph database  

**SCG is a substrate.**  
Domains attach above it as envelopes.

---

## 🏗️ Architecture

### High-Level Diagram

```
Application / Domain Envelopes
            │
            ▼
  MCP Interface (JSON-RPC 2.0)
            │
            ▼
      SCG Substrate Core
┌──────────────────────────────┐
│ Nodes • Edges • ESV • Lineage │
└──────────────────────────────┘
```

### Substrate Layer

**Nodes**  
Belief-weighted entities with confidence ∈ [0.0, 1.0].

**Edges**  
Directional relationships supporting:
- acyclic flows  
- cycles with energy conservation  
- bounded self-loops  

**Governor (ESV)**  
Constraint enforcement layer ensuring:
- drift ≤ **1e-10**  
- deterministic rejection of unsafe operations  
- fixed error codes (`-32001`)  

**Lineage**  
Merkle-style chain with:
- SHA-256 proof per operation  
- invariant preservation proofs  
- episode-based receipts  

### MCP Interface (JSON-RPC 2.0)

```json
{
  "jsonrpc": "2.0",
  "method": "governor.status",
  "id": 1
}
```

---

## ⚡ Quick Start

### Clone Repository

```bash
git clone https://github.com/OnlySGSolutions/scg_mcp_server.git
cd scg_mcp_server
```

### Download Certified Demo Package

```bash
wget https://github.com/OnlySGSolutions/scg_mcp_server/releases/download/v1.0_scg_demo_certified/scg_demo_package_v1.0_certified.zip
sha256sum -c scg_demo_package_v1.0_certified.sha256
unzip scg_demo_package_v1.0_certified.zip
cd scg_demo_package
```

---

## 🚀 Run 60-Second Determinism Demo

### Option 1: Docker (Recommended)

```bash
docker build -t scg-demo-package:v1.0 .
docker run --rm scg-demo-package:v1.0
```

### Option 2: Native

```bash
export SCG_TIMESTAMP_MODE=deterministic
export SCG_DETERMINISM=1
./demos/scg_demo.sh
```

**Expected Output:**

```
DETERMINISM VERIFIED - All checksums match
```

---

## 💡 Usage Examples

### Create Nodes

```bash
echo '{"jsonrpc":"2.0","method":"node.create","params":{"belief":0.7,"energy":1.0},"id":1}' | ./target/release/scg_mcp_server
```

### Bind Edges

```bash
echo '{"jsonrpc":"2.0","method":"edge.bind","params":{"src":"<node_uuid>","dst":"<node_uuid>","weight":0.5},"id":2}' | ./target/release/scg_mcp_server
```

### Query Governor Status

```bash
echo '{"jsonrpc":"2.0","method":"governor.status","params":{},"id":3}' | ./target/release/scg_mcp_server
```

---

## 🔒 Certified Demo Package

| Property     | Value                                                              |
| ------------ | ------------------------------------------------------------------ |
| Script Hash  | `588153f3c00a95c3296b576a74f4d8bea8ea556fb2a0366236bd7b21b899d1df` |
| Package Hash | `9FAEA83409F014066EEA2483E364C83A9AACC3F59BA884206A88D5B0BEF07158` |
| Determinism  | Dual-run SHA-256 equality                                          |
| Compliance   | Zero domain logic, zero IP leakage                                 |
| Git Commit   | `bc59b75`                                                          |

---

## 📄 Documentation

- [RUN_CERTIFICATION.md](scg_demo_package/RUN_CERTIFICATION.md)
- [SUBSTRATE_OVERVIEW.md](scg_demo_package/SUBSTRATE_OVERVIEW.md)
- [DEMO_WALKTHROUGH.md](scg_demo_package/DEMO_WALKTHROUGH.md)
- [RUNBOOK.md](scg_demo_package/RUNBOOK.md)
- [SCG_Demo_Executive_Summary.md](docs/SCG_Demo_Executive_Summary.md)
- [SCG_Demo_Validation_Report_v1.0.md](docs/SCG_Demo_Validation_Report_v1.0.md)

*All documentation is deterministic, reproducible, and part of the certification record.*

---

## 📦 Repository Structure

```
scg_mcp_server/
├── src/                      # Substrate core (Rust)
├── demos/                    # Demo scripts and configs
├── demo_expected/            # Expected output artifacts
├── docs/                     # Technical documentation
├── scg_demo_package/         # Certified demo package
├── certification/v1.0/       # Certification artifacts
│   ├── reports/              # Certification reports
│   ├── directives/           # Architecture directives
│   ├── harness/              # Test harness scripts
│   └── expected_results/     # Determinism baselines
├── Dockerfile                # Container build
└── README.md                 # This file
```

---

## 🚀 Roadmap

### v0.2.0 (Planned)

- Distributed coherence
- Streaming propagation
- GraphQL interface

### v1.0.0 (Future)

- Formal verification (TLA+)
- GPU propagation
- Envelope libraries (healthcare, automotive, finance)

---

## 📞 Contact

- **Research:** research@onlysgsolutions.com
- **Enterprise:** enterprise@onlysgsolutions.com
- **Security:** security@onlysgsolutions.com

---

<div align="center">

**[⬆ Back to Top](#-scg-substrate)**

Made with 🧠 by Only SG Solutions

© 2025 All Rights Reserved

</div>
