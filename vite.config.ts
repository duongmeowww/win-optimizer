import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Vite config for Tauri (fixed port, clearScreen disabled for Tauri)
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    // Watch only real source; ignore Rust target + node_modules to avoid EBUSY
    // from chokidar touching files cargo is writing.
    watch: {
      ignored: ["**/src-tauri/target/**", "**/node_modules/**", "**/dist/**"],
    },
  },
  build: {
    target: "es2021",
    outDir: "dist",
    // Relative paths so assets resolve under tauri://localhost/
    // (without this, /assets/... 404s or hits http://localhost refused)
    assetsDir: "assets",
    emptyOutDir: true,
    chunkSizeWarningLimit: 600,
    rollupOptions: {
      output: {
        manualChunks: (id: string) => {
          if (id.includes("node_modules/antd") || id.includes("node_modules/@ant-design")) return "antd";
          if (id.includes("node_modules/react")) return "react";
          if (id.includes("node_modules/i18next") || id.includes("node_modules/react-i18next")) return "i18n";
        },
      },
    },
  },
  // base: "" => Vite emits ./assets/... instead of /assets/...
  base: "",
});
