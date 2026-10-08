"""Real transport EOF regressions; runnable with standard-library unittest."""

import asyncio
import sys
import unittest

from iter_sdk import IterClient
from iter_sdk.exceptions import BackpressureError, ConnectionClosedError, ConnectionError, RequestTimeoutError
from iter_sdk.types import State


CHILD = r"""
import json, os, sys, time
mode = sys.argv[1]
if mode == "blocked_write":
    sys.stdin.buffer.read(1)
else:
    request = json.loads(sys.stdin.buffer.readline())
if mode == "two_requests":
    json.loads(sys.stdin.buffer.readline())
if mode == "silent":
    while True:
        time.sleep(1)
if mode == "response":
    print(json.dumps({"jsonrpc": "2.0", "id": request["id"], "result": {"ok": True}}), flush=True)
sys.stderr.write("transport fixture diagnostic\n")
sys.stderr.flush()
if mode == "read_error":
    sys.stdout.buffer.write(b"\xff\n")
    sys.stdout.buffer.flush()
os.close(1)
if mode != "exit":
    while True:
        time.sleep(1)
"""


class TransportEofTests(unittest.IsolatedAsyncioTestCase):
    async def make_client(self, mode):
        client = IterClient(max_inflight=4)
        process = await asyncio.create_subprocess_exec(
            sys.executable, "-u", "-c", CHILD, mode,
            stdin=asyncio.subprocess.PIPE,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.PIPE,
        )
        client.process = process
        client.stdin, client.stdout, client.stderr = process.stdin, process.stdout, process.stderr
        client._reader_task = asyncio.create_task(client._read_stdout())
        client._stderr_task = asyncio.create_task(client._read_stderr())

        async def cleanup():
            # Cleanup must run even when the regression fails on the old client.
            try:
                await asyncio.wait_for(client.close(), 10)
            finally:
                if process.returncode is None:
                    process.kill()
                await process.wait()

        self.addAsyncCleanup(cleanup)
        return client, process

    async def terminal_state(self, client):
        async def wait():
            while client._state == State.OPEN:
                await asyncio.sleep(0)
        await asyncio.wait_for(wait(), 5)

    async def assert_transport_failure(self, mode, params=None):
        client, process = await self.make_client(mode)
        with self.assertRaisesRegex(ConnectionError, "stdout|UTF-8|utf-8"):
            await asyncio.wait_for(client.send("test", params), 5)
        self.assertEqual(client._response_queue, {})
        self.assertEqual(client._state, State.CLOSING)
        with self.assertRaises(ConnectionClosedError):
            await client.send("later")
        if mode != "exit":
            self.assertIsNone(process.returncode, "stdout EOF is not process exit")
        await asyncio.wait_for(client.close(), 5)
        await asyncio.wait_for(client.close(), 1)
        self.assertIsNotNone(process.returncode)
        self.assertEqual(client._state, State.CLOSED)

    async def test_exit_without_response_fails_promptly(self):
        await self.assert_transport_failure("exit")

    async def test_stdout_closed_child_alive_is_still_reaped(self):
        await self.assert_transport_failure("alive")

    async def test_eof_interrupts_blocked_stdin_drain(self):
        await self.assert_transport_failure("blocked_write", {"data": "x" * (2 * 1024 * 1024)})

    async def test_read_failure_is_terminal(self):
        await self.assert_transport_failure("read_error")

    async def test_complete_response_precedes_terminal_eof(self):
        client, process = await self.make_client("response")
        response = await asyncio.wait_for(client.send("test"), 5)
        self.assertEqual(response.result, {"ok": True})
        await self.terminal_state(client)
        with self.assertRaises(ConnectionClosedError):
            await client.send("later")
        self.assertIsNone(process.returncode)

    async def test_eof_rejects_all_pending_and_keeps_stderr(self):
        client, _ = await self.make_client("two_requests")
        # The child waits for both requests before closing stdout.
        outcomes = await asyncio.wait_for(asyncio.gather(
            client.send("first"), client.send("second"), return_exceptions=True,
        ), 5)
        self.assertTrue(all(isinstance(error, ConnectionError) for error in outcomes))
        self.assertEqual(client._response_queue, {})

        async def diagnostic():
            while b"transport fixture diagnostic" not in client._stderr_bytes:
                await asyncio.sleep(0)
        await asyncio.wait_for(diagnostic(), 5)
        self.assertLessEqual(len(client._stderr_bytes), 10 * 1024)

    async def test_eof_during_close_does_not_wait_for_drain_timeout(self):
        client, _ = await self.make_client("alive")
        pending = asyncio.create_task(client.send("test"))
        while not client._response_queue:
            await asyncio.sleep(0)
        await asyncio.wait_for(client.close(), 2)
        with self.assertRaises(ConnectionError):
            await pending
        await client.close()

    async def test_timeout_releases_capacity_without_closing_transport(self):
        client, _ = await self.make_client("silent")
        client.max_inflight = 1
        pending = asyncio.create_task(client.send("first", timeout_ms=200))
        while not client._response_queue:
            await asyncio.sleep(0)
        with self.assertRaises(BackpressureError):
            await client.send("overflow")
        with self.assertRaises(RequestTimeoutError):
            await asyncio.wait_for(pending, 5)
        self.assertEqual(client._response_queue, {})
        self.assertEqual(client._state, State.OPEN)
        with self.assertRaises(RequestTimeoutError):
            await asyncio.wait_for(client.send("next", timeout_ms=20), 5)

    async def test_cancelled_pending_does_not_break_terminal_drain(self):
        client, _ = await self.make_client("alive")
        cancelled = asyncio.get_running_loop().create_future()
        cancelled.cancel()
        client._response_queue[99] = cancelled
        with self.assertRaises(ConnectionError):
            await asyncio.wait_for(client.send("test"), 5)
        self.assertEqual(client._response_queue, {})


if __name__ == "__main__":
    unittest.main()
