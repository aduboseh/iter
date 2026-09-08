"""Negative regression tests for the local-only verification harness."""

import copy
import importlib.util
import json
import os
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

SPEC = importlib.util.spec_from_file_location(
    "local_stack", Path(__file__).resolve().parents[1] / "scripts/verify_local_stack.py")
LOCAL = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(LOCAL)


class LocalStackTests(unittest.TestCase):
    def setUp(self):
        self.outcome = {
            "mode": "governed", "verdict": "ALLOW", "authoritative_pdp": True,
            "trace_available": True, "replay_sufficient": True,
            "packet": {"governance_hash": "authority", "state_snapshot_hash": "snapshot",
                       "execution_trace": ["step"]},
        }

    def test_valid_outcome_requires_all_observable_guarantees(self):
        self.assertEqual(LOCAL.validate_outcome(self.outcome, "authority", "snapshot"),
                         self.outcome["packet"])

    def test_each_missing_guarantee_fails(self):
        for field in self.outcome:
            with self.subTest(field=field):
                outcome = copy.deepcopy(self.outcome)
                del outcome[field]
                with self.assertRaises(RuntimeError):
                    LOCAL.validate_outcome(outcome, "authority", "snapshot")

    def test_demo_fallback_is_not_a_pass(self):
        self.outcome["mode"] = "demo"
        with self.assertRaises(RuntimeError):
            LOCAL.validate_outcome(self.outcome, "authority", "snapshot")

    def test_changed_provenance_and_empty_trace_fail(self):
        for field in ("governance_hash", "state_snapshot_hash", "execution_trace"):
            with self.subTest(field=field):
                outcome = copy.deepcopy(self.outcome)
                outcome["packet"][field] = ""
                with self.assertRaises(RuntimeError):
                    LOCAL.validate_outcome(outcome, "authority", "snapshot")

    def test_false_guarantees_fail(self):
        for field in ("authoritative_pdp", "trace_available", "replay_sufficient"):
            outcome = copy.deepcopy(self.outcome)
            outcome[field] = False
            with self.assertRaises(RuntimeError):
                LOCAL.validate_outcome(outcome, "authority", "snapshot")

    def test_error_responses_cannot_become_results(self):
        for response in ({"error": {}}, {"result": {"error": {}}}, {"result": {"isError": True}}):
            with self.assertRaises(RuntimeError):
                LOCAL.tool_payload(response)

    def test_mismatched_response_identity_fails(self):
        for request_id in (2, "1", True, None):
            with self.assertRaises(RuntimeError):
                LOCAL.decode_response(json.dumps({"jsonrpc": "2.0", "id": request_id}), 1)

    def test_invalid_json_and_protocol_fail(self):
        with self.assertRaises(json.JSONDecodeError):
            LOCAL.decode_response("not JSON", 1)
        with self.assertRaises(RuntimeError):
            LOCAL.decode_response('{"jsonrpc":"1.0","id":1}', 1)

    def test_ambient_identity_and_proxy_are_removed(self):
        with patch.dict(os.environ, {"SCG_AUTH_TOKEN": "private", "ITER_AUDIT_LEDGER_PATH": "real",
                                     "CARGO_TARGET_DIR": "unexpected", "HTTPS_PROXY": "remote"}):
            env = LOCAL.runtime_env()
        for key in ("SCG_AUTH_TOKEN", "ITER_AUDIT_LEDGER_PATH", "CARGO_TARGET_DIR", "HTTPS_PROXY"):
            self.assertNotIn(key, env)

    def test_output_cannot_overwrite_source_or_previous_run(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = root / "repo"
            repo.mkdir()
            with self.assertRaises(RuntimeError):
                LOCAL.verify_output_directory(repo / "evidence", (repo,))
            with self.assertRaises(RuntimeError):
                LOCAL.verify_output_directory(root, (repo,))
            self.assertEqual(LOCAL.verify_output_directory(root / "new", (repo,)), root / "new")

    def test_child_is_stopped_on_failure(self):
        with tempfile.TemporaryDirectory() as directory:
            with self.assertRaisesRegex(RuntimeError, "test failure"):
                with LOCAL.process([sys.executable, "-c", "import time; time.sleep(60)"],
                                   directory, LOCAL.runtime_env(), Path(directory) / "stderr.log") as child:
                    raise RuntimeError("test failure")
            self.assertIsNotNone(child.poll())

    def test_hash_binds_raw_bytes_not_normalized_text(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "input"
            path.write_bytes(b"line\r\n")
            crlf = LOCAL.sha256(path)
            path.write_bytes(b"line\n")
            self.assertNotEqual(crlf, LOCAL.sha256(path))


if __name__ == "__main__":
    unittest.main()
