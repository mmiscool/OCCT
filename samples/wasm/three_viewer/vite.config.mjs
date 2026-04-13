import { resolve } from "node:path";
import { defineConfig } from "vite";

const outputDir = process.env.OCCT_WASM_WEB_OUTPUT_DIR || resolve(__dirname, "dist");

export default defineConfig({
  base: "./",
  build: {
    emptyOutDir: false,
    outDir: outputDir,
    rollupOptions: {
      input: resolve(__dirname, "OcctThreeDemo.html")
    }
  }
});
