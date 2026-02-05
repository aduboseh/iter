# Iter Identity & Access Model

**Classification:** External-Safe  
**Version:** 1.0  
**Last Updated:** January 2026

---

## Overview

This document describes Iter's identity and access management approach for Azure deployments, including authentication, authorization, and role-based access control.

Not all identity and RBAC capabilities described here are implemented today; this document reflects the intended enterprise identity architecture.

---

## Identity Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        IDENTITY FLOW                                         │
│                                                                             │
│  ┌─────────────────┐                           ┌─────────────────┐         │
│  │   Consumer App  │                           │  Microsoft      │         │
│  │   (Client)      │─────── OAuth 2.0 ────────►│  Entra ID       │         │
│  └────────┬────────┘                           └────────┬────────┘         │
│           │                                             │                   │
│           │ Access Token (JWT)                          │ Validate Token    │
│           │                                             │                   │
│           ▼                                             ▼                   │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                        Iter Control Plane                            │   │
│  │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐     │   │
│  │  │ Token Validation│  │ Role Extraction │  │ Policy Lookup   │     │   │
│  │  └────────┬────────┘  └────────┬────────┘  └────────┬────────┘     │   │
│  │           │                    │                    │               │   │
│  │           └────────────────────┴────────────────────┘               │   │
│  │                                │                                    │   │
│  │                    ┌───────────┴───────────┐                       │   │
│  │                    │  Authorization Check  │                       │   │
│  │                    └───────────┬───────────┘                       │   │
│  │                                │                                    │   │
│  │                    ┌───────────┴───────────┐                       │   │
│  │                    │  Governance Decision  │                       │   │
│  │                    └───────────────────────┘                       │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Authentication

### Consumer Authentication (Entra ID)

| Method | Use Case | Configuration |
|--------|----------|---------------|
| **OAuth 2.0 Client Credentials** | Service-to-service | App registration, client secret/certificate |
| **OAuth 2.0 Authorization Code** | User-delegated access | App registration, redirect URIs |
| **Managed Identity** | Azure-hosted consumers | Federated identity credentials |

### Entra ID App Registration

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        Iter API App Registration                            │
│                                                                             │
│  Application (client) ID:  xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx            │
│  Directory (tenant) ID:    xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx            │
│                                                                             │
│  API Permissions:                                                           │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  Iter.Governance.Read      - Query governance status                │   │
│  │  Iter.Governance.Execute   - Execute governed operations            │   │
│  │  Iter.Audit.Read           - Read audit events                      │   │
│  │  Iter.Admin.Configure      - Configure policies (admin only)        │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  Exposed APIs:                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  api://iter-governance                                               │   │
│  │    └── .default (all permissions)                                   │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Service-to-Service (Managed Identity)

For Azure-hosted consumers, Managed Identity eliminates credential management:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                     MANAGED IDENTITY FLOW                                    │
│                                                                             │
│  ┌─────────────────┐                           ┌─────────────────┐         │
│  │  Consumer Pod   │                           │  Azure IMDS     │         │
│  │  (Workload ID)  │──── Token Request ───────►│                 │         │
│  └────────┬────────┘                           └────────┬────────┘         │
│           │                                             │                   │
│           │ Access Token                                │ Federated Token   │
│           │                                             │                   │
│           ▼                                             ▼                   │
│  ┌─────────────────┐                           ┌─────────────────┐         │
│  │  Iter API Call  │─────────────────────────►│  Iter Server    │         │
│  │  (with token)   │                           │  (validates)    │         │
│  └─────────────────┘                           └─────────────────┘         │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Authorization

### Role-Based Access Control (RBAC)

| Role | Permissions | Typical Assignment |
|------|-------------|-------------------|
| **Iter.Reader** | Read governance status, query nodes | Monitoring systems, dashboards |
| **Iter.Operator** | Execute governed operations | AI agents, workflow systems |
| **Iter.Auditor** | Read audit events, replay lineage | Compliance teams, auditors |
| **Iter.Admin** | Configure policies, manage permits | Platform administrators |

