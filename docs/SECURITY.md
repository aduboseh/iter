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

The server is designed to protect internal execution details and other sensitive data, including (non-exhaustive):

- internal topology/structure
- internal policy inputs and scoring signals
- internal energy/distribution details
- debug traces and implementation details
- internal audit-chain details

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

