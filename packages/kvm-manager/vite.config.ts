import { defineConfig } from "vite";
import reactRefresh from "@vitejs/plugin-react-refresh";
import react from "@vitejs/plugin-react";
import { resolve } from "path";

// https://vitejs.dev/config/
export default defineConfig(async () => ({
 plugins: [reactRefresh(), react()],

 // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
 clearScreen: false,
 server: {
 port: 3000,
 strictPort: true,
 watch: {
 ignored: ["**/src-tauri/**"],
 },
 },
 build: {
 rollupOptions: {
 output: {
 manualChunks: {
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
 },

 chunkSizeWarningLimit: 1000, // Increase chunk size warning limit to 1000kb (from default 500kb)
 sourcemap: true, // Enable source maps for better debugging
 },

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

 resolve: {
 alias: {
 '@': resolve(__dirname, './src'),
 },
 },
}));