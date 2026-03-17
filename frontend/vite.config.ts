import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'
import tailwindcss from '@tailwindcss/vite'

export default defineConfig({
  plugins: [svelte(), tailwindcss()],
  server: {
    port: 1738,
    proxy: {
      '/api': {
        target: 'http://localhost:8008',
        ws: true,
      },
    },
  },
})
