"""Contract invariants tests for Iter Python SDK."""

import asyncio
import pytest
from unittest.mock import AsyncMock, Mock

from iter_sdk import IterClient
from iter_sdk.exceptions import ConnectionClosedError, BackpressureError, RequestTimeoutError
from iter_sdk.types import State


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
