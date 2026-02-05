"""CT4.1-G drain test for Iter Python SDK."""

import asyncio
import pytest

from iter_sdk import IterClient
from iter_sdk.types import State


@pytest.mark.asyncio
async def test_ct41g_server_responds_during_drain_window():
    client = IterClient(max_inflight=1)
    client._state = State.OPEN

    future = asyncio.Future()
    client._response_queue[1] = future

    async def respond_during_drain():
        await asyncio.sleep(0.05)
        if not future.done():
            future.set_result({"jsonrpc": "2.0", "id": 1, "result": {}})
            client._response_queue.pop(1, None)

    close_task = asyncio.create_task(client._perform_close())
    asyncio.create_task(respond_during_drain())

    await asyncio.wait_for(future, timeout=1.0)
    await asyncio.wait_for(close_task, timeout=1.0)

    assert client._state == State.CLOSED
