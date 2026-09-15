"""Focused tests for APEX certification status and external evidence binding."""

from __future__ import annotations

import hashlib
import copy
import contextlib
import importlib.util
import io
import json
import os
from pathlib import Path
import shlex
import subprocess
import sys
import tempfile
import textwrap
import unittest
from unittest import mock


SCRIPT = (
    Path(__file__).resolve().parents[1] / "scripts" / "verify_productization_matrix.py"
)
SPEC = importlib.util.spec_from_file_location("verify_productization_matrix", SCRIPT)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError(f"cannot load verifier: {SCRIPT}")
VERIFIER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(VERIFIER)


def results(count: int, status: str = "PASS") -> list[dict[str, str]]:
    """Build minimal control results for certification-status tests."""

    return [{"status": status} for _ in range(count)]


def evidence_fixture() -> tuple[dict, dict[str, bytes]]:
    """Load shared synthetic conformance data, never certification evidence."""
    fixture = json.loads(
        (SCRIPT.parents[1] / "tests/data/productization_evidence_v2.json").read_text(
            encoding="utf-8"
        )
    )
    return fixture["record"], {
        path: content.encode("utf-8") for path, content in fixture["files"].items()
    }


def run_git(root: Path, *args: str) -> None:
    """Run one deterministic Git setup command for worktree-state tests."""

    subprocess.run(
        ["git", "-c", "commit.gpgsign=false", *args],
        cwd=root,
        text=True,
        capture_output=True,
        check=True,
    )


class MatrixValidationTests(unittest.TestCase):
    """Prove malformed authority metadata is rejected during validation."""

    def test_missing_directive_id_is_rejected(self) -> None:
        """Validation fails before execution when directive_id is absent."""

        matrix = VERIFIER.load_json(
            Path(__file__).resolve().parents[1]
            / "productization"
            / "APEX_RELEASE_MATRIX_V1.json"
        )
        matrix.pop("directive_id")
        with self.assertRaisesRegex(ValueError, "directive_id"):
            VERIFIER.validate_matrix(matrix)

    def test_noncanonical_directive_id_is_rejected(self) -> None:
        """A matrix cannot substitute a different release authority."""

        matrix = VERIFIER.load_json(
            Path(__file__).resolve().parents[1]
            / "productization"
            / "APEX_RELEASE_MATRIX_V1.json"
        )
        matrix["directive_id"] = "APEX-UNRELATED-001"
        with self.assertRaisesRegex(ValueError, VERIFIER.DIRECTIVE_ID):
            VERIFIER.validate_matrix(matrix)

    def test_missing_authority_amendment_is_rejected(self) -> None:
        """The Path B authority amendment cannot be omitted from the matrix."""

        matrix = VERIFIER.load_json(
            Path(__file__).resolve().parents[1]
            / "productization"
            / "APEX_RELEASE_MATRIX_V1.json"
        )
        matrix.pop("authority_amendment")
        with self.assertRaisesRegex(ValueError, "authority_amendment"):
            VERIFIER.validate_matrix(matrix)

    def test_full_substrate_control_requires_explicit_rejection(self) -> None:
        """G0-03 proves Path B rejection rather than requiring a false build claim."""

        matrix = VERIFIER.load_json(
            Path(__file__).resolve().parents[1]
            / "productization"
            / "APEX_RELEASE_MATRIX_V1.json"
        )
        control = next(item for item in matrix["controls"] if item["id"] == "G0-03")
        self.assertEqual(len(control["checks"]), 2)
        for check in control["checks"]:
            self.assertEqual(check["expected_exit"], 101)
            self.assertEqual(
                check["output_pattern"],
                "FULL_SUBSTRATE_UNSUPPORTED_IN_PUBLIC_REPO",
            )

    def test_canonical_control_id_substitution_is_rejected(self) -> None:
        """A syntactically valid replacement cannot hide a required control."""

        matrix = VERIFIER.load_json(
            Path(__file__).resolve().parents[1]
            / "productization"
            / "APEX_RELEASE_MATRIX_V1.json"
        )
        control = next(item for item in matrix["controls"] if item["id"] == "G2-01")
        control["id"] = "G2-99"
        with self.assertRaisesRegex(ValueError, r"canonical v1\.1 set"):
            VERIFIER.validate_matrix(matrix)

    def test_canonical_control_checks_substitution_is_rejected(self) -> None:
        """A valid-looking check cannot replace a canonical control definition."""

        matrix = VERIFIER.load_json(
            Path(__file__).resolve().parents[1]
            / "productization"
            / "APEX_RELEASE_MATRIX_V1.json"
        )
        control = next(item for item in matrix["controls"] if item["id"] == "G0-01")
        control["checks"] = [
            {"type": "path_exists", "repo": "iter", "path": "Cargo.toml"}
        ]
        with self.assertRaisesRegex(ValueError, "canonical definition"):
            VERIFIER.validate_matrix(matrix)

    def test_repository_path_escape_is_rejected_during_validation(self) -> None:
        """Repository checks cannot declare absolute or parent-traversal paths."""

        matrix_path = (
            Path(__file__).resolve().parents[1]
            / "productization"
            / "APEX_RELEASE_MATRIX_V1.json"
        )
        for escaped_path in ("../outside", str(Path.cwd().anchor + "outside")):
            with self.subTest(path=escaped_path):
                matrix = VERIFIER.load_json(matrix_path)
                check = next(
                    check
                    for control in matrix["controls"]
                    for check in control["checks"]
                    if check["type"] == "path_exists"
                )
                check["path"] = escaped_path
                with self.assertRaisesRegex(ValueError, "repository-relative"):
                    VERIFIER.validate_matrix(matrix)

    def test_cross_repository_path_escape_is_rejected_during_validation(self) -> None:
        """Cross-repository checks cannot escape either declared authority root."""

        matrix_path = (
            Path(__file__).resolve().parents[1]
            / "productization"
            / "APEX_RELEASE_MATRIX_V1.json"
        )
        for field in ("left_path", "right_path"):
            with self.subTest(field=field):
                matrix = VERIFIER.load_json(matrix_path)
                control = next(
                    item for item in matrix["controls"] if item["id"] == "G0-13"
                )
                check = next(
                    item
                    for item in control["checks"]
                    if item["type"] == "cross_repo_equal"
                )
                check[field] = "../outside"
                with self.assertRaisesRegex(ValueError, "repository-relative"):
                    VERIFIER.validate_matrix(matrix)


class CertificationStatusTests(unittest.TestCase):
    """Prove only a complete successful matrix can report full PASS."""

    def test_complete_matrix_is_pass(self) -> None:
        """Thirty successful controls and a valid mirror are a full PASS."""

        self.assertEqual(
            VERIFIER.certification_status(True, results(30)), ("PASS", True)
        )

    def test_selected_subset_is_partial(self) -> None:
        """A successful selected subset is useful but never full certification."""

        self.assertEqual(
            VERIFIER.certification_status(True, results(1)), ("PARTIAL", True)
        )

    def test_failure_or_mirror_mismatch_is_fail(self) -> None:
        """Any failed control or directive mismatch fails the execution."""

        failed = results(30)
        failed[-1] = {"status": "FAIL"}
        self.assertEqual(VERIFIER.certification_status(True, failed), ("FAIL", False))
        self.assertEqual(
            VERIFIER.certification_status(False, results(30)), ("FAIL", False)
        )


