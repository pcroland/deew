import { resolve } from "node:path"
import tailwindcss from "@tailwindcss/vite"
import react from "@vitejs/plugin-react"
import { defineConfig } from "vite"

// https://vite.dev/config/
export default defineConfig({
  plugins: [react(), tailwindcss()],
  resolve: {
    alias: {
      "@": resolve(import.meta.dirname, "./src"),
    },
  },
  server: {
    port: 3000,
    strictPort: true,
    // deew-web (cargo run -p deew-web). No changeOrigin: the server compares
    // the Origin header with the Host header.
    proxy: {
      "/api": { target: "http://127.0.0.1:8765", ws: true },
    },
    watch: {
      ignored: ["**/target/**"],
    },
  },
})
