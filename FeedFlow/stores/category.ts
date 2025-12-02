import { defineStore } from 'pinia'
import { useDatabase } from '~/composables/useDatabase'

export const useCategoryStore = defineStore('category', {
  state: () => ({
    categories: [] as any[]
  }),

  actions: {
    async getAllCategories() {
      const db = useDatabase()
      const stmt = db.prepare('SELECT * FROM categories ORDER BY name')
      return await stmt.all() as any[]
    },

    async getCategoryById(id: number) {
      const db = useDatabase()
      const stmt = db.prepare('SELECT * FROM categories WHERE id = ?')
      return await stmt.get(id) as any
    },

    async createCategory(name: string, description?: string) {
      const db = useDatabase()
      
      // 检查是否已存在
      const existingStmt = db.prepare('SELECT id FROM categories WHERE name = ?')
      const existing = await existingStmt.get(name)
      if (existing) {
        return (existing as any).id
      }

      const insert = db.prepare(`
        INSERT INTO categories (name, description)
        VALUES (?, ?)
      `)
      
      const result = await insert.run(name, description || '')
      return result.lastInsertRowid as number
    },

    async updateCategory(id: number, name: string, description?: string) {
      const db = useDatabase()
      const update = db.prepare(`
        UPDATE categories
        SET name = ?, description = ?
        WHERE id = ?
      `)
      await update.run(name, description || '', id)
    },

    async deleteCategory(id: number) {
      const db = useDatabase()
      // 先清除该分类下的订阅源分类
      const clearCategory = db.prepare('UPDATE feeds SET category_id = NULL WHERE category_id = ?')
      await clearCategory.run(id)
      
      // 删除分类
      const deleteStmt = db.prepare('DELETE FROM categories WHERE id = ?')
      await deleteStmt.run(id)
    }
  }
})

