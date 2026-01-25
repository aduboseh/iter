# Iter Data Handling & Retention Policy

**Classification:** External-Safe  
**Version:** 1.0  
**Last Updated:** January 2026

---

## Overview

This document describes what data Iter stores, what it does not store, and the retention policies governing each data type.

---

## Data Classification

### Data Iter Stores

| Data Type | Classification | Description | Storage Location |
|-----------|---------------|-------------|------------------|
| **DecisionPackets** | Business Critical | Immutable governance decision records | Azure Blob Storage |
| **AuditEvents** | Compliance | Timestamped event stream for audit | Azure Blob Storage / Log Analytics |
| **Telemetry Metrics** | Operational | Performance and health metrics | Azure Monitor |
| **Structured Logs** | Operational | Request/response event logs | Azure Log Analytics |
| **Trace Spans** | Operational | Distributed tracing data | Application Insights |

### Data Iter Does NOT Store

| Data Type | Reason |
|-----------|--------|
| **User PII** | Not collected; Iter is a governance layer, not a user-facing system |
| **Raw Model Outputs** | Upstream systems retain; Iter only evaluates governance |
| **Training Data** | Iter does not perform training |
| **Credentials/Secrets** | Managed by Azure Key Vault; not persisted in Iter |
| **Session State** | Iter is stateless per-request |

---

## Data Flow

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           DATA FLOW                                          │
│                                                                             │
│  ┌─────────────────┐                           ┌─────────────────┐         │
│  │   Consumer      │                           │   Iter Server   │         │
│  │   Request       │───── Request Data ───────►│                 │         │
│  │   (ephemeral)   │                           │   (stateless)   │         │
│  └─────────────────┘                           └────────┬────────┘         │
│                                                         │                   │
│                                         ┌───────────────┼───────────────┐  │
│                                         │               │               │  │
│                                         ▼               ▼               ▼  │
│                               ┌───────────────┐ ┌─────────────┐ ┌────────┐│
│                               │DecisionPacket │ │ AuditEvent  │ │Metrics ││
│                               │  (persisted)  │ │ (persisted) │ │(stream)││
│                               └───────┬───────┘ └──────┬──────┘ └───┬────┘│
│                                       │                │            │      │
│                                       ▼                ▼            ▼      │
│                               ┌───────────────┐ ┌─────────────┐ ┌────────┐│
│                               │ Blob Storage  │ │Log Analytics│ │Monitor ││
│                               │ (immutable)   │ │ (retained)  │ │(stream)││
│                               └───────────────┘ └─────────────┘ └────────┘│
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## DecisionPacket Data

### Contents

A DecisionPacket contains the minimum data required for governance replay:

| Field | Data Type | Purpose |
|-------|-----------|---------|
| `tick` | Integer | Decision sequence number |
| `iter_build_hash` | String | Iter version identifier |
| `scg_build_hash` | String | Substrate version identifier (opaque identifier; does not expose internal logic) |
| `energy.*` | Float | Energy state envelope (nodes, reservoir, integrity) |
| `reasoning.*` | Float | Reasoning state envelope (quality, signals) |
| `learning.*` | Mixed | Learning state envelope (status, costs, quality) |
| `policy.*` | Mixed | Policy decision and reason codes |
| `checksum` | String | SHA-256 of canonical form |

### What DecisionPackets Do NOT Contain

| Data | Reason Excluded |
|------|-----------------|
| User identifiers | Not relevant to governance decision |
| Raw input payloads | Only evaluated signals stored |
| Internal substrate state | IP protection |
| Debug information | Security |

### Immutability

DecisionPackets are immutable by design:
- Written once to append-only blob storage
- Checksum prevents undetected modification
- No update or delete operations supported

---

## Retention Policy

### Standard Retention Schedule

| Data Type | Hot Tier | Cool Tier | Archive | Total Retention |
|-----------|----------|-----------|---------|-----------------|
| DecisionPackets | 90 days | 1 year | 6 years | **7 years** |
| AuditEvents | 90 days | 1 year | 6 years | **7 years** |
| Structured Logs | 30 days | 335 days | - | **365 days** |
| Telemetry Metrics | 90 days | - | - | **90 days** |
| Trace Spans | 30 days | - | - | **30 days** |

### Retention Rationale

| Period | Rationale |
|--------|-----------|
| **7 years** (DecisionPackets, AuditEvents) | SOC 2 / regulatory compliance; litigation hold support |
| **365 days** (Logs) | Operational debugging, security investigation |
| **90 days** (Metrics) | Performance trending, capacity planning |
| **30 days** (Traces) | Active incident investigation |

### Lifecycle Management

```json
{
  "rules": [
    {
      "name": "decision-packet-lifecycle",
      "type": "Lifecycle",
      "definition": {
        "actions": {
          "baseBlob": {
            "tierToCool": { "daysAfterModificationGreaterThan": 90 },
            "tierToArchive": { "daysAfterModificationGreaterThan": 365 },
            "delete": { "daysAfterModificationGreaterThan": 2555 }
          }
        },
        "filters": {
          "blobTypes": ["blockBlob"],
          "prefixMatch": ["decision-packets/"]
        }
      }
    }
  ]
}
```

