# Free-First Local Verification

Azure/AKS verification is deferred at the owner's request. This route runs on
existing hardware and does not provision cloud resources or dispatch hosted CI.
Do not publish private SCG code or logs to obtain public CI minutes. Private CI
must stay within verified included usage; otherwise run locally.

## Real-Process Smoke Test

Prerequisites: Python 3.11+, Git, Rust 1.93.0 with the host compiler/linker, and
local Iter and SCG checkouts. The verifier runs locked builds before testing; it
does not trust an arbitrary pre-existing binary. Package downloads may be needed.

From Iter, with SCG as a sibling:

```powershell
python scripts/verify_local_stack.py --scg-root ../SCG --output ../local-stack-run-001
python -m unittest discover -s tests -p test_local_stack.py
```

The same commands work in a Linux shell. Choose a new output directory outside
both repositories on each run. Existing evidence directories are never replaced.
Run from clean commits for reproducible source attribution; dirty development
runs are explicitly identified as such in the report.

The smoke test checks:

- Real SCG process startup on an ephemeral loopback port.
- Rejection of unauthenticated snapshot requests when bearer auth is configured.
- Real MCP registration and `decision.check` through Iter's `scg-backed` runtime.
- Governance and live snapshot binding, nonempty trace, and durable audit writes.
- Identical complete packets from two fresh Iter processes on unchanged SCG state.
- Offline `iter-cli replay` success and rejection of a checksum-tampered packet.
- Explicit Iter runtime error after the gateway process is stopped, with no stub fallback.

The test uses an empty SCG cluster, one fixed proposal, and ephemeral test-only
credentials. It is a smoke test, not comprehensive graph, load, recovery, tenant,
TLS, independent-signature, or cross-platform determinism certification.

`report.json` records executed build/replay commands, exits, source commits and
cleanliness, lockfile hashes, toolchain/platform, binary hashes, transcript hashes,
and failures. It uses `iter-scg-local-smoke/v1` and always sets
`release_certification: false`. It cannot satisfy the APEX evidence schema.
Runtime credentials are not written to the report or MCP transcripts. Logs may
still contain private paths and state; keep the output local and review it before
sharing. Build/test timeouts and child cleanup are bounded.

## Container Validation

The Dockerfiles are local/RC packaging definitions, not production certification.
They pin base-image digests and use locked Rust builds. Debian package repositories
are not snapshot-pinned, so bit-reproducible images are not claimed. SCG's image
enables metrics but excludes Swagger downloads and OTLP export.

With a working local container engine, build from each repository root:

```text
docker build --tag iter:local .
docker build --tag scg:local --file build/Dockerfile .
```

Iter is a stdio MCP server, not an HTTP service. Always select its runtime mode;
the image deliberately has no implicit demo default. Offline tools are included:

```text
docker run --rm --network none --entrypoint iter-cli iter:local --version
docker run --rm --network none -i iter:local --json-only --runtime-mode=governed-local
```

For the two-container seam, SCG can use `--network none` with loopback defaults
and Iter can share that container's network namespace using
`--network container:<scg-container>`. This keeps `SCG_ENDPOINT` on loopback and
does not weaken Iter's HTTPS requirement for non-loopback endpoints. Use ephemeral
test credentials and local volumes for ledgers; do not publish service ports for
this local test. Local Kubernetes is a later step after container builds work.

## Certification Boundary

The 30-control matrix is unchanged. Missing evidence remains FAIL. In particular,
local tests do not satisfy AKS determinism (G1-04), AKS operations (G5-01), or
independent operator acceptance. Resume Azure-specific validation only after the
owner reauthorizes it. A passing smoke test does not authorize Enterprise GA.