class EvidenceRunTests(unittest.TestCase):
    """Exercise GitHub metadata policy with fixtures, never certification records."""

    def setUp(self) -> None:
        """Build API fixtures without writing reusable certification evidence."""

        self.commit = "a" * 40
        self.branch = {"protected": True, "commit": {"sha": self.commit}}
        self.run = {
            "id": 123,
            "head_sha": self.commit,
            "head_branch": "main",
            "path": VERIFIER.TRUSTED_EVIDENCE_WORKFLOW,
            "event": "workflow_dispatch",
            "status": "completed",
            "conclusion": "success",
            "run_attempt": 1,
            "repository": {"full_name": "aduboseh/iter", "fork": False},
            "head_repository": {"full_name": "aduboseh/iter", "fork": False},
            "actor": {"id": 10},
            "triggering_actor": {"id": 11},
        }
        self.environment = {
            "id": 5,
            "name": VERIFIER.CERTIFICATION_ENVIRONMENT,
            "deployment_branch_policy": {
                "protected_branches": True,
                "custom_branch_policies": False,
            },
            "protection_rules": [
                {
                    "type": "required_reviewers",
                    "prevent_self_review": True,
                    "reviewers": [{"type": "User", "reviewer": {"id": 20}}],
                }
            ],
        }
        self.reviews = [
            {
                "state": "approved",
                "user": {"id": 20, "type": "User"},
                "environments": [{"id": 5, "name": VERIFIER.CERTIFICATION_ENVIRONMENT}],
            }
        ]

    def check(self) -> tuple[bool, str]:
        """Exercise the complete policy with only GitHub transport substituted."""

        with mock.patch.object(
            VERIFIER,
            "github_json",
            side_effect=[
                self.run,
                self.branch,
                self.environment,
                self.reviews,
            ],
        ):
            return VERIFIER.evidence_run_check(123, self.commit)

    def test_first_attempt_with_authorized_environment_approval(self) -> None:
        """One authorized non-triggering account satisfies the approval policy."""

        passed, detail = self.check()
        self.assertTrue(passed, detail)

    def test_run_metadata_mutations_are_rejected(self) -> None:
        """Valid neighboring fields cannot compensate for an invalid identity."""

        for field, value in (
            ("id", 124),
            ("id", True),
            ("head_sha", "b" * 40),
            ("head_branch", None),
            ("head_branch", ""),
            ("path", ".github/workflows/other.yml"),
            ("event", "push"),
            ("status", "in_progress"),
            ("conclusion", "failure"),
            ("run_attempt", 2),
            ("run_attempt", True),
            ("repository", None),
            ("head_repository", {"full_name": "fork/iter", "fork": True}),
            ("actor", {}),
            ("triggering_actor", {"id": True}),
        ):
            with self.subTest(field=field, value=value):
                original = self.run[field]
                self.run[field] = value
                self.assertFalse(self.check()[0])
                self.run[field] = original

    def test_missing_run_fields_are_rejected(self) -> None:
        """Sparse metadata never inherits defaults from the expected policy."""

        for field in list(self.run):
            with self.subTest(field=field):
                value = self.run.pop(field)
                self.assertFalse(self.check()[0])
                self.run[field] = value

    def test_environment_policy_mutations_are_rejected(self) -> None:
        """Branch protection cannot substitute for environment review rules."""

        original = copy.deepcopy(self.environment)
        cases = [
            ("id", None),
            ("name", "other"),
            ("deployment_branch_policy", None),
            (
                "deployment_branch_policy",
                {"protected_branches": False, "custom_branch_policies": False},
            ),
            ("protection_rules", []),
            ("protection_rules", [{"type": "wait_timer"}]),
            (
                "protection_rules",
                [dict(original["protection_rules"][0], prevent_self_review=False)],
            ),
            ("protection_rules", [dict(original["protection_rules"][0], reviewers=[])]),
            (
                "protection_rules",
                [
                    dict(
                        original["protection_rules"][0],
                        reviewers=[{"type": "Team", "reviewer": {"id": 20}}],
                    )
                ],
            ),
        ]
        for field, value in cases:
            with self.subTest(field=field, value=value):
                self.environment = dict(original, **{field: value})
                self.assertFalse(self.check()[0])

    def test_missing_rejected_wrong_environment_and_self_reviews_fail(self) -> None:
        """Policy configuration alone is not attributable run-specific approval."""

        original = copy.deepcopy(self.reviews[0])
        cases = [
            [],
            [{}],
            [dict(original, state="rejected")],
            [
                dict(
                    original,
                    environments=[
                        {"id": 6, "name": VERIFIER.CERTIFICATION_ENVIRONMENT}
                    ],
                )
            ],
            [dict(original, environments=[{"id": 5, "name": "other"}])],
            [dict(original, user={"id": 20, "type": "Bot"})],
            [dict(original, user={"id": 30, "type": "User"})],
            [original, dict(original, state="rejected")],
        ]
        for actor_id in (10, 11):
            self.environment["protection_rules"][0]["reviewers"].append(
                {"type": "User", "reviewer": {"id": actor_id}}
            )
            cases.append([dict(original, user={"id": actor_id, "type": "User"})])
        for reviews in cases:
            with self.subTest(reviews=reviews):
                self.reviews = reviews
                self.assertFalse(self.check()[0])

    def test_api_error_at_every_query_fails_closed(self) -> None:
        """No failed metadata hop may be omitted from the trust chain."""

        responses = [self.run, self.branch, self.environment, self.reviews]
        for index in range(len(responses)):
            with self.subTest(query=index):
                with mock.patch.object(
                    VERIFIER,
                    "github_json",
                    side_effect=responses[:index] + [ValueError("API denied")],
                ):
                    passed, detail = VERIFIER.evidence_run_check(123, self.commit)
                self.assertFalse(passed)
                self.assertIn("API denied", detail)

    def test_unprotected_branch_fails(self) -> None:
        """Approval cannot bless code from an unprotected source branch."""

        with mock.patch.object(
            VERIFIER, "github_json", side_effect=[self.run, {"protected": False}]
        ):
            self.assertFalse(VERIFIER.evidence_run_check(123, self.commit)[0])

    def test_protected_release_branches_are_supported_and_url_encoded(self) -> None:
        """Release sources follow the same policy without being forced onto main."""

        self.run["head_branch"] = "release/v1.0"
        with mock.patch.object(
            VERIFIER,
            "github_json",
            side_effect=[
                self.run,
                self.branch,
                self.environment,
                self.reviews,
            ],
        ) as api:
            passed, detail = VERIFIER.evidence_run_check(123, self.commit)
        self.assertTrue(passed, detail)
        self.assertEqual(api.call_args_list[1].args, ("branches/release%2Fv1.0",))
        with mock.patch.object(
            VERIFIER, "github_json", side_effect=[self.run, {"protected": False}]
        ):
            self.assertFalse(VERIFIER.evidence_run_check(123, self.commit)[0])

    def test_protected_main_ancestry_is_required_not_just_ref_name(self) -> None:
        """The ref label must correspond to the subject's actual ancestry."""

        tip = "b" * 40
        for comparison, expected in (
            ({"status": "ahead", "merge_base_commit": {"sha": self.commit}}, True),
            ({"status": "diverged", "merge_base_commit": {"sha": "c" * 40}}, False),
            ({"status": "behind", "merge_base_commit": {"sha": tip}}, False),
            ({"status": "ahead", "merge_base_commit": {"sha": "c" * 40}}, False),
            ({}, False),
        ):
            with (
                self.subTest(comparison=comparison),
                mock.patch.object(
                    VERIFIER,
                    "github_json",
                    side_effect=[
                        self.run,
                        {"protected": True, "commit": {"sha": tip}},
                        comparison,
                        self.environment,
                        self.reviews,
                    ],
                ) as api,
            ):
                self.assertEqual(
                    VERIFIER.evidence_run_check(123, self.commit)[0], expected
                )
                self.assertEqual(
                    api.call_args_list[2].args, (f"compare/{self.commit}...{tip}",)
                )

    def test_missing_main_commit_and_ancestry_api_failure_fail_closed(self) -> None:
        """Unknown branch state cannot be treated as a matching source."""

        for tip in (None, {}, {"sha": "main"}):
            with (
                self.subTest(tip=tip),
                mock.patch.object(
                    VERIFIER,
                    "github_json",
                    side_effect=[self.run, {"protected": True, "commit": tip}],
                ),
            ):
                self.assertFalse(VERIFIER.evidence_run_check(123, self.commit)[0])
        with mock.patch.object(
            VERIFIER,
            "github_json",
            side_effect=[
                self.run,
                {"protected": True, "commit": {"sha": "b" * 40}},
                ValueError("API denied"),
            ],
        ):
            self.assertFalse(VERIFIER.evidence_run_check(123, self.commit)[0])

    def test_invalid_subjects_do_not_query_api(self) -> None:
        """Reject ambiguous IDs before forming authenticated API requests."""

        for run_id, commit in (
            (None, self.commit),
            (0, self.commit),
            (True, self.commit),
            ("123", self.commit),
            (123, "main"),
            (123, None),
            (123, "A" * 40),
        ):
            with self.subTest(run_id=run_id, commit=commit):
                with mock.patch.object(VERIFIER, "github_json") as api:
                    self.assertFalse(VERIFIER.evidence_run_check(run_id, commit)[0])
                api.assert_not_called()

    def test_cli_errors_and_timeout_are_not_success_or_secret_disclosure(self) -> None:
        """Errors stay fail-closed without exposing token-bearing CLI stderr."""

        failures = [
            FileNotFoundError("TOKEN-SENTINEL"),
            subprocess.TimeoutExpired("gh", 30, stderr="TOKEN-SENTINEL"),
            subprocess.CompletedProcess([], 1, "", "TOKEN-SENTINEL"),
            subprocess.CompletedProcess([], 0, "not json", ""),
        ]
        for failure in failures:
            with self.subTest(failure=type(failure).__name__):
                options = (
                    {"side_effect": failure}
                    if isinstance(failure, Exception)
                    else {"return_value": failure}
                )
                with mock.patch.object(VERIFIER.subprocess, "run", **options):
                    passed, detail = VERIFIER.evidence_run_check(123, self.commit)
                self.assertFalse(passed)
                self.assertNotIn("TOKEN-SENTINEL", detail)

    def test_api_uses_fixed_host_read_only_and_timeout(self) -> None:
        """Pin the authority host and prohibit shell or indefinite execution."""

        with mock.patch.object(
            VERIFIER.subprocess,
            "run",
            return_value=subprocess.CompletedProcess([], 0, "{}", ""),
        ) as command:
            self.assertEqual(VERIFIER.github_json("actions/runs/123"), {})
        argv = command.call_args.args[0]
        self.assertEqual(
            argv,
            [
                "gh",
                "api",
                "--hostname",
                "github.com",
                "--method",
                "GET",
                "repos/aduboseh/iter/actions/runs/123",
            ],
        )
        self.assertEqual(command.call_args.kwargs["timeout"], 30)
        self.assertFalse(command.call_args.kwargs.get("shell", False))

    def test_subject_commands_do_not_inherit_api_credentials(self) -> None:
        """Neither inherited nor declared environment gives code an API token."""

        check = {
            "argv": ["cargo", "test"],
            "repo": "iter",
            "env": {"GH_TOKEN": "override"},
        }
        with (
            mock.patch.dict(
                VERIFIER.os.environ, {"GH_TOKEN": "secret", "GITHUB_TOKEN": "secret"}
            ),
            mock.patch.object(
                VERIFIER.subprocess,
                "run",
                return_value=subprocess.CompletedProcess(
                    [], 0, "test result: ok. 1 passed", ""
                ),
            ) as command,
        ):
            self.assertTrue(VERIFIER.command_check(check, {"iter": SCRIPT.parent})[0])
        environment = command.call_args.kwargs["env"]
        self.assertNotIn("GH_TOKEN", environment)
        self.assertNotIn("GITHUB_TOKEN", environment)

    def test_matrix_cannot_consume_evidence_or_allow_failure_after_auth_rejection(
        self,
    ) -> None:
        """Advisory mode cannot turn failed authentication into accepted evidence."""

        with (
            mock.patch(
                "sys.argv",
                [
                    str(SCRIPT),
                    "--control",
                    "G1-01",
                    "--evidence-run-id",
                    "123",
                    "--allow-failures",
                ],
            ),
            mock.patch.object(VERIFIER, "git_head", return_value=self.commit),
            mock.patch.object(
                VERIFIER, "git_worktree_clean", return_value=(True, "clean")
            ),
            mock.patch.object(
                VERIFIER, "directive_mirror_check", return_value=(True, "matches")
            ),
            mock.patch.object(
                VERIFIER, "scg_release_ref_check", return_value=(True, "matches")
            ),
            mock.patch.object(
                VERIFIER, "evidence_run_check", return_value=(False, "not approved")
            ) as authenticate,
            mock.patch.object(VERIFIER, "evidence_check") as consume,
            contextlib.redirect_stdout(io.StringIO()),
        ):
            self.assertEqual(VERIFIER.main(), 1)
        authenticate.assert_called_once_with(123, self.commit)
        consume.assert_not_called()

    def test_preflight_and_both_workflows_share_authentication(self) -> None:
        """Both workflows authenticate before downloading any evidence bundle."""

        with (
            mock.patch(
                "sys.argv",
                [str(SCRIPT), "--verify-evidence-run", "--evidence-run-id", "123"],
            ),
            mock.patch.object(VERIFIER, "git_head", return_value=self.commit),
            mock.patch.object(
                VERIFIER, "evidence_run_check", return_value=(False, "not approved")
            ) as authenticate,
            contextlib.redirect_stdout(io.StringIO()),
        ):
            self.assertEqual(VERIFIER.main(), 1)
        authenticate.assert_called_once_with(123, self.commit)
        workflows = SCRIPT.parents[1] / ".github" / "workflows"
        for name in ("apex_productization.yml", "release_gate.yml"):
            workflow = (workflows / name).read_text(encoding="utf-8")
            self.assertEqual(workflow.count("--verify-evidence-run"), 1)
            self.assertNotIn("uses: actions/download-artifact", workflow)
            self.assertNotIn("--evidence-dir", workflow)
            self.assertLess(
                workflow.index("--verify-evidence-run"),
                workflow.index("--scg-root SCG"),
            )
        required = (workflows / "mcp_integration.yml").read_text(encoding="utf-8")
        self.assertIn("python -B tests/test_productization_matrix.py", required)
        self.assertIn("python -BO tests/test_productization_matrix.py", required)

    def test_preflight_uses_explicit_subject_not_authority_commit(self) -> None:
        """A separate verifier checkout must authenticate the candidate's HEAD."""

        subject = Path("candidate")
        with (
            mock.patch(
                "sys.argv",
                [
                    str(SCRIPT),
                    "--iter-root",
                    str(subject),
                    "--verify-evidence-run",
                    "--evidence-run-id",
                    "123",
                ],
            ),
            mock.patch.object(VERIFIER, "git_head", return_value=self.commit) as head,
            mock.patch.object(
                VERIFIER, "evidence_run_check", return_value=(False, "not approved")
            ) as authenticate,
            contextlib.redirect_stdout(io.StringIO()),
        ):
            self.assertEqual(VERIFIER.main(), 1)
        head.assert_called_once_with(subject)
        authenticate.assert_called_once_with(123, self.commit)

    def test_hosted_verifiers_are_separate_from_candidate_code(self) -> None:
        """Preflight and controls use isolated, main-sourced authority in both jobs."""

        workflows = SCRIPT.parents[1] / ".github" / "workflows"
        for name in ("apex_productization.yml", "release_gate.yml"):
            with self.subTest(workflow=name):
                workflow = (workflows / name).read_text(encoding="utf-8")
                authority = workflow.split("name: Check out verifier authority", 1)[1]
                authority = authority.split("\n  release-gate:", 1)[0]
                checkout = authority.split("\n      - ", 1)[0]
                self.assertIn("repository: aduboseh/iter", checkout)
                self.assertIn("ref: refs/heads/main", checkout)
                self.assertIn("path: authority", checkout)
                self.assertIn("persist-credentials: false", checkout)
                commands = [
                    shlex.split(line.strip())
                    for line in authority.replace("\\\n", " ").splitlines()
                    if line.strip().startswith("python3 ")
                ]
                self.assertEqual(len(commands), 2)
                for command in commands:
                    self.assertEqual(
                        command[:4],
                        [
                            "python3",
                            "-I",
                            "-B",
                            "authority/scripts/verify_productization_matrix.py",
                        ],
                    )
                    index = command.index("--iter-root")
                    self.assertEqual(command[index + 1], "iter")
                    self.assertNotIn("--matrix", command)
        release = (workflows / "release_gate.yml").read_text(encoding="utf-8")
        certification = release.split("  apex-productization:", 1)[1].split(
            "    steps:", 1
        )[0]
        self.assertIn("github.event_name == 'push'", certification)
        manual = (workflows / "apex_productization.yml").read_text(encoding="utf-8")
        certification = manual.split("  release-certification:", 1)[1]
        condition = certification.split("    steps:", 1)[0]
        self.assertIn("if: github.event_name == 'workflow_dispatch'", condition)
        self.assertNotIn("github.ref", condition)
        self.assertLess(
            certification.index("name: Reject non-main certification dispatch"),
            certification.index("name: Check out verifier authority"),
        )

    def test_non_main_dispatch_fails_instead_of_skipping_certification(self) -> None:
        """A selected non-main ref produces failure, not a green skipped job."""

        workflow = (
            SCRIPT.parents[1] / ".github/workflows/apex_productization.yml"
        ).read_text(encoding="utf-8")
        guard = workflow.split("name: Reject non-main certification dispatch", 1)[
            1
        ].split("\n      - ", 1)[0]
        self.assertIn("DISPATCH_REF: ${{ github.ref }}", guard)
        self.assertNotIn("\n        if:", guard)
        source = textwrap.dedent(
            guard.split("python3 -I - <<'PY'\n", 1)[1].split("          PY", 1)[0]
        )
        for ref in ("refs/heads/main", "refs/heads/release/v1.0", "refs/tags/v1.0", ""):
            with self.subTest(ref=ref):
                environment = os.environ.copy()
                environment["DISPATCH_REF"] = ref
                result = subprocess.run(
                    [sys.executable, "-I", "-c", source],
                    env=environment,
                    capture_output=True,
                    text=True,
                    timeout=30,
                    check=False,
                )
                self.assertEqual(result.returncode == 0, ref == "refs/heads/main")
                if ref != "refs/heads/main":
                    self.assertIn("::error::Certification dispatch", result.stderr)

    def test_workflow_preflight_cannot_execute_candidate_or_import_shadow(self) -> None:
        """Actual workflow argv rejects a run despite candidate scripts that exit 0."""

        workflows = SCRIPT.parents[1] / ".github" / "workflows"
        for name in ("apex_productization.yml", "release_gate.yml"):
            with self.subTest(workflow=name), tempfile.TemporaryDirectory() as temp:
                root = Path(temp)
                authority = root / "authority" / "scripts"
                candidate = root / "iter" / "scripts"
                authority.mkdir(parents=True)
                candidate.mkdir(parents=True)
                (authority / SCRIPT.name).write_bytes(SCRIPT.read_bytes())
                attack = "from pathlib import Path\nPath('candidate-executed').touch()\nraise SystemExit(0)\n"
                (candidate / SCRIPT.name).write_text(attack, encoding="utf-8")
                (candidate / "sitecustomize.py").write_text(attack, encoding="utf-8")
                workflow = (workflows / name).read_text(encoding="utf-8")
                line = next(
                    line
                    for line in workflow.replace("\\\n", " ").splitlines()
                    if "--verify-evidence-run" in line
                )
                command = shlex.split(line.strip())
                command[0] = sys.executable
                command[command.index("--evidence-run-id") + 1] = "0"
                environment = os.environ.copy()
                environment["PYTHONPATH"] = str(candidate)
                result = subprocess.run(
                    command,
                    cwd=root,
                    env=environment,
                    text=True,
                    capture_output=True,
                    timeout=30,
                    check=False,
                )
                self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
                self.assertIn("FAIL EVIDENCE-RUN", result.stdout)
                self.assertFalse((root / "candidate-executed").exists())

    def test_final_release_gate_distinguishes_readiness_from_certification(
        self,
    ) -> None:
        """Execute the workflow decision code against every non-success outcome."""

        workflow = (SCRIPT.parents[1] / ".github/workflows/release_gate.yml").read_text(
            encoding="utf-8"
        )
        final_gate = workflow.split("  release-gate:", 1)[1]
        self.assertIn("always()", final_gate)
        source = textwrap.dedent(
            final_gate.split("python3 -I - <<'PY'\n", 1)[1].split("          PY", 1)[0]
        )
        gates = (
            "governance",
            "sdk-rust",
            "sdk-typescript",
            "version-check",
            "changelog-check",
            "boundary-check",
        )
        passing = {gate: {"result": "success"} for gate in gates}
        passing["sbom"] = {"result": "skipped"}
        needs = final_gate.split("needs: [", 1)[1].split("]", 1)[0]
        self.assertEqual(
            {name.strip() for name in needs.split(",")},
            set(gates) | {"sbom", "apex-productization"},
        )
        cases = []
        for event in ("push", "pull_request", "workflow_dispatch"):
            for status in ("success", "failure", "cancelled", "skipped", None):
                values = copy.deepcopy(passing)
                if status is not None:
                    values["apex-productization"] = {"result": status}
                expected = (event, status) in (
                    ("push", "success"),
                    ("pull_request", "skipped"),
                )
                cases.append((event, "refs/heads/release/v1.0", values, expected))
        for event, certification in (("push", "success"), ("pull_request", "skipped")):
            for gate in gates:
                for status in ("failure", "cancelled", "skipped", None):
                    values = copy.deepcopy(passing)
                    values["apex-productization"] = {"result": certification}
                    if status is None:
                        del values[gate]
                    else:
                        values[gate] = {"result": status}
                    cases.append((event, "refs/heads/release/v1.0", values, False))
        for event, ref in (
            ("push", "refs/tags/v1.0"),
            ("push", "refs/heads/release/v1.0"),
            ("pull_request", "refs/pull/72/merge"),
        ):
            for status in ("success", "failure", "cancelled", "skipped", None):
                values = copy.deepcopy(passing)
                values["apex-productization"] = {
                    "result": "skipped" if event == "pull_request" else "success"
                }
                if status is None:
                    del values["sbom"]
                else:
                    values["sbom"] = {"result": status}
                expected = status == (
                    "success" if ref.startswith("refs/tags/v") else "skipped"
                )
                cases.append((event, ref, values, expected))
        for event, ref, values, expected in cases:
            with self.subTest(event=event, ref=ref, results=values):
                environment = os.environ.copy()
                environment.update(
                    EVENT_NAME=event, RELEASE_REF=ref, GATE_RESULTS=json.dumps(values)
                )
                result = subprocess.run(
                    [sys.executable, "-I", "-c", source],
                    env=environment,
                    capture_output=True,
                    text=True,
                    timeout=30,
                    check=False,
                )
                self.assertEqual(
                    result.returncode == 0, expected, result.stdout + result.stderr
                )
                if event == "pull_request" and expected:
                    self.assertIn("certification is NOT executed", result.stdout)


