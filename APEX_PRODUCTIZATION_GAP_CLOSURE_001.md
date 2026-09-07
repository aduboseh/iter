# APEX DIRECTIVE — PRODUCTIZATION GAP CLOSURE

**Directive ID:** `APEX-PRODUCTIZATION-GAP-CLOSURE-001`
**Status:** ACTIVE
**Authority:** Productization / Release Certification
**Applies To:** Iter, SCG, associated SDKs, deployment assets, verification infrastructure, release pipeline
**Release Posture:** FAIL-CLOSED
**Current Certification State:** `14 PASS / 16 FAIL`
**Target Certification State:** `30 PASS / 0 FAIL`

---

## 1. Mission

Advance Iter + SCG from the current Release Candidate certification state toward a fully certifiable production release by closing every control gap that can be **implemented, independently exercised, reproduced, and evidenced now**.

The objective is **not** to make the matrix green.

The objective is to make the system deserving of a green matrix.

No control may transition to PASS because evidence was manually created, inferred, reconstructed, asserted, or committed solely to satisfy certification logic.

A control transitions to PASS only when:

1. The required capability exists.
2. The capability is exercised by the required execution environment.
3. The collector independently evaluates the result.
4. The resulting evidence is bound to exact source and artifact identity.
5. The trusted evidence workflow publishes the evidence.
6. The canonical matrix independently consumes that evidence and returns PASS.

**Verification beats generation.**

---

# 2. Governing Invariants

These rules are mandatory across all closure work.

### 2.1 No Synthetic Certification

Repository-authored JSON, manually constructed evidence, expected-output substitution, or other artificial certification artifacts SHALL NOT satisfy failed controls.

### 2.2 Commit-Bound Evidence

Every certification artifact MUST bind, where applicable:

* Iter commit SHA
* SCG commit SHA
* private substrate commit SHA
* dependency lockfiles
* compiler/toolchain identity
* runner identity
* operating system
* architecture
* container/binary digest
* corpus/input digest
* configuration digest
* output/result identity
* collector version
* workflow identity
* workflow run ID
* artifact SHA-256

### 2.3 Immutable Inputs

Certification runs SHALL NOT depend on:

* moving branches
* mutable image tags
* unpinned GitHub Actions
* developer workstations
* implicit caches
* undocumented environment state
* manual artifact substitution

### 2.4 Independent Collection

Production claims SHALL be supported by collectors whose success condition is derived from observable system behavior, not from a predeclared PASS value.

### 2.5 Fail Closed

Missing evidence, invalid evidence, incomplete collectors, signature failures, provenance mismatch, commit mismatch, unsupported capabilities, or ambiguous identity SHALL evaluate to FAIL.

### 2.6 Product Claims Must Match Product Reality

Any surface, capability, deployment configuration, SDK, substrate, security property, or operational behavior that cannot be implemented and certified MUST be removed from the current product contract until it can be proven.

---

# 3. Immediate Decision Gate — `full_substrate`

The first architectural decision defines the actual product boundary.

## Path A — Full Substrate Is a Production Dependency

Use this path ONLY if the private substrate:

* already exists,
* is available to protected CI,
* can be versioned,
* can be maintained as a release dependency,
* and can participate in repeatable certification.

Required closure:

1. Identify the canonical private substrate repository/workspace.
2. Bind an exact substrate commit to every release.
3. Build the substrate in protected CI against the exact Iter commit.
4. Prove compilation cannot succeed using stub semantics.
5. Bind Iter + substrate + SCG + lockfiles + toolchain into the release attestation.
6. Convert G0-03 from a public repository check into trusted external evidence.
7. Amend the canonical productization matrix under an approved revision such as:

`apex-productization-matrix/v1.1`

### PASS condition

The private substrate is a first-class, reproducible, provenance-bound release dependency.

---

## Path B — Public Iter Is the Complete Product Boundary

Use this path if the private substrate cannot currently satisfy the requirements above.

