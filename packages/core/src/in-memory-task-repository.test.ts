import { InMemoryTaskRepository } from "./in-memory-task-repository";
import { taskRepositoryContract } from "./task-repository.contract";

taskRepositoryContract(() => new InMemoryTaskRepository());