class ArtifactBindingTests(unittest.TestCase):
    """Exercise authenticated archive binding independently of producer activation."""

    def test_repository_metadata_endpoint_has_no_trailing_slash(self):
        """Use the REST repository root route proven by the live API probe."""
        with mock.patch.object(
            VERIFIER.subprocess,
            "run",
            return_value=subprocess.CompletedProcess([], 0, '{"id":789}', ""),
        ) as api:
            self.assertEqual(VERIFIER.github_json(""), {"id": 789})
        self.assertEqual(api.call_args.args[0][-1], "repos/aduboseh/iter")

    def setUp(self) -> None:
        """Use a real ZIP with API-shaped metadata, not a claimed local digest."""
        self.heads = {"iter": "a" * 40, "scg": "b" * 40}
        self.archive = self.zip_bytes([("result.txt", b"observed result")])
        self.metadata = {
            "id": 456,
            "name": "apex-productization-evidence-" + "a" * 40 + "-" + "b" * 40,
            "expired": False,
            "size_in_bytes": len(self.archive),
            "digest": "sha256:" + hashlib.sha256(self.archive).hexdigest(),
            "workflow_run": {
                "id": 123,
                "head_sha": self.heads["iter"],
                "repository_id": 789,
                "head_repository_id": 789,
            },
        }

    @staticmethod
    def zip_bytes(entries) -> bytes:
        """Create archive bytes including deliberately unsafe member fixtures."""
        import zipfile

        buffer = io.BytesIO()
        with zipfile.ZipFile(buffer, "w", zipfile.ZIP_DEFLATED) as archive:
            for name, data in entries:
                archive.writestr(name, data)
        return buffer.getvalue()

    def api(self, endpoint):
        """Return only the expected repository, listing and immutable-ID records."""
        if endpoint == "":
            return {"id": 789, "full_name": "aduboseh/iter", "fork": False}
        if endpoint.startswith("actions/runs/123/artifacts?"):
            return {"total_count": 1, "artifacts": [self.metadata]}
        if endpoint == "actions/artifacts/456":
            return self.metadata
        raise AssertionError(endpoint)

    def bind(self):
        """Invoke the production selector and ZIP verifier using a mock transport."""
        with (
            mock.patch.object(VERIFIER, "github_json", side_effect=self.api),
            mock.patch.object(
                VERIFIER, "download_evidence_archive", return_value=self.archive
            ),
        ):
            return VERIFIER.evidence_artifact_check(123, self.heads)

    def test_api_digest_binds_immutable_consumed_bytes_and_receipt(self):
        """A valid API-bound ZIP yields immutable bytes and an API-derived receipt."""
        files, receipt = self.bind()
        self.assertEqual(dict(files), {"result.txt": b"observed result"})
        self.assertEqual(receipt["artifact_id"], 456)
        self.assertEqual(receipt["run_id"], 123)
        self.assertEqual(receipt["digest"], self.metadata["digest"])
        self.assertEqual(receipt["subject_commits"], self.heads)
        with self.assertRaises(TypeError):
            files["result.txt"] = b"replaced"

    def test_archive_substitution_rejected_even_with_recomputed_local_hashes(self):
        """A real ZIP with rewritten content cannot use the original API digest."""
        self.archive = self.zip_bytes([("result.txt", b"forged result")])
        self.metadata["size_in_bytes"] = len(self.archive)
        with self.assertRaisesRegex(ValueError, "digest"):
            self.bind()

    def test_invalid_artifact_metadata_fails_before_download(self):
        """Wrong origin, expiry, identity, size and absent digest fail closed."""
        mutations = [
            ("id", True),
            ("id", 0),
            ("name", "other"),
            ("expired", True),
            ("expired", None),
            ("digest", None),
            ("digest", "sha256:" + "A" * 64),
            ("size_in_bytes", 0),
            ("size_in_bytes", True),
            ("size_in_bytes", 2**40),
            ("workflow_run.id", 999),
            ("workflow_run.id", True),
            ("workflow_run.head_sha", "c" * 40),
            ("workflow_run.repository_id", 999),
            ("workflow_run.head_repository_id", 999),
        ]
        original = copy.deepcopy(self.metadata)
        for field, value in mutations:
            with self.subTest(field=field, value=value):
                self.metadata = copy.deepcopy(original)
                target = self.metadata
                parts = field.split(".")
                if len(parts) == 2:
                    target = target[parts[0]]
                target[parts[-1]] = value
                with (
                    mock.patch.object(VERIFIER, "github_json", side_effect=self.api),
                    mock.patch.object(
                        VERIFIER, "download_evidence_archive"
                    ) as download,
                    self.assertRaises(ValueError),
                ):
                    VERIFIER.evidence_artifact_check(123, self.heads)
                download.assert_not_called()

    def test_ambiguous_missing_or_incomplete_listing_rejected(self):
        """A missing page or duplicate selection cannot become a unique artifact."""
        for count, records in [
            (0, []),
            (2, [self.metadata] * 2),
            (101, [self.metadata]),
            (True, [self.metadata]),
        ]:
            with self.subTest(count=count):

                def api(endpoint):
                    if endpoint.startswith("actions/runs/"):
                        return {"total_count": count, "artifacts": records}
                    return self.api(endpoint)

                with (
                    mock.patch.object(VERIFIER, "github_json", side_effect=api),
                    mock.patch.object(
                        VERIFIER, "download_evidence_archive"
                    ) as download,
                    self.assertRaises(ValueError),
                ):
                    VERIFIER.evidence_artifact_check(123, self.heads)
                download.assert_not_called()

    def test_unsafe_member_names_and_aliases_rejected(self):
        """No platform can reinterpret a member as a path outside the bundle."""
        names = [
            "../escape",
            "/abs",
            "C:/drive",
            "a\\b",
            "a//b",
            "a/./b",
            "a/../b",
            "a:b",
            "file.",
            "NUL",
            "COM1.txt",
            "a\x00suffix",
            "space ",
            "\u00e9",
        ]
        for name in names:
            with self.subTest(name=name):
                raw = self.zip_bytes([(name, b"x")])
                if "\\" in name:
                    raw = raw.replace(b"a/b", b"a\\b")
                # ZipInfo truncates NUL names when writing; patch both headers.
                if "\x00" in name:
                    raw = self.zip_bytes([("abc", b"x")]).replace(b"abc", b"a\x00c")
                with self.assertRaises(ValueError):
                    VERIFIER.verified_archive_files(
                        raw, "sha256:" + hashlib.sha256(raw).hexdigest()
                    )
        for entries in [
            [("a", b"x"), ("A", b"y")],
            [("a", b"x"), ("a/b", b"y")],
            [("a/b", b"x"), ("A/c", b"y")],
        ]:
            with self.subTest(entries=entries):
                raw = self.zip_bytes(entries)
                with self.assertRaises(ValueError):
                    VERIFIER.verified_archive_files(
                        raw, "sha256:" + hashlib.sha256(raw).hexdigest()
                    )

    def test_links_encryption_corruption_and_limits_rejected(self):
        """Even an API-matching digest cannot bless an unsafe or oversized ZIP."""
        import stat
        import zipfile

        link = zipfile.ZipInfo("link")
        link.create_system = 3
        link.external_attr = (stat.S_IFLNK | 0o777) << 16
        encrypted = bytearray(self.archive)
        encrypted[6] |= 1
        offset = encrypted.index(b"PK\x01\x02")
        encrypted[offset + 8] |= 1
        for raw in [
            b"not zip",
            self.archive[:-10],
            self.zip_bytes([(link, b"target")]),
            bytes(encrypted),
        ]:
            with self.subTest(raw=raw[:20]), self.assertRaises(ValueError):
                VERIFIER.verified_archive_files(
                    raw, "sha256:" + hashlib.sha256(raw).hexdigest()
                )
        for limit in [
            "MAX_ARCHIVE_BYTES",
            "MAX_EVIDENCE_BYTES",
            "MAX_EVIDENCE_FILE_BYTES",
            "MAX_EVIDENCE_MEMBERS",
        ]:
            with (
                self.subTest(limit=limit),
                mock.patch.object(VERIFIER, limit, 1),
                self.assertRaises(ValueError),
            ):
                raw = self.zip_bytes([("a", b"xx"), ("b", b"xx")])
                VERIFIER.verified_archive_files(
                    raw, "sha256:" + hashlib.sha256(raw).hexdigest()
                )

    def test_cli_binding_failure_cannot_be_allowed_or_consume_evidence(self):
        """Full execution cannot downgrade artifact rejection to advisory status."""
        with (
            mock.patch(
                "sys.argv",
                [
                    str(SCRIPT),
                    "--control",
                    "G1-01",
                    "--evidence-run-id",
                    "123",
                    "--allow-failures",
                ],
            ),
            mock.patch.object(VERIFIER, "git_head", return_value="a" * 40),
            mock.patch.object(
                VERIFIER, "git_worktree_clean", return_value=(True, "clean")
            ),
            mock.patch.object(
                VERIFIER, "directive_mirror_check", return_value=(True, "matches")
            ),
            mock.patch.object(
                VERIFIER, "scg_release_ref_check", return_value=(True, "matches")
            ),
            mock.patch.object(
                VERIFIER, "evidence_run_check", return_value=(True, "approved")
            ),
            mock.patch.object(
                VERIFIER,
                "evidence_artifact_check",
                side_effect=ValueError("digest mismatch"),
            ),
            mock.patch.object(VERIFIER, "evidence_check") as consume,
            contextlib.redirect_stdout(io.StringIO()),
        ):
            self.assertEqual(VERIFIER.main(), 1)
        consume.assert_not_called()

    def test_artifact_identity_cannot_change_between_listing_and_download(self):
        """Replaced metadata and a mismatched repository cannot authorize download."""
        for endpoint in ("actions/artifacts/456", ""):
            with self.subTest(endpoint=endpoint):

                def api(value):
                    result = copy.deepcopy(self.api(value))
                    if value == endpoint:
                        result["id"] = 999
                    return result

                with (
                    mock.patch.object(VERIFIER, "github_json", side_effect=api),
                    mock.patch.object(
                        VERIFIER, "download_evidence_archive"
                    ) as download,
                    self.assertRaises(ValueError),
                ):
                    VERIFIER.evidence_artifact_check(123, self.heads)
                download.assert_not_called()

    def test_duplicate_members_bad_crc_and_size_mismatch_rejected(self):
        """ZIP integrity errors cannot be masked by a valid transport digest."""
        import warnings
        import zipfile

        with warnings.catch_warnings():
            warnings.simplefilter("ignore", UserWarning)
            duplicate = self.zip_bytes([("same", b"x"), ("same", b"y")])
        buffer = io.BytesIO()
        with zipfile.ZipFile(buffer, "w", zipfile.ZIP_STORED) as archive:
            archive.writestr("a", b"original")
        corrupt = buffer.getvalue().replace(b"original", b"tampered")
        for raw in (duplicate, corrupt):
            with self.assertRaises(ValueError):
                VERIFIER.verified_archive_files(
                    raw, "sha256:" + hashlib.sha256(raw).hexdigest()
                )
        self.metadata["size_in_bytes"] += 1
        with self.assertRaisesRegex(ValueError, "size mismatch"):
            self.bind()

    def test_download_is_bounded_and_does_not_forward_or_log_credentials(self):
        """Exercise actual transport code with a signed redirect and bounded stream."""
        from urllib.error import HTTPError

        token = "fixture-secret"
        signed_url = "https://storage.example.test/archive?secret=signed"
        opener = mock.Mock()
        response = mock.MagicMock()
        response.status = 200
        response.__enter__.return_value = response
        response.read1.side_effect = [self.archive, b""]
        opener.open.side_effect = [
            HTTPError(
                "https://api.github.com", 302, "Found", {"Location": signed_url}, None
            ),
            response,
        ]
        with (
            mock.patch.object(VERIFIER, "build_opener", return_value=opener),
            mock.patch.object(
                VERIFIER.subprocess,
                "run",
                return_value=subprocess.CompletedProcess([], 0, token, ""),
            ),
        ):
            self.assertEqual(VERIFIER.download_evidence_archive(456), self.archive)
        api_request = opener.open.call_args_list[0].args[0]
        storage_request = opener.open.call_args_list[1].args[0]
        self.assertEqual(
            api_request.full_url,
            "https://api.github.com/repos/aduboseh/iter/actions/artifacts/456/zip",
        )
        self.assertEqual(api_request.get_header("Authorization"), "Bearer " + token)
        self.assertIsNone(storage_request.get_header("Authorization"))
        from http.client import IncompleteRead

        for failure in (
            IncompleteRead(b"protected-fixture", 5),
            TimeoutError(signed_url),
            HTTPError(signed_url, 403, token, {}, None),
        ):
            opener.open.side_effect = failure
            with (
                mock.patch.object(VERIFIER, "build_opener", return_value=opener),
                mock.patch.object(
                    VERIFIER.subprocess,
                    "run",
                    return_value=subprocess.CompletedProcess([], 0, token, ""),
                ),
                self.assertRaises(ValueError) as error,
            ):
                VERIFIER.download_evidence_archive(456)
            self.assertNotIn(token, str(error.exception))
            self.assertNotIn(signed_url, str(error.exception))
        self.assertIsNone(
            VERIFIER.NoArtifactRedirect().redirect_request(
                None, None, 302, "", {}, signed_url
            )
        )

    def test_download_rejects_unsafe_redirect_and_stream_limits(self):
        """Neither metadata nor a slow storage stream can escape the transport limits."""
        from urllib.error import HTTPError

        for url, oversized, late in [
            ("http://storage.test/file", False, False),
            ("https://user:secret@storage.test/file", False, False),
            ("https://storage.test:444/file", False, False),
            ("https://storage.test/file", True, False),
            ("https://storage.test/file", False, True),
        ]:
            with self.subTest(url=url, oversized=oversized, late=late):
                opener = mock.Mock()
                response = mock.MagicMock()
                response.status = 200
                response.__enter__.return_value = response
                response.read1.side_effect = [b"012345678", b""]
                opener.open.side_effect = [
                    HTTPError(
                        "https://api.github.com", 302, "Found", {"Location": url}, None
                    ),
                    response,
                ]
                with (
                    mock.patch.object(VERIFIER, "build_opener", return_value=opener),
                    mock.patch.object(
                        VERIFIER.subprocess,
                        "run",
                        return_value=subprocess.CompletedProcess([], 0, "secret", ""),
                    ),
                    mock.patch.object(
                        VERIFIER, "MAX_ARCHIVE_BYTES", 8 if oversized else 1024
                    ),
                    mock.patch.object(
                        VERIFIER.time,
                        "monotonic",
                        side_effect=[0, 121 if late else 0, 0],
                    ),
                    self.assertRaises(ValueError),
                ):
                    VERIFIER.download_evidence_archive(456)

    def test_local_directory_is_rejected_even_after_run_authentication(self):
        """A caller cannot substitute a local bundle or recompute its declarations."""
        with (
            mock.patch(
                "sys.argv",
                [
                    str(SCRIPT),
                    "--control",
                    "G1-01",
                    "--evidence-run-id",
                    "123",
                    "--evidence-dir",
                    "forged",
                    "--allow-failures",
                ],
            ),
            mock.patch.object(VERIFIER, "git_head", return_value="a" * 40),
            mock.patch.object(
                VERIFIER, "git_worktree_clean", return_value=(True, "clean")
            ),
            mock.patch.object(
                VERIFIER, "directive_mirror_check", return_value=(True, "matches")
            ),
            mock.patch.object(
                VERIFIER, "scg_release_ref_check", return_value=(True, "matches")
            ),
            mock.patch.object(
                VERIFIER, "evidence_run_check", return_value=(True, "approved")
            ),
            mock.patch.object(VERIFIER, "evidence_artifact_check") as bind,
            contextlib.redirect_stdout(io.StringIO()),
        ):
            self.assertEqual(VERIFIER.main(), 1)
        bind.assert_not_called()

    def main_record_report(self, evidence, allow_failures=False):
        """Exercise main with authenticated transport fixtures and real record checks."""
        _, files = evidence_fixture()
        self.archive = self.zip_bytes(
            [
                ("G1-01-linux-x86_64.json", json.dumps(evidence).encode()),
                *files.items(),
            ]
        )
        self.metadata["size_in_bytes"] = len(self.archive)
        self.metadata["digest"] = "sha256:" + hashlib.sha256(self.archive).hexdigest()
        with tempfile.TemporaryDirectory() as temp:
            report = Path(temp) / "report.json"
            with (
                mock.patch(
                    "sys.argv",
                    [
                        str(SCRIPT),
                        "--control",
                        "G1-01",
                        "--evidence-run-id",
                        "123",
                        "--report",
                        str(report),
                        *(["--allow-failures"] if allow_failures else []),
                    ],
                ),
                mock.patch.object(
                    VERIFIER,
                    "git_head",
                    side_effect=[
                        self.heads["iter"],
                        self.heads["scg"],
                        "c" * 40,
                        self.heads["iter"],
                        self.heads["scg"],
                    ],
                ),
                mock.patch.object(
                    VERIFIER, "git_worktree_clean", return_value=(True, "clean")
                ),
                mock.patch.object(
                    VERIFIER, "directive_mirror_check", return_value=(True, "matches")
                ),
                mock.patch.object(
                    VERIFIER, "scg_release_ref_check", return_value=(True, "matches")
                ),
                mock.patch.object(
                    VERIFIER, "evidence_run_check", return_value=(True, "approved")
                ),
                mock.patch.object(VERIFIER, "github_json", side_effect=self.api),
                mock.patch.object(
                    VERIFIER, "download_evidence_archive", return_value=self.archive
                ),
                contextlib.redirect_stdout(io.StringIO()),
            ):
                code = VERIFIER.main()
            return code, json.loads(report.read_text())

    def test_main_consumes_archive_bytes_and_reports_separate_authority(self):
        """A synthetic bundle reaches evaluation and reports authenticated bindings."""
        evidence, _ = evidence_fixture()
        code, result = self.main_record_report(evidence)
        self.assertEqual(code, 0)
        self.assertEqual(result["verifier_authority_commit"], "c" * 40)
        self.assertEqual(result["evidence_artifact"]["digest"], self.metadata["digest"])
        self.assertEqual(result["evidence_artifact"]["artifact_id"], 456)
        self.assertEqual(result["subject_commits"], self.heads)
        self.assertEqual(result["summary"]["status"], "PARTIAL")
        self.assertEqual(result["controls"][0]["status"], "PASS")

    def test_incomplete_record_remains_fail_in_advisory_mode(self):
        """Transport authentication and advisory exit zero never upgrade a failed record."""
        evidence, _ = evidence_fixture()
        del evidence["execution_identity"]
        for advisory in (False, True):
            with self.subTest(advisory=advisory):
                code, result = self.main_record_report(evidence, advisory)
                self.assertEqual(code, 0 if advisory else 1)
                self.assertEqual(result["controls"][0]["status"], "FAIL")
                self.assertEqual(result["summary"]["status"], "FAIL")


