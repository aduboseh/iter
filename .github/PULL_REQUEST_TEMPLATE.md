# Pull Request

## Summary

Provide a 1–3 sentence summary of what changed and why.

## Checklist (required)

- **Authorship:** All commits are authored by me and use a valid email. No anonymous or third-party commits.
- **Governance freeze:** This PR does not change maintainers, license, policies, or governance.
- **Public/private boundary:** No secrets, internal IDs, or non-public artifacts. Logs and examples are sanitized.
- **Iter consumption:** If this affects public surface (API, CLI, models, schemas), docs/examples are updated and migration impact is noted.
- **CI gates:** Typecheck, lint, security, and tests pass; coverage ≥ 80% and not reduced.
- **Audit signals:** Audit logs and tracing updated where behavior crosses auth, data, or side-effect boundaries (if applicable).

## Testing

List commands run, evidence, and results demonstrating the change works.

## Release Notes (user-visible changes)

None, or a one-line note per change.
