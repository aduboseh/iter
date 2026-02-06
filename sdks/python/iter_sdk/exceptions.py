"""Iter SDK Exception Types"""

class RpcError(Exception):
    """Base RPC error for iter SDK."""
    pass


class SdkError(Exception):
    """Base SDK error."""
    pass


class VersionMismatchError(SdkError):
    """Client and server protocol version mismatch."""

    def __init__(self, client_version: str, server_version: str):
        self.client_version = client_version
        self.server_version = server_version
        super().__init__(f"Version mismatch: client={client_version}, server={server_version}")
        self.name = "VersionMismatchError"


class ConnectionError(SdkError):
    """Connection failed."""

    def __init__(self, message: str):
        super().__init__(f"Connection failed: {message}")
        self.name = "ConnectionError"


class RequestError(SdkError):
    """Request failed with server error."""

    def __init__(self, rpc_error: "RpcError"):
        self.rpc_error = rpc_error
        super().__init__(f"Request failed: {rpc_error.message} ({rpc_error.code})")
        self.name = "RequestError"


class BackpressureError(SdkError):
    """Max inflight requests exceeded."""

    def __init__(self, max_inflight: int):
        self.max_inflight = max_inflight
        super().__init__(f"Backpressure: maxInflight={max_inflight} exceeded")
        self.name = "BackpressureError"


class RequestTimeoutError(SdkError):
    """Request timed out."""

    def __init__(self, method: str, timeout_ms: int):
        self.method = method
        self.timeout_ms = timeout_ms
        super().__init__(f"Request timeout: {method} exceeded {timeout_ms}ms")
        self.name = "RequestTimeoutError"


class ConnectionClosedError(SdkError):
    """Connection closed."""

    def __init__(self, message: str = "Connection closed", pending_count_at_close: int = None):
        self.pending_count_at_close = pending_count_at_close
        super().__init__(message)
        self.name = "ConnectionClosedError"