class EvidenceSchemaTests(unittest.TestCase):
    """Reject incomplete declarations without claiming execution attestation."""

    def setUp(self):
        """Keep each case's declarations and raw artifact bytes isolated."""
        self.record, self.files = evidence_fixture()
        self.heads = dict(self.record["subject_commits"])

    def check(self, record=None, raw=None, run_id=123):
        """Evaluate bytes through the production evidence entry point."""
        if raw is None:
            raw = json.dumps(self.record if record is None else record).encode()
        return VERIFIER.evidence_check(
            "G1-01",
            {"file": "G1-01.json"},
            {**self.files, "G1-01.json": raw},
            self.heads,
            run_id,
        )

    def test_complete_v2_and_negative_exit_commands_pass(self):
        """Preserve empty argv operands, empty logs and intentional negative tests."""
        self.assertTrue(self.check()[0])
        self.record["commands"][0].update(exit_code=101, expected_exit=101)
        self.record["commands"][0]["cwd"] = "tests/fixtures"
        self.assertTrue(self.check()[0])

    def test_missing_unknown_and_null_fields_at_every_object_fail(self):
        """Every declared field is required; silent typo/default acceptance is forbidden."""

        def objects(value, path=()):
            if isinstance(value, dict):
                yield path, value
                for key, child in value.items():
                    yield from objects(child, (*path, key))
            elif isinstance(value, list):
                for index, child in enumerate(value):
                    yield from objects(child, (*path, index))

        for path, original in objects(self.record):
            for key in (*original, "unexpected"):
                for action in ("delete", "null") if key != "unexpected" else ("add",):
                    with self.subTest(path=path, key=key, action=action):
                        record = copy.deepcopy(self.record)
                        target = record
                        for part in path:
                            target = target[part]
                        if action == "delete":
                            del target[key]
                        else:
                            target[key] = None
                        self.assertFalse(self.check(record)[0])

    def test_legacy_version_and_incomplete_identity_rejected(self):
        """Neither a v1 record nor a renamed v1 record can claim completeness."""
        self.record["schema_version"] = "apex-productization-evidence/v1"
        self.assertFalse(self.check()[0])
        self.record["commands"] = ["synthetic-collector"]
        del self.record["execution_identity"]
        self.assertFalse(self.check()[0])
        self.record["schema_version"] = "apex-productization-evidence/v2"
        self.assertFalse(self.check()[0])

    def test_ambiguous_and_nonstandard_json_rejected(self):
        """Reject duplicate keys, NaN/Infinity and malformed or overnested JSON."""
        raw = json.dumps(self.record)
        for malformed in (
            raw.replace('"result": "PASS"', '"result": "FAIL", "result": "PASS"', 1),
            raw.replace('"run_id": 123', '"run_id": 999, "run_id": 123', 1),
            raw.replace('"exit_code": 0', '"exit_code": NaN', 1),
            raw.replace('"exit_code": 0', '"exit_code": Infinity', 1),
            raw.replace('"exit_code": 0', '"exit_code": -Infinity', 1),
            "null",
            "[]",
            "{",
            "[" * 1200 + "]" * 1200,
        ):
            with self.subTest(raw=malformed[:80]):
                self.assertFalse(self.check(raw=malformed.encode())[0])

    def test_identity_strings_arrays_and_subjects_rejected(self):
        """Reject blank identities, missing toolchains and malformed commit strings."""
        paths = (
            ("execution_identity", "runner", "os"),
            ("execution_identity", "runner", "architecture"),
            ("execution_identity", "runner", "image"),
            ("execution_identity", "toolchains", 0, "name"),
            ("execution_identity", "toolchains", 0, "version"),
        )
        for path in paths:
            for invalid in ("", " \t", 12, True, [], {}):
                record = copy.deepcopy(self.record)
                target = record
                for part in path[:-1]:
                    target = target[part]
                target[path[-1]] = invalid
                with self.subTest(path=path, invalid=invalid):
                    self.assertFalse(self.check(record)[0])
        for path in (
            ("commands",),
            ("artifacts",),
            ("execution_identity", "toolchains"),
        ):
            for invalid in ([], {}, "not-an-array", [None]):
                record = copy.deepcopy(self.record)
                target = record
                for part in path[:-1]:
                    target = target[part]
                target[path[-1]] = invalid
                self.assertFalse(self.check(record)[0])
        for invalid in ("a" * 39, "A" * 40, "a" * 40 + "\n", True):
            record = copy.deepcopy(self.record)
            record["subject_commits"]["iter"] = invalid
            self.heads["iter"] = invalid
            self.assertFalse(self.check(record)[0])

    def test_run_identity_is_strict_integer_first_attempt(self):
        """Python bool/float equality must not impersonate a run or first attempt."""
        for key, invalid in (
            ("run_id", True),
            ("run_id", 123.0),
            ("run_id", 0),
            ("run_id", "123"),
            ("run_id", 124),
            ("run_attempt", True),
            ("run_attempt", 1.0),
            ("run_attempt", 2),
        ):
            record = copy.deepcopy(self.record)
            record["producer"][key] = invalid
            with self.subTest(key=key, invalid=invalid):
                self.assertFalse(self.check(record)[0])
        for invalid in (None, True, 123.0, 0):
            self.assertFalse(self.check(run_id=invalid)[0])

    def test_every_reference_requires_declared_verified_bytes(self):
        """Hash-only claims and undeclared logs cannot satisfy execution identity."""
        references = [
            ("execution_identity", "corpus"),
            ("execution_identity", "configuration"),
            ("execution_identity", "result"),
            ("execution_identity", "toolchains", 0, "version_log"),
            ("commands", 0, "stdout"),
            ("commands", 0, "stderr"),
        ]
        for path in references:
            for replacement in ("undeclared.txt", "../escape", "A" * 64, 1, ""):
                record = copy.deepcopy(self.record)
                target = record
                for part in path[:-1]:
                    target = target[part]
                target[path[-1]] = replacement
                with self.subTest(path=path, replacement=replacement):
                    self.assertFalse(self.check(record)[0])
        for artifact in self.record["artifacts"]:
            path = artifact["path"]
            original = self.files.pop(path)
            with self.subTest(path=path, case="missing"):
                self.assertFalse(self.check()[0])
            self.files[path] = original + b"tampered"
            with self.subTest(path=path, case="tampered"):
                self.assertFalse(self.check()[0])
            self.files[path] = original
            record = copy.deepcopy(self.record)
            record["artifacts"] = [a for a in record["artifacts"] if a["path"] != path]
            with self.subTest(path=path, case="undeclared"):
                self.assertFalse(self.check(record)[0])

    def test_duplicate_artifact_declaration_rejected(self):
        """Repeated declarations are ambiguous even with the same digest."""
        self.record["artifacts"].append(dict(self.record["artifacts"][0]))
        self.assertFalse(self.check()[0])

    def test_command_shape_paths_and_failed_exit_rejected(self):
        """Require exact argv, contained cwd and matching non-boolean exit codes."""
        cases = (
            ("argv", "cargo test"),
            ("argv", []),
            ("argv", [""]),
            ("argv", [" "]),
            ("argv", ["cargo", 1]),
            ("repo", "host"),
            ("cwd", "../other"),
            ("cwd", "/tmp"),
            ("cwd", "C:\\tmp"),
            ("cwd", ""),
            ("cwd", 1),
            ("exit_code", 1),
            ("exit_code", True),
            ("exit_code", 0.0),
            ("expected_exit", False),
            ("expected_exit", 0.0),
        )
        for key, invalid in cases:
            record = copy.deepcopy(self.record)
            record["commands"][0][key] = invalid
            with self.subTest(key=key, invalid=invalid):
                self.assertFalse(self.check(record)[0])


