# Iter Server Security

> Visitor-facing security posture for the Iter Server MCP boundary.

---

## Threat model (high level)

Assume an adversary who can:

- send arbitrary MCP (JSON-RPC) requests
- attempt prompt injection or data exfiltration through responses
- attempt to coerce the server into returning sensitive details

---

## Assets protected

The server is designed to protect non-public execution details and other sensitive data, including (non-exhaustive):

- non-public topology/structure
- non-public policy inputs and scoring signals
- non-public energy/distribution details
- debug traces and implementation details
- non-public audit-chain details

---

## Boundary guarantees

Iter Server aims to provide:

- a small, explicitly enumerated tool surface
- strict input validation at the boundary
- sanitized responses that do not disclose sensitive internals
- sanitized error messages (no stack traces / no internal dumps)

The precise sanitization rules and attack-resistance measures are intentionally not documented publicly.

---

## Reporting security issues

If you believe you have found a security issue:

- email: security@onlysgsolutions.com
- do not open a public issue for vulnerabilities

