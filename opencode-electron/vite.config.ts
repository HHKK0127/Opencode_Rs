import { defineConfig } from 'vite';
import solid from 'vite-plugin-solid';
import electron from 'vite-plugin-electron';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

export default defineConfig({
  envDir: '.',
  root: __dirname,
  plugins: [
    solid(),
    electron([
      {
        entry: 'electron/main/index.ts',
        onstart: (options) => options.startup(),
        vite: {
          build: {
            sourcemap: true,
            minify: process.env.NODE_ENV === 'production',
            outDir: 'dist-electron/main',
            rollupOptions: {
              external: ['electron']
            }
          }
        }
      },
      {
        entry: 'electron/preload/index.ts',
        onstart: (options) => options.reload(),
        vite: {
          build: {
            sourcemap: true,
            minify: false,
            outDir: 'dist-electron/preload',
            rollupOptions: {
              external: ['electron']
            }
          }
        }
      }
    ])
  ],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
      '@electron': path.resolve(__dirname, './electron')
    }
  },
  build: {
    target: 'esnext',
    outDir: 'dist',
    emptyOutDir: true,
    rollupOptions: {
      input: path.resolve(__dirname, 'index.html')
    }
  }
});