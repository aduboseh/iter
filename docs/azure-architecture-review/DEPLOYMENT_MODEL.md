# Iter Deployment Model

**Classification:** External-Safe  
**Version:** 1.0  
**Last Updated:** January 2026

---

## Overview

Iter is designed for containerized deployment on Azure compute services. This document describes the runtime topology, scaling approach, and tenancy model.

---

## Runtime Topology

### Primary: Azure Kubernetes Service (AKS)

Iter's recommended production deployment is on AKS for enterprise customers requiring:
- Fine-grained control over resource allocation
- Custom networking configurations
- Integration with existing Kubernetes infrastructure

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        Azure Kubernetes Service (AKS)                        │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                         Iter Namespace                               │   │
│  │                                                                      │   │
│  │  ┌────────────────┐  ┌────────────────┐  ┌────────────────┐        │   │
│  │  │   iter-server  │  │   iter-server  │  │   iter-server  │        │   │
│  │  │    (Pod 1)     │  │    (Pod 2)     │  │    (Pod N)     │        │   │
│  │  └────────────────┘  └────────────────┘  └────────────────┘        │   │
│  │           │                  │                  │                   │   │
│  │           └──────────────────┼──────────────────┘                   │   │
│  │                              │                                      │   │
│  │                    ┌─────────┴─────────┐                           │   │
│  │                    │   ClusterIP Svc   │                           │   │
│  │                    └─────────┬─────────┘                           │   │
│  │                              │                                      │   │
│  └──────────────────────────────┼──────────────────────────────────────┘   │
│                                 │                                          │
│                       ┌─────────┴─────────┐                               │
│                       │   Ingress / AGIC  │                               │
│                       └─────────┬─────────┘                               │
│                                 │                                          │
└─────────────────────────────────┼──────────────────────────────────────────┘
                                  │
                                  ▼
                           Consumer Traffic
```

### Alternative: Azure Container Apps

For simpler deployments or smaller scale, Azure Container Apps provides:
- Managed Kubernetes abstraction
- Built-in autoscaling
- Simplified networking

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        Azure Container Apps Environment                      │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                        iter-server App                               │   │
│  │                                                                      │   │
│  │  Replicas: 1-10 (auto-scaled by request concurrency)                │   │
│  │  CPU: 0.5-2 cores per replica                                       │   │
│  │  Memory: 1-4 GB per replica                                         │   │
│  │                                                                      │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                 │                                          │
│                       ┌─────────┴─────────┐                               │
│                       │   HTTPS Ingress   │                               │
│                       └───────────────────┘                               │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Scaling Strategy

### Horizontal Pod Autoscaling (AKS)

```yaml
# iter-hpa.yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: iter-server-hpa
  namespace: iter
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: iter-server
  minReplicas: 2
  maxReplicas: 20
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
```

### Scaling Characteristics

| Metric | Target | Notes |
|--------|--------|-------|
| CPU utilization | 70% | Governance evaluation is CPU-bound |
| Memory utilization | 80% | DecisionPacket serialization uses working memory |
| Request latency P99 | ~100ms target | Typical governance evaluation target |
| Concurrent requests | 50/pod | Soft limit based on testing |

---

## Tenancy Model

### Current: Single-Tenant

Each Iter deployment serves a single logical tenant (organization or application).

```
┌───────────────────┐    ┌───────────────────┐    ┌───────────────────┐
│   Customer A      │    │   Customer B      │    │   Customer C      │
│   ┌───────────┐   │    │   ┌───────────┐   │    │   ┌───────────┐   │
│   │Iter Cluster│   │    │   │Iter Cluster│   │    │   │Iter Cluster│   │
│   │(Dedicated) │   │    │   │(Dedicated) │   │    │   │(Dedicated) │   │
│   └───────────┘   │    │   └───────────┘   │    │   └───────────┘   │
└───────────────────┘    └───────────────────┘    └───────────────────┘
```

**Benefits:**
- Complete data isolation
- Customer-specific policy configurations
- Independent scaling per customer
- Simplified compliance (data residency)

### Future: Multi-Tenant (Roadmap)

Planned for future releases with namespace-level isolation:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        Shared Iter Control Plane                            │
│                                                                             │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐            │
│  │  Namespace: A   │  │  Namespace: B   │  │  Namespace: C   │            │
│  │  (Customer A)   │  │  (Customer B)   │  │  (Customer C)   │            │
│  │  ┌───────────┐  │  │  ┌───────────┐  │  │  ┌───────────┐  │            │
│  │  │  Pods     │  │  │  │  Pods     │  │  │  │  Pods     │  │            │
│  │  │  Config   │  │  │  │  Config   │  │  │  │  Config   │  │            │
│  │  │  Secrets  │  │  │  │  Secrets  │  │  │  │  Secrets  │  │            │
│  │  └───────────┘  │  │  └───────────┘  │  │  └───────────┘  │            │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘            │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Container Specification

### Docker Image

```dockerfile
# Production image (from repository Dockerfile)
FROM debian:bookworm-slim
WORKDIR /app
COPY iter-server /app/iter-server
USER iter
EXPOSE 3000
ENTRYPOINT ["/app/iter-server"]
```

### Resource Requirements

| Environment | CPU Request | CPU Limit | Memory Request | Memory Limit |
|-------------|-------------|-----------|----------------|--------------|
| Development | 250m | 500m | 256Mi | 512Mi |
| Staging | 500m | 1000m | 512Mi | 1Gi |
| Production | 1000m | 2000m | 1Gi | 2Gi |

### Health Probes

```yaml
livenessProbe:
  exec:
    command: ["pgrep", "iter-server"]
  initialDelaySeconds: 10
  periodSeconds: 30

