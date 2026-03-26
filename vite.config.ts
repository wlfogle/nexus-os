import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import { resolve } from 'path'

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  plugins: [react()],

  // Base path for assets - important for Tauri
  base: './',

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  // prevent vite from obscuring rust errors
  clearScreen: false,
  
  // Tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      // Tell vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },

  resolve: {
    alias: {
      '@': resolve(__dirname, 'src'),
      '~': resolve(__dirname, 'src'),
    },
  },

  build: {
    // Tauri supports es2021
    target: process.env.TAURI_PLATFORM == 'windows' ? 'chrome105' : 'safari13',
    // Don't minify for debug builds
    minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
    // Produce sourcemaps for debug builds
    sourcemap: !!process.env.TAURI_DEBUG,

    rollupOptions: {
      external: [
        'chromadb-default-embed',
        'hnswlib-node',
        '@tensorflow/tfjs-node',
        'cohere-ai',
        'openai',
        '@xenova/transformers',
        'sharp',
        'canvas',
        'pdf-parse'
      ],
      // Optimize chunks
      output: {
        manualChunks: {
          'react-vendor': ['react', 'react-dom', 'react-router-dom'],
          'redux-vendor': ['@reduxjs/toolkit', 'react-redux'],
          'terminal': ['@xterm/xterm', '@xterm/addon-fit', '@xterm/addon-web-links', '@xterm/addon-search', '@xterm/addon-canvas'],
          'ai-vendor': ['axios', 'marked', 'highlight.js'],
          'ui-vendor': ['@headlessui/react', '@heroicons/react'],
        },
      },
    },

    // Increase chunk size warning limit
    chunkSizeWarningLimit: 1000,
  },

  // CSS configuration
  css: {
    preprocessorOptions: {
      scss: {
        additionalData: `@import "@/styles/variables.scss";`
      }
    }
  },

  // Development options
  esbuild: {
    // Remove console logs and debugger statements in production (except error and warn)
    drop: process.env.NODE_ENV === 'production' ? ['console', 'debugger'] : [],
    // Keep error and warn console methods in production for debugging
    pure: process.env.NODE_ENV === 'production' ? ['console.log', 'console.info', 'console.debug'] : [],
  },

  // Environment variables
  envPrefix: ['VITE_', 'TAURI_'],

  // Optimizations
  optimizeDeps: {
    include: [
      'react',
      'react-dom',
      'react-router-dom',
      '@reduxjs/toolkit',
      'react-redux',
      '@xterm/xterm',
      '@xterm/addon-fit',
      '@xterm/addon-web-links',
      '@xterm/addon-search',
      '@xterm/addon-canvas',
      '@tauri-apps/api/core',
      '@tauri-apps/api/event',
      '@tauri-apps/api/window',
    ],
    exclude: ['@tauri-apps/api'],
  },
}))
