# Day-1 Invariant Violations Log

**Directive**: SG-SCG-PILOT-ACT-04 v1.0.0 §5

---

## Overview

This directory contains forensic logs for any invariant violations detected during Day-1.

## Escalation Protocol

Per ACT-04 §8, the following conditions trigger immediate escalation:

| Condition | Action | Priority |
|-----------|--------|----------|
| ΔE > 1×10⁻¹⁰ (2+ cycles) | Immediate alert | P0 |
| quarantine=true | Immediate alert | P0 |
| Replay variance > 1×10⁻¹⁰ | Immediate alert | P1 |
| Ledger hash mismatch | Critical stop | P0 |
| Parser failure > 5min | Alert | P2 |

## Forensic Capture

Each violation generates:
- Timestamp and duration
- Raw telemetry block
- Invariant violated (with threshold)
- Governor correction status
- Pod restart count
- Last 200 lines of logs

## Expected State

**ZERO violations** for Day-1 certification approval.

If this directory contains files, Day-1 has failed certification criteria.

---

**Current Status**: No violations recorded (Day-1 in progress)

**Last Updated**: 2025-11-17 (Day-1 activation)
