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
  test.each(["EOF", "close"])("%s cancels the exit drain timer after buffered replies", async terminal => {
    jest.useFakeTimers({ doNotFake: ["nextTick", "setImmediate"] });
    try {
      const proc = new TransportChild();
      await withClient(proc, async client => {
        const request = outcome(client);
        proc.exitCode = 0;
        proc.emit("exit", 0, null);
        proc.stdout.write(JSON.stringify({ jsonrpc: "2.0", id: 1, result: "buffered" }) + "\n");
        expect(await request).toMatchObject({ result: "buffered" });
        expect(jest.getTimerCount()).toBe(1);
        if (terminal === "EOF") {
          const ended = once((client as any).lineReader, "close");
          proc.stdout.end();
          await ended;
        } else await client.close();
        expect(jest.getTimerCount()).toBe(0);
        await client.close();
        await jest.advanceTimersByTimeAsync(6000);
        expect((client as any)._state).toBe("closed");
      });
    } finally {
      jest.useRealTimers();
    }
  });

  test("exit bounds unanswered requests even when stdout and process close never arrive", async () => {
    jest.useFakeTimers({ doNotFake: ["nextTick", "setImmediate"] });
    try {
      const proc = new TransportChild();
      await withClient(proc, async client => {
        const answered = outcome(client);
        const unanswered = outcome(client);
        proc.exitCode = 7;
        proc.emit("exit", 7, null);
        const response = { jsonrpc: "2.0", id: 1, result: "buffered" };
        proc.stdout.write(JSON.stringify(response) + "\n");
        expect(await answered).toEqual(response);
        await jest.advanceTimersByTimeAsync(4999);
        expect((client as any).responseQueue.size).toBe(1);
        await jest.advanceTimersByTimeAsync(1);
        expect((client as any).responseQueue.size).toBe(0);
        const error = await unanswered;
        expect(error).toBeInstanceOf(ConnectionError);
        expect((error as Error).message).toContain("code 7");
        await client.close();
        expect(jest.getTimerCount()).toBe(0);
        expect(proc.stdin.destroyed).toBe(true);
        expect(proc.stdout.destroyed).toBe(true);
        expect(proc.stderr.destroyed).toBe(true);
      });
    } finally {
      jest.useRealTimers();
    }
  });

  test.each([
    [7, null, "code 7"],
    [null, "SIGTERM", "signal SIGTERM"],
  ] as const)("EOF preserves observed exit status %s/%s", async (code, signal, diagnostic) => {
    const proc = new TransportChild();
    await withClient(proc, async client => {
      proc.stderr.write("exit diagnostic\n");
      const request = outcome(client);
      proc.exitCode = code;
      proc.signalCode = signal;
      proc.emit("exit", code, signal);
      proc.stdout.end();
      const error = await bounded(request);
      expect(error).toBeInstanceOf(ConnectionError);
      expect((error as Error).message).toContain(diagnostic);
      expect((error as Error).message).toContain("exit diagnostic");
    });
  });

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
  // Windows pipe lifetime differs for both remote EOF and descendant inheritance.
  const unixTest = process.platform === "win32" ? test.skip : test;

  function stopHolder(pid: number): void {
    try { process.kill(pid, "SIGKILL"); }
    catch (error) { if ((error as NodeJS.ErrnoException).code !== "ESRCH") throw error; }
  }

  function start(mode: "exit" | "eof" | "response" | "held"): ChildProcess {
    return childProcess.spawn(process.execPath, ["-e", `
      setInterval(() => {}, 1000);
      process.stdin.once('data', () => {
        process.stdin.pause();
        if (${JSON.stringify(mode)} === 'exit') process.exit(7);
        else if (${JSON.stringify(mode)} === 'response') {
          process.stdout.write(JSON.stringify({jsonrpc:'2.0',id:1,result:'complete'})+'\\n',
            () => process.exit(0));
        } else if (${JSON.stringify(mode)} === 'held') {
          const holder = require('child_process').spawn(process.execPath,
            ['-e', 'setTimeout(() => {}, 20000)'], {stdio:['ignore',1,2]});
          holder.once('spawn', () => process.send({holderPid:holder.pid}, () => process.exit(7)));
        } else require('fs').closeSync(1);
      });
    `], { stdio: ["pipe", "pipe", "pipe", "ipc"] });
  }

  unixTest("descendant-held pipes do not defer failure to the request timeout", async () => {
    const proc = start("held");
    let holderPid: number | undefined;
    proc.once("message", (message: { holderPid: number }) => { holderPid = message.holderPid; });
    try {
      await withClient(proc, async client => {
        const request = outcome(client);
        await bounded(once(proc, "exit"));
        expect(holderPid).toBeDefined();
        expect(proc.stdout!.readableEnded).toBe(false);
        // The existing close drain budget is 5s, independent of the 30s request timeout.
        await new Promise(resolve => setTimeout(resolve, 5500));
        expect((client as any).responseQueue.size).toBe(0);
        const error = await bounded(request);
        expect(error).toBeInstanceOf(ConnectionError);
        expect((error as Error).message).toContain("code 7");
        await client.close();
        expect(proc.stdout!.destroyed).toBe(true);
        expect(proc.stderr!.destroyed).toBe(true);
      });
    } finally {
      if (holderPid !== undefined) stopHolder(holderPid);
    }
  }, 10000);

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
