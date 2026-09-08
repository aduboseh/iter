#!/usr/bin/env python3
"""Exercise real Iter and SCG processes locally; never issue a release certificate."""

import argparse
import contextlib
import hashlib
import json
import os
from pathlib import Path
import platform
import queue
import secrets
import socket
import subprocess
import threading
import time
import urllib.error
import urllib.request


def require(condition, message):
    """Keep validation active even when Python runs with optimization enabled."""
    if not condition:
        raise RuntimeError(message)


def sha256(path):
    """Hash raw file bytes without normalization."""
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def runtime_env():
    """Do not inherit developer runtime identities, ledgers, or HTTP proxies."""
    return {k: v for k, v in os.environ.items()
            if not k.upper().startswith(("ITER_", "SCG_", "CARGO_", "RUST"))
            and k.upper() not in {"HTTP_PROXY", "HTTPS_PROXY", "ALL_PROXY", "NO_PROXY"}}


def decode_response(line, request_id):
    """Require a JSON-RPC response to the exact request, not just any JSON."""
    response = json.loads(line)
    require(response.get("jsonrpc") == "2.0", "invalid JSON-RPC version")
    require(type(response.get("id")) is int and response["id"] == request_id, "response ID mismatch")
    return response


def tool_payload(response):
    """Reject transport/tool errors rather than interpreting them as results."""
    require("error" not in response, f"RPC failed: {response}")
    result = response["result"]
    require("error" not in result and not result.get("isError"), f"tool failed: {result}")
    return json.loads(result["content"][0]["text"])


def validate_outcome(outcome, governance_hash, snapshot):
    """Require an authoritative, replayable packet bound to the live SCG state."""
    require(outcome.get("mode") == "governed", "demo fallback is forbidden")
    require(outcome.get("verdict") == "ALLOW", "expected matching empty-cluster ALLOW")
    for key in ("authoritative_pdp", "trace_available", "replay_sufficient"):
        require(outcome.get(key) is True, f"missing guarantee: {key}")
    packet = outcome.get("packet") or {}
    require(packet.get("governance_hash") == governance_hash, "governance identity mismatch")
    require(packet.get("state_snapshot_hash") == snapshot, "SCG snapshot mismatch")
    require(bool(packet.get("execution_trace")), "SCG trace missing")
    return packet


def verify_output_directory(output, roots):
    """Keep logs/state outside both repositories and refuse to overwrite a run."""
    output = output.resolve()
    for root in roots:
        require(not output.is_relative_to(root.resolve()), "output must be outside source repos")
    require(not output.exists(), "output directory already exists; choose a fresh run directory")
    return output


@contextlib.contextmanager
def process(argv, cwd, env, log):
    """Own and bound the lifetime of only the child started by this harness."""
    with log.open("w", encoding="utf-8") as errors:
        child = subprocess.Popen(
            argv, cwd=cwd, env=env, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
            stderr=errors, text=True, encoding="utf-8", bufsize=1,
            creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0),
        )
        try:
            yield child
        finally:
            if child.poll() is None:
                child.terminate()
            try:
                child.wait(timeout=5)
            except subprocess.TimeoutExpired:
                child.kill()
                child.wait(timeout=5)
            for reader in getattr(child, "verification_readers", []):
                reader.join(timeout=5)
                require(not reader.is_alive(), "child output reader did not stop")
            child.stdin.close()
            child.stdout.close()


class McpClient:
    """Read MCP with a deadline on Windows and Linux, without blocking on EOF."""
    def __init__(self, child, transcript):
        self.child, self.transcript = child, transcript
        self.lines = queue.Queue()
        self.next_id = 0

        def read():
            for line in child.stdout:
                self.lines.put(line)
            self.lines.put(None)

        reader = threading.Thread(target=read, daemon=True)
        child.verification_readers = [reader]
        reader.start()

    def call(self, method, params):
        """Write one request and retain the exact request/response transcript."""
        self.next_id += 1
        request = {"jsonrpc": "2.0", "id": self.next_id, "method": method, "params": params}
        line = json.dumps(request, ensure_ascii=True)
        self.child.stdin.write(line + "\n")
        self.child.stdin.flush()
        try:
            response = self.lines.get(timeout=40)
        except queue.Empty as exc:
            raise RuntimeError("MCP response deadline exceeded") from exc
        require(response is not None, "Iter exited before returning a response")
        with self.transcript.open("a", encoding="utf-8") as stream:
            stream.write(line + "\n" + response)
        return decode_response(response, self.next_id)

    def tool(self, name, arguments):
        """Use the real tools/call transport rather than calling Rust helpers."""
        return self.call("tools/call", {"name": name, "arguments": arguments})


