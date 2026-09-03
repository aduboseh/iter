APEX DIRECTIVE
SCG / iter Productization and Enterprise Release

Directive ID: APEX-SCG-ITER-PROD-001
Classification: Productization / Release Authority
Priority: P0
Status: ACTIVE
Owner: Only SG Solutions
Systems: SCG / iter
Target: Enterprise Product v1.0
Operating Principle: Verification beats generation.

1. MISSION

Convert SCG and iter from validated technical systems into a commercially deployable, independently verifiable, enterprise-grade product.

The objective is not additional invention.

The objective is to establish that an external organization can:

deploy → integrate → govern → verify → replay → audit → upgrade

SCG/iter without requiring direct intervention from the system architect.

Productization is complete only when the system survives independent operation, adversarial testing, deployment variance, security review, audit scrutiny, and deterministic replay.

2. PRODUCT DEFINITION

The initial commercial product shall be:

iter Enterprise Runtime

A deterministic governance and verification runtime for AI systems and autonomous agents.

The customer submits a proposed action or decision.

iter evaluates that proposal against governed state and policy.

SCG provides the deterministic substrate required for canonical state, provenance, execution lineage, and replay.

The system returns a cryptographically bound DecisionPacket establishing what was decided, why it was decided, what evidence and policy governed the decision, and whether the decision can be independently verified and reproduced.

Commercial abstraction
AI / Agent
    │
    │ Proposed Action
    ▼
┌─────────────────────────────┐
│            iter             │
│                             │
│ Governance                  │
│ Verification                │
│ Policy Enforcement          │
└──────────────┬──────────────┘
               │
               │ governed evaluation
               ▼
┌─────────────────────────────┐
│             SCG             │
│                             │
│ Canonical State             │
│ Determinism                 │
│ Provenance                  │
│ Replay                      │
└──────────────┬──────────────┘
               │
               ▼
         DecisionPacket

The commercial promise is not:

AI that is always correct.

The commercial promise is:

AI decisions whose governance path can be verified, attributed, inspected, and replayed.

3. ARCHITECTURAL BOUNDARY

The product boundary SHALL remain explicit.

iter

Customer-facing governance runtime.

Responsibilities:

API/MCP ingress
policy enforcement
authorization
governed evaluation
DecisionPacket generation
verification
replay
audit export
SDK integration
operational telemetry
SCG

Proprietary deterministic substrate.

Responsibilities:

canonical state
graph semantics
deterministic execution
provenance
lineage
governance-state authority
execution trace integrity
replay substrate
Trust boundary

The AI model SHALL remain outside the trusted computing boundary.

MODEL
  │
  │ untrusted proposal
  ▼
ITER
  │
  │ governed request
  ▼
SCG
  │
  │ canonical evidence
  ▼
ITER
  │
  ▼
DECISIONPACKET

Model output is never authoritative merely because it was generated successfully.

4. PRIMARY EXECUTION ORDER

Effective immediately:

FREEZE NON-PRODUCT FEATURE DEVELOPMENT.

No new SCG or iter capability shall enter the critical path unless it directly contributes to one of the following:

correctness
determinism
security
provenance
replay
deployment
observability
auditability
customer integration
commercial release

Features outside these categories are deferred until after v1.0.

The governing question for every engineering task is:

Does this reduce the distance between the current system and a sellable, independently operable product?

If not, defer.

5. GATE 0 — CORRECTNESS CLOSURE

Priority: BLOCKING

No production claim shall precede correctness closure.

iter requirements
full_substrate production configuration compiles cleanly.
SCG-backed runtime is authoritative where configured.
Production tests exercise the actual GovernedRuntime::evaluate execution path.
Replay fails closed when required evidence is absent.
Missing provenance cannot produce Match.
Unsupported contracts fail deterministically.
Invalid governance state cannot produce an authoritative decision.
Degraded execution cannot masquerade as verified execution.
SCG requirements
Establish one authoritative governance-hash implementation.
Execution traces enforce validity fail-closed.
Deterministic execution cannot depend on unordered collection iteration.
Gateway persistence receives tamper-evident integrity protection.
Contract/version rejection precedence is explicitly defined.
Adversarial regression coverage exists for every critical invariant.
Required build gate
cargo fmt --check                 PASS
cargo clippy -- -D warnings       PASS
cargo test --workspace            PASS
cargo audit                       PASS