class ExternalEvidenceTests(unittest.TestCase):
    """Prove evidence can bind immutable commits without entering the source tree."""

    def test_evidence_json_path_escape_is_rejected(self) -> None:
        """Relative and absolute paths cannot escape the downloaded bundle."""

        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            evidence_dir = root / "evidence"
            evidence_dir.mkdir()
            outside = root / "outside.json"
            outside.write_text("{}", encoding="utf-8")
            heads = {"iter": "a" * 40, "scg": "b" * 40}

            for escaped_path in ("../outside.json", str(outside.resolve())):
                with self.subTest(path=escaped_path):
                    passed, detail, _ = VERIFIER.evidence_check(
                        "G1-01",
                        {"file": escaped_path},
                        {
                            p.relative_to(evidence_dir).as_posix(): p.read_bytes()
                            for p in evidence_dir.rglob("*")
                            if p.is_file()
                        },
                        heads,
                        123,
                    )
                    self.assertFalse(passed)
                    self.assertIn("evidence member path", detail)

    def test_external_evidence_binds_subjects_and_artifact(self) -> None:
        """A complete external record binds execution files and producer identity."""
        evidence, files = evidence_fixture()

        def check():
            return VERIFIER.evidence_check(
                "G1-01",
                {"file": "G1-01.json"},
                {**files, "G1-01.json": json.dumps(evidence).encode()},
                {"iter": "a" * 40, "scg": "b" * 40},
                123,
            )

        passed, detail, _ = check()
        self.assertTrue(passed, detail)
        commands = evidence["commands"]
        evidence["commands"] = [None]
        passed, detail, _ = check()
        self.assertFalse(passed)
        self.assertIn("command must be an object", detail)
        evidence["commands"] = commands
        evidence["producer"]["workflow"] = ".github/workflows/untrusted.yml"
        passed, detail, _ = check()
        self.assertFalse(passed)
        self.assertIn("producer.workflow", detail)

    def test_stale_external_evidence_fails_closed(self) -> None:
        """A stale subject fails even when execution artifact digests are valid."""
        evidence, files = evidence_fixture()
        evidence["subject_commits"]["iter"] = "c" * 40
        passed, detail, _ = VERIFIER.evidence_check(
            "G1-01",
            {"file": "G1-01.json"},
            {**files, "G1-01.json": json.dumps(evidence).encode()},
            {"iter": "a" * 40, "scg": "b" * 40},
            123,
        )
        self.assertFalse(passed)
        self.assertIn("does not match", detail)


