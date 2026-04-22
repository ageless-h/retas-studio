import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig(async () => ({
  plugins: [react()],
  clearScreen: false,
  build: {
    rollupOptions: {
      external: ["@tauri-apps/plugin-dialog"],
      output: {
        manualChunks: {
          vendor: ["react", "react-dom"],
          blueprint: ["@blueprintjs/core", "@blueprintjs/icons"],
          dockview: ["dockview"],
          grid: ["ag-grid-react", "ag-grid-community"],
          icons: ["lucide-react"],
        },
      },
    },
    chunkSizeWarningLimit: 600,
  },
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
}));
