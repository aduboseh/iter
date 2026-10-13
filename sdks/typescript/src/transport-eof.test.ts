import childProcess, { ChildProcess } from "child_process";
import { EventEmitter, once } from "events";
import { PassThrough } from "stream";
import { ConnectionClosedError, ConnectionError, IterClient } from "./index";

class TransportChild extends EventEmitter {
  stdin = new PassThrough();
  stdout = new PassThrough();
  stderr = new PassThrough();
  exitCode: number | null = null;
  signalCode: NodeJS.Signals | null = null;

  kill(signal: NodeJS.Signals = "SIGTERM"): boolean {
    this.signalCode = signal;
    this.emit("exit", null, signal);
    this.stdout.end();
    this.stderr.end();
    this.emit("close", null, signal);
    return true;
  }
}

async function bounded<T>(promise: Promise<T>): Promise<T> {
  let timer: ReturnType<typeof setTimeout> | undefined;
  try {
    return await Promise.race([
      promise,
      new Promise<never>((_, reject) => {
        timer = setTimeout(() => reject(new Error("transport did not settle promptly")), 2000);
      }),
    ]);
  } finally {
    clearTimeout(timer);
  }
}

async function withClient(
  proc: ChildProcess | TransportChild,
  run: (client: IterClient) => Promise<void>
): Promise<void> {
  const spawn = jest.spyOn(childProcess, "spawn").mockReturnValueOnce(proc as ChildProcess);
  const client = await IterClient.connect("test-transport", { maxInflight: 2 });
  spawn.mockRestore();
  try {
    await run(client);
  } finally {
    // Also reap children and clear request timers when a regression assertion fails.
    for (const pending of (client as any).responseQueue.values()) {
      pending.reject(new ConnectionError("test cleanup"));
    }
    (client as any).responseQueue.clear();
    if (proc.exitCode === null && proc.signalCode === null) proc.kill("SIGKILL");
    await client.close();
    proc.stdin?.destroy();
    proc.stdout?.destroy();
    proc.stderr?.destroy();
  }
}

function outcome(client: IterClient, method = "test", params?: unknown): Promise<unknown> {
  return client.send(method, params).then(value => value, (error: Error) => error);
}

describe("Terminal transport event ordering", () => {
  test("stdout EOF fails all pending requests, retains stderr, and blocks admission", async () => {
    const proc = new TransportChild();
    await withClient(proc, async client => {
      proc.stderr.write("transport diagnostic\n");
      const first = outcome(client);
      const second = outcome(client);
      proc.stdout.end();
      const results = await bounded(Promise.all([first, second]));
      for (const result of results) {
        expect(result).toBeInstanceOf(ConnectionError);
        expect((result as Error).message).toContain("transport diagnostic");
      }
      expect((client as any).responseQueue.size).toBe(0);
      expect((client as any)._state).toBe("closing");
      expect((client as any)._closePromise).toBeNull();
      expect(proc.signalCode).toBeNull();
      await expect(bounded(client.send("later"))).rejects.toBeInstanceOf(ConnectionClosedError);
      await bounded(client.close());
      expect(proc.signalCode).not.toBeNull();
      await client.close();
    });
  });

  test("exit stops admission without discarding responses still buffered on stdout", async () => {
    const proc = new TransportChild();
    await withClient(proc, async client => {
      const first = outcome(client);
      const second = outcome(client);
      proc.exitCode = 0;
      proc.emit("exit", 0, null);
      const late = outcome(client, "later");
      const response = { jsonrpc: "2.0", id: 1, result: { complete: true } };
      proc.stdout.end(JSON.stringify(response));
      expect(await bounded(first)).toEqual(response);
      expect(await bounded(second)).toBeInstanceOf(ConnectionError);
      expect(await bounded(late)).toBeInstanceOf(ConnectionClosedError);
      expect((client as any).responseQueue.size).toBe(0);
    });
  });

  test.each(["process", "stdout", "stdin"])("%s error is terminal and handled", async source => {
    const proc = new TransportChild();
    await withClient(proc, async client => {
      const request = outcome(client);
      const target = source === "process" ? proc : source === "stdout" ? proc.stdout : proc.stdin;
      expect(() => target.emit("error", new Error("transport fault"))).not.toThrow();
      const result = await bounded(request);
      expect(result).toBeInstanceOf(ConnectionError);
      expect((result as Error).message).toContain("transport fault");
      await expect(bounded(client.send("later"))).rejects.toBeInstanceOf(ConnectionClosedError);
      expect((client as any).responseQueue.size).toBe(0);
    });
  });

  test("EOF during close releases the drain and clears all request timers", async () => {
    const proc = new TransportChild();
    await withClient(proc, async client => {
      const request = outcome(client);
      const closed = client.close();
      proc.stdout.end();
      expect(await bounded(request)).toBeInstanceOf(ConnectionError);
      await bounded(closed);
      expect((client as any)._state).toBe("closed");
      proc.emit("exit", 0, null);
      proc.emit("close", 0, null);
      expect((client as any)._state).toBe("closed");
    });
  });

  test("stdout destruction without end is terminal", async () => {
    const proc = new TransportChild();
    await withClient(proc, async client => {
      const request = outcome(client);
      proc.stdout.destroy();
      expect(await bounded(request)).toBeInstanceOf(ConnectionError);
      await expect(bounded(client.send("later"))).rejects.toBeInstanceOf(ConnectionClosedError);
    });
  });

  test("a complete response preceding EOF stays successful", async () => {
    const proc = new TransportChild();
    await withClient(proc, async client => {
      const request = outcome(client);
      const response = { jsonrpc: "2.0", id: 1, result: "complete" };
      const ended = once((client as any).lineReader, "close");
      proc.stdout.end(JSON.stringify(response) + "\n");
      await bounded(ended);
      expect(await request).toEqual(response);
      await expect(bounded(client.send("later"))).rejects.toBeInstanceOf(ConnectionClosedError);
    });
  });
});

