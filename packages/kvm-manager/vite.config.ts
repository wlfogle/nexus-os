import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { resolve } from "path";

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  plugins: [react()],

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 3000,
    strictPort: true,
    watch: {
      // 3. tell vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
  // Build optimizations
  build: {
    rollupOptions: {
      output: {
        manualChunks: {
          // Separate vendor libraries
          'react-vendor': ['react', 'react-dom'],
          'mui-vendor': [
            '@mui/material',
            '@mui/icons-material',
            '@mui/x-charts',
            '@mui/x-data-grid',
            '@emotion/react',
            '@emotion/styled'
          ],
          'ui-vendor': [
            '@headlessui/react',
            '@heroicons/react',
            'framer-motion',
            'notistack',
            'react-hot-toast'
          ],
          'form-vendor': [
            'react-hook-form',
            '@hookform/resolvers',
            'yup',
            'zod'
          ],
          'utils-vendor': [
            'clsx',
            'tailwind-merge',
            'date-fns',
            'uuid'
          ],
          'tauri-vendor': [
            '@tauri-apps/api',
            '@tauri-apps/plugin-shell'
          ],
          'charts-vendor': ['recharts'],
          'routing-vendor': ['react-router-dom'],
          'query-vendor': ['@tanstack/react-query'],
          'virtualization-vendor': [
            'react-window',
            'react-virtualized-auto-sizer',
            'react-draggable'
          ]
        }
      }
    },
    // Increase chunk size warning limit to 1000kb (from default 500kb)
    chunkSizeWarningLimit: 1000,
    // Enable source maps for better debugging
    sourcemap: true
  },
  // Vitest configuration
  test: {
    globals: true,
    environment: 'jsdom',
    setupFiles: ['./src/test/setup.ts'],
    css: true,
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html'],
      exclude: [
        'node_modules/',
        'src/test/',
        '**/*.d.ts',
        '**/*.config.ts',
      ],
    },
  },
  // Path resolution for imports
  resolve: {
    alias: {
      '@': resolve(__dirname, './src'),
    },
  },
}));
