"""Iter Python SDK - MCP Client

Async client for Iter MCP protocol with STDIO transport.
Follows the SDK lifecycle contract: sdks/sdk-contract.md
"""

import asyncio
import json
from dataclasses import asdict
from typing import Any, Dict, Optional

from .exceptions import (
    ConnectionError,
    RequestError,
    BackpressureError,
    RequestTimeoutError,
    ConnectionClosedError,
)
from .types import (
    TraceContext,
    RpcRequest,
    RpcResponse,
    RpcError,
    ToolInfo,
    NodeState,
    GovernorStatus,
    State,
)

STDERR_RING_MAX_BYTES = 10 * 1024
MAX_PROTOCOL_VIOLATIONS = 3


class IterClient:
    """Iter MCP client (STDIO transport, async)."""

    def __init__(self, max_inflight: int = 1):
        self.max_inflight: int = max_inflight
        self._state: State = State.OPEN
        self._close_lock: asyncio.Lock = asyncio.Lock()
        self._close_future: Optional[asyncio.Future] = None

        self.process: Optional[asyncio.subprocess.Process] = None
        self.stdin: Optional[asyncio.StreamWriter] = None
        self.stdout: Optional[asyncio.StreamReader] = None
        self.stderr: Optional[asyncio.StreamReader] = None

        self._request_id: int = 0
        self._response_queue: Dict[int, asyncio.Future] = {}
        self._trace_context: Optional[TraceContext] = None

        self._stderr_bytes: bytearray = bytearray()
        self._protocol_violation_count: int = 0
        self._circuit_breaker_close_started: bool = False

        self._reader_task: Optional[asyncio.Task] = None
        self._stderr_task: Optional[asyncio.Task] = None

    @property
    def trace_context(self) -> Optional[TraceContext]:
        """Get current trace context."""
        return self._trace_context

    def with_trace(self, trace: TraceContext) -> "IterClient":
        """Set trace context for subsequent requests."""
        self._trace_context = trace
        return self

    @staticmethod
    async def connect(binary_path: str, max_inflight: int = 1) -> "IterClient":
        """Connect to an Iter server process."""
        client = IterClient(max_inflight=max_inflight)

        client.process = await asyncio.create_subprocess_exec(
            binary_path,
            stdin=asyncio.subprocess.PIPE,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.PIPE,
        )

        if not client.process.stdin or not client.process.stdout or not client.process.stderr:
            raise ConnectionError("Failed to open stdio")

        client.stdin = client.process.stdin
        client.stdout = client.process.stdout
        client.stderr = client.process.stderr

        client._reader_task = asyncio.create_task(client._read_stdout())
        client._stderr_task = asyncio.create_task(client._read_stderr())

        return client

    async def send(self, method: str, params: Optional[Any] = None, timeout_ms: int = 30000) -> RpcResponse:
        """Send a JSON-RPC request.

        Contract:
        - Precondition: state must be OPEN
        - Precondition: response_queue.size < max_inflight
        - Returns Promise that resolves on matching response or rejects on timeout
        """
        if self._state != State.OPEN:
            raise ConnectionClosedError("Client is closing or closed, cannot send request")

        if len(self._response_queue) >= self.max_inflight:
            raise BackpressureError(self.max_inflight)

        if not self.stdin:
            raise ConnectionError("Not connected")

        self._request_id += 1
        request_id = self._request_id

        request = RpcRequest(
            jsonrpc="2.0",
            method=method,
            params=params,
            id=request_id,
        )

        future: asyncio.Future = asyncio.Future()
        self._response_queue[request_id] = future

        self.stdin.write((json.dumps(asdict(request)) + "\n").encode("utf-8"))
        await self.stdin.drain()

        try:
            response = await asyncio.wait_for(future, timeout=timeout_ms / 1000)
            return response
        except asyncio.TimeoutError:
            self._response_queue.pop(request_id, None)
            raise RequestTimeoutError(method, timeout_ms)
        finally:
            self._response_queue.pop(request_id, None)

    async def tools_list(self) -> list[ToolInfo]:
        """List available tools."""
        response = await self.send("tools/list")
        if response.error:
            raise RequestError(response.error)

        result = response.result or {}
        tools = result.get("tools", [])
        return [ToolInfo(**t) for t in tools]

    async def node_create(self, belief: float, energy: float) -> NodeState:
        """Create a node."""
        response = await self.send("tools/call", {
            "name": "node.create",
            "arguments": {"belief": belief, "energy": energy},
        })
        return self._parse_tool_result(response)

    async def node_query(self, node_id: int) -> NodeState:
        """Query a node."""
        response = await self.send("tools/call", {
            "name": "node.query",
            "arguments": {"node_id": node_id},
        })
        return self._parse_tool_result(response)

    async def governor_status(self) -> GovernorStatus:
        """Get governor status."""
        response = await self.send("tools/call", {
            "name": "governor.status",
            "arguments": {},
        })
        return self._parse_tool_result(response)

    async def close(self) -> None:
        """Close the connection.

        Contract:
        - Idempotent: multiple calls return same future
        - Set state to CLOSING
        - Await drain (bounded timeout)
        - Kill subprocess (SIGTERM → SIGKILL)
        - Set state to CLOSED
        """
        async with self._close_lock:
            if self._close_future:
                await self._close_future
                return

            self._close_future = asyncio.Future()
            await self._perform_close()
            self._close_future.set_result(None)

    async def _perform_close(self) -> None:
        """Internal close implementation."""
        self._state = State.CLOSING

        await self._wait_for_drain(5000)

        pending_count = len(self._response_queue)
        if pending_count > 0:
            error = ConnectionClosedError(
                "Client closed during drain, request did not complete in time",
                pending_count_at_close=pending_count,
            )
            for pending in self._response_queue.values():
                pending.set_exception(error)
            self._response_queue.clear()

        if self._reader_task:
            self._reader_task.cancel()
            try:
                await self._reader_task
            except asyncio.CancelledError:
                pass
            self._reader_task = None

        if self._stderr_task:
            self._stderr_task.cancel()
            try:
                await self._stderr_task
            except asyncio.CancelledError:
                pass
            self._stderr_task = None

        if self.process and self.process.returncode is None:
            self.process.terminate()

            try:
                await asyncio.wait_for(self.process.wait(), timeout=2.0)
            except asyncio.TimeoutError:
                if self.process and self.process.returncode is None:
                    self.process.kill()
                    try:
                        await asyncio.wait_for(self.process.wait(), timeout=1.0)
                    except asyncio.TimeoutError:
                        raise ConnectionError("Failed to reap process after SIGKILL (zombie detected)")

        self.process = None
        self.stdin = None
        self.stdout = None
        self.stderr = None
        self._state = State.CLOSED

    async def _wait_for_drain(self, timeout_ms: int) -> None:
        """Wait for response queue to drain."""
        interval_ms = 25
        max_iterations = timeout_ms // interval_ms

        for _ in range(max_iterations):
            if len(self._response_queue) == 0:
                return
            await asyncio.sleep(interval_ms / 1000)

    async def _read_stdout(self) -> None:
        """Read stdout line by line and process responses."""
        while self._state in (State.OPEN, State.CLOSING):
            try:
                line_bytes = await self.stdout.readline() if self.stdout else None
                if not line_bytes:
                    break

                line = line_bytes.decode("utf-8").strip()
                if line:
                    self._handle_stdout_line(line)
            except Exception:
                if self._state == State.CLOSED:
                    break

    async def _read_stderr(self) -> None:
        """Read stderr into ring buffer."""
        while self._state in (State.OPEN, State.CLOSING):
            try:
                chunk = await self.stderr.read(4096) if self.stderr else None
                if not chunk:
                    break

                self._append_stderr_bytes(chunk)
            except Exception:
                break

    def _handle_stdout_line(self, line: str) -> None:
        """Handle a stdout line (JSON-RPC response).

        Contract:
        - MUST process responses when state is OPEN or CLOSING
        - MUST ignore when state is CLOSED
        """
        if self._state not in (State.OPEN, State.CLOSING):
            return

        try:
            parsed = json.loads(line)
        except json.JSONDecodeError:
            self._protocol_violation_count += 1

            if len(self._response_queue) > 0:
                self._fail_closed_protocol_violation("Protocol violation: malformed stdout")
                return

            if self._protocol_violation_count >= MAX_PROTOCOL_VIOLATIONS:
                self._fail_closed_protocol_violation("Protocol violation: malformed stdout")
            return

        request_id = parsed.get("id")

        if not isinstance(request_id, int):
            self._protocol_violation_count += 1
            if self._protocol_violation_count >= MAX_PROTOCOL_VIOLATIONS:
                self._fail_closed_protocol_violation("Protocol violation: invalid JSON-RPC")
            return

        pending = self._response_queue.get(request_id)
        if not pending:
            return

        if parsed.get("jsonrpc") != "2.0":
            self._response_queue.pop(request_id, None)
            pending.set_exception(RequestError(RpcError(code=-32700, message="Malformed JSON-RPC response")))
            return

        response = RpcResponse(**parsed)
        self._response_queue.pop(request_id, None)
        pending.set_result(response)

    def _append_stderr_bytes(self, chunk: bytes) -> None:
        """Append stderr bytes to ring buffer (10KB max)."""
        if len(chunk) == 0:
            return

        combined = self._stderr_bytes + chunk
        if len(combined) > STDERR_RING_MAX_BYTES:
            self._stderr_bytes = bytearray(combined[-STDERR_RING_MAX_BYTES:])
        else:
            self._stderr_bytes = bytearray(combined)

    def _fail_closed_protocol_violation(self, reason: str) -> None:
        """Fail-closed on protocol violation."""
        if self._state == State.CLOSED:
            return

        error = ConnectionError(reason)
        for pending in self._response_queue.values():
            pending.set_exception(error)
        self._response_queue.clear()

        if not self._circuit_breaker_close_started:
            self._circuit_breaker_close_started = True
            asyncio.create_task(self.close())

    def _parse_tool_result(self, response: RpcResponse) -> Any:
        """Parse tool result from response."""
        if response.error:
            raise RequestError(response.error)

        result = response.result or {}
        content = result.get("content", [])

        if not content or not isinstance(content, list) or len(content) == 0:
            raise RequestError(RpcError(code=-1, message="Invalid tool response format"))

        text = content[0].get("text")
        if not text:
            raise RequestError(RpcError(code=-1, message="Invalid tool response format"))

        return json.loads(text)
