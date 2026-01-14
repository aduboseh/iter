# Iter Compliance Intent Statement

**Classification:** External-Safe  
**Version:** 1.0  
**Last Updated:** January 2026

---

## Overview

This document describes Iter's compliance posture, alignment with industry standards, and roadmap toward formal certifications. Iter is designed from inception with auditability, governance, and enterprise compliance requirements in mind.

---

## Compliance Philosophy

Iter's architecture embeds compliance-enabling properties at the core:

| Property | Compliance Benefit |
|----------|-------------------|
| **Determinism** | Reproducible decisions for audit and review |
| **Immutability** | DecisionPackets cannot be altered post-emission |
| **Auditability** | Complete audit trail with cryptographic verification |
| **Fail-closed** | Safe defaults prevent accidental policy bypass |
| **Separation** | Clear boundaries between public protocol and proprietary IP |

Iter is not currently certified under SOC 2, ISO 27001, or other frameworks; this document reflects design alignment and planned certification milestones.

---

## Framework Alignment

### SOC 2 Type II (Intent)

**Status:** Roadmap — Targeting Q4 2026

Iter's design aligns with SOC 2 Trust Service Criteria:

| Trust Criteria | Iter Capability |
|---------------|-----------------|
| **Security** | Fail-closed validation, input sanitization, output redaction |
| **Availability** | Stateless architecture, horizontal scaling design, health probes |
| **Processing Integrity** | Deterministic execution, checksum verification |
| **Confidentiality** | IP boundary enforcement, audit field redaction |
| **Privacy** | No PII stored in DecisionPackets; data minimization |

**Planned Controls:**
- Access logging and review
- Change management procedures
- Vulnerability management
- Incident response procedures

### ISO 27001 (Awareness)

**Status:** Awareness — Design aligns with ISMS principles

Iter's controls map to ISO 27001 Annex A domains:

| Domain | Iter Implementation |
|--------|-------------------|
| A.5 Information Security Policies | Governance manifest, fail-closed defaults |
| A.8 Asset Management | Sealed IP boundary, component responsibility map |
| A.9 Access Control | RBAC design, Entra ID integration roadmap |
| A.10 Cryptography | SHA-256 checksums, canonical serialization |
| A.12 Operations Security | Structured logging, audit events |
| A.14 System Development | Governance tests, CI enforcement |

### NIST AI Risk Management Framework (Awareness)

**Status:** Awareness — Architecture designed for AI governance

Iter supports NIST AI RMF functions:

| Function | Iter Contribution |
|----------|------------------|
| **GOVERN** | Policy enforcement, economic controls, learning permits |
| **MAP** | Clear system boundaries, component responsibilities |
| **MEASURE** | Audit events, governance metrics, decision tracing |
| **MANAGE** | Fail-closed enforcement, deterministic replay |

**Key Alignments:**
- Transparency: Explicit reason codes for all governance decisions
- Accountability: Immutable DecisionPackets with cryptographic verification
- Robustness: Fail-closed behavior on invalid inputs
- Safety: Learning gate controls, scarcity enforcement

### NIST Cybersecurity Framework (Awareness)

| Function | Iter Controls |
|----------|---------------|
| **Identify** | Component responsibility map, architecture boundary |
| **Protect** | Input validation, output sanitization, RBAC |
| **Detect** | Audit events, metrics, anomaly alerting |
| **Respond** | Error taxonomy, incident procedures |
| **Recover** | Stateless architecture, container redeployment |

---

## Compliance Roadmap

| Milestone | Target | Status |
|-----------|--------|--------|
| Compliance-ready architecture | Q4 2025 | ✓ Complete |
| SOC 2 Type I readiness assessment | Q2 2026 | Planned |
| SOC 2 Type II audit | Q4 2026 | Planned |
| ISO 27001 gap analysis | Q1 2027 | Planned |
| ISO 27001 certification | Q4 2027 | Planned |

Timelines are indicative and subject to customer demand, audit scope, and third-party availability.

---

## Current Controls

### Implemented

| Control | Description | Evidence |
|---------|-------------|----------|
| Schema Stability | Type shapes enforced by 71 governance tests | `tests/governance_invariants.rs` |
| Error Taxonomy | Exhaustive error codes, no catch-all errors | `docs/contracts_v1.md` |
| Deterministic Execution | Byte-identical outputs for identical inputs | `examples/determinism_demo.rs` |
| Audit Events | Structured JSON Lines with phase tracking | `src/types/telemetry.rs` |
| Checksum Verification | SHA-256 over canonical JSON | `src/types/decision_packet.rs` |
| Input Validation | NaN/Inf rejection, range enforcement | `src/types/validation.rs` |
| Release Gates | CI blocks non-compliant releases | `.github/workflows/release_gate.yml` |

### Planned

| Control | Target | Description |
|---------|--------|-------------|
| Entra ID Integration | Q1 2026 | Enterprise authentication |
| RBAC Enforcement | Q2 2026 | Role-based access control |
| Audit Log Archival | Q2 2026 | 7-year retention for compliance |
| Penetration Testing | Q3 2026 | Third-party security assessment |
| SOC 2 Audit | Q4 2026 | Independent compliance attestation |

---

## Audit Support

### What Iter Provides for Auditors

| Artifact | Description | Location |
|----------|-------------|----------|
| DecisionPackets | Immutable governance records | Blob Storage |
| Audit Events | Timestamped event stream | Log Analytics |
| Checksum Verification | Integrity verification tool | `iter-verify` CLI (planned) |
| Governance Tests | Automated invariant verification | Public repository |
| Architecture Boundary | Clear public/private delineation | `ARCHITECTURE_BOUNDARY.md` |

### Audit Procedures

1. **Request Scope Definition**
   - Define audit period and data requirements
   - Identify relevant DecisionPackets and AuditEvents

2. **Evidence Collection**
   - Export DecisionPackets from Blob Storage
   - Export AuditEvents from Log Analytics
   - Run governance tests against repository snapshot

3. **Verification**
   - Verify DecisionPacket checksums
   - Confirm deterministic replay
   - Review policy decision reasons

4. **Reporting**
   - Summary of decisions by outcome
   - Policy violation analysis
   - Recommendation report

---

## Data Sovereignty

### Regional Deployment

Iter supports data residency requirements through regional deployment:

| Region | Use Case |
|--------|----------|
| East US 2 | Default North America |
| West Europe | EU GDPR compliance |
| Southeast Asia | APAC requirements |

### Data Localization

| Data Type | Location | Cross-Border |
|-----------|----------|--------------|
| DecisionPackets | Regional Blob Storage | No |
| Audit Events | Regional Log Analytics | No |
| Telemetry | Regional Application Insights | Configurable |

---

## Third-Party Attestations (Planned)

| Attestation | Provider | Target |
|-------------|----------|--------|
| SOC 2 Type II | Independent CPA firm | Q4 2026 |
| Penetration Test | Third-party security firm | Q3 2026 |
| Code Audit | Security research firm | Q2 2026 |

---

## Contact for Compliance Inquiries

For compliance, audit, or certification inquiries:

**Armonti Du-Bose-Hill**  
Email: armontidubosehill@gmail.com

Detailed governance rules and internal compliance documentation available under NDA for audit, compliance, or partnership review.
