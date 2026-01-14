# Iter Azure Services Mapping

**Classification:** External-Safe  
**Version:** 1.0  
**Last Updated:** January 2026

---

## Overview

This document maps Iter components and requirements to Azure services for production deployment.

---

## Service Mapping Summary

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           ITER on AZURE                                      │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                          COMPUTE                                     │   │
│  │  ┌─────────────────────┐  ┌─────────────────────┐                   │   │
│  │  │ Azure Kubernetes    │  │ Azure Container     │                   │   │
│  │  │ Service (AKS)       │  │ Apps (ACA)          │                   │   │
│  │  │ [Primary]           │  │ [Alternative]       │                   │   │
│  │  └─────────────────────┘  └─────────────────────┘                   │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                          IDENTITY                                    │   │
│  │  ┌─────────────────────┐  ┌─────────────────────┐                   │   │
│  │  │ Microsoft Entra ID  │  │ Managed Identities  │                   │   │
│  │  │ (Authentication)    │  │ (Service-to-Service)│                   │   │
│  │  └─────────────────────┘  └─────────────────────┘                   │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                          SECRETS                                     │   │
│  │  ┌─────────────────────┐  ┌─────────────────────┐                   │   │
│  │  │ Azure Key Vault     │  │ AKS Secret Store    │                   │   │
│  │  │ (Secrets/Certs)     │  │ CSI Driver          │                   │   │
│  │  └─────────────────────┘  └─────────────────────┘                   │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                         MONITORING                                   │   │
│  │  ┌─────────────────────┐  ┌─────────────────────┐                   │   │
│  │  │ Azure Monitor       │  │ Log Analytics       │                   │   │
│  │  │ (Metrics/Alerts)    │  │ Workspace           │                   │   │
│  │  └─────────────────────┘  └─────────────────────┘                   │   │
│  │  ┌─────────────────────┐                                            │   │
│  │  │ Application         │                                            │   │
│  │  │ Insights            │                                            │   │
│  │  └─────────────────────┘                                            │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                          STORAGE                                     │   │
│  │  ┌─────────────────────┐  ┌─────────────────────┐                   │   │
│  │  │ Azure Blob Storage  │  │ Azure Table Storage │                   │   │
│  │  │ (DecisionPackets)   │  │ (Audit Index)       │                   │   │
│  │  └─────────────────────┘  └─────────────────────┘                   │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                         NETWORKING                                   │   │
│  │  ┌─────────────────────┐  ┌─────────────────────┐                   │   │
│  │  │ Azure Front Door    │  │ Azure Private Link  │                   │   │
│  │  │ / App Gateway       │  │ (Private Endpoints) │                   │   │
│  │  └─────────────────────┘  └─────────────────────┘                   │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Detailed Mapping

### Compute

| Iter Component | Azure Service | Configuration | Purpose |
|---------------|---------------|---------------|---------|
| iter-server | **AKS** | Standard_D4s_v3 nodes, 3 node pool | Primary production runtime |
| iter-server | **Container Apps** | Consumption plan, 0.5-2 vCPU | Development/staging or simple deployments |
| Container Registry | **ACR** | Premium tier, geo-replication | Store and distribute container images |

### Identity & Access

| Iter Requirement | Azure Service | Configuration | Purpose |
|-----------------|---------------|---------------|---------|
| Service authentication | **Entra ID** | App registration, service principal | Authenticate API consumers |
| Pod identity | **Workload Identity** | Federated identity credentials | Access Azure resources from pods |
| Service-to-service | **Managed Identity** | System-assigned or user-assigned | Access Key Vault, Storage, Monitor |

### Secrets Management

| Iter Requirement | Azure Service | Configuration | Purpose |
|-----------------|---------------|---------------|---------|
| API keys | **Key Vault** | Standard tier, soft-delete enabled | Store consumer API keys |
| TLS certificates | **Key Vault** | Certificate management | HTTPS termination |
| Kubernetes integration | **Secret Store CSI** | Key Vault provider | Mount secrets as volumes |

### Monitoring & Observability

| Iter Requirement | Azure Service | Configuration | Purpose |
|-----------------|---------------|---------------|---------|
| Metrics | **Azure Monitor** | Container insights enabled | CPU, memory, request metrics |
| Logs | **Log Analytics** | 30-90 day retention | Structured log aggregation |
| Tracing | **Application Insights** | W3C trace context | Distributed tracing correlation |
| Alerts | **Azure Monitor Alerts** | Action groups configured | Incident notification |

