# Iter Observability & Telemetry

**Classification:** External-Safe  
**Version:** 1.0  
**Last Updated:** January 2026

---

## Overview

This document describes Iter's observability architecture, including logging, metrics, distributed tracing, and audit capabilities on Azure.

---

## Telemetry Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        ITER TELEMETRY FLOW                                   │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                        Iter Server                                   │   │
│  │                                                                      │   │
│  │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐     │   │
│  │  │ Structured Logs │  │ Metrics Emitter │  │ TraceContext    │     │   │
│  │  └────────┬────────┘  └────────┬────────┘  └────────┬────────┘     │   │
│  │           │                    │                    │               │   │
│  │  ┌────────┴────────┐  ┌────────┴────────┐  ┌────────┴────────┐     │   │
│  │  │ AuditEvent      │  │ DecisionPacket  │  │ Request Span    │     │   │
│  │  │ (JSON Lines)    │  │ (SHA-256)       │  │ (W3C format)    │     │   │
│  │  └────────┬────────┘  └────────┬────────┘  └────────┬────────┘     │   │
│  │           │                    │                    │               │   │
│  └───────────┼────────────────────┼────────────────────┼───────────────┘   │
│              │                    │                    │                   │
│              ▼                    ▼                    ▼                   │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐           │
│  │   OTEL Agent    │  │   Blob Storage  │  │ App Insights    │           │
│  │   (sidecar)     │  │   (archive)     │  │ (APM)           │           │
│  └────────┬────────┘  └────────┬────────┘  └────────┬────────┘           │
│           │                    │                    │                     │
│           ▼                    ▼                    ▼                     │
│  ┌─────────────────────────────────────────────────────────────────────┐ │
│  │                     Azure Monitor / Log Analytics                    │ │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌────────────┐ │ │
│  │  │   Logs      │  │   Metrics   │  │   Traces    │  │   Alerts   │ │ │
│  │  └─────────────┘  └─────────────┘  └─────────────┘  └────────────┘ │ │
│  └─────────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Logging

### Structured Log Format

All Iter logs use structured JSON format for machine parsing:

```json
{
  "timestamp": "2026-01-13T15:30:00.000Z",
  "level": "INFO",
  "target": "iter_server::mcp",
  "message": "Governance decision completed",
  "fields": {
    "request_id": "req-abc123",
    "tool": "node.create",
    "decision": "ALLOW",
    "latency_ms": 12,
    "trace_id": "4bf92f3577b34da6a3ce929d0e0e4736",
    "span_id": "00f067aa0ba902b7"
  }
}
```

### Log Levels

| Level | Purpose | Examples |
|-------|---------|----------|
| ERROR | Unrecoverable failures | Request parse failure, internal error |
| WARN | Recoverable issues | Policy violation, near-limit conditions |
| INFO | Significant events | Governance decisions, tool invocations |
| DEBUG | Diagnostic detail | Request/response flow (disabled in prod) |
| TRACE | Verbose internals | Not emitted in production |

### Log Categories

| Category | Description | Retention |
|----------|-------------|-----------|
| `iter_server::mcp` | Protocol layer events | 30 days |
| `iter_server::governance` | Governance decisions | 90 days |
| `iter_server::audit` | Audit events | 365 days |
| `iter_server::health` | Health check results | 7 days |

---

## Metrics

### Key Metrics

| Metric | Type | Description | Alert Threshold |
|--------|------|-------------|-----------------|
| `iter_requests_total` | Counter | Total requests by tool and status | - |
| `iter_request_duration_ms` | Histogram | Request latency distribution | P99 > 100ms |
| `iter_governance_decisions` | Counter | Decisions by outcome (ALLOW/DENY/etc.) | - |
| `iter_active_permits` | Gauge | Currently active learning permits | - |
| `iter_policy_violations` | Counter | Policy violations by type | > 10/min |
| `iter_checksum_failures` | Counter | DecisionPacket checksum mismatches | Any |

### Prometheus Format

```prometheus
# HELP iter_requests_total Total number of MCP requests
# TYPE iter_requests_total counter
iter_requests_total{tool="node.create",status="success"} 1542
iter_requests_total{tool="node.create",status="error"} 3
iter_requests_total{tool="governance.status",status="success"} 8721

# HELP iter_request_duration_ms Request duration in milliseconds
# TYPE iter_request_duration_ms histogram
iter_request_duration_ms_bucket{tool="node.create",le="10"} 1200
iter_request_duration_ms_bucket{tool="node.create",le="50"} 1500
iter_request_duration_ms_bucket{tool="node.create",le="100"} 1540
iter_request_duration_ms_bucket{tool="node.create",le="+Inf"} 1545

# HELP iter_governance_decisions Governance decisions by outcome
# TYPE iter_governance_decisions counter
iter_governance_decisions{decision="ALLOW"} 14532
iter_governance_decisions{decision="DENY"} 89
iter_governance_decisions{decision="FREEZE_LEARNING"} 12
```

### Azure Monitor Integration

Metrics are exported to Azure Monitor via OpenTelemetry:

```yaml
# otel-collector-config.yaml
exporters:
  azuremonitor:
    connection_string: "${APPLICATIONINSIGHTS_CONNECTION_STRING}"

processors:
  batch:
    timeout: 10s

receivers:
  prometheus:
    config:
      scrape_configs:
        - job_name: 'iter-server'
          static_configs:
            - targets: ['localhost:9090']

service:
  pipelines:
    metrics:
      receivers: [prometheus]
      processors: [batch]
      exporters: [azuremonitor]
```

