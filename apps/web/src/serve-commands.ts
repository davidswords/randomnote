import type { Reply, Request } from "./worker-invoke";

/** The Rust app compiled to WebAssembly, as far as the worker needs it. */
export type Command = {
  /** Takes and gives JSON; rejects with the message to show the user. */
  invoke(command: string, args: string): Promise<string>;
};

/** The worker's end of the channel to the page. */
export type WorkerScope = {
  postMessage(reply: Reply): void;
  addEventListener(type: "message", listener: (event: { data: Request }) => unknown): void;
};

/**
 * Answers the page's commands with the app once it has opened. Commands that
 * arrive while it opens wait for it; if it can't open, each gets the reason.
 */
export function serveCommands(scope: WorkerScope, opening: Promise<Command>): void {
  scope.addEventListener("message", async ({ data: { id, command, args } }) => {
    try {
      const app = await opening;
      const result: unknown = JSON.parse(await app.invoke(command, JSON.stringify(args)));
      scope.postMessage({ id, result });
    } catch (error) {
      scope.postMessage({ id, error: String(error) });
    }
  });
}
