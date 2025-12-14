# Contributing to Iter

Thank you for your interest in contributing to Iter!

## Public Surface Guidelines

Public surfaces must not reference internal substrate or governance names. All user-facing content—including README, documentation, demos, examples, marketplace listings, CLI help text, and Rustdoc comments on public APIs—should use "Iter" branding and approved terminology only. Internal crate names, workflow identifiers, and test baselines may retain their existing naming conventions.

## Getting Started

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run the test suite: `cargo test`
5. Submit a pull request

## Code Style

- Follow existing patterns in the codebase
- Run `cargo fmt` before committing
- Ensure `cargo clippy` passes without warnings
