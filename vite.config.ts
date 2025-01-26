import path from "path";
import { fileURLToPath } from "url";
import { defineConfig } from "vite";
import solidPlugin from "vite-plugin-solid";
import { viteSingleFile } from "vite-plugin-singlefile";
import tailwindcss from "@tailwindcss/vite";
import { visualizer } from "rollup-plugin-visualizer";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

// https://vite.dev/config/
export default defineConfig({
    plugins: [
        solidPlugin(),
        viteSingleFile(),
        tailwindcss(),
        visualizer({
            open: true, // 构建完成后自动打开报告
        }),
    ],
    server: {
        proxy: {
            "/api": {
                target: "http://127.0.0.1:41280",
                changeOrigin: true,
            },
        },
    },
    resolve: {
        alias: {
            "@": path.resolve(__dirname, "./src"),
        },
    },
});
