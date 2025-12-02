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
    }
  }
})

