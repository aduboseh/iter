"""Iter Python SDK Type Definitions"""

from dataclasses import dataclass
from enum import Enum
from typing import Any, Dict, Optional


class State(Enum):
    """Client state: OPEN → CLOSING → CLOSED"""
    OPEN = "open"
    CLOSING = "closing"
    CLOSED = "closed"


@dataclass
class TraceContext:
    """Trace context for request correlation"""
    trace_id: str
    span_id: str
    parent_span_id: Optional[str] = None


def create_trace_context(trace_id: str) -> TraceContext:
    """Create a new trace context."""
    return TraceContext(trace_id=trace_id, span_id=trace_id)


@dataclass
class RpcError:
    """JSON-RPC 2.0 Error"""
    code: int
    message: str
    data: Optional[Any] = None


@dataclass
class RpcRequest:
    """JSON-RPC 2.0 Request"""
    jsonrpc: str = "2.0"
    method: str = ""
    params: Optional[Any] = None
    id: int = 0


@dataclass
class RpcResponse:
    """JSON-RPC 2.0 Response"""
    jsonrpc: str = "2.0"
    result: Optional[Any] = None
    error: Optional[RpcError] = None
    id: int = 0


@dataclass
class ToolInfo:
    """Tool information"""
    name: str
    description: str
    input_schema: Dict[str, Any]


@dataclass
class NodeState:
    """Node state"""
    id: int
    belief: float
    energy: float
    esv_valid: bool
    stability: float


@dataclass
class GovernorStatus:
    """Governor status"""
    drift_ok: bool
    energy_drift: float
    coherence: float
    node_count: int
    edge_count: int
    healthy: bool
