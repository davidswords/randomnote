import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { defineConfig } from "vite";

const desktop = JSON.parse(readFileSync(new URL("../desktop/package.json", import.meta.url), "utf8"));

export default defineConfig({
  define: { __APP_VERSION__: JSON.stringify(desktop.version) },
  resolve: {
    alias: {
      "randomtask-web-wasm": fileURLToPath(new URL("./src/wasm/randomtask_web.js", import.meta.url)),
    },
  },
  server: { port: 1430, strictPort: true },
  preview: { port: 1430, strictPort: true },
  worker: { format: "es" },
});
