"""Contract invariants tests for Iter Python SDK."""

import asyncio
import json
import pytest
from unittest.mock import AsyncMock, Mock

from iter_sdk import IterClient
from iter_sdk.exceptions import ConnectionClosedError, BackpressureError, RequestTimeoutError, RequestError
from iter_sdk.types import State, RpcResponse


@pytest.mark.asyncio
@pytest.mark.parametrize("args,expected", [
    ({}, {"start_sequence": 0, "limit": 100}),
    ({"start_sequence": 7, "limit": 2}, {"start_sequence": 7, "limit": 2}),
])
async def test_audit_history_returns_integrity_page_without_replay(args, expected):
    client = IterClient(max_inflight=1)
    page = {"verification": "integrity_only", "records": [], "next_sequence": None,
            "verified_next_sequence": 7, "verified_record_hash": "a" * 64}
    client.send = AsyncMock(return_value=RpcResponse(result={
        "content": [{"type": "text", "text": json.dumps(page)}],
    }))

    assert await client.audit_history(**args) == page
    client.send.assert_awaited_once_with("tools/call", {
        "name": "audit.history", "arguments": expected,
    })


@pytest.mark.asyncio
@pytest.mark.parametrize("method,code", [("audit_history", 5002), ("audit_replay", 5003)])
async def test_audit_tool_errors_preserve_code_without_fallback(method, code):
    client = IterClient(max_inflight=1)
    client.send = AsyncMock(return_value=RpcResponse(result={
        "error": {"code": code, "message": "audit unavailable"},
        "content": [{"text": "{\"verification\":\"integrity_only\"}"}],
    }))
    with pytest.raises(RequestError) as error:
        await getattr(client, method)()
    assert error.value.rpc_error.code == code
    assert error.value.rpc_error.message == "audit unavailable"
    assert client.send.await_count == 1


@pytest.mark.asyncio
async def test_send_rejects_when_not_open():
    client = IterClient(max_inflight=1)
    client._state = State.CLOSING

    with pytest.raises(ConnectionClosedError):
        await client.send("test", {})


@pytest.mark.asyncio
async def test_backpressure_blocks_when_at_max_inflight():
    client = IterClient(max_inflight=1)
    client._state = State.OPEN
    class FakeStdin:
        def write(self, *_args, **_kwargs):
            return None

        async def drain(self):
            return None

    client.stdin = FakeStdin()
    client._response_queue[1] = asyncio.Future()

    with pytest.raises(BackpressureError):
        await client.send("test", {})


@pytest.mark.asyncio
async def test_timeout_rejects_and_evicts():
    client = IterClient(max_inflight=1)
    class FakeStdin:
        def write(self, *_args, **_kwargs):
            return None

        async def drain(self):
            return None

    client._state = State.OPEN
    client.stdin = FakeStdin()

    with pytest.raises(RequestTimeoutError):
        await client.send("test", {}, timeout_ms=1)

    assert client._response_queue == {}


@pytest.mark.asyncio
async def test_decision_preview_uses_canonical_tool_name_and_args():
    client = IterClient(max_inflight=1)
    client.send = AsyncMock(return_value=object())
    client._parse_tool_result = Mock(return_value={"verdict": "ALLOW", "simulation": True})

    result = await client.decision_preview(
        proposal_id="proposal-1",
        state_snapshot_hash="sha256:state",
        requested_action="deploy_capsule",
        constraints={"tenant": "alpha"},
    )

    client.send.assert_awaited_once_with("tools/call", {
        "name": "decision.preview",
        "arguments": {
            "proposal_id": "proposal-1",
            "state_snapshot_hash": "sha256:state",
            "requested_action": "deploy_capsule",
            "constraints": {"tenant": "alpha"},
        },
    })
    client._parse_tool_result.assert_called_once()
    assert result["verdict"] == "ALLOW"


@pytest.mark.asyncio
async def test_audit_search_uses_canonical_tool_name_and_filter_map():
    client = IterClient(max_inflight=1)
    client.send = AsyncMock(return_value=object())
    client._parse_tool_result = Mock(return_value={"count": 0, "results": []})

    result = await client.audit_search(principal="alice", limit=10)

    client.send.assert_awaited_once_with("tools/call", {
        "name": "audit.search",
        "arguments": {
            "principal": "alice",
            "limit": 10,
        },
    })
    client._parse_tool_result.assert_called_once()
    assert result["count"] == 0

@pytest.mark.asyncio
async def test_register_resource_uses_canonical_tool_name_and_args():
    client = IterClient(max_inflight=1)
    client.send = AsyncMock(return_value=object())
    client._parse_tool_result = Mock(return_value={"registered": True})

    result = await client.register_resource(
        resource_path="docs/README.md",
        expected_hash="sha256:abc",
    )

    client.send.assert_awaited_once_with("tools/call", {
        "name": "register_resource",
        "arguments": {
            "resource_path": "docs/README.md",
            "expected_hash": "sha256:abc",
        },
    })
    client._parse_tool_result.assert_called_once()
    assert result["registered"] is True