class RepositoryPathTests(unittest.TestCase):
    """Prove repository checks cannot consume sibling or host files."""

    def test_runtime_path_escape_is_rejected_for_path_and_regex(self) -> None:
        """Both repository-path executors fail closed on escaped targets."""

        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo = root / "iter"
            repo.mkdir()
            outside = root / "outside.txt"
            outside.write_text("trusted-looking content", encoding="utf-8")
            roots = {"iter": repo}

            checks = (
                (
                    VERIFIER.path_check,
                    {"type": "path_exists", "repo": "iter"},
                ),
                (
                    VERIFIER.regex_check,
                    {"type": "regex", "repo": "iter", "pattern": "trusted"},
                ),
            )
            for runner, check in checks:
                for escaped_path in ("../outside.txt", str(outside.resolve())):
                    with self.subTest(check=check["type"], path=escaped_path):
                        passed, detail, _ = runner(
                            {**check, "path": escaped_path}, roots
                        )
                        self.assertFalse(passed)
                        self.assertIn("escapes declared repository", detail)


class CrossRepositoryContractTests(unittest.TestCase):
    """Prove the pinned SCG contract is compared to Iter's vendored contract."""

    def test_equal_files_pass_and_byte_divergence_fails(self) -> None:
        """Raw-byte equality passes, while any mutation fails with both digests."""

        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            iter_root = root / "iter"
            scg_root = root / "scg"
            iter_root.mkdir()
            scg_root.mkdir()
            (iter_root / "contract.rs").write_bytes(b"canonical\n")
            (scg_root / "contract.rs").write_bytes(b"canonical\n")
            check = {
                "type": "cross_repo_equal",
                "left_repo": "iter",
                "left_path": "contract.rs",
                "right_repo": "scg",
                "right_path": "contract.rs",
            }
            roots = {"iter": iter_root, "scg": scg_root}

            passed, detail, _ = VERIFIER.cross_repo_equal_check(check, roots)
            self.assertTrue(passed, detail)

            (scg_root / "contract.rs").write_bytes(b"mutated\n")
            passed, detail, _ = VERIFIER.cross_repo_equal_check(check, roots)
            self.assertFalse(passed)
            self.assertIn("cross-repository mismatch", detail)
            self.assertIn(hashlib.sha256(b"canonical\n").hexdigest(), detail)
            self.assertIn(hashlib.sha256(b"mutated\n").hexdigest(), detail)

    def test_escaped_path_fails_closed(self) -> None:
        """Runtime containment rejects a path outside either repository root."""

        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            iter_root = root / "iter"
            scg_root = root / "scg"
            iter_root.mkdir()
            scg_root.mkdir()
            (root / "outside.rs").write_bytes(b"canonical\n")
            (scg_root / "contract.rs").write_bytes(b"canonical\n")
            check = {
                "type": "cross_repo_equal",
                "left_repo": "iter",
                "left_path": "../outside.rs",
                "right_repo": "scg",
                "right_path": "contract.rs",
            }

            passed, detail, _ = VERIFIER.cross_repo_equal_check(
                check, {"iter": iter_root, "scg": scg_root}
            )
            self.assertFalse(passed)
            self.assertIn("escapes declared repository", detail)


