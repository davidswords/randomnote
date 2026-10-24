import { describe, expect, it } from "vitest";
import { type Reply, type Request, workerInvoke } from "./worker-invoke";

/** A worker that records what it's sent and replies when a test says so. */
function fakeWorker() {
  const sent: Request[] = [];
  const listeners: ((event: { data: Reply }) => void)[] = [];
  return {
    sent,
    port: {
      postMessage: (request: Request) => sent.push(request),
      addEventListener: (_type: "message", listener: (event: { data: Reply }) => void) =>
        listeners.push(listener),
    },
    reply: (reply: Reply) => listeners.forEach((listener) => listener({ data: reply })),
  };
}

describe("workerInvoke", () => {
  it("sends the command and its arguments to the worker", () => {
    const worker = fakeWorker();
    const invoke = workerInvoke(worker.port);

    void invoke("find_task", { id: "task-1" });

    expect(worker.sent).toEqual([{ id: 0, command: "find_task", args: { id: "task-1" } }]);
  });

  it("sends empty arguments when there are none", () => {
    const worker = fakeWorker();
    const invoke = workerInvoke(worker.port);

    void invoke("list_tasks");

    expect(worker.sent).toEqual([{ id: 0, command: "list_tasks", args: {} }]);
  });

  it("resolves with the result the worker sends back", async () => {
    const worker = fakeWorker();
    const invoke = workerInvoke(worker.port);
    const result = invoke<boolean>("has_account");

    worker.reply({ id: 0, result: true });

    await expect(result).resolves.toBe(true);
  });

  it("rejects with the message the worker sends back", async () => {
    const worker = fakeWorker();
    const invoke = workerInvoke(worker.port);
    const result = invoke("unlock", { password: "wrong" });

    worker.reply({ id: 0, error: "Wrong password" });

    await expect(result).rejects.toBe("Wrong password");
  });

  it("matches each reply to its own command, in whatever order they arrive", async () => {
    const worker = fakeWorker();
    const invoke = workerInvoke(worker.port);
    const first = invoke("find_task", { id: "a" });
    const second = invoke("find_task", { id: "b" });

    worker.reply({ id: 1, result: "second" });
    worker.reply({ id: 0, result: "first" });

    await expect(first).resolves.toBe("first");
    await expect(second).resolves.toBe("second");
  });

  it("ignores a reply to a command it isn't waiting for", async () => {
    const worker = fakeWorker();
    const invoke = workerInvoke(worker.port);
    const result = invoke("has_account");

    worker.reply({ id: 7, result: "stray" });
    worker.reply({ id: 0, result: false });
    worker.reply({ id: 0, result: "duplicate" });

    await expect(result).resolves.toBe(false);
  });
});
