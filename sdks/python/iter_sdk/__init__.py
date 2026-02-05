"""Iter Python SDK

Thin client for Iter MCP protocol.
"""

from .client import IterClient
from .exceptions import (
    SdkError,
    VersionMismatchError,
    ConnectionError,
    RequestError,
    BackpressureError,
    RequestTimeoutError,
    ConnectionClosedError,
)
from .types import (
    TraceContext,
    create_trace_context,
    RpcRequest,
    RpcResponse,
    RpcError,
    ToolInfo,
    NodeState,
    GovernorStatus,
    State,
)

__version__ = "1.0.0"
SDK_PROTOCOL_VERSION = "1.0.0"
MIN_SERVER_VERSION = "1.0.0"
MAX_SERVER_VERSION = "1.99.99"

__all__ = [
    "IterClient",
    "SdkError",
    "VersionMismatchError",
    "ConnectionError",
    "RequestError",
    "BackpressureError",
    "RequestTimeoutError",
    "ConnectionClosedError",
    "TraceContext",
    "create_trace_context",
    "RpcRequest",
    "RpcResponse",
    "RpcError",
    "ToolInfo",
    "NodeState",
    "GovernorStatus",
    "State",
]