### Permission Matrix

Operation names are illustrative and reflect logical capabilities, not stable external APIs.

| Operation | Reader | Operator | Auditor | Admin |
|-----------|--------|----------|---------|-------|
| `governance.status` | ✓ | ✓ | ✓ | ✓ |
| `governor.status` | ✓ | ✓ | ✓ | ✓ |
| `node.query` | ✓ | ✓ | ✓ | ✓ |
| `node.create` | - | ✓ | - | ✓ |
| `node.mutate` | - | ✓ | - | ✓ |
| `edge.bind` | - | ✓ | - | ✓ |
| `edge.propagate` | - | ✓ | - | ✓ |
| `lineage.replay` | - | - | ✓ | ✓ |
| `esv.audit` | - | - | ✓ | ✓ |
| Policy configuration | - | - | - | ✓ |
| Permit management | - | - | - | ✓ |

### Role Assignment

Roles are assigned via Entra ID groups or direct app role assignments:

```json
{
  "appRoles": [
    {
      "id": "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
      "displayName": "Iter Reader",
      "value": "Iter.Reader",
      "description": "Read-only access to governance status"
    },
    {
      "id": "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
      "displayName": "Iter Operator",
      "value": "Iter.Operator",
      "description": "Execute governed operations"
    },
    {
      "id": "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
      "displayName": "Iter Auditor",
      "value": "Iter.Auditor",
      "description": "Read audit events and replay lineage"
    },
    {
      "id": "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
      "displayName": "Iter Admin",
      "value": "Iter.Admin",
      "description": "Full administrative access"
    }
  ]
}
```

---

## Infrastructure Identity

### AKS Workload Identity

Iter pods use Workload Identity to access Azure resources without storing credentials:

```yaml
# iter-service-account.yaml
apiVersion: v1
kind: ServiceAccount
metadata:
  name: iter-server
  namespace: iter
  annotations:
    azure.workload.identity/client-id: "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
  labels:
    azure.workload.identity/use: "true"
```

### Resource Access via Managed Identity

| Resource | Access Type | RBAC Role |
|----------|-------------|-----------|
| Azure Key Vault | Secrets, certificates | Key Vault Secrets User |
| Azure Storage | DecisionPackets, audit logs | Storage Blob Data Contributor |
| Azure Monitor | Metrics, logs | Monitoring Metrics Publisher |
| Azure Container Registry | Container images | AcrPull |

---

## Token Validation

### JWT Claims Validated

| Claim | Validation | Purpose |
|-------|------------|---------|
| `iss` | Must match Entra ID issuer | Token source verification |
| `aud` | Must match Iter API audience | Prevent token reuse |
| `exp` | Must be in the future | Token freshness |
| `roles` | Must contain required role | Authorization |
| `tid` | Must match expected tenant | Tenant isolation |

### Example Token Payload

```json
{
  "iss": "https://login.microsoftonline.com/{tenant-id}/v2.0",
  "aud": "api://iter-governance",
  "sub": "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
  "roles": ["Iter.Operator"],
  "tid": "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
  "exp": 1736784000,
  "iat": 1736780400
}
```

---

## Security Considerations

### Least Privilege

- Assign minimum required role for each consumer
- Use time-limited access for elevated permissions
- Review role assignments quarterly

### Token Security

- Access tokens are short-lived (1 hour default)
- Refresh tokens not used for service-to-service
- Token caching handled by MSAL libraries

### Audit Trail

- All authentication events logged
- Role changes tracked in audit events
- Failed authentication attempts monitored

---

## Implementation Roadmap

| Phase | Capability | Status |
|-------|------------|--------|
| Phase 1 | API key authentication (current) | ✓ Implemented |
| Phase 2 | Entra ID integration | Planned Q1 2026 |
| Phase 3 | Workload Identity for pods | Planned Q2 2026 |
| Phase 4 | Fine-grained RBAC | Planned Q3 2026 |

---

## Configuration Reference

