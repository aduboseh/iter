# SCG-PILOT-01 — Day-0 Baseline

**Status**: Collection in progress  
**Directive**: SG-SCG-PILOT-ACT-03 v1.0.0

---

## Overview

This directory contains Day-0 baseline measurements for the SCG-PILOT-01 7-day certification pilot.

---

## Files

| File | Status | Purpose |
|------|--------|---------|
| `day0_summary_template.json` |  Ready | Template awaiting real measurements |
| `day0_attestation.md` |  Complete | Day-0 preparation documentation |
| `day0_summary_final.json` | ⏳ Pending | Final baseline (6-12h collection) |
| `README.md` |  This file | Directory documentation |

---

## Collection Requirements

**Minimum Duration**: 6 hours continuous runtime  
**Preferred Duration**: 12 hours for statistical confidence  
**Sampling Rate**: Every 60 seconds (360-720 data points)

**Required Metrics**:
- Energy drift (ΔE): min/max/mean/stddev
- Coherence C(t): stability over time
- ESV valid ratio: must remain 1.0
- Quarantine events: must remain 0
- Node/edge counts: growth rate
- Governor corrections: frequency

---

## Collection Command

```powershell
.\deployment\pilot\monitor-invariants.ps1 -IntervalSeconds 60 -OutputPath ".\pilot_reports\day0"
```

**Output**:
- CSV log: `pilot-monitoring/<timestamp>/invariant-data.csv`
- Text log: `pilot-monitoring/<timestamp>/invariant-monitoring.log`

---

## Baseline Validation

Once collection is complete, validate baseline with:

```powershell
# Extract min/max/mean from CSV
$data = Import-Csv ".\pilot-monitoring\<timestamp>\invariant-data.csv"
$energyDrift = $data | Measure-Object -Property Energy_Drift -Average -Minimum -Maximum

# Verify all invariants pass
$violations = $data | Where-Object { 
    double]$_.Energy_Drift -gt 1e-10 -or 
    double]$_.Coherence -lt 0.97 -or
    int]$_.Quarantine_Events -gt 0
}

if ($violations.Count -eq 0) {
    Write-Host " Baseline VALID - All invariants passed" -ForegroundColor Green
} else {
    Write-Host " Baseline INVALID - $($violations.Count) violations detected" -ForegroundColor Red
}
```

---

## Next Steps

1. Run monitoring for ≥6 hours
2. Generate `day0_summary_final.json` with actual values
3. Update CERTIFICATION_DOSSIER.md with Day-0 summary
4. Authorize Day-1 commencement

---

**Last Updated**: 2025-11-17 (Day-0 preparation complete)
