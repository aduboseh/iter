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
| 0 (stateless) | minutes | Redeployment from container registry (environment-dependent) |

Iter is stateless; all persistent state is external (DecisionPackets, audit logs).

---

## Deployment Regions

### Supported Azure Regions

| Region | Primary Use Case |
|--------|-----------------|
| East US 2 | Primary production (North America) |
| West Europe | EU data residency requirements |
| Southeast Asia | APAC deployments |

Multi-region deployment follows active-passive model with traffic manager.
