import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'
import tailwindcss from '@tailwindcss/vite'

export default defineConfig({
  plugins: [svelte(), tailwindcss()],
  server: {
    port: 3817,
    proxy: {
      '/api': {
        target: 'http://localhost:8337',
        ws: true,
      },
    },
  },
})