readinessProbe:
  tcpSocket:
    port: 3000
  initialDelaySeconds: 5
  periodSeconds: 10
```

---

## Transport Options

### STDIO (Current)

Current primary transport for local and containerized deployments:
- Direct process communication
- No network overhead for co-located clients
- Used by SDKs and CLI tools

### HTTP/HTTPS (Roadmap)

Planned for distributed deployments:
- RESTful wrapper over MCP
- TLS termination at ingress
- API gateway integration

Timing and scope depend on customer deployment requirements and security posture.

---

## High Availability

### AKS Configuration

| Component | Configuration |
|-----------|--------------|
| Replica count | Minimum 2, recommended 3+ |
| Pod disruption budget | maxUnavailable: 1 |
| Pod anti-affinity | Spread across availability zones |
| Node pools | System pool + User pool (dedicated) |

### Disaster Recovery

| RPO | RTO | Strategy |
|-----|-----|----------|
| 0 for configured external evidence stores | minutes | Redeployment from container registry plus existing audit ledger/blob storage |

Iter process state is disposable. Production evidence is externalized through configured DecisionPacket/audit storage. The executable local control is `ITER_AUDIT_LEDGER_PATH`; set `ITER_REQUIRE_AUDIT_LEDGER=1` so governed and `scg-backed` runtimes fail closed when the mounted single-writer ledger is absent, invalid, or unwritable.

---

## Deployment Regions

### Supported Azure Regions

| Region | Primary Use Case |
|--------|-----------------|
| East US 2 | Primary production (North America) |
| West Europe | EU data residency requirements |
| Southeast Asia | APAC deployments |

Multi-region deployment follows active-passive model with traffic manager.
---

## Lifecycle Framing: Day-0 / Day-1 / Day-2 Operations

### Day-0: Marketplace Provisioning

**Objective:** Instantiate Iter infrastructure in customer Azure subscription

**Activities:**
1. Customer initiates deployment from Azure Marketplace
2. Iter ARM template provisions:
   - AKS cluster (or integrates with existing cluster)
   - Azure Cosmos DB account (governance state)
   - Azure Storage account (decision ledger)
   - Managed Identity for Iter service principal
3. Network configuration applied (private endpoint, vNet integration)
4. Health checks confirm service availability

**Success Criteria:**
- Iter API responds to `tools/list` with 200 OK
- Managed Identity can authenticate to Cosmos DB
- Storage account accessible via private endpoint

**Typical Duration:** 15–30 minutes (automated)

---

### Day-1: Identity + Policy Bootstrap

**Objective:** Configure Entra ID integration and install initial governance policies

**Activities:**
1. **Identity Binding**
   - Customer grants Iter Managed Identity access to required Azure resources
   - Entra ID application registration created for consumer authentication
   - RBAC roles assigned (Governance Admin, Policy Auditor)

2. **Policy Installation**
   - Default "safe-start" policy installed (DENY all until explicitly allowed)
   - Customer imports policy templates for their use case:
     - Healthcare: clinical decision review gates
     - Financial: model risk management thresholds
     - General: basic quality and safety constraints

3. **Test Governance Request**
   - Customer submits sample decision to Iter
   - Verifies DecisionPacket returned with expected outcome
   - Confirms AuditEvent written to ledger

**Success Criteria:**
- Consumer can authenticate via Entra ID
- Policy evaluation returns deterministic results
- Audit ledger queryable via Azure Storage

**Typical Duration:** 2–4 hours (semi-automated with validation)

---

### Day-2: Policy Evolution + Audit Replay

**Objective:** Ongoing governance operations and compliance maintenance

**Activities:**

1. **Policy Lifecycle Management**
   - Update policies to reflect changing business rules
   - Version policies with semantic versioning
   - Test policy changes in isolated namespace before production promotion

2. **Audit and Compliance**
   - Schedule daily exports of DecisionPackets for archive
   - Execute replay operations to verify historical decisions
   - Generate compliance reports (e.g., monthly governance summaries)

3. **Operational Monitoring**
   - Monitor Iter latency and availability via Azure Monitor
   - Alert on policy violation patterns (e.g., high DENY rates)
   - Scale AKS nodes based on decision volume

4. **Incident Response**
   - When AI system behavior is questioned, retrieve relevant DecisionPackets
   - Replay decisions to reconstruct governance logic
   - Update policies to close identified gaps

**Key Operations:**

| Operation | Frequency | Tooling |
|-----------|-----------|---------|
| Policy updates | Weekly–Monthly | Git + CI/CD pipeline |
| Audit exports | Daily | Azure Storage lifecycle policy |
| Compliance reporting | Monthly | Custom scripts + Azure Monitor |
| Replay verification | On-demand | Iter replay API |

**Success Criteria:**
- Zero unplanned policy bypasses
- <5 minute MTTR for governance investigations (via replay)
- 100% audit coverage (no missing DecisionPackets)

---

### Operational Maturity Model

| Maturity Level | Day-0 | Day-1 | Day-2 |
|----------------|-------|-------|-------|
| **Deployed** | ✅ Service running | ✅ Identity configured | ✅ Policies active |
| **Observable** | Health checks | Basic telemetry | Full audit pipeline |
| **Governed** | Default deny | Test policies | Production policies |
| **Compliant** | N/A | Baseline policies | 7-year retention |

This lifecycle framing ensures customers understand Iter as an **operable service**, not just deployable infrastructure.
---

## Lifecycle Framing: Day-0 / Day-1 / Day-2 Operations

### Day-0: Marketplace Provisioning

**Objective:** Instantiate Iter infrastructure in customer Azure subscription

**Activities:**
1. Customer initiates deployment from Azure Marketplace
2. Iter ARM template provisions:
   - AKS cluster (or integrates with existing cluster)
   - Azure Cosmos DB account (governance state)
   - Azure Storage account (decision ledger)
   - Managed Identity for Iter service principal
3. Network configuration applied (private endpoint, vNet integration)
4. Health checks confirm service availability

**Success Criteria:**
- Iter API responds to `tools/list` with 200 OK
- Managed Identity can authenticate to Cosmos DB
- Storage account accessible via private endpoint

**Typical Duration:** 15–30 minutes (automated)

---

### Day-1: Identity + Policy Bootstrap

**Objective:** Configure Entra ID integration and install initial governance policies

**Activities:**
1. **Identity Binding**
   - Customer grants Iter Managed Identity access to required Azure resources
   - Entra ID application registration created for consumer authentication
   - RBAC roles assigned (Governance Admin, Policy Auditor)

2. **Policy Installation**
   - Default "safe-start" policy installed (DENY all until explicitly allowed)
   - Customer imports policy templates for their use case:
     - Healthcare: clinical decision review gates
     - Financial: model risk management thresholds
     - General: basic quality and safety constraints

3. **Test Governance Request**
   - Customer submits sample decision to Iter
   - Verifies DecisionPacket returned with expected outcome
   - Confirms AuditEvent written to ledger

**Success Criteria:**
- Consumer can authenticate via Entra ID
- Policy evaluation returns deterministic results
- Audit ledger queryable via Azure Storage

**Typical Duration:** 2–4 hours (semi-automated with validation)

---

### Day-2: Policy Evolution + Audit Replay

**Objective:** Ongoing governance operations and compliance maintenance

**Activities:**

1. **Policy Lifecycle Management**
   - Update policies to reflect changing business rules
   - Version policies with semantic versioning
   - Test policy changes in isolated namespace before production promotion

2. **Audit and Compliance**
   - Schedule daily exports of DecisionPackets for archive
   - Execute replay operations to verify historical decisions
   - Generate compliance reports (e.g., monthly governance summaries)

3. **Operational Monitoring**
   - Monitor Iter latency and availability via Azure Monitor
   - Alert on policy violation patterns (e.g., high DENY rates)
   - Scale AKS nodes based on decision volume

4. **Incident Response**
   - When AI system behavior is questioned, retrieve relevant DecisionPackets
   - Replay decisions to reconstruct governance logic
   - Update policies to close identified gaps

**Key Operations:**

| Operation | Frequency | Tooling |
|-----------|-----------|---------|
| Policy updates | Weekly–Monthly | Git + CI/CD pipeline |
| Audit exports | Daily | Azure Storage lifecycle policy |
| Compliance reporting | Monthly | Custom scripts + Azure Monitor |
| Replay verification | On-demand | Iter replay API |

**Success Criteria:**
- Zero unplanned policy bypasses
- <5 minute MTTR for governance investigations (via replay)
- 100% audit coverage (no missing DecisionPackets)

---

### Operational Maturity Model

| Maturity Level | Day-0 | Day-1 | Day-2 |
|----------------|-------|-------|-------|
| **Deployed** | ✅ Service running | ✅ Identity configured | ✅ Policies active |
| **Observable** | Health checks | Basic telemetry | Full audit pipeline |
| **Governed** | Default deny | Test policies | Production policies |
| **Compliant** | N/A | Baseline policies | 7-year retention |

This lifecycle framing ensures customers understand Iter as an **operable service**, not just deployable infrastructure.