iter → SCG                        PASS
SCG → DecisionPacket              PASS
DecisionPacket → replay           PASS

tampered evidence                 REJECT
missing evidence                  REJECT
invalid trace                     REJECT
unsupported contract              REJECT
unauthorized mutation             REJECT
Exit criterion

ZERO unresolved P0 correctness defects.

6. GATE 1 — DETERMINISM CERTIFICATION

SCG/iter SHALL demonstrate determinism empirically.

Determinism is not accepted as an architectural claim.

It must be continuously proven.

Certification environments

Execute the canonical verification corpus across:

Linux x86_64
Linux ARM64
Windows x86_64
Azure AKS
clean-room CI

Given identical:

INPUT
+
POLICY
+
SCG STATE
+
CONTRACT VERSION
+
CONFIGURATION

the resulting canonical decision identity SHALL be identical.

HASH_A
=
HASH_B
=
HASH_C
=
HASH_D
=
HASH_E

Semantic equivalence is insufficient.

Canonical output must be byte-stable wherever the specification requires canonical identity.

Mutation suite

Deliberately mutate:

policy
input
evidence
SCG state
contract version
execution trace
runtime identity
configuration

Every mutation SHALL either:

produce the expected identity change, or
invalidate verification.
Deliverable

SCG/iter Determinism Certification Suite

This suite becomes a permanent release-blocking CI control.

7. GATE 2 — SECURITY AND TRUST BOUNDARY

Production deployment SHALL assume a hostile environment.

Required controls:

TLS
OAuth2/OIDC
explicit workload identity
tenant identity
RBAC
least privilege
secrets management
network isolation
rate limiting
request identity
privileged-operation auditing
credential rotation
immutable security logging

SCG SHALL NOT require direct public exposure.

Preferred topology:

EXTERNAL CLIENT
      │
      ▼
API GATEWAY
      │
      ├── Identity
      ├── Authentication
      ├── Authorization
      ├── Rate Limiting
      └── Request Provenance
             │
             ▼
            iter
             │
             ▼
            SCG
       PRIVATE BOUNDARY

No production behavior may depend on implicit localhost trust.

Every privileged state mutation SHALL produce attributable evidence.

8. GATE 3 — DECISIONPACKET PRODUCT CONTRACT

DecisionPacket becomes the primary customer-visible evidence artifact.

The contract SHALL include sufficient information to determine:

what happened
why it happened
what governed it
what evidence was used
which runtime produced it
which substrate state governed it
whether it has been altered
whether it can be replayed

Minimum conceptual contract:

DecisionPacket
├── decision_id
├── verdict
├── reason
├── policy_id
├── policy_version
├── governance_hash
├── input_hash
├── evidence_hash
├── execution_trace
├── runtime_identity
├── substrate_identity
├── contract_version
├── timestamp
├── replay_identity
└── signature

The schema SHALL be:

versioned
canonical
backwards-aware
independently verifiable
documented
testable
migration-controlled

DecisionPacket verification SHALL NOT require trust in the originating AI model.

9. GATE 4 — SOFTWARE SUPPLY CHAIN

The provenance chain SHALL extend beyond runtime decisions into the software itself.

Every production release SHALL produce:

SOURCE COMMIT
      ↓
BUILD ENVIRONMENT
      ↓
DEPENDENCY GRAPH
      ↓
TOOLCHAIN
      ↓
BINARY
      ↓
CONTAINER
      ↓
DEPLOYMENT
      ↓
RUNTIME IDENTITY
      ↓
DECISIONPACKET

Required release artifacts:

signed iter binaries
signed container images
SCG substrate artifact
Rust SDK
Python SDK
schemas
SBOM
checksums
build provenance
release manifest
migration manifest
signatures

Release artifacts SHALL be traceable to source.

Production artifacts SHALL NOT be manually assembled.

Builds SHALL be reproducible to the maximum practical extent and generated through controlled CI.

10. GATE 5 — DEPLOYMENT AND OPERABILITY

A customer SHALL be capable of deploying the product without the architect manually reconstructing the environment.

