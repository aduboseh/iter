"""Focused tests for APEX certification status and external evidence binding."""

from __future__ import annotations

import hashlib
import importlib.util
import json
from pathlib import Path
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
                "G1-01", {"file": "G1-01.json"}, evidence_dir, heads
            )

            self.assertTrue(passed, detail)

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
            )

            self.assertFalse(passed)
            self.assertIn("does not match", detail)


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
