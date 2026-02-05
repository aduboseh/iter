"""Backpressure and timeout tests for Iter Python SDK."""

import asyncio
import pytest

from iter_sdk import IterClient
from iter_sdk.exceptions import BackpressureError, RequestTimeoutError
from iter_sdk.types import State


@pytest.mark.asyncio
async def test_backpressure_blocks_when_queue_full():
    client = IterClient(max_inflight=1)
    class FakeStdin:
        def write(self, *_args, **_kwargs):
            return None

        async def drain(self):
            return None

    client._state = State.OPEN
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
