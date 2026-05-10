import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import path from "path";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [react(), tailwindcss()],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  build: {
    chunkSizeWarningLimit: 1000,
  },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: "ws", host, port: 1421 } : undefined,
    watch: {
      ignored: ["**/src-tauri/**", "**/api-server/**"],
    },
    // Web-deployment Phase 1: proxy /api/v1/* to the local Axum server
    // so the dev experience for web-mode mirrors the production routing.
    // Override the target via WT_API_PROXY env var (e.g., when the
    // server runs in Docker on a different port). Tauri builds ignore
    // this proxy because the Tauri transport uses IPC not fetch.
    proxy: {
      "/api/v1": {
        target: process.env.WT_API_PROXY || "http://127.0.0.1:8787",
        changeOrigin: false,
      },
    },
  },
});
