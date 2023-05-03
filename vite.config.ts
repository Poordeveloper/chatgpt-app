import path from 'node:path'
import type { PluginOption } from 'vite'
import { defineConfig, loadEnv } from 'vite'
import vue from '@vitejs/plugin-vue'
import { VitePWA } from 'vite-plugin-pwa'

function setupPlugins(env: ImportMetaEnv): PluginOption[] {
  return [
    vue(),
    env.VITE_GLOB_APP_PWA === 'true'
      && VitePWA({
        injectRegister: 'auto',
        manifest: {
          name: 'chatGPT',
          short_name: 'chatGPT',
          icons: [
            { src: 'pwa-192x192.png', sizes: '192x192', type: 'image/png' },
            { src: 'pwa-512x512.png', sizes: '512x512', type: 'image/png' },
          ],
        },
      }),
  ]
}

// https://vitejs.dev/config/
export default defineConfig(async (env) => {
  const { internalIpV4 } = await import('internal-ip')
  const viteEnv = loadEnv(env.mode, process.cwd()) as unknown as ImportMetaEnv
  const host = await internalIpV4()
  return {
    resolve: {
      alias: {
        '@': path.resolve(process.cwd(), 'src'),
      },
    },
    plugins: setupPlugins(viteEnv),

    // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
    // prevent vite from obscuring rust errors
    clearScreen: false,
    // tauri expects a fixed port, fail if that port is not available
    server: {
      host: '0.0.0.0', // listen on all addresses
      port: 1420,
      strictPort: true,
      /*
      hmr: {
        protocol: 'ws',
        host,
        port: 5183,
      },
      */
      proxy: {
        '/api': {
          target: 'http://localhost:8080/',
          changeOrigin: true,
          // rewrite: path => path.replace('/api/', '/'),
        },
      },
    },
    // to make use of `TAURI_DEBUG` and other env variables
    // https://tauri.studio/v1/api/config#buildconfig.beforedevcommand
    envPrefix: ['VITE_', 'TAURI_'],
    build: {
      // Tauri supports es2021
      target:
        process.env.TAURI_PLATFORM === 'windows' ? 'chrome105' : 'safari13',
      // don't minify for debug builds
      minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
      // produce sourcemaps for debug builds
      sourcemap: !!process.env.TAURI_DEBUG,
    },
  }
})
