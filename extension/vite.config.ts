import { defineConfig } from 'vite';
import webExtension from 'vite-plugin-web-extension';
import { resolve } from 'path';

export default defineConfig({
  plugins: [
    webExtension({
      manifest: 'manifest.json',
      additionalInputs: ['src/overlay/index.ts'],
    }),
  ],

  build: {
    outDir: 'dist',
    emptyOutDir: true,
  },

  resolve: {
    alias: {
      '@loci/core': resolve(__dirname, '../packages/core/src'),
    },
  },
});
