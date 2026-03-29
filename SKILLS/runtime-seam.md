WO-ITER-RUNTIME-001
Phase 0 Recon
Date: 2026-03-27
Repo: Iter
Branch: codex/wo-iter-runtime-001
Base commit: 99b047cb6cf6a91b3a9fba3f2c7b7667fee26f34

1. Runtime construction at server startup
file: src/main.rs
line: 58
current_state: The live STDIO server boot path imports and constructs `StubRuntime` directly, then routes all requests through `handle_stub_request`.
insertion_point: Replace `StubRuntime`-specific boot and dispatch with a generic runtime boundary only if a real non-stub authoritative runtime exists; otherwise downgrade claims and banner here.

2. GovernedRuntime structure
file: src/governed.rs
line: 30
current_state: `GovernedRuntime` exists, implements authoritative governance semantics, but wraps `StubRuntime` as its substrate state carrier.
insertion_point: Any authoritative promotion must first remove or fence this `StubRuntime` dependency, or explicitly scope GovernedRuntime as policy-authoritative but substrate-stub-backed.

3. Runtime trait and implementors
file: src/runtime.rs
line: 244
current_state: The canonical runtime trait is `GovernanceRuntime`; current implementors are `GovernedRuntime` in `src/governed.rs:135` and `StubRuntime` via `GovernanceRuntimeTrait` in `src/substrate/stub.rs:835`.
insertion_point: Server dispatch should target this trait boundary, but currently does not for `decision.check`; that redirection belongs here and in the MCP dispatch path.

4. MCP dispatch for decision.check
file: src/main.rs
line: 489
current_state: There is no separate `src/mcp_handler.rs`; `handle_stub_tool(...) -> serde_json::Value` handles MCP dispatch inline, and `decision.check` is processed at `src/main.rs:580` by calling `runtime.evaluate_governance(&proposal)` directly on `StubRuntime`.
insertion_point: Replace the inline stub-specific `decision.check` path with trait-based runtime dispatch here if and only if the authoritative runtime can be constructed safely.

5. DecisionPacket definition and live emission status
file: src/audit/mod.rs
line: 34
current_state: `DecisionPacket` fields are `iter_build_hash`, `substrate_build_hash`, `tick`, `energy`, `reasoning`, `learning`, `policy`, `permit_hash`, `economics_hash`, `evaluated_rules`, and `checksum`; it has neither `governance_hash` nor `execution_trace`, and it is emitted in governed/test paths but not on the live server `decision.check` path.
insertion_point: Any authoritative runtime closure requires packet schema expansion here plus live-server emission wiring from MCP dispatch.

6. SCG runtime connection file
file: src/substrate_runtime.rs
line: n/a
current_state: `src/substrate_runtime.rs` does not exist in the public repo.
insertion_point: A real SCG-backed runtime connector would need to be introduced or restored before full authoritative seam closure is possible.

7. Canonical governance hash
file: governance/governance.hash
line: 1
current_state: Canonical governance hash is `327D6A6BC2956507DD77A1C1C5EFD78C67E190C5BC29DD04202CF62BE3D0656E`.
insertion_point: Any future governed packet binding or boot-time authority validation must load and bind this value here.

8. Existing fail-closed SCG unreachability logic
file: src/
line: n/a
current_state: No SCG-specific fail-closed connectivity or unreachability logic exists in the current public Iter source tree.
insertion_point: Explicit SCG transport failure handling must be added at the future substrate connector boundary; there is nothing to hook into today.

ASCII call chain
main.rs:28 main
  -> main.rs:46 run_stdio_server
    -> main.rs:62 StubRuntime::new
      -> main.rs:100 / 110 handle_stub_request
        -> main.rs:443 handle_stub_tool
          -> main.rs:580 decision.check
            -> src/substrate/stub.rs evaluate_governance
              -> GovernanceEvaluation JSON text

Compile status
- `GovernedRuntime` currently compiles and links in the crate and test suite.
- It is orphaned from the live server boot path.

Entanglement assessment
- Status: ENTANGLED
- Reason 1: server boot path is hard-wired to `StubRuntime`
- Reason 2: MCP dispatch signatures are `StubRuntime`-specific
- Reason 3: `decision.check` bypasses the `GovernanceRuntime` trait and returns stub evaluation output directly
- Reason 4: `DecisionPacket` schema lacks the fields required by the requested seam program
- Reason 5: no `src/substrate_runtime.rs` or equivalent SCG connector exists in the public repo

Execution decision
- Full WO-ITER-RUNTIME-001 seam closure is not safely completable in one truthful PR from the current public repo state.
- Correct action: execute downgrade protocol now, then split future runtime closure into:
  - 001A: packet/schema and trait-path normalization
  - 001B: real SCG-backed boot/runtime integration once the connector exists

DOWNGRADE EXECUTED: 2026-03-27 — full wiring deferred, live-server claims reduced to match the current stub-mode boot path.

001B DOWNGRADE: 2026-03-27
Reason: no stable public SCG connector contract exists in the Iter repo, the public Cargo manifest carries no scg-* dependency, SCG exposes simulation/energy APIs but no governance packet or execution_trace response contract, and the private iter-internal facade is not a usable public seam.
ScgBacked mode added as fail-closed stub.
Real connector deferred pending: an inspectable SCG-backed governance interface with stable request/response signatures, governance identity binding, and SCG-originated deterministic execution_trace data.

001B STATUS: 2026-03-27
ScgRuntime connector: deferred — no stable public SCG interface or response contract
ScgBacked mode: fail-closed stub
Trace-level replay test: deferred — no SCG-originated execution_trace surface exists
INV-RUNTIME-001 CI guard: pending
Canonical hash bound at boot: no — connector deferred, so no SCG-backed boot path exists

001B-RETRY STATUS: 2026-03-29
src/governance_connector.rs: created
ScgRuntime::connect(): implemented
Four fail-closed checks: all present
governance_hash boot load: yes
execution_trace from SCG: yes
AuditEvent.decision field: yes
replay_trace_is_identical test: passing
contract_version check: present
governance_hash mismatch check: present
verify_replay_id check: present
SCG↔Iter seam: CLOSED