### Environment Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `AZURE_TENANT_ID` | Entra ID tenant | `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx` |
| `AZURE_CLIENT_ID` | Iter API app ID | `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx` |
| `AZURE_AUTHORITY` | Entra ID authority | `https://login.microsoftonline.com/{tenant}` |

### SDK Configuration

```typescript
// TypeScript SDK with Entra ID
import { IterClient } from "@iter/sdk";
import { DefaultAzureCredential } from "@azure/identity";

const credential = new DefaultAzureCredential();
const client = await IterClient.connect("iter-server", {
  credential,
  scope: "api://iter-governance/.default"
});
```

```rust
// Rust SDK with Entra ID
use iter_sdk::IterClient;
use azure_identity::DefaultAzureCredential;

let credential = DefaultAzureCredential::default();
let client = IterClient::connect_with_credential(
    "iter-server",
    credential,
    "api://iter-governance/.default"
)?;
```
---

## Tenant Isolation Guarantees

Iter enforces contractual tenant isolation at three distinct layers:

### Policy Namespace Isolation

**Guarantee:** Policy definitions are tenant-scoped and cryptographically bound to tenant identity.

- Policy objects are stored with `tenant_id` as partition key
- Cross-tenant policy reads return empty sets, not access-denied errors
- Policy evaluation context includes tenant scope verification before execution

### Decision Ledger Isolation

**Guarantee:** DecisionPackets and AuditEvents are scoped to originating tenant with no cross-tenant read paths.

- Audit streams are written to tenant-specific storage partitions
- Ledger queries require tenant-scoped credentials; operator access requires explicit tenant consent
- Replay operations verify tenant identity match before reconstructing decisions

### Runtime Isolation

**Guarantee:** No shared state exists between tenant evaluation contexts.

- Each governance evaluation runs in an isolated execution context
- Memory and compute resources are tenant-partitioned at the process level
- Telemetry and logging include tenant identifiers but prevent cross-tenant correlation

### Operator Access Model

**Platform operators cannot:**
- Read tenant policy definitions without tenant-granted RBAC role
- Access DecisionPackets or audit ledgers without tenant authorization
- Observe governance outcomes across tenant boundaries

**Platform operators can:**
- Monitor aggregate system health metrics (no per-tenant detail)
- Perform infrastructure operations (scaling, patching) without data access
- Respond to tenant-initiated support requests with time-limited, audited access grants

This isolation model meets Microsoft Marketplace multi-tenancy requirements and supports compliance regimes (HIPAA, GDPR) that prohibit data commingling.
---

## Tenant Isolation Guarantees

Iter enforces contractual tenant isolation at three distinct layers:

### Policy Namespace Isolation

**Guarantee:** Policy definitions are tenant-scoped and cryptographically bound to tenant identity.

- Policy objects are stored with `tenant_id` as partition key
- Cross-tenant policy reads return empty sets, not access-denied errors
- Policy evaluation context includes tenant scope verification before execution

### Decision Ledger Isolation

**Guarantee:** DecisionPackets and AuditEvents are scoped to originating tenant with no cross-tenant read paths.

- Audit streams are written to tenant-specific storage partitions
- Ledger queries require tenant-scoped credentials; operator access requires explicit tenant consent
- Replay operations verify tenant identity match before reconstructing decisions

### Runtime Isolation

**Guarantee:** No shared state exists between tenant evaluation contexts.

- Each governance evaluation runs in an isolated execution context
- Memory and compute resources are tenant-partitioned at the process level
- Telemetry and logging include tenant identifiers but prevent cross-tenant correlation

### Operator Access Model

**Platform operators cannot:**
- Read tenant policy definitions without tenant-granted RBAC role
- Access DecisionPackets or audit ledgers without tenant authorization
- Observe governance outcomes across tenant boundaries

**Platform operators can:**
- Monitor aggregate system health metrics (no per-tenant detail)
- Perform infrastructure operations (scaling, patching) without data access
- Respond to tenant-initiated support requests with time-limited, audited access grants

This isolation model meets Microsoft Marketplace multi-tenancy requirements and supports compliance regimes (HIPAA, GDPR) that prohibit data commingling.