---

## Data Minimization

### Collection Principles

| Principle | Implementation |
|-----------|---------------|
| **Purpose Limitation** | Only collect data required for governance decisions |
| **Data Minimization** | DecisionPackets contain evaluated state, not raw inputs |
| **Storage Limitation** | Automatic lifecycle management and deletion |
| **Accuracy** | Checksums ensure data integrity |

### Redaction

Sensitive fields are redacted before storage:

| Field Category | Stored | Redacted |
|----------------|--------|----------|
| Decision outcome | ✓ | - |
| Policy reason codes | ✓ | - |
| State envelopes | ✓ | - |
| Internal scores | - | ✓ |
| Substrate details | - | ✓ |
| Debug traces | - | ✓ |

---

## Data Access

### Access Controls

| Role | DecisionPackets | AuditEvents | Logs | Metrics |
|------|-----------------|-------------|------|---------|
| Iter.Reader | - | - | - | ✓ |
| Iter.Operator | - | - | ✓ | ✓ |
| Iter.Auditor | ✓ | ✓ | ✓ | ✓ |
| Iter.Admin | ✓ | ✓ | ✓ | ✓ |

### Data Export

Authorized users can export data for audit or analysis:

```bash
# Export DecisionPackets for date range
az storage blob download-batch \
  --source decision-packets \
  --destination ./export \
  --pattern "2026-01-*"

# Export AuditEvents from Log Analytics
az monitor log-analytics query \
  --workspace $WORKSPACE_ID \
  --analytics-query "AuditEvents | where TimeGenerated > ago(30d)"
```

---

## Data Deletion

### Deletion Triggers

| Trigger | Process |
|---------|---------|
| Retention expiry | Automatic lifecycle policy deletion |
| Customer request | Manual deletion with audit trail |
| Legal hold release | Manual deletion after hold period |

### Deletion Procedure

1. Verify authorization (Iter.Admin required)
2. Document deletion request and rationale
3. Create audit event for deletion
4. Execute deletion via lifecycle policy or manual command
5. Verify deletion completion
6. Update retention records

### Data That Cannot Be Deleted

| Data | Reason |
|------|--------|
| DecisionPackets under legal hold | Litigation preservation |
| AuditEvents required for active audit | Audit integrity |
| Data within minimum retention period | Compliance requirements |

---

## Cross-Border Data Transfer

### Default: No Cross-Border Transfer

By default, all Iter data remains within the deployment region:

| Data Type | Cross-Border | Encryption |
|-----------|--------------|------------|
| DecisionPackets | No | AES-256 at rest |
| AuditEvents | No | AES-256 at rest |
| Logs | No | TLS in transit |
| Metrics | Configurable | TLS in transit |

### Multi-Region (If Enabled)

For disaster recovery or global deployments:

| Scenario | Data Replication | Compliance |
|----------|-----------------|------------|
| Active-Passive DR | DecisionPackets replicated to secondary region | Same jurisdiction |
| Global Deployment | Separate instances per region | No cross-border |

---

## Encryption

### At Rest

| Data Store | Encryption | Key Management |
|------------|------------|----------------|
| Blob Storage | AES-256 | Azure-managed or customer-managed (Key Vault) |
| Log Analytics | AES-256 | Azure-managed |
| Application Insights | AES-256 | Azure-managed |

### In Transit

| Connection | Encryption | Minimum Version |
|------------|------------|-----------------|
| Client → Iter | TLS 1.2+ | TLS 1.2 |
| Iter → Storage | TLS 1.2+ | TLS 1.2 |
| Internal (AKS) | mTLS (optional) | TLS 1.2 |

---

## Backup and Recovery

### Backup Strategy

| Data Type | Backup Method | RPO |
|-----------|--------------|-----|
| DecisionPackets | Geo-redundant storage | near-zero (synchronous replication dependent on Azure configuration) |
| AuditEvents | Geo-redundant storage | 0 (synchronous) |
| Configuration | Git repository | N/A (source of truth) |

### Recovery Procedure

Iter is stateless; recovery involves redeploying containers and pointing to existing data stores:

1. Deploy new AKS cluster or Container Apps instance
2. Configure storage account connections
3. Verify DecisionPacket accessibility
4. Resume operations

**RTO:** < 30 minutes for infrastructure redeployment
---

## Healthcare Data Processing Clarification

**Critical Distinction:** Iter governs actions taken by systems that process PHI; it does not ingest, transform, or persist clinical data.

### PHI Boundary

