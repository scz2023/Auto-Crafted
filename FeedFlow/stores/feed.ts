import { defineStore } from 'pinia'
import { useDatabase } from '~/composables/useDatabase'
import { parseFeed, refreshFeed as refreshFeedRss } from '~/composables/useRss'

export const useFeedStore = defineStore('feed', {
  state: () => ({
    feeds: [] as any[]
  }),

  actions: {
    async getAllFeeds() {
      const db = useDatabase()
      // 只返回已订阅的订阅源（用于全部文章显示）
      const stmt = db.prepare(`
        SELECT 
          f.*,
          c.name as categoryName
        FROM feeds f
        LEFT JOIN categories c ON f.category_id = c.id
        WHERE f.is_subscribed = 1
        ORDER BY c.name, f.title
      `)
      return await stmt.all() as any[]
    },

    async getAllFeedsForManagement(categoryId?: number | null, limit?: number, offset?: number) {
      const db = useDatabase()
      // 返回所有订阅源（用于管理页面）
      let sql = `
        SELECT 
          f.*,
          c.name as categoryName
        FROM feeds f
        LEFT JOIN categories c ON f.category_id = c.id
      `
      const params: any[] = []
      if (categoryId !== undefined && categoryId !== null) {
        sql += ` WHERE f.category_id = ?`
        params.push(categoryId)
      } else if (categoryId === null) {
        sql += ` WHERE f.category_id IS NULL`
      }
      sql += ` ORDER BY c.name, f.title`
      
      // 添加分页
      if (limit !== undefined) {
        sql += ` LIMIT ?`
        params.push(limit)
        if (offset !== undefined) {
          sql += ` OFFSET ?`
          params.push(offset)
        }
      }
      
      const stmt = db.prepare(sql)
      return await stmt.all(...params) as any[]
    },

    async getFeedsCount(categoryId?: number | null) {
      const db = useDatabase()
      let sql = `SELECT COUNT(*) as count FROM feeds`
      const params: any[] = []
      
      if (categoryId !== undefined && categoryId !== null) {
        sql += ` WHERE category_id = ?`
        params.push(categoryId)
      } else if (categoryId === null) {
        sql += ` WHERE category_id IS NULL`
      }
      
      const stmt = db.prepare(sql)
      const result = await stmt.get(...params) as any
      return result.count || 0
    },

    async toggleSubscribe(feedId: number) {
      const db = useDatabase()
      // 先获取当前状态
      const stmt = db.prepare('SELECT is_subscribed FROM feeds WHERE id = ?')
      const feed = await stmt.get(feedId) as any
      
      if (!feed) {
        throw new Error('订阅源不存在')
      }
      
      // 切换订阅状态
      const newSubscribe = feed.is_subscribed ? 0 : 1
      const update = db.prepare('UPDATE feeds SET is_subscribed = ? WHERE id = ?')
      await update.run(newSubscribe, feedId)
      
      return newSubscribe === 1
    },

    async addFeed(url: string, title?: string) {
      const db = useDatabase()
      
      // 检查是否已存在
      const existingStmt = db.prepare('SELECT id FROM feeds WHERE url = ?')
      const existing = await existingStmt.get(url)
      if (existing) {
        throw new Error('该订阅已存在')
      }

      // 获取超时设置
      const { useSettingsStore } = await import('~/stores/settings')
      const settingsStore = useSettingsStore()
      const settings = await settingsStore.getSettings()
      const timeout = settings.refreshTimeout ? settings.refreshTimeout * 1000 : undefined

      // 解析 RSS
      const feedData = await parseFeed(url, timeout)
      
      // 尝试获取favicon
      let favicon = null
      try {
        const feedLink = feedData.link || url
        if (feedLink) {
          const feedUrl = new URL(feedLink)
          favicon = `${feedUrl.protocol}//${feedUrl.host}/favicon.ico`
        }
      } catch {
        // 忽略favicon获取错误
      }
      
      const insert = db.prepare(`
        INSERT INTO feeds (title, url, description, favicon, is_subscribed, last_update)
        VALUES (?, ?, ?, ?, ?, ?)
      `)
      
      const result = await insert.run(
        title || feedData.title || '未命名订阅',
        url,
        feedData.description || '',
        favicon,
        1, // 新添加的 RSS 自动订阅
        new Date().toISOString()
      )
      
      const feedId = result.lastInsertRowid as number

      // 保存文章
      await this.saveArticles(feedId, feedData.items)

      return feedId
    },

    async addFeedWithCategory(url: string, title?: string, categoryId?: number | null, description?: string, skipRssParse = false) {
      const db = useDatabase()
      
      // 检查是否已存在
      const existingStmt = db.prepare('SELECT id FROM feeds WHERE url = ?')
      const existing = await existingStmt.get(url)
      if (existing) {
        // 如果已存在，更新分类
        if (categoryId !== undefined && categoryId !== null) {
          const update = db.prepare('UPDATE feeds SET category_id = ? WHERE id = ?')
          await update.run(categoryId, (existing as any).id)
        }
        return (existing as any).id
      }

      let feedData: any = null
      let parseError: Error | null = null

      // 尝试解析 RSS（除非明确跳过）
      if (!skipRssParse) {
        try {
          // 获取超时设置
          const { useSettingsStore } = await import('~/stores/settings')
          const settingsStore = useSettingsStore()
          const settings = await settingsStore.getSettings()
          const timeout = settings.refreshTimeout ? settings.refreshTimeout * 1000 : undefined

          // 解析 RSS
          feedData = await parseFeed(url, timeout)
        } catch (error: any) {
          // 如果解析失败，记录错误但继续保存订阅源
          parseError = error
          console.warn(`RSS 解析失败，将仅保存订阅源信息: ${url}`, error.message)
        }
      }
      
      // 尝试获取favicon
      let favicon = null
      try {
        const feedLink = feedData?.link || url
        if (feedLink) {
          const feedUrl = new URL(feedLink)
          favicon = `${feedUrl.protocol}//${feedUrl.host}/favicon.ico`
        }
      } catch {
        // 忽略favicon获取错误
      }
      
      const insert = db.prepare(`
        INSERT INTO feeds (title, url, description, favicon, category_id, is_subscribed, last_update)
        VALUES (?, ?, ?, ?, ?, ?, ?)
      `)
      
      const result = await insert.run(
        title || feedData?.title || '未命名订阅',
        url,
        description || feedData?.description || '',
        favicon,
        categoryId || null,
        1, // 新添加的 RSS 自动订阅
        new Date().toISOString()
      )
      
      const feedId = result.lastInsertRowid as number
      
      // 验证分类是否正确关联
      if (categoryId) {
        const verifyStmt = db.prepare('SELECT category_id FROM feeds WHERE id = ?')
        const verify = await verifyStmt.get(feedId) as any
        if (verify.category_id !== categoryId) {
          console.warn(`警告: 订阅源 ${feedId} 的分类 ID 不匹配。期望: ${categoryId}, 实际: ${verify.category_id}`)
        } else {
          console.log(`✓ 订阅源 ${feedId} 已正确关联到分类 ${categoryId}`)
        }
      }
      
      // 如果解析成功，保存文章
      if (feedData && feedData.items) {
        try {
          await this.saveArticles(feedId, feedData.items)
          
          // 更新最后更新时间
          const updateStmt = db.prepare('UPDATE feeds SET last_update = ? WHERE id = ?')
          await updateStmt.run(new Date().toISOString(), feedId)
        } catch (error) {
          console.warn(`保存文章失败 (feedId: ${feedId}):`, error)
        }
      }
      
      // 如果解析失败，抛出错误以便调用者知道
      if (parseError) {
        throw parseError
      }
      
      return feedId
    },

    async updateFeed(id: number, data: any) {
      const db = useDatabase()
      const update = db.prepare(`
        UPDATE feeds
        SET title = ?, description = ?, category_id = ?
        WHERE id = ?
      `)
      await update.run(
        data.title,
        data.description || '',
        data.category_id !== undefined ? data.category_id : null,
        id
      )
    },

    async deleteFeed(id: number) {
      const db = useDatabase()
      const deleteFeed = db.prepare('DELETE FROM feeds WHERE id = ?')
      const deleteArticles = db.prepare('DELETE FROM articles WHERE feed_id = ?')
      
      await deleteArticles.run(id)
      await deleteFeed.run(id)
    },

    async refreshFeed(id: number) {
      const db = useDatabase()
      const stmt = db.prepare('SELECT * FROM feeds WHERE id = ?')
      const feed = await stmt.get(id) as any
      
      if (!feed) {
        throw new Error('订阅不存在')
      }

      // 获取超时设置
      const { useSettingsStore } = await import('~/stores/settings')
      const settingsStore = useSettingsStore()
      const settings = await settingsStore.getSettings()
      const timeout = settings.refreshTimeout ? settings.refreshTimeout * 1000 : undefined
      
      const feedData = await refreshFeedRss(feed.url, timeout)
      
      // 更新订阅信息
      const update = db.prepare(`
        UPDATE feeds
        SET title = ?, description = ?, last_update = ?
        WHERE id = ?
      `)
      await update.run(
        feedData.title || feed.title,
        feedData.description || feed.description || '',
        new Date().toISOString(),
        id
      )

      // 保存新文章
      await this.saveArticles(id, feedData.items)
    },

    async saveArticles(feedId: number, items: any[]) {
      const db = useDatabase()
      const insert = db.prepare(`
        INSERT OR IGNORE INTO articles (feed_id, title, link, content, snippet, pub_date, guid)
        VALUES (?, ?, ?, ?, ?, ?, ?)
      `)

      for (const item of items) {
        const content = item.content || item.contentSnippet || item.summary || ''
        const snippet = item.contentSnippet || item.summary || content.substring(0, 200)
        const guid = item.guid || item.id || item.link || ''
        
        await insert.run(
          feedId,
          item.title || '无标题',
          item.link || '',
          content,
          snippet,
          item.pubDate || item.isoDate || new Date().toISOString(),
          guid
        )
      }

      // 检查并清理超出限制的文章
      await this.cleanupOldArticles()
    },

    async cleanupOldArticles() {
      const db = useDatabase()
      const { useSettingsStore } = await import('~/stores/settings')
      const settingsStore = useSettingsStore()
      const settings = await settingsStore.getSettings()
      const maxArticles = settings.maxArticles || 1000

      // 获取当前文章总数
      const countResult = db.prepare('SELECT COUNT(*) as count FROM articles').get() as any
      const currentCount = countResult.count

      if (currentCount > maxArticles) {
        // 删除最旧的文章，保留最新的 maxArticles 篇
        const deleteStmt = db.prepare(`
          DELETE FROM articles
          WHERE id NOT IN (
            SELECT id FROM articles
            ORDER BY pub_date DESC, created_at DESC
            LIMIT ?
          )
        `)
        await deleteStmt.run(maxArticles)
      }
    }
  }
})