Required closure:

1. Remove `full_substrate` from current production claims.
2. Define public Iter as the governed verifier/runtime product boundary.
3. Change G0-03 so the expected certified behavior is:

`FULL_SUBSTRATE_UNSUPPORTED_IN_PUBLIC_REPO`

4. Ensure documentation, packaging, API claims, release artifacts, and acceptance criteria reflect this boundary.

### PASS condition

The product makes no production claim that requires an unavailable substrate.

---

## Directive

**Do not carry an unshippable architecture as a production promise.**

Path A is authorized only if the private substrate can immediately participate in the controlled release chain.

Otherwise execute Path B.

---

# 4. Priority Zero — Trusted Evidence Producer

The highest-leverage infrastructure gap is the absence of:

`.github/workflows/apex_productization_evidence.yml`

This workflow becomes the canonical certification evidence producer.

## Required Characteristics

The workflow SHALL:

* use `workflow_dispatch` only,
* require a protected production-certification environment,
* require independent approval,
* accept exact Iter commit SHA,
* accept exact SCG commit SHA,
* accept exact substrate commit SHA when applicable,
* reject branch references,
* use OIDC/workload identity,
* prohibit long-lived PATs,
* pin every third-party GitHub Action to an immutable commit SHA,
* execute control-specific collectors,
* hash every produced artifact,
* preserve raw logs,
* produce attestations where required,
* reject incomplete collector output,
* prevent collectors from self-declaring unsupported PASS states.

## Canonical Artifact Name

`apex-productization-evidence-<iter-commit>-<scg-commit>`

## Every evidence record SHALL include

* control identifier,
* exact subject commits,
* exact command executed,
* runner OS,
* runner architecture,
* runner image identity,
* compiler/toolchain identity,
* input corpus digest,
* configuration digest,
* execution result,
* result identity/digest,
* raw logs,
* artifact SHA-256,
* producer workflow identity,
* producer run ID,
* signature or attestation where applicable.

### Exit Criterion

No remaining production control depends on repository-authored PASS evidence.

---

# 5. Determinism Closure — G1

Determinism is the first technical certification domain to close because all higher-order provenance claims depend upon stable identity.

## G1-01 — Linux x86_64

Establish the canonical reference execution.

Required:

* canonical corpus,
* versioned corpus manifest,
* corpus SHA-256,
* canonical configuration,
* expected DecisionPacket/result identity,
* exact toolchain,
* clean execution,
* evidence publication.

This becomes the baseline certificate against which all other execution environments are compared.

---

## G1-03 — Windows x86_64

Prove identical semantic identity across Windows.

Specifically test:

* path normalization,
* path separator behavior,
* newline handling,
* UTF-8 behavior,
* sorting,
* map/set iteration,
* filesystem ordering,
* serialization,
* timezone/environment leakage.

PASS requires canonical identity equivalence with G1-01.

---

## G1-02 — Linux ARM64

Prove architecture independence.

Test:

* floating-point behavior,
* integer width assumptions,
* endianness assumptions,
* architecture-specific dependency behavior,
* ordering,
* serialization,
* hashing.

PASS requires canonical identity equivalence with the reference execution or a formally versioned architecture-aware identity model.

---

## G1-05 — Clean Room

Execute from:

* source,
* lockfiles,
* explicitly documented toolchains,
* empty dependency caches,
* empty build caches,
* no developer credentials,
* no developer filesystem state.

The build MUST NOT depend upon hidden workstation state.

---

## G1-06 — Mutation Suite

Mutation testing SHALL cover:

* input,
* policy,
* evidence,
* contract,
* trace,
* SCG state,
* runtime configuration,
* runtime identity,
* replay identity.

For every security- or decision-relevant mutation, the system MUST either:

1. produce a different canonical identity, or
2. reject verification.

Silent equivalence is a failure.

---

## G1-04 — AKS Determinism

Execute the canonical corpus against immutable AKS images.

