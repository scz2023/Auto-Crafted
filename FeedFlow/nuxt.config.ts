// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  compatibilityDate: '2024-04-03',
  devtools: { enabled: true },
  
  modules: [
    '@element-plus/nuxt',
    '@pinia/nuxt'
  ],

  elementPlus: {
    /** Options */
  },

  css: [
    'element-plus/dist/index.css',
    '~/assets/css/global.css'
  ],

  typescript: {
    strict: true
  },

  ssr: false, // Tauri 需要客户端渲染

  nitro: {
    prerender: {
      routes: ['/'],
      crawlLinks: false
    },
    // 确保服务器端 API 在 Tauri 中可用
    experimental: {
      wasm: true
    },
    // 在 Tauri 环境中，需要确保服务器端代码能运行
    storage: {
      fs: {
        driver: 'fs',
        base: './.nitro/storage'
      }
    }
  }
})

