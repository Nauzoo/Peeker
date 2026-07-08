import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'
import tailwindcss from '@tailwindcss/vite'

// https://vite.dev/config/
export default defineConfig({
  server: {
    proxy: {
      '/api' : {
        target: 'http://localhost:3000',
        changeOrigin: true
      }
    }
  },
  plugins: [
    svelte(), tailwindcss()
  ],
})
