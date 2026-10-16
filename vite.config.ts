import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import tailwindcss from '@tailwindcss/vite'

const host = process.env.TAURI_DEV_HOST

// https://vite.dev/config/
export default defineConfig({
  plugins: [vue(), tailwindcss()],
  // Tauri używa własnego targetu, więc nie czyścimy ekranu i nie ukrywamy błędów.
  clearScreen: false,
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
      // `reference/` to materiał referencyjny MCC, `src-tauri/` buduje Cargo.
      ignored: ['**/src-tauri/**', '**/reference/**'],
    },
  },
})
