## Report

- iter-internal created (private): yes
- extraction method: git filter-repo on fresh clone

### Extracted paths (C-class)

Docs / directives
- docs/Iter_Hardening_Directive_v2.0.md
- docs/ARCHITECTURE.md
- docs/APEX_DEMO_DIRECTIVE_v2.0.md
- docs/DIRECTIVE_CHANGELOG.md
- docs/directives/SG-ITER-PILOT-AUTH-02_v2.1.0.md
- docs/pilot/**
- demos/LLM_IN_THE_LOOP.md

Governance / lineage extras
- governance/GOVERNANCE_PROTOCOL.md
- governance/SCG_Governance_v1.1.0.md
- governance/SCG_Governance_v1.1.1.md
- lineage/SCG_GOVERNANCE/**

Tests (internal / reconstructive)
- tests/integration/adversarial_tests.rs
- tests/hardening_fuzz.rs
- tests/mcp_fuzz_scenarios.rs
- tests/hardening_concurrency.rs
- tests/governance_enforcement.rs
- tests/contract_tests.rs
- tests/lineage_roundtrip.rs

### Custody verification (iter-internal)
- head SHA: a01db1e1bc74b2fb8dc0dda34aeb14456a2fada3
- commit count: 17
- contents: only C-classified paths

### Public repo state (before cleanup)
- public HEAD SHA (main): 9223f132d43d655cb5a0792f905f2b15169453ca

### Public deletions
Removed the C-classified files listed above from the public repo.

### Public stubs added
- docs/ARCHITECTURE.md
- docs/Iter_Hardening_Directive_v2.0.md

### Constraints
- no logic changes
- no dependency changes
- no CI/workflow changes
- SCG substrate repo untouched

### Tests
- cargo test: PASS
