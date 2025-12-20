import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';
import path from 'path';
import checker from 'vite-plugin-checker';
import wasm from 'vite-plugin-wasm';
import topLevelAwait from 'vite-plugin-top-level-await';

// https://vitejs.dev/config/
export default defineConfig({
  build: {
    sourcemap: true,
    rollupOptions: {
      output: {
        manualChunks: (id: string) => {
          if (id.includes('@biscuit-auth/biscuit-wasm')) return 'vendor-biscuit';
          if (id.includes('ag-grid')) return 'vendor-ag-grid';
          if (id.includes('swagger-ui')) return 'vendor-swagger';
          if (id.includes('codemirror') || id.includes('@codemirror')) return 'vendor-codemirror';
          if (id.includes('node_modules')) {
            if (id.includes('vue') || id.includes('vue-router') || id.includes('axios')) {
              return 'vendor-core';
            }
          }
        },
      },
    },
  },
  plugins: [
    vue(),
    checker({
      eslint: {
        lintCommand: 'eslint --ext .js,.ts,.vue --ignore-path .gitignore ./src',
      },
      typescript: true,
      vueTsc: true,
    }),
    wasm(),
    topLevelAwait(),
  ],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  define: {
    global: 'window',
  },
});
