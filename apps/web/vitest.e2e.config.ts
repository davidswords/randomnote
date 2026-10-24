import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    include: ["e2e/**/*.e2e.test.ts"],
    globalSetup: ["e2e/setup.ts"],
    // Argon2 runs in the browser's WebAssembly, so signing up takes a few seconds.
    testTimeout: 120_000,
    hookTimeout: 120_000,
    fileParallelism: false,
  },
});
