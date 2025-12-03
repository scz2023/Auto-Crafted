import { defineStore } from 'pinia'
import { useDatabase } from '~/composables/useDatabase'
import { parseFeed, refreshFeed as refreshFeedRss } from '~/composables/useRss'

export const useFeedStore = defineStore('feed', {
  state: () => ({
    feeds: [] as any[]
  }),

  actions: {
    // 仅返回已订阅的订阅源（用于阅读侧边栏等「已订阅」场景）
    async getAllFeeds() {
      const db = useDatabase()
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

    // 返回所有订阅源（包含未订阅），用于管理页面
    async getAllFeedsWithStatus() {
      const db = useDatabase()
      const stmt = db.prepare(`
        SELECT 
          f.*,
          c.name as categoryName
        FROM feeds f
        LEFT JOIN categories c ON f.category_id = c.id
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
      let feedData: any = null
      try {
        feedData = await parseFeed(url, timeout)
        // 确保 feedData 是对象
        if (!feedData || typeof feedData !== 'object') {
          console.warn('RSS 解析返回无效数据，将使用默认值')
          feedData = null
        }
      } catch (error: any) {
        console.warn(`RSS 解析失败，将仅保存订阅源信息: ${url}`, error.message)
        feedData = null
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
        INSERT INTO feeds (title, url, description, favicon, is_subscribed, last_update)
        VALUES (?, ?, ?, ?, ?, ?)
      `)
      
      const result = await insert.run(
        title || feedData?.title || '未命名订阅',
        url,
        feedData?.description || '',
        favicon,
        1, // 新添加的 RSS 自动订阅
        new Date().toISOString()
      )
      
      const feedId = result.lastInsertRowid as number

      // 保存文章（如果解析成功且有文章）
      if (feedData && feedData.items && Array.isArray(feedData.items)) {
        try {
          await this.saveArticles(feedId, feedData.items)
        } catch (error) {
          console.warn(`保存文章失败 (feedId: ${feedId}):`, error)
        }
      }

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

    // 从 OPML 导入订阅源：只创建记录，不自动订阅、不解析 RSS
    async importFeedFromOpml(url: string, title?: string, categoryId?: number | null, description?: string) {
      const db = useDatabase()

      // 检查是否已存在
      const existingStmt = db.prepare('SELECT id, is_subscribed FROM feeds WHERE url = ?')
      const existing = await existingStmt.get(url) as any
      if (existing) {
        // 如果已存在，只在未订阅且给了分类的情况下更新分类
        if (!existing.is_subscribed && categoryId !== undefined && categoryId !== null) {
          const update = db.prepare('UPDATE feeds SET category_id = ? WHERE id = ?')
          await update.run(categoryId, existing.id)
        }
        return existing.id as number
      }

      // 尝试获取 favicon（仅根据 URL 推断，不请求网络）
      let favicon: string | null = null
      try {
        const feedUrl = new URL(url)
        favicon = `${feedUrl.protocol}//${feedUrl.host}/favicon.ico`
      } catch {
        // 忽略 favicon 获取错误
      }

      const insert = db.prepare(`
        INSERT INTO feeds (title, url, description, favicon, category_id, is_subscribed, last_update)
        VALUES (?, ?, ?, ?, ?, ?, ?)
      `)

      const result = await insert.run(
        title || '未命名订阅',
        url,
        description || '',
        favicon,
        categoryId || null,
        0, // 从 OPML 导入的默认未订阅
        null // 暂无最后更新时间
      )

      return result.lastInsertRowid as number
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
      
      let feedData: any = null
      try {
        feedData = await refreshFeedRss(feed.url, timeout)
        // 确保 feedData 是对象
        if (!feedData || typeof feedData !== 'object') {
          console.warn('RSS 刷新返回无效数据')
          feedData = null
        }
      } catch (error: any) {
        console.error(`刷新订阅失败 (id: ${id}):`, error)
        throw new Error(error.message || '刷新订阅失败')
      }
      
      if (!feedData) {
        throw new Error('无法获取订阅数据')
      }
      
      // 更新订阅信息
      const update = db.prepare(`
        UPDATE feeds
        SET title = ?, description = ?, last_update = ?
        WHERE id = ?
      `)
      await update.run(
        feedData?.title || feed.title,
        feedData?.description || feed.description || '',
        new Date().toISOString(),
        id
      )

      // 保存新文章（如果存在）
      if (feedData?.items && Array.isArray(feedData.items)) {
        try {
          await this.saveArticles(id, feedData.items)
        } catch (error) {
          console.warn(`保存文章失败 (feedId: ${id}):`, error)
        }
      }
    },

    async saveArticles(feedId: number, items: any[]) {
      if (!items || !Array.isArray(items) || items.length === 0) {
        console.warn(`[saveArticles] 没有文章需要保存 (feedId: ${feedId})`)
        return
      }

      console.log(`[saveArticles] 开始保存文章 (feedId: ${feedId}, 数量: ${items.length})`)
      
      const db = useDatabase()
      const insert = db.prepare(`
        INSERT OR IGNORE INTO articles (feed_id, title, link, content, snippet, pub_date, guid)
        VALUES (?, ?, ?, ?, ?, ?, ?)
      `)

      let savedCount = 0
      let skippedCount = 0
      let errorCount = 0

      for (const item of items) {
        try {
          if (!item) {
            console.warn('[saveArticles] 跳过空项目')
            skippedCount++
            continue
          }

          const content = item.content || item.contentSnippet || item.summary || ''
          const snippet = item.contentSnippet || item.summary || content.substring(0, 200)
          const guid = item.guid || item.id || item.link || ''
          
          // 如果没有 guid 和 link，跳过这篇文章（无法唯一标识）
          if (!guid && !item.link) {
            console.warn('[saveArticles] 跳过没有 guid 和 link 的文章:', item.title || '无标题')
            skippedCount++
            continue
          }
          
          const result = await insert.run(
            feedId,
            item.title || '无标题',
            item.link || '',
            content,
            snippet,
            item.pubDate || item.isoDate || new Date().toISOString(),
            guid
          )
          
          // INSERT OR IGNORE 不会返回 changes，所以无法直接判断是否插入成功
          // 但如果没有错误，就认为成功了
          savedCount++
        } catch (error: any) {
          errorCount++
          console.error(`[saveArticles] 保存文章失败:`, { 
            feedId, 
            itemTitle: item?.title || '无标题',
            error: error.message || error.toString() 
          })
          // 继续处理下一篇文章，不中断整个流程
        }
      }

      console.log(`[saveArticles] 保存完成 (feedId: ${feedId}): 成功 ${savedCount}, 跳过 ${skippedCount}, 错误 ${errorCount}`)

      // 检查并清理超出限制的文章
      try {
        await this.cleanupOldArticles()
      } catch (error) {
        console.warn('[saveArticles] 清理旧文章失败:', error)
      }
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

