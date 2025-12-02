import { useSettingsStore } from '~/stores/settings'
import { useFeedStore } from '~/stores/feed'

let refreshTimer: NodeJS.Timeout | null = null
let isRefreshing = false

export async function initAutoRefresh() {
  const settingsStore = useSettingsStore()
  const settings = await settingsStore.getSettings()

  // 启动时自动刷新
  if (settings.autoRefreshOnStart) {
    await refreshAllFeedsAuto()
  }

  // 设置定时刷新
  if (settings.autoRefreshInterval) {
    startAutoRefresh(settings.autoRefreshInterval)
  }
}

export function startAutoRefresh(intervalMinutes: number) {
  stopAutoRefresh()
  
  if (intervalMinutes <= 0) return

  const intervalMs = intervalMinutes * 60 * 1000
  refreshTimer = setInterval(async () => {
    if (!isRefreshing) {
      await refreshAllFeedsAuto()
    }
  }, intervalMs)
}

export function stopAutoRefresh() {
  if (refreshTimer) {
    clearInterval(refreshTimer)
    refreshTimer = null
  }
}

export async function refreshAllFeedsAuto() {
  if (isRefreshing) return

  isRefreshing = true
  try {
    const feedStore = useFeedStore()
    const feeds = await feedStore.getAllFeeds()
    
    const settingsStore = useSettingsStore()
    const settings = await settingsStore.getSettings()
    const concurrentRefresh = settings.concurrentRefresh || 3

    // 并发刷新
    const chunks: any[][] = []
    for (let i = 0; i < feeds.length; i += concurrentRefresh) {
      chunks.push(feeds.slice(i, i + concurrentRefresh))
    }

    for (const chunk of chunks) {
      await Promise.all(
        chunk.map(async (feed) => {
          try {
            await feedStore.refreshFeed(feed.id)
            // 在每个请求之间添加延迟，避免过于频繁的请求导致 429 错误
            // 延迟时间：500ms - 2s，随机分布
            const delay = 500 + Math.random() * 1500
            await new Promise(resolve => setTimeout(resolve, delay))
          } catch (error: any) {
            console.error(`刷新订阅 ${feed.title} 失败:`, error)
            // 如果是 429 错误，增加延迟时间
            if (error.message?.includes('429') || error.message?.includes('请求频率过高')) {
              console.warn(`订阅 ${feed.title} 遇到频率限制，等待 5 秒后继续`)
              await new Promise(resolve => setTimeout(resolve, 5000))
            }
          }
        })
      )
      // 在每批请求之间添加额外延迟
      if (chunks.indexOf(chunk) < chunks.length - 1) {
        await new Promise(resolve => setTimeout(resolve, 1000))
      }
    }
  } finally {
    isRefreshing = false
  }
}

export function updateAutoRefreshSettings() {
  const settingsStore = useSettingsStore()
  settingsStore.getSettings().then((settings) => {
    if (settings.autoRefreshInterval) {
      startAutoRefresh(settings.autoRefreshInterval)
    } else {
      stopAutoRefresh()
    }
  })
}