describe("Real subprocess transport loss", () => {
  function start(mode: "exit" | "eof" | "response"): ChildProcess {
    return childProcess.spawn(process.execPath, ["-e", `
      setInterval(() => {}, 1000);
      process.stdin.once('data', () => {
        process.stdin.pause();
        if (${JSON.stringify(mode)} === 'exit') process.exit(7);
        else if (${JSON.stringify(mode)} === 'response') {
          process.stdout.write(JSON.stringify({jsonrpc:'2.0',id:1,result:'complete'})+'\\n',
            () => process.exit(0));
        } else require('fs').closeSync(1);
      });
    `], { stdio: ["pipe", "pipe", "pipe"] });
  }

  test("unexpected exit rejects the pending request and every later send", async () => {
    const proc = start("exit");
    await withClient(proc, async client => {
      expect(await bounded(outcome(client))).toBeInstanceOf(ConnectionError);
      await expect(bounded(client.send("later"))).rejects.toBeInstanceOf(ConnectionClosedError);
      await bounded(client.close());
      expect(proc.exitCode !== null || proc.signalCode !== null).toBe(true);
    });
  });

  // Node retains Windows stdout handles after closeSync(1); Unix exercises remote EOF.
  const unixTest = process.platform === "win32" ? test.skip : test;
  unixTest("child-alive remote EOF releases a backpressured send and close reaps it", async () => {
    const proc = start("eof");
    await withClient(proc, async client => {
      const request = outcome(client, "large", "x".repeat(2 * 1024 * 1024));
      expect(proc.stdin!.writableNeedDrain).toBe(true);
      expect(await bounded(request)).toBeInstanceOf(ConnectionError);
      expect(proc.exitCode).toBeNull();
      expect(proc.signalCode).toBeNull();
      await expect(bounded(client.send("later"))).rejects.toBeInstanceOf(ConnectionClosedError);
      await bounded(client.close());
      expect(proc.exitCode !== null || proc.signalCode !== null).toBe(true);
    });
  });

  test("closing a live child's stdout reader releases a backpressured send and reaps it", async () => {
    const proc = start("eof");
    await withClient(proc, async client => {
      const request = outcome(client, "large", "x".repeat(2 * 1024 * 1024));
      expect(proc.stdin!.writableNeedDrain).toBe(true);
      proc.stdout!.destroy();
      expect(await bounded(request)).toBeInstanceOf(ConnectionError);
      expect(proc.exitCode).toBeNull();
      expect(proc.signalCode).toBeNull();
      await expect(bounded(client.send("later"))).rejects.toBeInstanceOf(ConnectionClosedError);
      await bounded(client.close());
      expect(proc.exitCode !== null || proc.signalCode !== null).toBe(true);
    });
  });

  test("a real final response is delivered before transport loss", async () => {
    const proc = start("response");
    await withClient(proc, async client => {
      const closed = once(proc, "close");
      expect(await bounded(outcome(client))).toEqual({ jsonrpc: "2.0", id: 1, result: "complete" });
      await bounded(closed);
      await expect(bounded(client.send("later"))).rejects.toBeInstanceOf(ConnectionClosedError);
      await client.close();
    });
  });
});
