import type { Invoke } from "@randomtask/ui";

/** A command sent to the worker that runs the Rust app. */
export type Request = { id: number; command: string; args: Record<string, unknown> };

/** The worker's answer: the command's result, or the message it was refused with. */
export type Reply = { id: number; result: unknown } | { id: number; error: string };

/** The page's end of the channel to the worker. */
export type WorkerPort = {
  postMessage(request: Request): void;
  addEventListener(type: "message", listener: (event: { data: Reply }) => void): void;
};

/**
 * Runs the Rust app's commands in the worker, as Tauri's invoke does on the
 * desktop: resolving with the result, or rejecting with the message.
 */
export function workerInvoke(port: WorkerPort): Invoke {
  const waiting = new Map<number, (reply: Reply) => void>();
  let nextId = 0;

  port.addEventListener("message", ({ data: reply }) => {
    waiting.get(reply.id)?.(reply);
    waiting.delete(reply.id);
  });

  return <T>(command: string, args: Record<string, unknown> = {}) =>
    new Promise<T>((resolve, reject) => {
      const id = nextId++;
      waiting.set(id, (reply) => ("error" in reply ? reject(reply.error) : resolve(reply.result as T)));
      port.postMessage({ id, command, args });
    });
}
