# Iter Attack Surface

> Visitor-facing summary of what the Iter Server exposes and what it intentionally withholds.

---

## What is exposed (high level)

Iter Server exposes a small, tool-oriented surface via MCP. Responses are intended to be useful for callers while remaining safe to share with untrusted clients.

Typical outputs include:

- IDs and user-provided values needed to continue a session
- high-level status indicators for governance/health
- audit and integrity summaries (for example, checksums)

---

## What is intentionally not exposed

The server is designed to **not** expose information that could be used to reconstruct internal execution mechanisms or internal state, including (non-exhaustive):

- internal topology/structure
- raw policy inputs or internal scoring vectors
- internal energy/distribution details
- debug traces or implementation details
- full audit-chain internals

---

## Sanitization posture

All outbound responses are sanitized before being returned to the caller.

To preserve security and prevent reconstruction, the exact filtering rules and attack-resistance measures are **not documented publicly**.

For threat model and disclosure guidance, see `docs/SECURITY.md`.

