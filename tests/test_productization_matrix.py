"""Focused tests for APEX certification status and external evidence binding."""

from __future__ import annotations

import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import tempfile
import unittest


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
        ["git", *args],
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

    def test_canonical_control_id_substitution_is_rejected(self) -> None:
        """A syntactically valid replacement cannot hide a required control."""

        matrix = VERIFIER.load_json(
            Path(__file__).resolve().parents[1]
            / "productization"
            / "APEX_RELEASE_MATRIX_V1.json"
        )
        control = next(item for item in matrix["controls"] if item["id"] == "G2-01")
        control["id"] = "G2-99"
        with self.assertRaisesRegex(ValueError, "canonical v1 set"):
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
