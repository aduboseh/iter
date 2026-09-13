"""Focused tests for APEX certification status and external evidence binding."""

from __future__ import annotations

import hashlib
import copy
import contextlib
import importlib.util
import io
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest import mock


SCRIPT = Path(__file__).resolve().parents[1] / "scripts" / "verify_productization_matrix.py"
SPEC = importlib.util.spec_from_file_location("verify_productization_matrix", SCRIPT)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError(f"cannot load verifier: {SCRIPT}")
VERIFIER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(VERIFIER)


def results(count: int, status: str = "PASS") -> list[dict[str, str]]:
    """Build minimal control results for certification-status tests."""

    return [{"status": status} for _ in range(count)]


def producer(run_id: int = 123) -> dict[str, object]:
    """Build the trusted producer metadata required by evidence files."""

    return {
        "repository": VERIFIER.EVIDENCE_PRODUCER_REPOSITORY,
        "workflow": VERIFIER.TRUSTED_EVIDENCE_WORKFLOW,
        "run_id": run_id,
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
        passed, detail = self.check()
        self.assertTrue(passed, detail)

    def test_run_metadata_mutations_are_rejected(self) -> None:
        for field, value in (
            ("id", 124),
            ("id", True),
            ("head_sha", "b" * 40),
            ("head_branch", "unprotected"),
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
        for field in list(self.run):
            with self.subTest(field=field):
                value = self.run.pop(field)
                self.assertFalse(self.check()[0])
                self.run[field] = value

    def test_environment_policy_mutations_are_rejected(self) -> None:
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
        with mock.patch.object(
            VERIFIER, "github_json", side_effect=[self.run, {"protected": False}]
        ):
            self.assertFalse(VERIFIER.evidence_run_check(123, self.commit)[0])

    def test_protected_main_ancestry_is_required_not_just_ref_name(self) -> None:
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
            self.assertLess(
                workflow.index("--verify-evidence-run"),
                workflow.index("uses: actions/download-artifact"),
            )
        required = (workflows / "mcp_integration.yml").read_text(encoding="utf-8")
        self.assertIn("python -B tests/test_productization_matrix.py", required)
        self.assertIn("python -BO tests/test_productization_matrix.py", required)


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
                        evidence_dir,
                        heads,
                        123,
                    )
                    self.assertFalse(passed)
                    self.assertIn("escapes evidence directory", detail)

    def test_external_evidence_binds_subjects_and_artifact(self) -> None:
        """A valid external bundle passes exact commit and SHA-256 checks."""

        with tempfile.TemporaryDirectory() as temp_dir:
            evidence_dir = Path(temp_dir)
            artifact_dir = evidence_dir / "artifacts" / "G1-01"
            artifact_dir.mkdir(parents=True)
            artifact = artifact_dir / "certification.json"
            artifact.write_bytes(b'{"deterministic":true}\n')
            digest = hashlib.sha256(artifact.read_bytes()).hexdigest()
            heads = {"iter": "a" * 40, "scg": "b" * 40}
            evidence = {
                "schema_version": VERIFIER.EVIDENCE_SCHEMA,
                "control_id": "G1-01",
                "result": "PASS",
                "producer": producer(),
                "subject_commits": heads,
                "commands": ["cargo test --locked --workspace"],
                "artifacts": [
                    {
                        "path": "artifacts/G1-01/certification.json",
                        "sha256": digest,
                    }
                ],
            }
            (evidence_dir / "G1-01.json").write_text(
                json.dumps(evidence), encoding="utf-8"
            )

            passed, detail, _ = VERIFIER.evidence_check(
                "G1-01",
                {"file": "G1-01.json"},
                evidence_dir,
                heads,
                123,
            )

            self.assertTrue(passed, detail)

            evidence["commands"] = [None]
            (evidence_dir / "G1-01.json").write_text(
                json.dumps(evidence), encoding="utf-8"
            )
            passed, detail, _ = VERIFIER.evidence_check(
                "G1-01",
                {"file": "G1-01.json"},
                evidence_dir,
                heads,
                123,
            )
            self.assertFalse(passed)
            self.assertIn("non-empty strings", detail)

            evidence["commands"] = ["cargo test --locked --workspace"]
            evidence["producer"] = producer()
            evidence["producer"]["workflow"] = ".github/workflows/untrusted.yml"
            (evidence_dir / "G1-01.json").write_text(
                json.dumps(evidence), encoding="utf-8"
            )
            passed, detail, _ = VERIFIER.evidence_check(
                "G1-01",
                {"file": "G1-01.json"},
                evidence_dir,
                heads,
                123,
            )
            self.assertFalse(passed)
            self.assertIn("producer.workflow", detail)

    def test_stale_external_evidence_fails_closed(self) -> None:
        """A stale iter commit is rejected even when the artifact digest is valid."""

        with tempfile.TemporaryDirectory() as temp_dir:
            evidence_dir = Path(temp_dir)
            artifact = evidence_dir / "artifact.txt"
            artifact.write_text("evidence", encoding="utf-8")
            evidence = {
                "schema_version": VERIFIER.EVIDENCE_SCHEMA,
                "control_id": "G1-01",
                "result": "PASS",
                "producer": producer(),
                "subject_commits": {"iter": "c" * 40, "scg": "b" * 40},
                "commands": ["certify"],
                "artifacts": [
                    {
                        "path": "artifact.txt",
                        "sha256": hashlib.sha256(artifact.read_bytes()).hexdigest(),
                    }
                ],
            }
            (evidence_dir / "G1-01.json").write_text(
                json.dumps(evidence), encoding="utf-8"
            )

            passed, detail, _ = VERIFIER.evidence_check(
                "G1-01",
                {"file": "G1-01.json"},
                evidence_dir,
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
