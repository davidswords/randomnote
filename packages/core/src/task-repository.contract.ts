import { describe, expect, it } from "vitest";
import type { Task } from "./task";
import type { TaskRepository } from "./task-repository";

const aTask = (id: string, title: string): Task => ({ id, title });

/**
 * The behaviour every TaskRepository must have. Each implementation runs
 * this suite: the in-memory one as a unit test, real storage adapters as
 * integration tests.
 */
export function taskRepositoryContract(
  makeRepository: () => TaskRepository | Promise<TaskRepository>,
): void {
  describe("TaskRepository contract", () => {
    it("finds a saved task by its ID", async () => {
      const repository = await makeRepository();
      const task = aTask("task-1", "Buy milk");

      await repository.save(task);

      expect(await repository.findById("task-1")).toEqual(task);
    });

    it("finds nothing for an ID that was never saved", async () => {
      const repository = await makeRepository();

      expect(await repository.findById("missing")).toBeUndefined();
    });
  });
}