class DirectiveMirrorTests(unittest.TestCase):
    """Prove both authority documents are required and byte-identical."""

    def test_both_directives_are_verified(self) -> None:
        """A missing or divergent amendment fails the authority check."""

        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            iter_root = root / "iter"
            scg_root = root / "scg"
            iter_root.mkdir()
            scg_root.mkdir()
            for name in VERIFIER.DIRECTIVE_MIRRORS:
                (iter_root / name).write_bytes(b"authority\n")
                (scg_root / name).write_bytes(b"authority\n")

            passed, detail = VERIFIER.directive_mirror_check(iter_root, scg_root)
            self.assertTrue(passed, detail)
            self.assertIn(VERIFIER.AUTHORITY_AMENDMENT, detail)

            (scg_root / VERIFIER.AUTHORITY_AMENDMENT).write_bytes(b"mutated\n")
            passed, detail = VERIFIER.directive_mirror_check(iter_root, scg_root)
            self.assertFalse(passed)
            self.assertIn(VERIFIER.AUTHORITY_AMENDMENT, detail)


class GitWorktreeStateTests(unittest.TestCase):
    """Prove certification rejects source content outside the recorded commit."""

    def test_clean_then_dirty_worktree(self) -> None:
        """A committed tree passes and an untracked source file fails closed."""

        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            run_git(root, "init")
            run_git(root, "config", "user.name", "APEX Test")
            run_git(root, "config", "user.email", "apex-test@example.invalid")
            (root / "source.txt").write_text("stable\n", encoding="utf-8")
            run_git(root, "add", "source.txt")
            run_git(root, "commit", "-m", "fixture")

            self.assertEqual(
                VERIFIER.git_worktree_clean(root),
                (True, "repository worktree is clean"),
            )
            (root / "untracked.txt").write_text("mutation\n", encoding="utf-8")
            clean, detail = VERIFIER.git_worktree_clean(root)
            self.assertFalse(clean)
            self.assertIn("untracked.txt", detail)


