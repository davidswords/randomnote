import { configDefaults, defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    // End-to-end tests need the built app, a browser and the server; they run with vitest.e2e.config.ts.
    exclude: [...configDefaults.exclude, "e2e/**"],
    coverage: {
      provider: "v8",
      include: ["src/**", "scripts/**"],
      // main.ts and worker.ts only start the app, so only the end-to-end tests reach them.
      // check-tools.ts only runs the commands whose output tools.ts judges.
      exclude: [
        "src/**/*.test.ts",
        "scripts/**/*.test.ts",
        "src/main.ts",
        "src/worker.ts",
        "src/wasm/**",
        "src/vite-env.d.ts",
        "scripts/check-tools.ts",
      ],
      thresholds: { lines: 100, branches: 100, functions: 100, statements: 100 },
    },
  },
});
