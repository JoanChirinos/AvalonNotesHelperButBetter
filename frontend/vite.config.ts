import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'
import tailwindcss from '@tailwindcss/vite'

const basePath = process.env.VITE_BASE_PATH || '/'

export default defineConfig({
  base: basePath,
  plugins: [svelte(), tailwindcss()],
  server: {
    port: 3817,
    proxy: {
      [`${basePath}api`]: {
        target: 'http://localhost:8337',
        rewrite: (path) => path.replace(new RegExp(`^${basePath}`), '/'),
        ws: true,
      },
    },
  },
})
