import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { resolve } from 'path'

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST

// https://vite.dev/config/
export default defineConfig(async () => ({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src'),
    },
  },

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: 'ws',
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: ['**/src-tauri/**'],
    },
  },

  build: {
    target: 'esnext',
    minify: 'terser',
    chunkSizeWarningLimit: 5000,
    terserOptions: {
      compress: {
        drop_console: true,
        drop_debugger: true,
      },
    },
    rollupOptions: {
      output: {
        manualChunks(id) {
          // ECharts + vue-echarts
          if (id.includes('echarts') || id.includes('vue-echarts')) return 'echarts'

          // Vue ecosystem
          if (id.includes('node_modules/vue') || id.includes('node_modules/pinia') || id.includes('node_modules/@vue'))
            return 'vue-vendor'

          // Tauri plugins
          if (id.includes('@tauri-apps')) return 'tauri'

          // Other node_modules
          if (id.includes('node_modules')) return 'vendor'
        },
      },
    },
  },
}))
