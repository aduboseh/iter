#!/usr/bin/env python3
"""Execute the APEX SCG/iter v1 release matrix.

A control passes only when every declared check passes. Missing commands,
repositories, evidence, or stale evidence are failures. There is no pending or
skipped state.
"""

from __future__ import annotations

import argparse
from collections.abc import Mapping
import hashlib
from http.client import HTTPException
import io
import json
import os
from pathlib import Path
import re
import stat
import subprocess
import sys
import time
from types import MappingProxyType
from typing import Any
from urllib.error import HTTPError
from urllib.parse import quote, urlsplit
from urllib.request import HTTPRedirectHandler, Request, build_opener
import zipfile
import zlib

MATRIX_SCHEMA = "apex-productization-matrix/v1.1"
EVIDENCE_SCHEMA = "apex-productization-evidence/v1"
DIRECTIVE_ID = "APEX-SCG-ITER-PROD-001"
AUTHORITY_DOCUMENT = "APEX_PRODUCTIZATION_V1.md"
AUTHORITY_AMENDMENT = "APEX_PRODUCTIZATION_GAP_CLOSURE_001.md"
DIRECTIVE_MIRRORS = (AUTHORITY_DOCUMENT, AUTHORITY_AMENDMENT)
EXPECTED_CONTROL_COUNT = 30
EXPECTED_CONTROL_CHECK_DIGESTS = {
    "G0-01": "4364a791a1a3e48c542c472254c417f0c6d94f6279005b706984a2b7b0cbf79d",
    "G0-02": "e366086fad6b7cd3a9effc2ba18715e3f4bba9993cfa683d94620e596add7019",
    "G0-03": "fb9199d2e024d7db94b7f34594d91f513234c401f6fb53b14a497cf324468266",
    "G0-04": "3173dd818e17dba46ef848c5bc3d83d46f98541ed760df91e33a4ba6ae6ee517",
    "G0-05": "802218a091d41d636292ae7bb147e5b588655ca9f913b0727c4bc838b31c845e",
    "G0-06": "71118f52a178f45421d9226fb3c53573b86f1c5eed86644c2395b75beb6abc1a",
    "G0-07": "66db7d3f8aa1973c4c7e26b3aa034e37098191e3f910e2a30de6dc4366971eea",
    "G0-08": "bcedd7adf6823474d9f4bfe3c07f4e049a53c3d6e8e26456134bea67e37ec585",
    "G0-09": "aef272532d4e2ff74c7cd6712e52e694d2fd1b79b9e313a3b92cff9505b86e1d",
    "G0-10": "8b5a34376c814aa901d7dc0b276cf75a5bbf0e978ba70d751e8e8b099f38e6c6",
    "G0-11": "28657cd805d5153c12ae272c745f84679f88bbf1970045499199b345d1d463c9",
    "G0-12": "3539a37d38ce3b44ab63fc9a4d264c208a1711240c5e4eb8fef90c8ced109fbe",
    "G0-13": "27601335bb9bb41ed2d3d975d5b527227348b25f25871945be2335d99bf04f0a",
    "G0-14": "3d73bf0be6f2607983403f38a489958f27bf78c774024a927dff96f50c7ba644",
    "G1-01": "e556dbc7e1718c34f1a4c21495d75a56d7c6ffc555bb38c817fef0e217fe7ef4",
    "G1-02": "005d22c92bf49329b20ccec6fc4fe559a04273132a67bf460a1c763c1924fbbf",
    "G1-03": "b61dbf78a9c6cdb1d8b1ebd6bc517737b69bdc2173e6d0c67e977c7e3cc0cfb3",
    "G1-04": "fdacf4a7ddde912075915382c6428438f1d2f48f144d160a9379b262504a1b64",
    "G1-05": "717fcce52b842794dc17859a6419bdbef3842c1ebfd2dfcd3c60115f85aa5720",
    "G1-06": "a5ce10fd07b4ba9605bed78f1d05d95f42bf2e02ede79347c9a7f48f39b04428",
    "G2-01": "acfece60a0d9b528f55a70c7dfa949e2342e83c267189217114fbb2a7c4b2ca4",
    "G2-02": "c5a85ac0aad99822763f9c69bbccaeecbd63988c2f1cc94e818658d9db8477fd",
    "G2-03": "9ccd0038e975a3dbfc86128f073c1c585addd902cf61373ed0956952aa15e071",
    "G2-04": "6fbc83be145a24a4b375c152af640ee3834e213b1c62e137f3cdf492bb9c43c1",
    "G3-01": "0d4b8b3f0464d573203594eb0d8a9518f6beeb01cb9aef2b626fd06203a5554f",
    "G3-02": "e61b5c4da54d2b58c9b79b76e66e594bfd56294710628db45d0695d6f703f69e",
    "G4-01": "1132b16be32ac4783c5d4d3ea9a0c2efe35c0018427697b3ef015344ad1cde1a",
    "G5-01": "9c115a1473a49639af84b4ffaf89e071069c898f0934039192fe9826818ff755",
    "G6-01": "f118fb677779e3d4ef4e2c61b240f95fff6d2af7ad659acebb022f953e25788e",
    "G6-02": "f6b335e7fded84e8359274f9333b160a30ea3a8eb09412a9044d029a834b2a11",
}
EXPECTED_CONTROL_IDS = frozenset(EXPECTED_CONTROL_CHECK_DIGESTS)
ALLOWED_CHECK_TYPES = {
    "command",
    "path_exists",
    "regex",
    "cross_repo_equal",
    "evidence",
}
EVIDENCE_PRODUCER_REPOSITORY = "aduboseh/iter"
TRUSTED_EVIDENCE_WORKFLOW = ".github/workflows/apex_productization_evidence.yml"
CERTIFICATION_ENVIRONMENT = "production-certification"
# Deliberately conservative transport limits; raising them requires collector evidence.
MAX_ARCHIVE_BYTES = 32 * 1024 * 1024
MAX_EVIDENCE_BYTES = 128 * 1024 * 1024
MAX_EVIDENCE_FILE_BYTES = 32 * 1024 * 1024
MAX_EVIDENCE_MEMBERS = 1024