Evidence SHALL bind:

* image digest,
* cluster deployment revision,
* configuration digest,
* workload identity,
* corpus digest,
* resulting canonical identity.

PASS requires identity equivalence with certified local executions.

---

# 6. Security Closure — G2

Security controls SHALL be tested negatively as well as positively.

A successful happy path alone does not constitute certification.

## G2-01 — Transport Security

Prove:

* TLS on all production ingress,
* authenticated Iter → SCG transport,
* plaintext rejection,
* protocol downgrade rejection,
* expired certificate rejection,
* wrong-authority rejection,
* SAN/hostname validation,
* certificate rotation without verification bypass.

---

## G2-02 — Identity

Implement and certify:

* OIDC request identity,
* service workload identity,
* explicit tenant claim binding,
* expired-token rejection,
* missing-token rejection,
* invalid-signature rejection,
* incorrect-issuer rejection,
* incorrect-audience rejection,
* cross-tenant rejection,
* actor/workload/tenant correlation in audit records.

---

## G2-03 — Access Boundary

Implement:

* least-privilege RBAC,
* per-tenant quotas,
* per-tenant rate limiting,
* deny-by-default network policy,
* private SCG service boundary,
* no public management endpoint.

Certification MUST include:

* negative authorization tests,
* unauthorized tenant tests,
* lateral-access tests,
* direct SCG access tests,
* network-isolation tests.

---

## G2-04 — Secrets and Audit

Production secrets SHALL use managed secret infrastructure.

Preferred production implementation:

**Azure Key Vault + AKS Workload Identity**

Prove:

* workload-bound secret access,
* secret rotation,
* secret revocation,
* repository leak prevention,
* environment-file leak prevention,
* log-redaction behavior,
* privileged-operation auditing,
* actor attribution,
* workload attribution,
* tenant attribution,
* commit attribution,
* tamper-evident audit storage,
* documented retention.

---

# 7. Independent Verification — G3-02

DecisionPackets SHALL be independently verifiable outside the originating runtime.

## Required Architecture

Create a versioned signature envelope containing at minimum:

* envelope version,
* canonical packet digest,
* algorithm identifier,
* key identifier,
* signature,
* optional certification metadata that does not mutate packet identity.

The signature SHALL be over the canonical DecisionPacket digest rather than unstable serialized representation.

## Required Key Lifecycle

Define:

* key generation,
* storage,
* access policy,
* rotation,
* revocation,
* key identifiers,
* verification history.

## Required Verifiers

Implement:

* Rust verifier,
* Python verifier.

Neither verifier may require the originating Iter process.

## Required Golden Vectors

Publish:

* valid packet/signature,
* invalid packet,
* invalid signature,
* wrong key,
* revoked key,
* mutated policy hash,
* mutated evidence hash,
* mutated trace,
* mutated replay identity.

## Algorithm Gate

The signing algorithm MUST be selected against deployment compliance requirements.

If FIPS constraints apply, use a FIPS-compatible managed signing path such as P-256 where required.

Do not select an algorithm purely because deterministic signing is technically convenient.

---

# 8. Supply Chain Closure — G4

One controlled release pipeline SHALL produce the complete release.

It SHALL create, rather than merely inspect:

* Iter binaries,
* SCG container artifacts,
* Rust SDK packages,
* Python SDK packages,
* TypeScript SDK packages if TypeScript remains a supported surface,
* versioned schemas,
* SBOM,
* checksums,
* container digests,
* signed provenance,
* source-to-artifact mapping,
* release manifest.

## Mandatory Release Invariants

* no manually uploaded production artifact,
* exact source commits,
* lockfile-bound dependencies,
* immutable action pins,
* CI signing,
* SLSA-compatible provenance,
* external verification command,
* substrate commit included when Path A applies.

## SBOM

Produce SPDX or CycloneDX output.

## Release Manifest

The manifest SHALL identify the complete provenance chain:

