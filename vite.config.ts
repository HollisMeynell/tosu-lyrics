import { defineConfig } from 'vite'
import solidPlugin from 'vite-plugin-solid';
import {viteSingleFile} from "vite-plugin-singlefile";

// https://vite.dev/config/
export default defineConfig({
  plugins: [solidPlugin(), viteSingleFile()],
  server: {
    proxy: {
      '/api': {
        target: 'http://127.0.0.1:41280',
        changeOrigin: true,
      }
    }
  }
})