### Storage

| Iter Data | Azure Service | Configuration | Purpose |
|-----------|---------------|---------------|---------|
| DecisionPackets | **Blob Storage** | Hot tier, immutable blobs | Long-term decision archive |
| Audit Events | **Blob Storage** | Append blobs, cool tier | Audit log retention |
| Audit Index | **Table Storage** | Standard tier | Query audit events by key |
| Telemetry | **Log Analytics** | Workspace tables | Query and analyze telemetry |

### Networking

| Iter Requirement | Azure Service | Configuration | Purpose |
|-----------------|---------------|---------------|---------|
| Load balancing | **Azure Load Balancer** | Standard SKU | AKS service load balancing |
| Ingress | **Application Gateway** | WAF_v2 SKU | HTTP/HTTPS ingress with WAF |
| Global routing | **Front Door** | Premium tier | Multi-region traffic management |
| Private connectivity | **Private Link** | Private endpoints | Secure access to PaaS services |
| Network isolation | **NSG** | Deny-all default | Pod-level network policies |

---

## Reference Architecture (Azure)

```
                                    Internet
                                        │
                                        ▼
                            ┌───────────────────────┐
                            │    Azure Front Door   │
                            │    (WAF + CDN)        │
                            └───────────┬───────────┘
                                        │
                            ┌───────────┴───────────┐
                            │   Application Gateway │
                            │   (Regional LB)       │
                            └───────────┬───────────┘
                                        │
┌───────────────────────────────────────┼───────────────────────────────────────┐
│                              Virtual Network                                   │
│                                       │                                        │
│  ┌────────────────────────────────────┼────────────────────────────────────┐  │
│  │                        AKS Cluster (Private)                             │  │
│  │                                    │                                     │  │
│  │  ┌─────────────────┐  ┌───────────┴───────────┐  ┌─────────────────┐   │  │
│  │  │  System Pool    │  │       User Pool       │  │  Monitoring     │   │  │
│  │  │  (AKS system)   │  │   (iter-server pods)  │  │  (otel-agent)   │   │  │
│  │  └─────────────────┘  └───────────────────────┘  └─────────────────┘   │  │
│  │                                                                          │  │
│  └──────────────────────────────────────────────────────────────────────────┘  │
│                                       │                                        │
│                          Private Endpoints                                     │
│                                       │                                        │
│  ┌────────────────┐  ┌────────────────┤  ┌────────────────┐                   │
│  │                │  │                │  │                │                   │
│  ▼                ▼  ▼                ▼  ▼                ▼                   │
│ Key Vault     Storage Account    Log Analytics    Container Registry          │
│                                                                                │
└────────────────────────────────────────────────────────────────────────────────┘
```

---

## Cost Estimation (Monthly)

All costs are indicative estimates for architectural discussion only and do not represent a committed pricing model.

| Component | SKU | Estimated Cost | Notes |
|-----------|-----|----------------|-------|
| AKS | 3x Standard_D4s_v3 | ~$400 | Production cluster |
| ACR | Premium | ~$50 | Geo-replicated |
| Key Vault | Standard | ~$5 | Secrets + certificates |
| Storage | Hot + Cool | ~$20 | DecisionPackets + Audit |
| Log Analytics | Pay-as-you-go | ~$50 | 30-day retention |
| App Gateway | WAF_v2 | ~$250 | Production ingress |
| **Total** | | **~$775/month** | Base production deployment |

*Costs are estimates and vary by region and usage.*

---

## Deployment Prerequisites

### Azure Resources (Terraform/Bicep)

Representative resources shown for architectural clarity; implementation may vary by deployment model.

```hcl
# Required resources
resource "azurerm_resource_group" "iter" { ... }
resource "azurerm_kubernetes_cluster" "iter" { ... }
resource "azurerm_container_registry" "iter" { ... }
resource "azurerm_key_vault" "iter" { ... }
resource "azurerm_storage_account" "iter" { ... }
resource "azurerm_log_analytics_workspace" "iter" { ... }
resource "azurerm_application_insights" "iter" { ... }
```

### Azure RBAC Roles

| Identity | Role | Scope |
|----------|------|-------|
| AKS Managed Identity | AcrPull | Container Registry |
| AKS Managed Identity | Key Vault Secrets User | Key Vault |
| AKS Managed Identity | Storage Blob Data Contributor | Storage Account |
| AKS Managed Identity | Monitoring Metrics Publisher | Log Analytics |