class ScgReleaseRefTests(unittest.TestCase):
    """Prove certification cannot drift from iter's declared SCG subject."""

    def test_release_ref_must_match_checked_out_scg(self) -> None:
        """The exact pinned SCG commit passes and any other commit fails."""

        with tempfile.TemporaryDirectory() as temp_dir:
            iter_root = Path(temp_dir)
            productization = iter_root / "productization"
            productization.mkdir()
            pinned = "d" * 40
            (productization / "SCG_RELEASE_REF").write_text(
                pinned + "\n", encoding="utf-8"
            )

            self.assertEqual(
                VERIFIER.scg_release_ref_check(iter_root, pinned),
                (True, f"SCG release pin matches checked-out subject: {pinned}"),
            )
            passed, detail = VERIFIER.scg_release_ref_check(iter_root, "e" * 40)
            self.assertFalse(passed)
            self.assertIn("mismatch", detail)

    def test_release_ref_rejects_noncanonical_value(self) -> None:
        """A moving branch name or malformed digest fails closed."""

        with tempfile.TemporaryDirectory() as temp_dir:
            iter_root = Path(temp_dir)
            productization = iter_root / "productization"
            productization.mkdir()
            (productization / "SCG_RELEASE_REF").write_text(
                "master\n", encoding="utf-8"
            )

            passed, detail = VERIFIER.scg_release_ref_check(iter_root, "d" * 40)
            self.assertFalse(passed)
            self.assertIn("invalid", detail)


if __name__ == "__main__":
    unittest.main()