---

## Distributed Tracing

### W3C Trace Context

Iter propagates and respects W3C Trace Context headers:

```
traceparent: 00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01
tracestate: iter=decision-123
```

### TraceContext Structure

```rust
pub struct TraceContext {
    pub trace_id: String,      // 32 hex chars
    pub span_id: String,       // 16 hex chars
    pub parent_span_id: Option<String>,
}
```

### Span Hierarchy

```
Consumer Request
└── iter-server (root span)
    ├── input_validation
    ├── policy_evaluation
    │   ├── reasoning_quality_gate
    │   ├── energy_integrity_gate
    │   └── learning_permission_gate
    ├── governance_decision
    ├── decision_packet_emit
    └── audit_event_emit
```

### Application Insights Integration

```typescript
// SDK injects trace context automatically
const client = await IterClient.connect("iter-server");
client.withTrace(createTraceContext("parent-trace-id"));

// Subsequent calls are correlated in Application Insights
const result = await client.nodeCreate(0.7, 100.0);
```

---

## Audit Events

### AuditEvent Structure

```json
{
  "event_id": "evt-abc123",
  "timestamp": "2026-01-13T15:30:00.000Z",
  "phase": "GOVERNANCE_DECISION",
  "outcome": "ALLOW",
  "trace_id": "4bf92f3577b34da6a3ce929d0e0e4736",
  "tool": "node.create",
  "decision_packet_checksum": "sha256:abc123...",
  "reason_codes": [],
  "redacted_fields": ["internal_score", "substrate_state"]
}
```

### Audit Event Phases

| Phase | Description |
|-------|-------------|
| `REQUEST_RECEIVED` | MCP request parsed and validated |
| `POLICY_EVALUATION` | Policy gates evaluated |
| `GOVERNANCE_DECISION` | Final decision rendered |
| `RESPONSE_SENT` | Response returned to client |
| `ERROR` | Error occurred during processing |

### Audit Redaction

Sensitive fields are redacted per AUDIT_ALLOWLIST/AUDIT_DENYLIST:

| Field Category | Redacted | Reason |
|----------------|----------|--------|
| Tool name, arguments | No | Needed for replay |
| Decision outcome | No | Core audit data |
| Reason codes | No | Explainability |
| Internal scores | Yes | IP protection |
| Substrate state | Yes | IP protection |
| Debug traces | Yes | Security |

---

## Alerting

### Alert Rules

| Alert | Condition | Severity | Action |
|-------|-----------|----------|--------|
| High Error Rate | error_rate > 5% for 5 min | Critical | Page on-call |
| High Latency | P99 > 200ms for 10 min | Warning | Slack notification |
| Policy Violations | violations > 50/min | Warning | Slack notification |
| Checksum Failure | Any confirmed occurrence | Critical | Page on-call |
| Health Check Fail | 3 consecutive failures | Critical | Auto-restart pod |

### Azure Monitor Alert Configuration

```json
{
  "alertRule": {
    "name": "iter-high-error-rate",
    "description": "High error rate in Iter server",
    "severity": 1,
    "evaluationFrequency": "PT1M",
    "windowSize": "PT5M",
    "criteria": {
      "allOf": [
        {
          "query": "iter_requests_total | where status == 'error' | summarize ErrorRate = count() / sum(count()) by bin(TimeGenerated, 5m)",
          "threshold": 0.05,
          "operator": "GreaterThan"
        }
      ]
    },
    "actions": [
      { "actionGroupId": "/subscriptions/.../actionGroups/iter-oncall" }
    ]
  }
}
```

---

## Dashboards

### Key Dashboard Panels

| Panel | Metrics | Purpose |
|-------|---------|---------|
| Request Rate | `iter_requests_total` | Traffic overview |
| Latency Distribution | `iter_request_duration_ms` | Performance monitoring |
| Decision Breakdown | `iter_governance_decisions` | Governance health |
| Error Rate | `iter_requests_total{status="error"}` | Reliability |
| Active Permits | `iter_active_permits` | Capacity planning |

### Azure Workbook Template

```json
{
  "version": "Notebook/1.0",
  "items": [
    {
      "type": 3,
      "content": {
        "version": "KqlItem/1.0",
        "query": "iter_requests_total | summarize sum(value) by tool, bin(TimeGenerated, 5m) | render timechart",
        "size": 0,
        "title": "Request Rate by Tool"
      }
    }
  ]
}
```

---

## Retention Policy

| Data Type | Hot Retention | Archive Retention | Purpose |
|-----------|---------------|-------------------|---------|
| Logs | 30 days | 365 days | Operational debugging |
| Metrics | 90 days | N/A | Performance analysis |
| Traces | 30 days | N/A | Distributed tracing |
| Audit Events | 90 days | 7 years | Compliance |
| DecisionPackets | 90 days | 7 years | Replay and audit |

---

## Implementation Status

| Capability | Status | Azure Service |
|------------|--------|---------------|
| Structured logging | ✓ Implemented | Log Analytics |
| Prometheus metrics | ✓ Implemented | Azure Monitor |
| W3C trace context | ✓ Implemented | Application Insights |
| Audit event stream | ✓ Implemented | Blob Storage |
| OTEL collector | Planned Q1 2026 | AKS sidecar |
| Azure Workbooks | Planned Q1 2026 | Azure Monitor |