`source → dependency graph → toolchain → build → artifact → digest → signature → release`

---

# 9. AKS Operability Closure — G5

Create version-controlled deployment infrastructure and an automated failure-injection certification suite.

## Infrastructure

AKS configuration SHALL be reproducible using version-controlled IaC.

Acceptable authoritative deployment sources include:

* Terraform,
* Bicep,
* Helm,
* or an explicitly versioned combination.

## Failure Scenarios

Certification SHALL exercise:

* idempotent deployment,
* readiness probes,
* liveness probes,
* invalid configuration,
* pod restart,
* node loss,
* SCG outage,
* network interruption,
* identity provider failure,
* secret rotation,
* rolling upgrade,
* failed upgrade,
* rollback,
* persistent state recovery,
* audit continuity,
* degraded-mode behavior.

## Recovery Evidence

Record:

* RTO,
* RPO where applicable,
* recovery outcome,
* image digests,
* Helm revision,
* IaC revision,
* source commits,
* audit continuity.

---

# 10. Distribution Completeness — G6-01

The shipped product SHALL be evaluated from a clean consumer machine.

Inventory every promised surface.

Current surfaces to certify or explicitly remove include:

* runtime,
* HTTP/API interface,
* MCP interface,
* Rust SDK,
* Python SDK,
* TypeScript SDK,
* policy interface,
* verification CLI,
* replay CLI,
* audit export,
* deployment assets,
* telemetry integration,
* DOC-001,
* DOC-002,
* DOC-003,
* DOC-004,
* DOC-005,
* DOC-006.

## Rule

A promised but absent interface is a product defect.

The permitted resolution is either:

1. implement and certify it, or
2. remove it from the product contract.

There is no third state.

---

# 11. Independent Operator Acceptance — G6-02

This control SHALL NOT be self-certified.

An independent operator or design partner must demonstrate the ability to:

1. install the product,
2. configure identity,
3. configure policy,
4. integrate a client,
5. execute governed requests,
6. verify DecisionPackets,
7. replay outcomes,
8. export audit evidence,
9. upgrade,
10. roll back,
11. recover from documented failures.

The system architect SHALL NOT perform the acceptance procedure on behalf of the operator.

## Acceptance Gate

All P0 and P1 findings MUST be closed before PASS.

Until an independent operator exists and completes the procedure, G6-02 remains FAIL.

This is an external dependency, not an engineering failure.

---

# 12. Closure Classification

Every failed control SHALL be classified into one of three states.

| State                   | Meaning                                                                                                      | Required Action                                                         |
| ----------------------- | ------------------------------------------------------------------------------------------------------------ | ----------------------------------------------------------------------- |
| **CLOSE NOW**           | Capability and environment are under current engineering control                                             | Implement, collect evidence, certify                                    |
| **BLOCKED**             | Requires infrastructure, credentials, substrate, external environment, or dependency not currently available | Document blocker and preserve FAIL                                      |
| **EXTERNAL ACCEPTANCE** | Requires independent human/operator certification                                                            | Prepare acceptance package; preserve FAIL until independently exercised |

A BLOCKED control SHALL NOT be reclassified as PASS to improve matrix appearance.

---

# 13. Authorized Execution Sequence

Execute in this order.

### Phase 0 — Product Boundary

1. Resolve `full_substrate`.
2. Amend product claims and matrix semantics if required.

### Phase 1 — Evidence Authority

3. Implement trusted evidence producer.
4. Define evidence schemas.
5. Define commit/artifact binding.
6. Protect certification environment.

### Phase 2 — Determinism

7. Establish canonical corpus.
8. Certify Linux x86_64.
9. Certify Windows x86_64.
10. Certify ARM64.
11. Certify clean-room execution.
12. Execute mutation suite.
13. Certify AKS equivalence.

### Phase 3 — Security

14. Close TLS.
15. Close identity.
16. Close access boundary.
17. Close secrets/audit.

