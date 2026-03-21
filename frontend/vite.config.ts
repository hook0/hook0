import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';
import path from 'path';
import checker from 'vite-plugin-checker';
import wasm from 'vite-plugin-wasm';
import tailwindcss from '@tailwindcss/vite';

// https://vitejs.dev/config/
export default defineConfig({
  server: {
    port: 3000,
    strictPort: true,
  },
  build: {
    sourcemap: true,
    rollupOptions: {
      output: {
        manualChunks: (id: string) => {
          if (id.includes('@biscuit-auth/biscuit-wasm')) return 'vendor-biscuit';
          if (id.includes('codemirror') || id.includes('@codemirror')) return 'vendor-codemirror';
          if (id.includes('echarts') || id.includes('vue-echarts')) return 'vendor-charts';
          if (id.includes('node_modules')) {
            if (
              id.includes('vue') ||
              id.includes('vue-router') ||
              id.includes('pinia') ||
              id.includes('@tanstack/vue-query') ||
              id.includes('axios')
            ) {
              return 'vendor-core';
            }
          }
        },
      },
    },
  },
  plugins: [
    vue(),
    tailwindcss(),
    checker({
      overlay: false,
      eslint: {
        lintCommand: 'eslint --ext .js,.ts,.vue --ignore-path .gitignore ./src',
      },
      typescript: true,
      vueTsc: true,
    }),
    wasm(),
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