```
┌──────────────────────────────────────────────────────────────────┐
│                     DATA FLOW EXAMPLE                            │
│                                                                  │
│  Healthcare AI Agent                                             │
│  │                                                               │
│  ├─ Reads PHI from EHR system                                    │
│  │  (patient diagnosis, medications, lab results)               │
│  │                                                               │
│  ├─ Proposes clinical action                                     │
│  │  "Prescribe medication X at dosage Y"                        │
│  │                                                               │
│  ├─ Submits governance request to Iter                           │
│  │  ┌────────────────────────────────────────────────────┐      │
│  │  │ Iter Receives:                                     │      │
│  │  │ • Action type: "prescribe_medication"              │      │
│  │  │ • Risk score: 0.15                                 │      │
│  │  │ • Confidence: 0.92                                 │      │
│  │  │                                                    │      │
│  │  │ Iter Does NOT Receive:                             │      │
│  │  │ • Patient name, MRN, or identifiers                │      │
│  │  │ • Diagnosis codes or clinical details              │      │
│  │  │ • Medication names or dosages                      │      │
│  │  └────────────────────────────────────────────────────┘      │
│  │                                                               │
│  └─ Receives governance decision                                 │
│     "ALLOW" or "REQUIRE_REVIEW"                                  │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
```

### Governance-Only Data Model

Iter processes **governance-relevant metadata**, not clinical data:

- Action classifications (e.g., "high_risk_intervention")
- Confidence scores and uncertainty bounds
- Policy compliance flags
- Agent identity and authorization context

**What Iter does not store:**
- Patient identifiers (names, MRNs, SSNs)
- Clinical observations (diagnoses, lab values, vitals)
- Treatment details (medications, procedures, orders)

### HIPAA Compliance Implications

Because Iter does not store PHI:

1. **Not a Covered Entity or Business Associate** (in typical deployments)
2. **No HIPAA Security Rule applicability** to Iter data stores
3. **Simplified audit scope** for healthcare customers

**Exception:** If a customer configures Iter to receive PHI-containing inputs (against design guidance), Iter's Azure-hosted infrastructure can support BAA execution. However, default deployment intentionally avoids this requirement.

### Recommended Integration Pattern

Healthcare customers should:

1. **Pre-process agent requests** to strip PHI before submitting to Iter
2. **Use risk scores and action types** as governance inputs, not clinical details
3. **Store PHI-containing agent logs separately** from Iter's decision ledger

This pattern maintains full governance capability while keeping Iter outside HIPAA compliance scope, reducing operational burden.
---

## Healthcare Data Processing Clarification

**Critical Distinction:** Iter governs actions taken by systems that process PHI; it does not ingest, transform, or persist clinical data.

### PHI Boundary

```
┌──────────────────────────────────────────────────────────────────┐
│                     DATA FLOW EXAMPLE                            │
│                                                                  │
│  Healthcare AI Agent                                             │
│  │                                                               │
│  ├─ Reads PHI from EHR system                                    │
│  │  (patient diagnosis, medications, lab results)               │
│  │                                                               │
│  ├─ Proposes clinical action                                     │
│  │  "Prescribe medication X at dosage Y"                        │
│  │                                                               │
│  ├─ Submits governance request to Iter                           │
│  │  ┌────────────────────────────────────────────────────┐      │
│  │  │ Iter Receives:                                     │      │
│  │  │ • Action type: "prescribe_medication"              │      │
│  │  │ • Risk score: 0.15                                 │      │
│  │  │ • Confidence: 0.92                                 │      │
│  │  │                                                    │      │
│  │  │ Iter Does NOT Receive:                             │      │
│  │  │ • Patient name, MRN, or identifiers                │      │
│  │  │ • Diagnosis codes or clinical details              │      │
│  │  │ • Medication names or dosages                      │      │
│  │  └────────────────────────────────────────────────────┘      │
│  │                                                               │
│  └─ Receives governance decision                                 │
│     "ALLOW" or "REQUIRE_REVIEW"                                  │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
```

### Governance-Only Data Model

Iter processes **governance-relevant metadata**, not clinical data:

- Action classifications (e.g., "high_risk_intervention")
- Confidence scores and uncertainty bounds
- Policy compliance flags
- Agent identity and authorization context

**What Iter does not store:**
- Patient identifiers (names, MRNs, SSNs)
- Clinical observations (diagnoses, lab values, vitals)
- Treatment details (medications, procedures, orders)

### HIPAA Compliance Implications

Because Iter does not store PHI:

1. **Not a Covered Entity or Business Associate** (in typical deployments)
2. **No HIPAA Security Rule applicability** to Iter data stores
3. **Simplified audit scope** for healthcare customers

**Exception:** If a customer configures Iter to receive PHI-containing inputs (against design guidance), Iter's Azure-hosted infrastructure can support BAA execution. However, default deployment intentionally avoids this requirement.

### Recommended Integration Pattern

Healthcare customers should:

1. **Pre-process agent requests** to strip PHI before submitting to Iter
2. **Use risk scores and action types** as governance inputs, not clinical details
3. **Store PHI-containing agent logs separately** from Iter's decision ledger

This pattern maintains full governance capability while keeping Iter outside HIPAA compliance scope, reducing operational burden.
