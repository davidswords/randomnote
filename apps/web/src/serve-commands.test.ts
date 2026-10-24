import { describe, expect, it } from "vitest";
import { type Command, serveCommands } from "./serve-commands";
import type { Reply, Request } from "./worker-invoke";

/** The worker's side: what it's sent, and what it posts back. */
function fakeScope() {
  const posted: Reply[] = [];
  let listener: (event: { data: Request }) => unknown = () => undefined;
  return {
    posted,
    scope: {
      postMessage: (reply: Reply) => posted.push(reply),
      addEventListener: (_type: "message", handler: (event: { data: Request }) => unknown) => {
        listener = handler;
      },
    },
    send: (request: Request) => listener({ data: request }),
  };
}

/** The WebAssembly app's invoke: JSON strings in and out, rejecting with a message. */
function fakeApp(run: (command: string, args: unknown) => unknown): Command {
  return {
    invoke: async (command, args) => JSON.stringify(run(command, JSON.parse(args))),
  };
}

const settle = () => new Promise((resolve) => setTimeout(resolve, 0));

describe("serveCommands", () => {
  it("runs each command with its arguments and posts the result back", async () => {
    const worker = fakeScope();
    serveCommands(
      worker.scope,
      Promise.resolve(fakeApp((command, args) => ({ command, args }))),
    );

    await worker.send({ id: 3, command: "find_task", args: { id: "task-1" } });
    await settle();

    expect(worker.posted).toEqual([
      { id: 3, result: { command: "find_task", args: { id: "task-1" } } },
    ]);
  });

  it("posts back a null result", async () => {
    const worker = fakeScope();
    serveCommands(worker.scope, Promise.resolve(fakeApp(() => null)));

    await worker.send({ id: 0, command: "find_task", args: { id: "missing" } });
    await settle();

    expect(worker.posted).toEqual([{ id: 0, result: null }]);
  });

  it("posts back the message a refused command rejects with", async () => {
    const worker = fakeScope();
    const app: Command = { invoke: () => Promise.reject("Wrong password") };
    serveCommands(worker.scope, Promise.resolve(app));

    await worker.send({ id: 1, command: "unlock", args: { password: "wrong" } });
    await settle();

    expect(worker.posted).toEqual([{ id: 1, error: "Wrong password" }]);
  });

  it("waits for the app to open before running commands sent early", async () => {
    const worker = fakeScope();
    let open: (app: Command) => void = () => undefined;
    serveCommands(worker.scope, new Promise((resolve) => (open = resolve)));

    void worker.send({ id: 0, command: "has_account", args: {} });
    await settle();
    expect(worker.posted).toEqual([]);
    open(fakeApp(() => false));
    await settle();

    expect(worker.posted).toEqual([{ id: 0, result: false }]);
  });

  it("answers every command with why the app couldn't open", async () => {
    const worker = fakeScope();
    serveCommands(worker.scope, Promise.reject("Couldn't open the browser's storage"));

    await worker.send({ id: 0, command: "has_account", args: {} });
    await settle();

    expect(worker.posted).toEqual([{ id: 0, error: "Couldn't open the browser's storage" }]);
  });
});
