import { defineConfig } from 'vite';
import { resolve } from 'node:path';

// Two pages: the index (landing with the globe) and the app. /api is proxied to the Rust backend.
export default defineConfig({
  server: { port: 5173, proxy: { '/api': 'http://127.0.0.1:8787' } },
  preview: { port: 4173, proxy: { '/api': 'http://127.0.0.1:8787' } },
  build: {
    rollupOptions: { input: { index: resolve(__dirname, 'index.html'), app: resolve(__dirname, 'app.html') } },
  },
});
