import {
  SDK_PROTOCOL_VERSION,
  MIN_SERVER_VERSION,
  MAX_SERVER_VERSION,
  isVersionCompatible,
  createTraceContext,
  SdkError,
  VersionMismatchError,
  ConnectionError,
  RequestError,
} from "./index";

describe("Protocol Version", () => {
  test("SDK_PROTOCOL_VERSION is valid", () => {
    expect(SDK_PROTOCOL_VERSION).toBe("1.0.0");
  });

  test("MIN_SERVER_VERSION is valid", () => {
    expect(MIN_SERVER_VERSION).toBe("1.0.0");
  });

  test("MAX_SERVER_VERSION allows minor versions", () => {
    expect(MAX_SERVER_VERSION).toBe("1.99.99");
  });
});

describe("Version Compatibility", () => {
  test("accepts current version", () => {
    expect(isVersionCompatible("1.0.0")).toBe(true);
  });

  test("accepts minor version bumps", () => {
    expect(isVersionCompatible("1.1.0")).toBe(true);
    expect(isVersionCompatible("1.5.0")).toBe(true);
    expect(isVersionCompatible("1.99.0")).toBe(true);
  });

  test("accepts patch version bumps", () => {
    expect(isVersionCompatible("1.0.1")).toBe(true);
    expect(isVersionCompatible("1.5.10")).toBe(true);
  });

  test("rejects major version bumps", () => {
    expect(isVersionCompatible("2.0.0")).toBe(false);
    expect(isVersionCompatible("3.0.0")).toBe(false);
  });

  test("rejects older major versions", () => {
    expect(isVersionCompatible("0.9.0")).toBe(false);
  });

  test("rejects invalid version strings", () => {
    expect(isVersionCompatible("")).toBe(false);
    expect(isVersionCompatible("1.0")).toBe(false);
    expect(isVersionCompatible("v1.0.0")).toBe(false);
    expect(isVersionCompatible("not-a-version")).toBe(false);
  });
});

describe("Trace Context", () => {
  test("createTraceContext creates valid context", () => {
    const trace = createTraceContext("test-trace-id");
    expect(trace.traceId).toBe("test-trace-id");
    expect(trace.spanId).toBe("test-trace-id");
    expect(trace.parentSpanId).toBeUndefined();
  });
});

describe("Error Types", () => {
  test("SdkError has correct name", () => {
    const err = new SdkError("test");
    expect(err.name).toBe("SdkError");
    expect(err.message).toBe("test");
  });

  test("VersionMismatchError formats message correctly", () => {
    const err = new VersionMismatchError("1.0.0", "2.0.0");
    expect(err.name).toBe("VersionMismatchError");
    expect(err.message).toBe("Version mismatch: client=1.0.0, server=2.0.0");
    expect(err.clientVersion).toBe("1.0.0");
    expect(err.serverVersion).toBe("2.0.0");
  });

  test("ConnectionError formats message correctly", () => {
    const err = new ConnectionError("timeout");
    expect(err.name).toBe("ConnectionError");
    expect(err.message).toBe("Connection failed: timeout");
  });

  test("RequestError formats message correctly", () => {
    const err = new RequestError({ code: -32600, message: "Invalid Request" });
    expect(err.name).toBe("RequestError");
    expect(err.message).toBe("Request failed: Invalid Request (-32600)");
    expect(err.rpcError.code).toBe(-32600);
  });
});
