import { configDefaults, defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    // End-to-end tests need a built app and a display; they run with vitest.e2e.config.ts.
    exclude: [...configDefaults.exclude, "e2e/**"],
  },
});
