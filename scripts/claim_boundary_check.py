"""Fail closed on evaluator-facing documentation claims that exceed current proof."""

from __future__ import annotations

import pathlib
import re
import sys


ROOT = pathlib.Path(__file__).resolve().parents[1]
CHECKED_PATHS = [
    "README.md",
    "docs/CLAIM_BOUNDARY.md",
    "docs/ITER_SCG_CONTRACT.md",
    "docs/ITER_VERIFY_SPEC.md",
    "docs/MCP_API.md",
    "docs/RELEASE_NOTES_PUBLIC_v0.3.0.md",
    "docs/iter-external-spec-v1.md",
    "docs/iter-mcp-contract-v1-outline.md",
    "docs/iter-mcp-tools-phase0-classification.md",
    "docs/iter-operator-tools.md",
]

SCOPING_WORDS = re.compile(
    r"\b(not claimed|not claim|must not|not an active|not active|deferred|before claiming|prohibited|same-binary|same binary|non-authoritative|out of scope|within scope)\b",
    re.IGNORECASE,
)

RULES = [
    (
        re.compile(r"\bcross[- ]platform\b", re.IGNORECASE),
        "cross-platform claims must be explicitly scoped or deferred",
    ),
    (
        re.compile(r"\bobservability[- ]complete\b", re.IGNORECASE),
        "observability-complete is not an active product claim",
    ),
    (
        re.compile(r"\bdistributed coordination\b", re.IGNORECASE),
        "distributed coordination is not an active product claim",
    ),
    (
        re.compile(r"\ball invariants (hold )?simultaneously\b", re.IGNORECASE),
        "full invariant simultaneity is deferred",
    ),
    (
        re.compile(r"\bperfect replay\b", re.IGNORECASE),
        "perfect replay overstates the current same-binary replay guarantee",
    ),
]


def main() -> int:
    failures: list[str] = []

    for relative in CHECKED_PATHS:
        path = ROOT / relative
        if not path.exists():
            failures.append(f"{relative}:0: required file is missing")
            continue
        for line_no, line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
            for pattern, message in RULES:
                if not pattern.search(line):
                    continue
                if SCOPING_WORDS.search(line):
                    continue
                failures.append(f"{relative}:{line_no}: {message}: {line.strip()}")

    if failures:
        print("CLAIM_BOUNDARY_VIOLATION")
        for failure in failures:
            print(failure)
        return 1

    print("CLAIM_BOUNDARY_PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