Primary deployment target:

Azure AKS

Deployment SHALL be automated, versioned, repeatable, and idempotent.

Target operational experience:

DEPLOY
  ↓
VERIFY
  ↓
READY

iter API                HEALTHY
SCG substrate           HEALTHY
policy registry         VERIFIED
audit persistence       HEALTHY
governance state        VERIFIED
replay                   ENABLED
telemetry                HEALTHY

Required observability:

OpenTelemetry traces
structured logs
metrics
correlation IDs
decision latency
substrate evaluation latency
policy violations
authorization failures
replay failures
hash mismatches
degraded evaluations
DecisionPacket verification failures
11. FAILURE INJECTION

The production system SHALL be tested under deliberate degradation.

Required scenarios:

terminate SCG during evaluation
terminate iter during execution
corrupt execution trace
mutate DecisionPacket
present stale policy
present unsupported contract
remove required evidence
interrupt database connectivity
rotate credentials during operation
replay against incompatible runtime
send malformed MCP/API traffic
exhaust resource limits
restart workloads during active evaluation

The invariant is:

Loss of confidence SHALL result in loss of authority, never fabricated certainty.

Fail-open behavior on authoritative governance paths is prohibited.

12. GATE 6 — CUSTOMER DISTRIBUTION
Customer receives

iter Enterprise Runtime

Including:

runtime
API
MCP interface
Rust SDK
Python SDK
policy interface
DecisionPacket specification
verification CLI
replay CLI
audit export
deployment package
observability integration
operational documentation
Customer does not automatically receive
SCG source
proprietary graph algorithms
internal simulation architecture
complete connectome implementation
proprietary substrate mechanisms

The commercial interface SHALL depend upon a stable versioned SCG contract rather than SCG implementation details.

SCG remains the proprietary substrate.

iter remains the product surface.

13. REQUIRED DOCUMENTATION

The following are release artifacts, not optional supporting material.

DOC-001

Architecture and Trust Boundaries

DOC-002

Threat Model

DOC-003

Determinism Specification

DOC-004

DecisionPacket and Verification Specification

DOC-005

Deployment and Operations Guide

DOC-006

Governance and Compliance Control Matrix

Controls should map, where applicable, against:

NIST AI RMF
NIST SSDF
SOC 2
ISO 27001
HIPAA
FHIR

Compliance SHALL NOT be asserted solely from architectural alignment.

Claims require evidence.

14. RELEASE LADDER
v0.9 — INTERNAL RELEASE CANDIDATE

Required:

Gate 0 complete
zero P0 correctness defects
production path integration tests
replay correctness established
v0.9.5 — DETERMINISM RELEASE CANDIDATE

Required:

Gate 1 complete
cross-platform determinism proven
mutation suite operational
adversarial testing operational
determinism certification generated automatically
v1.0-PP — PRIVATE PREVIEW

Target:

1–3 external design partners

Required:

authentication
RBAC
tenant isolation
deployment automation
observability
customer documentation
support procedures
upgrade procedure
rollback procedure

Private Preview exists to discover environmental assumptions not represented by internal testing.

v1.0 — ENTERPRISE GA

Release SHALL occur only when an external organization can independently:

INSTALL
INTEGRATE
EXECUTE
VERIFY
REPLAY
AUDIT
UPGRADE
ROLL BACK

without architect intervention.

15. DEFINITION OF DONE

SCG/iter crosses the product boundary when all of the following statements are TRUE:

Clean build passes.
Production configuration passes.
Determinism certification passes.
Adversarial suite passes.
Tampering is detected.
Missing evidence fails closed.
Unsupported contracts fail closed.
Replay is authoritative.
DecisionPackets independently verify.
Authentication is enforced.
Authorization is enforced.
Tenant boundaries are enforced.
Production secrets are managed.
Audit records are tamper evident.
Release artifacts are signed.
SBOM is generated.
Build provenance is retained.
Deployment is automated.
Deployment is idempotent.
Rollback is tested.
Upgrade is tested.
Failure injection passes.
Telemetry covers critical execution paths.
Documentation reflects actual implementation.
Compliance mappings reference actual controls.
A clean environment can deploy the product.
An external operator can operate the product.
An external auditor can reconstruct a governed decision.
A historical decision can be replayed against its authoritative evidence.
Private Preview completes without unresolved P0/P1 product defects.
Release authority signs v1.0.

