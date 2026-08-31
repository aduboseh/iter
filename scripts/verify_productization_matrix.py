#!/usr/bin/env python3
"""Execute the APEX SCG/iter v1 release matrix.

A control passes only when every declared check passes. Missing commands,
repositories, evidence, or stale evidence are failures. There is no pending or
skipped state.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import time
from typing import Any

MATRIX_SCHEMA = "apex-productization-matrix/v1"
EVIDENCE_SCHEMA = "apex-productization-evidence/v1"
EXPECTED_CONTROL_COUNT = 30
ALLOWED_CHECK_TYPES = {"command", "path_exists", "regex", "evidence"}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    script_root = Path(__file__).resolve().parents[1]
    parser.add_argument(
        "--matrix",
        type=Path,
        default=script_root / "productization" / "APEX_RELEASE_MATRIX_V1.json",
    )
    parser.add_argument("--iter-root", type=Path, default=script_root)
    parser.add_argument("--scg-root", type=Path, default=script_root.parent / "SCG")
    parser.add_argument(
        "--evidence-dir",
        type=Path,
        default=script_root / "productization" / "evidence",
    )
    parser.add_argument("--control", action="append", default=[])
    parser.add_argument("--validate-only", action="store_true")
    parser.add_argument("--allow-failures", action="store_true")
    parser.add_argument("--report", type=Path)
    return parser.parse_args()


def load_json(path: Path) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError as exc:
        raise ValueError(f"missing JSON file: {path}") from exc
    except json.JSONDecodeError as exc:
        raise ValueError(f"invalid JSON in {path}: {exc}") from exc
    if not isinstance(value, dict):
        raise ValueError(f"expected JSON object in {path}")
    return value


def require_repo(control_id: str, check: dict[str, Any]) -> None:
    if check.get("repo") not in {"iter", "scg"}:
        raise ValueError(f"{control_id}: check repo must be 'iter' or 'scg'")


def validate_matrix(matrix: dict[str, Any]) -> list[dict[str, Any]]:
    if matrix.get("schema_version") != MATRIX_SCHEMA:
        raise ValueError(
            f"matrix schema must be {MATRIX_SCHEMA!r}, got "
            f"{matrix.get('schema_version')!r}"
        )
    controls = matrix.get("controls")
    if not isinstance(controls, list):
        raise ValueError("matrix controls must be an array")
    if matrix.get("control_count") != EXPECTED_CONTROL_COUNT:
        raise ValueError(
            f"control_count must be {EXPECTED_CONTROL_COUNT}, got "
            f"{matrix.get('control_count')!r}"
        )
    if len(controls) != EXPECTED_CONTROL_COUNT:
        raise ValueError(
            f"matrix must contain exactly {EXPECTED_CONTROL_COUNT} controls, "
            f"got {len(controls)}"
        )

    ids: set[str] = set()
    for index, control in enumerate(controls):
        if not isinstance(control, dict):
            raise ValueError(f"control {index} must be an object")
        control_id = control.get("id")
        if (
            not isinstance(control_id, str)
            or re.fullmatch(r"G[0-6]-\d{2}", control_id) is None
        ):
            raise ValueError(f"invalid control id at index {index}: {control_id!r}")
        if control_id in ids:
            raise ValueError(f"duplicate control id: {control_id}")
        ids.add(control_id)
        if control.get("gate") != int(control_id[1]):
            raise ValueError(f"{control_id}: gate does not match control id")
        for field in ("system", "title", "invariant", "remediation"):
            if not isinstance(control.get(field), str) or not control[field].strip():
                raise ValueError(f"{control_id}: missing non-empty {field}")
        checks = control.get("checks")
        if not isinstance(checks, list) or not checks:
            raise ValueError(f"{control_id}: checks must be a non-empty array")
        for check_index, check in enumerate(checks):
            if not isinstance(check, dict):
                raise ValueError(f"{control_id}: check {check_index} must be an object")
            check_type = check.get("type")
            if check_type not in ALLOWED_CHECK_TYPES:
                raise ValueError(f"{control_id}: unsupported check type {check_type!r}")
            if check_type == "command":
                argv = check.get("argv")
                if (
                    not isinstance(argv, list)
                    or not argv
                    or not all(isinstance(item, str) and item for item in argv)
                ):
                    raise ValueError(f"{control_id}: command argv must be strings")
                require_repo(control_id, check)
            elif check_type in {"path_exists", "regex"}:
                require_repo(control_id, check)
                if not isinstance(check.get("path"), str) or not check["path"]:
                    raise ValueError(f"{control_id}: {check_type} requires path")
                if check_type == "regex" and not isinstance(check.get("pattern"), str):
                    raise ValueError(f"{control_id}: regex requires pattern")
            elif check_type == "evidence":
                if not isinstance(check.get("file"), str) or not check["file"]:
                    raise ValueError(f"{control_id}: evidence requires file")
    return controls


def git_head(root: Path) -> str | None:
    if not root.is_dir():
        return None
    result = subprocess.run(
        ["git", "rev-parse", "HEAD"],
        cwd=root,
        text=True,
        capture_output=True,
        check=False,
    )
    return result.stdout.strip() if result.returncode == 0 else None


def command_check(
    check: dict[str, Any], roots: dict[str, Path]
) -> tuple[bool, str, float]:
    root = roots[check["repo"]]
    if not root.is_dir():
        return False, f"repository root missing: {root}", 0.0
    argv = check["argv"]
    timeout_seconds = int(check.get("timeout_seconds", 900))
    env = os.environ.copy()
    extra_env = check.get("env", {})
    if not isinstance(extra_env, dict) or not all(
        isinstance(key, str) and isinstance(value, str)
        for key, value in extra_env.items()
    ):
        return False, "command env must contain string keys and values", 0.0
    env.update(extra_env)

    started = time.monotonic()
    try:
        result = subprocess.run(
            argv,
            cwd=root,
            env=env,
            text=True,
            capture_output=True,
            timeout=timeout_seconds,
            check=False,
        )
    except FileNotFoundError:
        return False, f"command not found: {argv[0]}", time.monotonic() - started
    except subprocess.TimeoutExpired:
        return (
            False,
            f"timed out after {timeout_seconds}s: {' '.join(argv)}",
            time.monotonic() - started,
        )

    elapsed = time.monotonic() - started
    combined = (result.stdout + "\n" + result.stderr).strip()
    tail = combined[-4000:] if combined else "<no output>"
    expected_exit = int(check.get("expected_exit", 0))
    if result.returncode != expected_exit:
        return (
            False,
            f"exit {result.returncode}, expected {expected_exit}: "
            f"{' '.join(argv)}\n{tail}",
            elapsed,
        )
    pattern = check.get("output_pattern")
    if pattern and re.search(pattern, combined, re.MULTILINE) is None:
        return False, f"required output pattern not found: {pattern!r}", elapsed
    if len(argv) >= 2 and argv[0] == "cargo" and argv[1] == "test":
        passed_counts = [
            int(value)
            for value in re.findall(r"test result: ok\. (\d+) passed", combined)
        ]
        if not passed_counts or sum(passed_counts) == 0:
            return False, "cargo test executed zero tests", elapsed
    return True, f"exit {result.returncode}: {' '.join(argv)}", elapsed


def path_check(
    check: dict[str, Any], roots: dict[str, Path]
) -> tuple[bool, str, float]:
    path = roots[check["repo"]] / check["path"]
    if not path.exists():
        return False, f"missing path: {path}", 0.0
    return True, f"path exists: {path}", 0.0


def regex_check(
    check: dict[str, Any], roots: dict[str, Path]
) -> tuple[bool, str, float]:
    path = roots[check["repo"]] / check["path"]
    if not path.is_file():
        return False, f"missing file: {path}", 0.0
    text = path.read_text(encoding="utf-8")
    matched = re.search(check["pattern"], text, re.MULTILINE) is not None
    if bool(check.get("negate", False)):
        matched = not matched
    return matched, f"regex {'passed' if matched else 'failed'}: {path}", 0.0


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def evidence_check(
    control_id: str,
    check: dict[str, Any],
    evidence_dir: Path,
    heads: dict[str, str | None],
) -> tuple[bool, str, float]:
    path = evidence_dir / check["file"]
    try:
        evidence = load_json(path)
    except ValueError as exc:
        return False, str(exc), 0.0

    failures: list[str] = []
    if evidence.get("schema_version") != EVIDENCE_SCHEMA:
        failures.append(f"schema_version must be {EVIDENCE_SCHEMA}")
    if evidence.get("control_id") != control_id:
        failures.append(f"control_id must be {control_id}")
    if evidence.get("result") != "PASS":
        failures.append("result must be PASS")

    subjects = evidence.get("subject_commits")
    if not isinstance(subjects, dict):
        failures.append("subject_commits must be an object")
    else:
        for repo in ("iter", "scg"):
            expected = heads[repo]
            actual = subjects.get(repo)
            if expected is None:
                failures.append(f"cannot resolve current {repo} commit")
            elif actual != expected:
                failures.append(
                    f"{repo} evidence commit {actual!r} does not match {expected}"
                )

    artifacts = evidence.get("artifacts")
    if not isinstance(artifacts, list) or not artifacts:
        failures.append("artifacts must be a non-empty array")
    else:
        for index, artifact in enumerate(artifacts):
            if not isinstance(artifact, dict):
                failures.append(f"artifact {index} must be an object")
                continue
            artifact_path = artifact.get("path")
            expected_hash = artifact.get("sha256")
            if not isinstance(artifact_path, str) or not artifact_path:
                failures.append(f"artifact {index} path missing")
                continue
            target = (evidence_dir / artifact_path).resolve()
            try:
                target.relative_to(evidence_dir.resolve())
            except ValueError:
                failures.append(f"artifact {index} escapes evidence directory")
                continue
            if not target.is_file():
                failures.append(f"artifact {index} missing: {target}")
                continue
            actual_hash = sha256_file(target)
            if expected_hash != actual_hash:
                failures.append(
                    f"artifact {index} hash mismatch: {actual_hash} != {expected_hash}"
                )

    commands = evidence.get("commands")
    if not isinstance(commands, list) or not commands:
        failures.append("commands must be a non-empty array")

    if failures:
        return False, "; ".join(failures), 0.0
    return True, f"commit-bound evidence verified: {path}", 0.0


def directive_mirror_check(iter_root: Path, scg_root: Path) -> tuple[bool, str]:
    iter_path = iter_root / "APEX_PRODUCTIZATION_V1.md"
    scg_path = scg_root / "APEX_PRODUCTIZATION_V1.md"
    if not iter_path.is_file():
        return False, f"missing iter directive: {iter_path}"
    if not scg_path.is_file():
        return False, f"missing SCG directive mirror: {scg_path}"
    iter_hash = sha256_file(iter_path)
    scg_hash = sha256_file(scg_path)
    if iter_hash != scg_hash:
        return False, f"directive mirror mismatch: iter={iter_hash}, scg={scg_hash}"
    return True, f"directive mirror byte-identical: {iter_hash}"


def write_report(path: Path, report: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        json.dumps(report, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )


def main() -> int:
    args = parse_args()
    try:
        matrix = load_json(args.matrix)
        controls = validate_matrix(matrix)
    except ValueError as exc:
        print(f"MATRIX INVALID: {exc}", file=sys.stderr)
        return 2

    selected = set(args.control)
    if selected:
        known = {control["id"] for control in controls}
        unknown = sorted(selected - known)
        if unknown:
            print(f"unknown controls: {', '.join(unknown)}", file=sys.stderr)
            return 2
        controls = [control for control in controls if control["id"] in selected]

    print(
        f"MATRIX VALID: {len(matrix['controls'])} controls, "
        f"schema={MATRIX_SCHEMA}"
    )
    if args.validate_only:
        return 0

    roots = {
        "iter": args.iter_root.resolve(),
        "scg": args.scg_root.resolve(),
    }
    heads = {repo: git_head(root) for repo, root in roots.items()}
    mirror_ok, mirror_detail = directive_mirror_check(roots["iter"], roots["scg"])
    print(f"{'PASS' if mirror_ok else 'FAIL'} DIRECTIVE-MIRROR {mirror_detail}")

    results: list[dict[str, Any]] = []
    for control in controls:
        check_results: list[dict[str, Any]] = []
        for check in control["checks"]:
            check_type = check["type"]
            if check_type == "command":
                passed, detail, elapsed = command_check(check, roots)
            elif check_type == "path_exists":
                passed, detail, elapsed = path_check(check, roots)
            elif check_type == "regex":
                passed, detail, elapsed = regex_check(check, roots)
            else:
                passed, detail, elapsed = evidence_check(
                    control["id"], check, args.evidence_dir.resolve(), heads
                )
            check_results.append(
                {
                    "type": check_type,
                    "passed": passed,
                    "detail": detail,
                    "elapsed_seconds": round(elapsed, 3),
                }
            )
            print(
                f"  {'PASS' if passed else 'FAIL'} "
                f"{control['id']} {check_type}: {detail}"
            )
        passed = all(result["passed"] for result in check_results)
        results.append(
            {
                "id": control["id"],
                "gate": control["gate"],
                "system": control["system"],
                "title": control["title"],
                "status": "PASS" if passed else "FAIL",
                "checks": check_results,
            }
        )
        print(f"{'PASS' if passed else 'FAIL'} {control['id']} {control['title']}")

    pass_count = sum(result["status"] == "PASS" for result in results)
    fail_count = len(results) - pass_count
    all_passed = mirror_ok and fail_count == 0
    report = {
        "schema_version": "apex-productization-report/v1",
        "directive_id": matrix["directive_id"],
        "subject_commits": heads,
        "directive_mirror": {
            "status": "PASS" if mirror_ok else "FAIL",
            "detail": mirror_detail,
        },
        "summary": {
            "status": "PASS" if all_passed else "FAIL",
            "passed": pass_count,
            "failed": fail_count,
            "selected_controls": len(results),
        },
        "controls": results,
    }
    if args.report:
        write_report(args.report, report)
        print(f"REPORT {args.report.resolve()}")

    print(
        f"APEX RELEASE MATRIX: {'PASS' if all_passed else 'FAIL'} "
        f"({pass_count} passed, {fail_count} failed)"
    )
    if all_passed or args.allow_failures:
        return 0
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
