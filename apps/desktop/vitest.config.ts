import { configDefaults, defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    // End-to-end tests need a built app and a display; they run with vitest.e2e.config.ts.
    exclude: [...configDefaults.exclude, "e2e/**"],
    coverage: {
      provider: "v8",
      include: ["src/**"],
      // main.ts only starts the app, so only the end-to-end tests reach it.
      exclude: ["src/**/*.test.ts", "src/main.ts"],
      thresholds: { lines: 100, branches: 100, functions: 100, statements: 100 },
    },
  },
});
