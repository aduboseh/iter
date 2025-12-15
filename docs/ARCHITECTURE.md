# Architecture

Iter uses a dual-build architecture:

- **`public_stub`** (default): Compiles without proprietary dependencies. Validates protocol types, governance invariants, and SDK surface.
- **`full_substrate`**: Requires proprietary SCG crates. Enables execution semantics, integration tests, and full MCP handler behavior.

## Certification Boundaries

See [ARCHITECTURE_BOUNDARY.md](../ARCHITECTURE_BOUNDARY.md) for what public vs. private CI certifies.

## Detailed Architecture

Execution architecture, substrate internals, and performance characteristics are documented privately. Available to partners, auditors, and acquirers under NDA.

Contact: architecture@onlysgsolutions.com