def parse_args() -> argparse.Namespace:
    """Parse verifier paths, control selection, and reporting options."""

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
        help="Legacy local bundles are rejected for evidence controls",
    )
    parser.add_argument("--evidence-run-id", type=int)
    parser.add_argument("--control", action="append", default=[])
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--validate-only", action="store_true")
    mode.add_argument("--verify-evidence-run", action="store_true")
    parser.add_argument("--allow-failures", action="store_true")
    parser.add_argument("--report", type=Path)
    return parser.parse_args()


def load_json(path: Path) -> dict[str, Any]:
    """Load one JSON object or raise a diagnostic ValueError."""

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
    """Require a check to target one of the two certified repositories."""

    if check.get("repo") not in {"iter", "scg"}:
        raise ValueError(f"{control_id}: check repo must be 'iter' or 'scg'")


def canonical_json_sha256(value: Any) -> str:
    """Hash a JSON value with fixed key ordering and separators."""

    encoded = json.dumps(
        value, sort_keys=True, separators=(",", ":"), ensure_ascii=True
    ).encode("utf-8")
    return hashlib.sha256(encoded).hexdigest()


def validate_matrix(matrix: dict[str, Any]) -> list[dict[str, Any]]:
    """Validate the fixed 30-control schema and return its controls."""

    if matrix.get("schema_version") != MATRIX_SCHEMA:
        raise ValueError(
            f"matrix schema must be {MATRIX_SCHEMA!r}, got "
            f"{matrix.get('schema_version')!r}"
        )
    directive_id = matrix.get("directive_id")
    if directive_id != DIRECTIVE_ID:
        raise ValueError(f"matrix directive_id must be {DIRECTIVE_ID!r}")
    if matrix.get("authority_document") != AUTHORITY_DOCUMENT:
        raise ValueError(
            f"matrix authority_document must be {AUTHORITY_DOCUMENT!r}"
        )
    if matrix.get("authority_amendment") != AUTHORITY_AMENDMENT:
        raise ValueError(
            f"matrix authority_amendment must be {AUTHORITY_AMENDMENT!r}"
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
                declared_path = Path(check["path"])
                if declared_path.is_absolute() or ".." in declared_path.parts:
                    raise ValueError(
                        f"{control_id}: {check_type} path must be repository-relative"
                    )
                if check_type == "regex" and not isinstance(check.get("pattern"), str):
                    raise ValueError(f"{control_id}: regex requires pattern")
            elif check_type == "cross_repo_equal":
                for side in ("left", "right"):
                    repo_field = f"{side}_repo"
                    path_field = f"{side}_path"
                    if check.get(repo_field) not in {"iter", "scg"}:
                        raise ValueError(
                            f"{control_id}: {repo_field} must be 'iter' or 'scg'"
                        )
                    declared = check.get(path_field)
                    if not isinstance(declared, str) or not declared:
                        raise ValueError(
                            f"{control_id}: cross_repo_equal requires {path_field}"
                        )
                    declared_path = Path(declared)
                    if declared_path.is_absolute() or ".." in declared_path.parts:
                        raise ValueError(
                            f"{control_id}: {path_field} must be repository-relative"
                        )
            elif check_type == "evidence":
                if not isinstance(check.get("file"), str) or not check["file"]:
                    raise ValueError(f"{control_id}: evidence requires file")
        expected_checks_digest = EXPECTED_CONTROL_CHECK_DIGESTS.get(control_id)
        if (
            expected_checks_digest is not None
            and canonical_json_sha256(checks) != expected_checks_digest
        ):
            raise ValueError(f"{control_id}: checks differ from canonical definition")
    if ids != EXPECTED_CONTROL_IDS:
        missing = sorted(EXPECTED_CONTROL_IDS - ids)
        unexpected = sorted(ids - EXPECTED_CONTROL_IDS)
        raise ValueError(
            "matrix control IDs differ from canonical v1.1 set: "
            f"missing={missing}, unexpected={unexpected}"
        )
    return controls


def git_head(root: Path) -> str | None:
    """Return the exact Git commit checked out at root, if resolvable."""

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


def git_worktree_clean(root: Path) -> tuple[bool, str]:
    """Require one repository to have no tracked or untracked source changes."""

    if not root.is_dir():
        return False, f"repository root missing: {root}"
    result = subprocess.run(
        [
            "git",
            "-C",
            str(root),
            "status",
            "--porcelain=v1",
            "--untracked-files=all",
        ],
        text=True,
        capture_output=True,
        check=False,
    )
    if result.returncode != 0:
        detail = (result.stdout + "\n" + result.stderr).strip()
        return False, f"cannot inspect repository state: {detail}"
    dirty = result.stdout.strip()
    if dirty:
        return False, f"repository is dirty: {dirty}"
    return True, "repository worktree is clean"


def command_check(
    check: dict[str, Any], roots: dict[str, Path]
) -> tuple[bool, str, float]:
    """Execute one declared command without a shell and verify its result."""

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
    # The API credential authenticates evidence; it must not reach subject code.
    env.pop("GH_TOKEN", None)
    env.pop("GITHUB_TOKEN", None)

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


def resolve_contained_path(root: Path, declared_path: str) -> Path | None:
    """Resolve a declared path only when it remains inside its authority root."""

    resolved_root = root.resolve()
    target = (resolved_root / declared_path).resolve()
    try:
        target.relative_to(resolved_root)
    except ValueError:
        return None
    return target


def path_check(
    check: dict[str, Any], roots: dict[str, Path]
) -> tuple[bool, str, float]:
    """Verify that a declared repository-relative path exists."""

    path = resolve_contained_path(roots[check["repo"]], check["path"])
    if path is None:
        return False, "path escapes declared repository", 0.0
    if not path.exists():
        return False, f"missing path: {path}", 0.0
    return True, f"path exists: {path}", 0.0


def regex_check(
    check: dict[str, Any], roots: dict[str, Path]
) -> tuple[bool, str, float]:
    """Evaluate a declared regular-expression invariant against one file."""

    path = resolve_contained_path(roots[check["repo"]], check["path"])
    if path is None:
        return False, "regex path escapes declared repository", 0.0
    if not path.is_file():
        return False, f"missing file: {path}", 0.0
    text = path.read_text(encoding="utf-8")
    matched = re.search(check["pattern"], text, re.MULTILINE) is not None
    if bool(check.get("negate", False)):
        matched = not matched
    return matched, f"regex {'passed' if matched else 'failed'}: {path}", 0.0


def sha256_file(path: Path) -> str:
    """Return the lowercase SHA-256 digest of a file's raw bytes."""

    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def cross_repo_equal_check(
    check: dict[str, Any], roots: dict[str, Path]
) -> tuple[bool, str, float]:
    """Require two contained repository files to have identical raw bytes."""

    left = resolve_contained_path(roots[check["left_repo"]], check["left_path"])
    right = resolve_contained_path(roots[check["right_repo"]], check["right_path"])
    if left is None or right is None:
        return False, "cross-repository path escapes declared repository", 0.0
    if not left.is_file():
        return False, f"missing cross-repository file: {left}", 0.0
    if not right.is_file():
        return False, f"missing cross-repository file: {right}", 0.0

    left_hash = sha256_file(left)
    right_hash = sha256_file(right)
    if left_hash != right_hash:
        return (
            False,
            f"cross-repository mismatch: {left}={left_hash}, {right}={right_hash}",
            0.0,
        )
    return True, f"cross-repository files match: {left_hash}: {left} == {right}", 0.0


def github_json(endpoint: str) -> Any:
    """Read GitHub-owned metadata without exposing CLI diagnostics or credentials."""

    repository_endpoint = f"repos/{EVIDENCE_PRODUCER_REPOSITORY}"
    result = subprocess.run(
        [
            "gh",
            "api",
            "--hostname",
            "github.com",
            "--method",
            "GET",
            f"{repository_endpoint}/{endpoint}" if endpoint else repository_endpoint,
        ],
        capture_output=True,
        text=True,
        encoding="utf-8",
        timeout=30,
        check=False,
    )
    if result.returncode != 0:
        raise ValueError(
            "GitHub metadata unavailable; check read permissions and connectivity"
        )
    return json.loads(result.stdout)


def evidence_run_check(run_id: int | None, iter_commit: str | None) -> tuple[bool, str]:
    """Authenticate a first-attempt producer run and its environment approval.

    This is account-level approval verification, not independent human acceptance
    or validation of the control-specific claims inside an artifact.
    """

    def require(condition: bool, message: str) -> None:
        """Raise rather than assert so optimization cannot remove policy checks."""

        if not condition:
            raise ValueError(message)

    def positive_id(value: Any) -> bool:
        """Reject booleans, which Python otherwise treats as integer IDs."""

        return isinstance(value, int) and not isinstance(value, bool) and value > 0

    try:
        require(positive_id(run_id), "positive evidence run ID required")
        require(
            isinstance(iter_commit, str)
            and re.fullmatch(r"[0-9a-f]{40}", iter_commit) is not None,
            "exact lowercase Iter commit required",
        )
        run = github_json(f"actions/runs/{run_id}")
        require(isinstance(run, dict), "invalid workflow run response")
        for key, expected in {
            "id": run_id,
            "head_sha": iter_commit,
            "path": TRUSTED_EVIDENCE_WORKFLOW,
            "event": "workflow_dispatch",
            "status": "completed",
            "conclusion": "success",
            "run_attempt": 1,
        }.items():
            require(
                type(run.get(key)) is type(expected) and run[key] == expected,
                f"invalid run.{key}",
            )
        for key in ("repository", "head_repository"):
            repo = run.get(key)
            require(
                isinstance(repo, dict)
                and repo.get("full_name") == EVIDENCE_PRODUCER_REPOSITORY
                and repo.get("fork") is False,
                f"invalid run.{key}",
            )
        actors = []
        for key in ("actor", "triggering_actor"):
            actor = run.get(key)
            require(
                isinstance(actor, dict) and positive_id(actor.get("id")),
                f"invalid run.{key}",
            )
            actors.append(actor["id"])

        branch_name = run.get("head_branch")
        require(
            isinstance(branch_name, str) and bool(branch_name),
            "invalid run.head_branch",
        )
        branch = github_json(f"branches/{quote(branch_name, safe='')}")
        require(
            isinstance(branch, dict) and branch.get("protected") is True,
            "producer source branch must be protected",
        )
        tip = branch.get("commit")
        require(
            isinstance(tip, dict)
            and isinstance(tip.get("sha"), str)
            and re.fullmatch(r"[0-9a-f]{40}", tip["sha"]) is not None,
            "protected source branch commit unavailable",
        )
        if tip["sha"] != iter_commit:
            comparison = github_json(f"compare/{iter_commit}...{tip['sha']}")
            require(
                isinstance(comparison, dict)
                and comparison.get("status") == "ahead"
                and isinstance(comparison.get("merge_base_commit"), dict)
                and comparison["merge_base_commit"].get("sha") == iter_commit,
                "evidence commit is not an ancestor of its protected source branch",
            )
        environment = github_json(f"environments/{CERTIFICATION_ENVIRONMENT}")
        require(
            isinstance(environment, dict), "invalid certification environment response"
        )
        require(
            environment.get("name") == CERTIFICATION_ENVIRONMENT
            and positive_id(environment.get("id")),
            "invalid certification environment identity",
        )
        policy = environment.get("deployment_branch_policy")
        require(
            isinstance(policy, dict)
            and policy.get("protected_branches") is True
            and policy.get("custom_branch_policies") is False,
            "certification environment must require protected branches",
        )
        rules = environment.get("protection_rules")
        require(isinstance(rules, list), "missing environment protection rules")
        review_rules = [
            rule
            for rule in rules
            if isinstance(rule, dict) and rule.get("type") == "required_reviewers"
        ]
        require(len(review_rules) == 1, "one required-reviewers rule required")
        rule = review_rules[0]
        require(
            rule.get("prevent_self_review") is True,
            "environment must prevent self-review",
        )
        reviewers = rule.get("reviewers")
        require(isinstance(reviewers, list), "missing required reviewers")
        reviewer_ids = {
            item["reviewer"]["id"]
            for item in reviewers
            if isinstance(item, dict)
            and item.get("type") == "User"
            and isinstance(item.get("reviewer"), dict)
            and positive_id(item["reviewer"].get("id"))
        }
        require(
            bool(reviewer_ids),
            "at least one individually named required reviewer required",
        )

        reviews = github_json(f"actions/runs/{run_id}/approvals")
        require(isinstance(reviews, list), "invalid run approval response")
        approved = False
        for review in reviews:
            require(isinstance(review, dict), "invalid run review")
            environments = review.get("environments")
            require(isinstance(environments, list), "invalid reviewed environments")
            if not any(
                isinstance(item, dict)
                and positive_id(item.get("id"))
                and item["id"] == environment["id"]
                and item.get("name") == CERTIFICATION_ENVIRONMENT
                for item in environments
            ):
                continue
            require(
                review.get("state") == "approved",
                "certification environment review is not approved",
            )
            reviewer = review.get("user")
            require(
                isinstance(reviewer, dict)
                and reviewer.get("type") == "User"
                and positive_id(reviewer.get("id"))
                and reviewer["id"] in reviewer_ids
                and reviewer["id"] not in actors,
                "approval must be from an authorized non-triggering user",
            )
            approved = True
        require(approved, "missing recorded certification environment approval")
    except (ValueError, OSError, subprocess.SubprocessError) as exc:
        # Do not echo subprocess output: it may contain token-bearing diagnostics.
        detail = (
            str(exc)
            if isinstance(exc, ValueError)
            else "GitHub metadata query failed or timed out"
        )
        return False, detail
    return (
        True,
        f"producer run {run_id} authenticated for {iter_commit}; environment approval verified",
    )


def positive_integer(value: Any) -> bool:
    """Reject booleans and non-positive API identities or size declarations."""
    return isinstance(value, int) and not isinstance(value, bool) and value > 0


class NoArtifactRedirect(HTTPRedirectHandler):
    """Keep credentials on the API host and refuse implicit storage redirects."""

    def redirect_request(self, req, fp, code, msg, headers, newurl):
        """Require callers to handle the API's signed redirect explicitly."""
        return None


def download_evidence_archive(artifact_id: int) -> bytes:
    """Download bounded ZIP bytes by immutable ID without forwarding API credentials."""
    if not positive_integer(artifact_id):
        raise ValueError("invalid artifact ID")
    try:
        credential = subprocess.run(
            ["gh", "auth", "token", "--hostname", "github.com"],
            capture_output=True,
            text=True,
            timeout=30,
            check=False,
        )
        token = credential.stdout.strip()
        if credential.returncode != 0 or not token:
            raise ValueError("artifact credential unavailable")
        opener = build_opener(NoArtifactRedirect())
        request = Request(
            f"https://api.github.com/repos/{EVIDENCE_PRODUCER_REPOSITORY}/actions/artifacts/{artifact_id}/zip",
            headers={
                "Authorization": f"Bearer {token}",
                "Accept": "application/vnd.github+json",
            },
        )
        try:
            with opener.open(request, timeout=30):
                raise ValueError("expected artifact API redirect")
        except HTTPError as response:
            try:
                if response.code != 302:
                    raise ValueError("artifact download API failed")
                location = response.headers.get("Location", "")
            finally:
                response.close()
        destination = urlsplit(location)
        if (
            destination.scheme != "https"
            or not destination.hostname
            or destination.username is not None
            or destination.password is not None
            or destination.port not in (None, 443)
        ):
            raise ValueError("invalid artifact storage redirect")
        # Never attach the API token to the signed storage request or log its URL.
        with opener.open(Request(location), timeout=30) as response:
            if response.status != 200:
                raise ValueError("artifact storage download failed")
            deadline = time.monotonic() + 120
            data = bytearray()
            while True:
                chunk = response.read1(
                    min(1024 * 1024, MAX_ARCHIVE_BYTES + 1 - len(data))
                )
                if time.monotonic() > deadline:
                    raise ValueError("artifact download deadline exceeded")
                if not chunk:
                    break
                data.extend(chunk)
                if len(data) > MAX_ARCHIVE_BYTES:
                    raise ValueError("artifact archive exceeds byte limit")
            return bytes(data)
    except (OSError, ValueError, HTTPException, subprocess.SubprocessError) as exc:
        # HTTP and CLI exceptions may contain signed URLs or credentials.
        raise ValueError(
            "artifact download failed, timed out, or exceeded limits"
        ) from exc


def evidence_member_path(name: str) -> str:
    """Require one portable, non-aliased relative evidence member name."""
    if not isinstance(name, str) or not name or len(name) > 240:
        raise ValueError("invalid evidence member path")
    for part in name.split("/"):
        if (
            re.fullmatch(r"[A-Za-z0-9_][A-Za-z0-9_.-]*", part) is None
            or part.endswith(".")
            or re.fullmatch(
                r"(?i:CON|PRN|AUX|NUL|COM[1-9]|LPT[1-9])", part.split(".")[0]
            )
        ):
            raise ValueError("invalid or aliased evidence member path")
    return name


def verified_archive_files(raw: bytes, digest: str) -> Mapping[str, bytes]:
    """Verify the API digest before parsing a bounded ZIP into immutable memory."""
    if len(raw) > MAX_ARCHIVE_BYTES:
        raise ValueError("artifact archive exceeds byte limit")
    if digest != "sha256:" + hashlib.sha256(raw).hexdigest():
        raise ValueError("artifact archive digest mismatch")
    files: dict[str, bytes] = {}
    members: set[str] = set()
    nodes: dict[str, tuple[str, bool]] = {}
    try:
        with zipfile.ZipFile(io.BytesIO(raw)) as archive:
            infos = archive.infolist()
            if not infos or len(infos) > MAX_EVIDENCE_MEMBERS:
                raise ValueError("invalid evidence member count")
            total = 0
            for info in infos:
                is_dir = info.is_dir()
                name = evidence_member_path(
                    info.orig_filename[:-1] if is_dir else info.orig_filename
                )
                if info.orig_filename != info.filename:
                    raise ValueError("truncated evidence member name")
                if name.casefold() in members:
                    raise ValueError("duplicate evidence member")
                members.add(name.casefold())
                parts = name.split("/")
                for index in range(1, len(parts) + 1):
                    node = "/".join(parts[:index])
                    kind = is_dir or index < len(parts)
                    previous = nodes.setdefault(node.casefold(), (node, kind))
                    if previous != (node, kind):
                        raise ValueError("aliased or conflicting evidence member")
                mode = stat.S_IFMT(info.external_attr >> 16)
                if mode not in (0, stat.S_IFDIR if is_dir else stat.S_IFREG):
                    raise ValueError("evidence links and special files are forbidden")
                if info.flag_bits & 1 or info.compress_type not in (
                    zipfile.ZIP_STORED,
                    zipfile.ZIP_DEFLATED,
                ):
                    raise ValueError("unsupported or encrypted evidence member")
                total += info.file_size
                if (
                    info.file_size > MAX_EVIDENCE_FILE_BYTES
                    or total > MAX_EVIDENCE_BYTES
                    or (is_dir and info.file_size != 0)
                ):
                    raise ValueError("evidence expansion exceeds byte limit")
                with archive.open(info) as stream:
                    data = stream.read(MAX_EVIDENCE_FILE_BYTES + 1)
                if len(data) != info.file_size or len(data) > MAX_EVIDENCE_FILE_BYTES:
                    raise ValueError("evidence member size mismatch")
                if not is_dir:
                    files[name] = data
    except (
        zipfile.BadZipFile,
        RuntimeError,
        NotImplementedError,
        EOFError,
        zlib.error,
    ) as exc:
        raise ValueError("invalid evidence ZIP") from exc
    if not files:
        raise ValueError("empty evidence archive")
    return MappingProxyType(files)


def evidence_artifact_check(
    run_id: int, heads: dict[str, str | None]
) -> tuple[Mapping[str, bytes], dict[str, Any]]:
    """After run approval, bind exact subject bytes to one GitHub artifact record."""
    if (
        not positive_integer(run_id)
        or any(
            not isinstance(heads.get(repo), str)
            or re.fullmatch(r"[0-9a-f]{40}", heads[repo]) is None
            for repo in ("iter", "scg")
        )
    ):
        raise ValueError("exact subjects and positive evidence run required")
    name = f"apex-productization-evidence-{heads['iter']}-{heads['scg']}"
    listing = github_json(f"actions/runs/{run_id}/artifacts?per_page=100&name={name}")
    if (
        not isinstance(listing, dict)
        or not positive_integer(listing.get("total_count"))
        or listing["total_count"] != 1
        or not isinstance(listing.get("artifacts"), list)
        or len(listing["artifacts"]) != 1
    ):
        raise ValueError(
            "exactly one evidence artifact required; incomplete listings rejected"
        )
    artifact = listing["artifacts"][0]
    if (
        not isinstance(artifact, dict)
        or not positive_integer(artifact.get("id"))
    ):
        raise ValueError("invalid artifact ID")
    if github_json(f"actions/artifacts/{artifact['id']}") != artifact:
        raise ValueError("artifact metadata changed during selection")
    repository = github_json("")
    if (
        not isinstance(repository, dict)
        or repository.get("full_name") != EVIDENCE_PRODUCER_REPOSITORY
        or repository.get("fork") is not False
        or not positive_integer(repository.get("id"))
    ):
        raise ValueError("invalid evidence repository identity")
    run = artifact.get("workflow_run")
    expected_run = {
        "id": run_id,
        "head_sha": heads["iter"],
        "repository_id": repository["id"],
        "head_repository_id": repository["id"],
    }
    if not isinstance(run, dict) or any(
        type(run.get(key)) is not type(value) or run[key] != value
        for key, value in expected_run.items()
    ):
        raise ValueError("artifact run or repository mismatch")
    digest = artifact.get("digest")
    size = artifact.get("size_in_bytes")
    if (
        artifact.get("name") != name
        or artifact.get("expired") is not False
        or not isinstance(digest, str)
        or re.fullmatch(r"sha256:[0-9a-f]{64}", digest) is None
        or not positive_integer(size)
        or not 0 < size <= MAX_ARCHIVE_BYTES
    ):
        raise ValueError("invalid evidence artifact name, expiration, digest or size")
    raw = download_evidence_archive(artifact["id"])
    if len(raw) != size:
        raise ValueError("artifact archive size mismatch")
    files = verified_archive_files(raw, digest)
    return files, {
        "repository": EVIDENCE_PRODUCER_REPOSITORY,
        "artifact_id": artifact["id"],
        "run_id": run_id,
        "name": name,
        "digest": digest,
        "size_in_bytes": len(raw),
        "subject_commits": dict(heads),
        "file_count": len(files),
    }


def evidence_check(
    control_id: str,
    check: dict[str, Any],
    evidence_files: Mapping[str, bytes],
    heads: dict[str, str | None],
    evidence_run_id: int | None,
) -> tuple[bool, str, float]:
    """Check declarations against the immutable API-authenticated bundle bytes."""

    try:
        path = evidence_member_path(check["file"])
        evidence = json.loads(evidence_files[path])
        if not isinstance(evidence, dict):
            raise ValueError("evidence must be a JSON object")
    except (ValueError, KeyError, UnicodeError, RecursionError) as exc:
        return False, str(exc), 0.0

    failures: list[str] = []
    if evidence.get("schema_version") != EVIDENCE_SCHEMA:
        failures.append(f"schema_version must be {EVIDENCE_SCHEMA}")
    if evidence.get("control_id") != control_id:
        failures.append(f"control_id must be {control_id}")
    if evidence.get("result") != "PASS":
        failures.append("result must be PASS")

    producer = evidence.get("producer")
    if evidence_run_id is None:
        failures.append("trusted evidence_run_id is required")
    if not isinstance(producer, dict):
        failures.append("producer must be an object")
    else:
        if producer.get("repository") != EVIDENCE_PRODUCER_REPOSITORY:
            failures.append(
                f"producer.repository must be {EVIDENCE_PRODUCER_REPOSITORY}"
            )
        if producer.get("workflow") != TRUSTED_EVIDENCE_WORKFLOW:
            failures.append(f"producer.workflow must be {TRUSTED_EVIDENCE_WORKFLOW}")
        if producer.get("run_id") != evidence_run_id:
            failures.append(f"producer.run_id must be {evidence_run_id}")

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
            try:
                target = evidence_member_path(artifact_path)
            except ValueError:
                failures.append(f"artifact {index} invalid evidence member path")
                continue
            if target not in evidence_files:
                failures.append(f"artifact {index} missing: {target}")
                continue
            actual_hash = hashlib.sha256(evidence_files[target]).hexdigest()
            if expected_hash != actual_hash:
                failures.append(
                    f"artifact {index} hash mismatch: {actual_hash} != {expected_hash}"
                )

    commands = evidence.get("commands")
    if (
        not isinstance(commands, list)
        or not commands
        or not all(isinstance(command, str) and command.strip() for command in commands)
    ):
        failures.append("commands must be a non-empty array of non-empty strings")

    if failures:
        return False, "; ".join(failures), 0.0
    return True, f"commit-bound evidence verified: {path}", 0.0


def directive_mirror_check(iter_root: Path, scg_root: Path) -> tuple[bool, str]:
    """Require every Iter/SCG productization authority to be byte-identical."""

    verified: list[str] = []
    for relative_path in DIRECTIVE_MIRRORS:
        iter_path = iter_root / relative_path
        scg_path = scg_root / relative_path
        if not iter_path.is_file():
            return False, f"missing iter directive: {iter_path}"
        if not scg_path.is_file():
            return False, f"missing SCG directive mirror: {scg_path}"
        iter_hash = sha256_file(iter_path)
        scg_hash = sha256_file(scg_path)
        if iter_hash != scg_hash:
            return (
                False,
                f"directive mirror mismatch for {relative_path}: "
                f"iter={iter_hash}, scg={scg_hash}",
            )
        verified.append(f"{relative_path}={iter_hash}")
    return True, "directive mirrors byte-identical: " + ", ".join(verified)


def scg_release_ref_check(iter_root: Path, scg_head: str | None) -> tuple[bool, str]:
    """Require the checked-out SCG commit to equal iter's immutable release pin."""

    path = iter_root / "productization" / "SCG_RELEASE_REF"
    try:
        pinned = path.read_text(encoding="utf-8").strip()
    except OSError as exc:
        return False, f"cannot read SCG release pin {path}: {exc}"
    if re.fullmatch(r"[0-9a-f]{40}", pinned) is None:
        return False, f"invalid SCG release pin in {path}: {pinned!r}"
    if scg_head is None:
        return False, "cannot resolve checked-out SCG commit"
    if pinned != scg_head:
        return False, f"SCG release pin mismatch: pinned={pinned}, checked_out={scg_head}"
    return True, f"SCG release pin matches checked-out subject: {pinned}"


def write_report(path: Path, report: dict[str, Any]) -> None:
    """Write one stable, sorted JSON certification report."""

    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        json.dumps(report, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )


def certification_status(
    authority_ok: bool, results: list[dict[str, Any]]
) -> tuple[str, bool]:
    """Classify a run without allowing a subset to claim full certification."""

    execution_passed = authority_ok and all(
        result["status"] == "PASS" for result in results
    )
    if not execution_passed:
        return "FAIL", False
    if len(results) == EXPECTED_CONTROL_COUNT:
        return "PASS", True
    return "PARTIAL", True


def main() -> int:
    """Validate or execute the release matrix and emit an auditable report."""

    args = parse_args()
    if args.verify_evidence_run:
        passed, detail = evidence_run_check(
            args.evidence_run_id, git_head(args.iter_root)
        )
        print(f"{'PASS' if passed else 'FAIL'} EVIDENCE-RUN {detail}")
        return 0 if passed else 1
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
        f"MATRIX VALID: {len(matrix['controls'])} controls, " f"schema={MATRIX_SCHEMA}"
    )
    if args.validate_only:
        return 0

    roots = {
        "iter": args.iter_root.resolve(),
        "scg": args.scg_root.resolve(),
    }
    heads = {repo: git_head(root) for repo, root in roots.items()}
    verifier_authority_commit = git_head(Path(__file__).resolve().parents[1])
    for repo, root in roots.items():
        clean, detail = git_worktree_clean(root)
        print(f"{'PASS' if clean else 'FAIL'} {repo.upper()}-INITIAL-STATE {detail}")
        if not clean:
            return 2
    mirror_ok, mirror_detail = directive_mirror_check(roots["iter"], roots["scg"])
    print(f"{'PASS' if mirror_ok else 'FAIL'} DIRECTIVE-MIRROR {mirror_detail}")
    scg_pin_ok, scg_pin_detail = scg_release_ref_check(roots["iter"], heads["scg"])
    print(f"{'PASS' if scg_pin_ok else 'FAIL'} SCG-RELEASE-REF {scg_pin_detail}")

    evidence_files: Mapping[str, bytes] = MappingProxyType({})
    evidence_artifact = None
    if any(
        check["type"] == "evidence"
        for control in controls
        for check in control["checks"]
    ):
        trusted, detail = evidence_run_check(args.evidence_run_id, heads["iter"])
        print(f"{'PASS' if trusted else 'FAIL'} EVIDENCE-RUN {detail}")
        if not trusted:
            return 1
        if args.evidence_dir is not None:
            print(
                "FAIL EVIDENCE-ARTIFACT local evidence directories are not authenticated"
            )
            return 1
        try:
            evidence_files, evidence_artifact = evidence_artifact_check(
                args.evidence_run_id, heads
            )
        except (ValueError, OSError, subprocess.SubprocessError) as exc:
            detail = (
                str(exc)
                if isinstance(exc, ValueError)
                else "artifact API failed or timed out"
            )
            print(f"FAIL EVIDENCE-ARTIFACT {detail}")
            return 1
        print(
            f"PASS EVIDENCE-ARTIFACT {evidence_artifact['artifact_id']} {evidence_artifact['digest']}"
        )

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
            elif check_type == "cross_repo_equal":
                passed, detail, elapsed = cross_repo_equal_check(check, roots)
            else:
                passed, detail, elapsed = evidence_check(
                    control["id"],
                    check,
                    evidence_files,
                    heads,
                    args.evidence_run_id,
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
    final_heads = {repo: git_head(root) for repo, root in roots.items()}
    repository_state: dict[str, dict[str, Any]] = {}
    repositories_unchanged = True
    for repo, root in roots.items():
        clean, clean_detail = git_worktree_clean(root)
        unchanged = clean and final_heads[repo] == heads[repo]
        repositories_unchanged = repositories_unchanged and unchanged
        detail = f"initial={heads[repo]}, final={final_heads[repo]}, {clean_detail}"
        print(
            f"{'PASS' if unchanged else 'FAIL'} " f"{repo.upper()}-FINAL-STATE {detail}"
        )
        repository_state[repo] = {
            "status": "PASS" if unchanged else "FAIL",
            "initial_commit": heads[repo],
            "final_commit": final_heads[repo],
            "clean": clean,
            "detail": detail,
        }

    authority_ok = mirror_ok and scg_pin_ok and repositories_unchanged
    report_status, execution_passed = certification_status(authority_ok, results)
    full_matrix_run = len(results) == EXPECTED_CONTROL_COUNT
    report = {
        "schema_version": "apex-productization-report/v1",
        "directive_id": matrix["directive_id"],
        "subject_commits": final_heads,
        "verifier_authority_commit": verifier_authority_commit,
        "evidence_artifact": evidence_artifact,
        "repository_state": repository_state,
        "evidence_producer": {
            "repository": EVIDENCE_PRODUCER_REPOSITORY,
            "workflow": TRUSTED_EVIDENCE_WORKFLOW,
            "run_id": args.evidence_run_id,
        },
        "directive_mirror": {
            "status": "PASS" if mirror_ok else "FAIL",
            "detail": mirror_detail,
        },
        "scg_release_ref": {
            "status": "PASS" if scg_pin_ok else "FAIL",
            "detail": scg_pin_detail,
        },
        "summary": {
            "status": report_status,
            "passed": pass_count,
            "failed": fail_count,
            "selected_controls": len(results),
            "total_controls": EXPECTED_CONTROL_COUNT,
            "full_matrix": full_matrix_run,
        },
        "controls": results,
    }
    if args.report:
        write_report(args.report, report)
        print(f"REPORT {args.report.resolve()}")

    print(
        f"APEX RELEASE MATRIX: {report_status} "
        f"({pass_count} passed, {fail_count} failed)"
    )
    if execution_passed or args.allow_failures:
        return 0
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
