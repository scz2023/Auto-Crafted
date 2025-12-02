import { defineStore } from 'pinia'
import { useDatabase } from '~/composables/useDatabase'

export const useSettingsStore = defineStore('settings', {
  state: () => ({
    settings: {} as any
  }),

  actions: {
    async getSettings() {
      const db = useDatabase()
      const stmt = db.prepare('SELECT key, value FROM settings')
      const rows = await stmt.all() as any[]
      const settings: any = {}
      
      for (const row of rows) {
        try {
          settings[row.key] = JSON.parse(row.value)
        } catch {
          settings[row.key] = row.value
        }
      }
      
      return settings
    },

    async saveSettings(settings: any) {
      const db = useDatabase()
      const insert = db.prepare(`
        INSERT OR REPLACE INTO settings (key, value)
        VALUES (?, ?)
      `)

      for (const [key, value] of Object.entries(settings)) {
        const valueStr = typeof value === 'string' ? value : JSON.stringify(value)
        await insert.run(key, valueStr)
      }
    }
  }
})

