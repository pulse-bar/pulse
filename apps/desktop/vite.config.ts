import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import path from 'node:path';

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  // Vite serves React from `ui/`; `shell/` next to us holds the Rust crate
  // and tauri.conf.json so the Tauri CLI finds it as a subfolder.
  root: path.resolve(__dirname, 'ui'),
  plugins: [react()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, 'ui/src'),
      '@pulse/types': path.resolve(__dirname, '../../packages/types/src/index.ts'),
    },
  },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: 'ws', host, port: 1421 } : undefined,
    watch: { ignored: ['**/shell/**'] },
  },
  envPrefix: ['VITE_', 'TAURI_ENV_*'],
  build: {
    outDir: path.resolve(__dirname, 'dist'),
    emptyOutDir: true,
    target: process.env.TAURI_ENV_PLATFORM === 'windows' ? 'chrome105' : 'safari14',
    minify: !process.env.TAURI_ENV_DEBUG ? 'esbuild' : false,
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
  },
});
