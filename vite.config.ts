import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";

import * as child from "child_process";
import * as fs from "node:fs";

const host = process.env.TAURI_DEV_HOST;

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  plugins: [
    vue(),
    {
      // Update version.ts when frontend built
      name: "update-version",
      async buildStart(options) {
        // Get commit from git
        const commitHash = child.execSync("git rev-parse --short HEAD").toString().trim();

        // Write to src/version.ts
        fs.writeFileSync("src/version.ts", "export const COMMIT_HASH = " + JSON.stringify(commitHash) + ";");
      },
    },
  ],

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
}));
