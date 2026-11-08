import { svelte } from '@sveltejs/vite-plugin-svelte';
import { defineConfig } from 'vitest/config';

// Tauri expects a fixed dev-server port and serves the built `dist` in release builds. Vitest runs
// in mode `test`, where Svelte must resolve its browser build for components to mount in jsdom.
export default defineConfig(({ mode }) => ({
  plugins: [svelte()],
  clearScreen: false,
  server: { port: 1420, strictPort: true },
  build: { target: 'safari16', outDir: 'dist' },
  resolve: mode === 'test' ? { conditions: ['browser'] } : undefined,
  test: {
    environment: 'jsdom',
    setupFiles: ['./src/test-setup.ts'],
    include: ['src/**/*.test.ts'],
    // The theme tests read the stylesheets as text (`?raw`), which Vitest blanks unless it processes them.
    css: { include: [/\/src\/styles\/[^/]+\.css/] },
  },
}));
