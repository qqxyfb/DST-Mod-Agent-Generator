import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";

// 前端开发服务器：Tauri dev 模式固定端口，见 src-tauri/tauri.conf.json
export default defineConfig({
  plugins: [vue()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: "127.0.0.1",
  },
  envPrefix: ["VITE_", "TAURI_"],
  build: {
    target: ["es2021", "chrome100", "safari13"],
    minify: "esbuild",
    sourcemap: false,
  },
});