Until these conditions are satisfied:

THE SYSTEM IS RELEASE CANDIDATE INFRASTRUCTURE, NOT ENTERPRISE GA.

16. DEFERRED UNTIL POST-v1.0

The following SHALL NOT block commercialization:

generalized cognitive architecture expansion
visual connectome tooling
advanced graph visualization
autonomous learning
broad plugin marketplace
large integration catalog
additional model-provider proliferation
sophisticated administrative GUI
speculative SCG research capabilities

These may resume after the commercial primitive is stable.

17. BLACK SWAN CONTROLS
BS-01 — CROSS-ENVIRONMENT NONDETERMINISM

Risk: Identical governed executions produce divergent canonical identities across architectures or environments.

Mitigation: Cross-platform deterministic replay becomes a release-blocking CI invariant.

BS-02 — CRYPTOGRAPHICALLY VALID, SEMANTICALLY WRONG

Risk: The system perfectly attests to execution against an incorrectly defined policy.

Mitigation: Separate integrity verification from semantic policy validation. Require policy conformance tests, version control, approval provenance, and authorized policy releases.

BS-03 — SCG/iter OPERATIONAL COUPLING

Risk: iter becomes dependent upon SCG implementation details, expanding the customer-facing attack surface and preventing independent evolution.

Mitigation: Freeze a narrow, explicit, versioned substrate contract. iter consumes the contract. It SHALL NOT depend upon SCG internals.

18. ENGINEERING PRIORITY ORDER

Until Enterprise GA:

P0  CORRECTNESS
 ↓
P1  DETERMINISM
 ↓
P2  SECURITY
 ↓
P3  EVIDENCE / ATTESTATION
 ↓
P4  DISTRIBUTION
 ↓
P5  OPERABILITY
 ↓
P6  PRIVATE PREVIEW
 ↓
P7  ENTERPRISE GA
 ↓
P8  NEW CAPABILITY

Feature novelty sits below product integrity.

19. APEX GOVERNING RULE

Every proposed change to SCG or iter SHALL answer:

1. Which release gate does this close?

2. Which invariant does this strengthen?

3. What evidence proves completion?

4. Can the result be independently reproduced?

If those questions cannot be answered, the work does not belong on the v1.0 critical path.

FINAL DIRECTIVE

SCG and iter have crossed the architectural feasibility threshold.

The next objective is not to demonstrate that the systems are sophisticated.

The next objective is to demonstrate that they are trustworthy without their creator present.

Freeze expansion.

Close correctness.

Certify determinism.

Harden the trust boundary.

Productize the DecisionPacket.

Sign the supply chain.

Automate deployment.

Attack the system.

Deploy with external operators.

Prove replay.

Prove auditability.

Then release.

APEX END STATE:

A customer can purchase iter, deploy it into their environment, place an AI system behind its governance boundary, receive independently verifiable DecisionPackets backed by SCG, and reconstruct why an authoritative decision occurred without trusting the model, the operator, or an undocumented assertion from Only SG Solutions.

That is the line.

Cross it before expanding the frontier.



OPERATIONALIZATION

This directive governs product release readiness. It does not replace the SCG
runtime-governance authority at governance/SCG_Governance_v1.0.md, its
integrity hash, or any stricter security control. If an APEX rule conflicts with
that manifest, the manifest controls; every APEX rule is subordinate to it.

The canonical 30-control release matrix is maintained in the customer-facing
iter repository at productization/APEX_RELEASE_MATRIX_V1.json. The SCG copy of
this directive is a controlled mirror and must remain byte-identical to the
iter copy.

Existing production-ready, enterprise-ready, acquisition-grade, or general
availability statements are subordinate to this directive. Until all 30
controls report PASS against the exact iter and SCG release commits, the
combined system is release-candidate infrastructure, not Enterprise GA.

Evidence semantics are strict: missing evidence is FAIL; stale evidence is
FAIL; skipped checks are FAIL; and a narrative assertion without its required
artifact is FAIL.
