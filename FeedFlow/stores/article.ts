import { defineStore } from 'pinia'
import { useDatabase } from '~/composables/useDatabase'

export const useArticleStore = defineStore('article', {
  state: () => ({
    articles: [] as any[]
  }),

  actions: {
    async getAllArticles(limit?: number, offset?: number, feedId?: number) {
      const db = useDatabase()
      
      // 构建查询语句
      let sql = `
        SELECT 
          a.*,
          f.title as feedName
        FROM articles a
        LEFT JOIN feeds f ON a.feed_id = f.id
      `
      // 注意：a.* 会自动包含所有 articles 表的字段，包括 ai_summary
      
      const params: any[] = []
      if (feedId !== undefined && feedId !== null) {
        sql += ` WHERE a.feed_id = ?`
        params.push(feedId)
      }
      
      const stmt = db.prepare(sql)
      const allArticles = await stmt.all(...params) as any[]
      
      // 按日期排序（将字符串日期转换为 Date 对象进行比较）
      // 这样可以正确处理各种日期格式（RFC 2822, ISO 8601 等）
      allArticles.sort((a, b) => {
        // 处理 pub_date
        let dateA = 0
        let dateB = 0
        
        if (a.pub_date) {
          const parsedA = new Date(a.pub_date)
          dateA = isNaN(parsedA.getTime()) ? 0 : parsedA.getTime()
        }
        
        if (b.pub_date) {
          const parsedB = new Date(b.pub_date)
          dateB = isNaN(parsedB.getTime()) ? 0 : parsedB.getTime()
        }
        
        // 按发布时间倒序排序（最新的在前）
        if (dateB !== dateA) {
          return dateB - dateA
        }
        
        // 如果发布时间相同，按创建时间倒序排序
        let createdA = 0
        let createdB = 0
        
        if (a.created_at) {
          const parsedA = new Date(a.created_at)
          createdA = isNaN(parsedA.getTime()) ? 0 : parsedA.getTime()
        }
        
        if (b.created_at) {
          const parsedB = new Date(b.created_at)
          createdB = isNaN(parsedB.getTime()) ? 0 : parsedB.getTime()
        }
        
        return createdB - createdA
      })
      
      // 应用分页
      if (limit !== undefined) {
        const start = offset || 0
        return allArticles.slice(start, start + limit)
      }
      
      return allArticles
    },

    async getTotalArticlesCount() {
      const db = useDatabase()
      const stmt = db.prepare('SELECT COUNT(*) as count FROM articles')
      const result = await stmt.get() as any
      return result.count || 0
    },

    async getArticlesByFeed(feedId: number, limit?: number, offset?: number) {
      return await this.getAllArticles(limit, offset, feedId)
    },

    async refreshAllFeeds() {
      const { refreshAllFeeds } = await import('~/composables/useRss')
      return await refreshAllFeeds()
    },

    async toggleFavorite(articleId: number) {
      const db = useDatabase()
      // 先获取当前状态
      const stmt = db.prepare('SELECT is_favorite FROM articles WHERE id = ?')
      const article = await stmt.get(articleId) as any
      
      if (!article) {
        throw new Error('文章不存在')
      }
      
      // 切换收藏状态
      const newFavorite = article.is_favorite ? 0 : 1
      const update = db.prepare('UPDATE articles SET is_favorite = ? WHERE id = ?')
      await update.run(newFavorite, articleId)
      
      return newFavorite === 1
    },

    async setFavorite(articleId: number, isFavorite: boolean) {
      const db = useDatabase()
      const update = db.prepare('UPDATE articles SET is_favorite = ? WHERE id = ?')
      await update.run(isFavorite ? 1 : 0, articleId)
    },

    async getFavoriteArticles(limit?: number, offset?: number) {
      const db = useDatabase()
      
      const sql = `
        SELECT 
          a.*,
          f.title as feedName
        FROM articles a
        LEFT JOIN feeds f ON a.feed_id = f.id
        WHERE a.is_favorite = 1
      `
      
      const stmt = db.prepare(sql)
      const allArticles = await stmt.all() as any[]
      
      // 按日期排序（将字符串日期转换为 Date 对象进行比较）
      allArticles.sort((a, b) => {
        let dateA = 0
        let dateB = 0
        
        if (a.pub_date) {
          const parsedA = new Date(a.pub_date)
          dateA = isNaN(parsedA.getTime()) ? 0 : parsedA.getTime()
        }
        
        if (b.pub_date) {
          const parsedB = new Date(b.pub_date)
          dateB = isNaN(parsedB.getTime()) ? 0 : parsedB.getTime()
        }
        
        // 按发布时间倒序排序（最新的在前）
        if (dateB !== dateA) {
          return dateB - dateA
        }
        
        // 如果发布时间相同，按创建时间倒序排序
        let createdA = 0
        let createdB = 0
        
        if (a.created_at) {
          const parsedA = new Date(a.created_at)
          createdA = isNaN(parsedA.getTime()) ? 0 : parsedA.getTime()
        }
        
        if (b.created_at) {
          const parsedB = new Date(b.created_at)
          createdB = isNaN(parsedB.getTime()) ? 0 : parsedB.getTime()
        }
        
        return createdB - createdA
      })
      
      // 应用分页
      if (limit !== undefined) {
        const start = offset || 0
        return allArticles.slice(start, start + limit)
      }
      
      return allArticles
    }
  }
})