def execute(args, output, report):
    """Build locked binaries, exercise the seam and reject negative test cases."""
    roots = {"iter": args.iter_root.resolve(), "scg": args.scg_root.resolve()}
    env = runtime_env()
    report["commands"] = []
    report["checks"] = []

    def run(argv, cwd, name, expected=0):
        result = subprocess.run(argv, cwd=cwd, env=env, capture_output=True, timeout=1800)
        (output / f"{name}.stdout.log").write_bytes(result.stdout)
        (output / f"{name}.stderr.log").write_bytes(result.stderr)
        report["commands"].append({"argv": [str(x) for x in argv], "cwd": str(cwd),
                                   "exit_code": result.returncode, "expected_exit": expected})
        require(result.returncode == expected, f"{name}: exit {result.returncode}, expected {expected}")
        return result.stdout.decode("utf-8")

    def passed(name):
        report["checks"].append(name)
        print(f"PASS {name}", flush=True)

    report["subjects"] = {}
    for name, root in roots.items():
        head = run(["git", "rev-parse", "HEAD"], root, f"{name}-head").strip()
        status = run(["git", "status", "--porcelain"], root, f"{name}-status")
        report["subjects"][name] = {"commit": head, "clean": not status.strip(),
                                     "lockfile_sha256": sha256(root / "Cargo.lock")}
    report["rustc"] = run(["rustc", f"+{args.toolchain}", "-vV"], roots["iter"], "rustc")
    run(["cargo", f"+{args.toolchain}", "build", "--locked", "--bin", "iter-server",
         "--bin", "iter-cli"], roots["iter"], "iter-build")
    run(["cargo", f"+{args.toolchain}", "build", "--locked", "-p", "scg-gateway",
         "--no-default-features", "--features", "ci"], roots["scg"], "scg-build")
    suffix = ".exe" if os.name == "nt" else ""
    binaries = {name: roots[repo] / "target/debug" / (name + suffix)
                for name, repo in (("iter-server", "iter"), ("iter-cli", "iter"), ("scg-gateway", "scg"))}
    report["binary_sha256"] = {name: sha256(path) for name, path in binaries.items()}
    governance_file = roots["scg"] / "governance/governance.hash"
    governance_hash = governance_file.read_text(encoding="utf-8").strip()
    require(governance_hash == (roots["iter"] / "governance/governance.hash").read_text().strip(),
            "Iter/SCG governance hash mismatch")
    with socket.socket() as sock:
        sock.bind(("127.0.0.1", 0))
        port = sock.getsockname()[1]
    endpoint = f"http://127.0.0.1:{port}"
    token = secrets.token_hex(32)
    gateway_env = dict(env, SCG_GATEWAY_HOST="127.0.0.1", SCG_GATEWAY_PORT=str(port),
                       SCG_GATEWAY_AUTH_TOKEN=token, SCG_GOVERNANCE_HASH_PATH=str(governance_file),
                       SCG_STATE_PATH=str(output / "cluster.json"))
    opener = urllib.request.build_opener(urllib.request.ProxyHandler({}))

    def http(path, authenticated=True):
        headers = {"Authorization": f"Bearer {token}"} if authenticated else {}
        with opener.open(urllib.request.Request(endpoint + path, headers=headers), timeout=2) as response:
            return response.read()

    with process([binaries["scg-gateway"]], output, gateway_env, output / "gateway.stderr.log") as gateway:
        # Drain gateway stdout so structured logging can never fill a pipe.
        def drain():
            with (output / "gateway.stdout.log").open("w", encoding="utf-8") as stream:
                for line in gateway.stdout:
                    stream.write(line)
        reader = threading.Thread(target=drain, daemon=True)
        gateway.verification_readers = [reader]
        reader.start()
        deadline = time.monotonic() + 20
        while True:
            require(gateway.poll() is None, "gateway exited before readiness")
            try:
                http("/ready")
                break
            except (OSError, urllib.error.URLError):
                require(time.monotonic() < deadline, "gateway readiness deadline exceeded")
                time.sleep(0.1)
        passed("real_gateway_ready")
        try:
            http("/governance/snapshot-hash", authenticated=False)
        except urllib.error.HTTPError as error:
            require(error.code == 401, f"wrong unauthorized status: {error.code}")
        else:
            raise RuntimeError("gateway accepted a request without the configured bearer token")
        passed("unauthenticated_request_rejected")
        snapshot = json.loads(http("/governance/snapshot-hash"))["state_snapshot_hash"]
        proposal = {"proposal_id": "local-stack-001", "requested_action": "approve",
                    "state_snapshot_hash": snapshot, "resource_path": "local-stack/state", "constraints": {}}
        (output / "proposal.json").write_text(json.dumps(proposal, indent=2) + "\n")
        packets = []
        for attempt in range(2):
            ledger = output / f"iter-{attempt}.jsonl"
            ledger.touch()
            iter_env = dict(env, SCG_ENDPOINT=endpoint, SCG_AUTH_TOKEN=token,
                            SCG_GOVERNANCE_HASH_PATH=str(governance_file),
                            ITER_AUDIT_LEDGER_PATH=str(ledger), ITER_REQUIRE_AUDIT_LEDGER="1")
            with process([binaries["iter-server"], "--json-only", "--runtime-mode=scg-backed"],
                         output, iter_env, output / f"iter-{attempt}.stderr.log") as child:
                client = McpClient(child, output / f"mcp-{attempt}.jsonl")
                require("error" not in client.call("initialize", {}), "initialization failed")
                registration = tool_payload(client.tool("register_resource", {
                    "resource_path": proposal["resource_path"], "expected_hash": snapshot}))
                require(registration.get("registered") is True, "resource registration failed")
                outcome = tool_payload(client.tool("decision.check", proposal))
                packet = validate_outcome(outcome, governance_hash, snapshot)
                packets.append(packet)
                require(ledger.stat().st_size > 0, "decision returned without a durable ledger record")
                packet_file = output / f"packet-{attempt}.json"
                packet_file.write_text(json.dumps(packet, indent=2) + "\n", encoding="utf-8")
                replay_args = [binaries["iter-cli"], "replay", "--decision-file", packet_file,
                               "--policy-version", outcome["policy_version"],
                               "--schema-version", outcome["schema_version"]]
                replay = json.loads(run(replay_args, output, f"replay-{attempt}"))
                require(replay.get("outcome") == "VERIFIED", "CLI did not verify the real packet")
                if attempt == 1:
                    mutated = dict(packet, checksum="0" * 64)
                    tamper_file = output / "packet-tampered.json"
                    tamper_file.write_text(json.dumps(mutated) + "\n", encoding="utf-8")
                    replay_args[3] = tamper_file
                    rejected = json.loads(run(replay_args, output, "replay-tampered", expected=2))
                    require(rejected.get("outcome") == "MISMATCH", "tamper was not rejected")
                    passed("offline_replay_and_tamper_rejection")
                    gateway.terminate()
                    gateway.wait(timeout=5)
                    failure = client.tool("decision.check", proposal)
                    error = failure.get("error") or failure.get("result", {}).get("error")
                    require(isinstance(error, dict) and error.get("code") == 1001,
                            "gateway outage did not fail closed at the Iter runtime boundary")
                    passed("gateway_outage_rejected_without_stub_fallback")
        require(packets[0] == packets[1], "identical proposal/state produced different packets across fresh processes")
        passed("fresh_process_packet_equality")
    passed("real_process_governed_seam")
    for name, root in roots.items():
        require(run(["git", "rev-parse", "HEAD"], root, f"{name}-final-head").strip() ==
                report["subjects"][name]["commit"], "source commit changed during verification")
        require(sha256(root / "Cargo.lock") == report["subjects"][name]["lockfile_sha256"],
                "lockfile changed during verification")
        final_status = run(["git", "status", "--porcelain"], root, f"{name}-final-status")
        require(final_status == (output / f"{name}-status.stdout.log").read_text(),
                "worktree status changed during verification")


def main():
    """Always publish the local report, including failures; never publish PASS evidence."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--iter-root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--scg-root", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--toolchain", default="1.93.0")
    args = parser.parse_args()
    output = verify_output_directory(args.output, (args.iter_root, args.scg_root))
    output.mkdir(parents=True)
    report = {"schema": "iter-scg-local-smoke/v1", "classification": "LOCAL_DEVELOPMENT_ONLY",
              "release_certification": False, "platform": platform.platform(),
              "python": platform.python_version(), "harness_sha256": sha256(Path(__file__)),
              "result": "FAIL"}
    try:
        execute(args, output, report)
        report["result"] = "PASS"
    except Exception as error:
        report["error"] = str(error)
        print(f"FAIL {error}", flush=True)
    finally:
        report["artifacts"] = {p.name: sha256(p) for p in output.iterdir() if p.is_file()}
        (output / "report.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(f"Local-only report: {output / 'report.json'}")
    return 0 if report["result"] == "PASS" else 1


if __name__ == "__main__":
    raise SystemExit(main())