### Phase 4 — Independent Verification

18. Version signature envelope.
19. Implement signing lifecycle.
20. Implement Rust verifier.
21. Implement Python verifier.
22. Publish golden vectors.
23. Run mutation verification.

### Phase 5 — Supply Chain

24. Build controlled release workflow.
25. Generate SBOM.
26. Generate provenance.
27. Sign artifacts.
28. Produce release manifest.

### Phase 6 — Operability

29. Version AKS infrastructure.
30. Execute failure-injection suite.
31. Record recovery evidence.

### Phase 7 — Distribution

32. Inventory every promised interface.
33. Implement or remove incomplete surfaces.
34. Run clean-machine installation and usage tests.
35. Validate DOC-001 through DOC-006.

### Phase 8 — External Acceptance

36. Provide independent operator package.
37. Run operator acceptance.
38. Close all P0/P1 findings.

### Phase 9 — Final Certification

39. Run full productization matrix.
40. Run without `--allow-failures`.
41. Verify repository cleanliness.
42. Verify directive mirror.
43. Verify SCG release pin.
44. Verify trusted evidence producer artifacts.

---

# 14. Work We Can Close Now

The engineering team is immediately authorized to close all gaps that do not require an unavailable external dependency.

Highest priority:

1. trusted evidence workflow,
2. canonical corpus authority,
3. deterministic collectors,
4. Windows/Linux determinism,
5. clean-room validation,
6. mutation framework,
7. security implementation and negative tests,
8. signature envelope,
9. Rust/Python verification,
10. release provenance pipeline,
11. SBOM generation,
12. release manifest,
13. AKS IaC,
14. AKS failure-injection tooling,
15. distribution inventory,
16. clean-machine acceptance harness,
17. product/documentation claim correction.

These items should be completed before waiting on independent acceptance.

---

# 15. Work That SHALL Remain FAIL Until Reality Exists

The following SHALL NOT be artificially closed.

### Private Substrate

If Path A is selected but the substrate repository, commit, or protected build environment is unavailable, its associated control remains FAIL.

### AKS Certification

If the required production-equivalent AKS environment is unavailable, AKS-specific certification remains FAIL.

Local simulation does not substitute for cluster evidence.

### Independent Operator Acceptance

G6-02 remains FAIL until performed by an independent operator.

Internal architect execution is not equivalent.

### External Compliance Requirements

Where algorithm or infrastructure requirements depend on unresolved FIPS, regulatory, or customer requirements, certification must remain conditional until that requirement is decided.

---

# 16. Definition of Done

Productization is complete only when the canonical certification run reports:

```text
30 controls selected
30 PASS
0 FAIL

full_matrix = true
directive mirror = PASS
SCG release pin = PASS

Iter repository clean
SCG repository clean

trusted evidence producer = SUCCESS
all evidence commit-bound
all release artifacts provenance-bound
all required signatures valid
all independent verification vectors PASS
all P0/P1 operator findings CLOSED
```

No `--allow-failures`.

No ignored controls.

No waiver disguised as certification.

No manually manufactured evidence.

---

# 17. Release Law

Until the Definition of Done is satisfied:

**The product remains Release Candidate.**

Current fail-closed behavior SHALL remain intact.

A failed certification gate blocks production promotion.

The release system must prefer:

* absence of a release over an unverifiable release,
* explicit unsupported state over implied capability,
* reproducible evidence over architectural assertion,
* independent verification over self-attestation,
* product-surface reduction over governance debt.

---

# 18. Final Directive

**Close what can be proven now. Isolate what cannot. Remove claims that exceed the current product boundary. Preserve every genuine FAIL until implementation, independent execution, and commit-bound evidence justify PASS.**

The objective is not certification theater.

The objective is an Iter + SCG release whose behavior, provenance, security boundary, artifact lineage, replay semantics, and operational recovery can survive independent scrutiny without requiring trust in the architect who built it.
