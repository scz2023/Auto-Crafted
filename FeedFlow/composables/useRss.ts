/**
 * 转换 RSSHub 协议 URL 为标准 HTTP URL
 * 支持的格式：
 * - rsshub://category/subcategory -> https://rsshub.app/category/subcategory
 * - rsshub://custom.domain.com/category/subcategory -> https://custom.domain.com/category/subcategory
 * - rsshub:category/subcategory -> https://rsshub.app/category/subcategory
 */
function normalizeRssUrl(url: string): string {
  // 处理 rsshub:// 协议
  if (url.startsWith('rsshub://')) {
    // 移除协议前缀
    const path = url.replace(/^rsshub:\/\//, '')
    
    // 检查是否包含自定义域名（包含点号且不是路径的一部分）
    // 例如: rsshub://custom.domain.com/path 或 rsshub://localhost:1200/path
    const domainMatch = path.match(/^([^\/]+?)(\/.*)?$/)
    
    if (domainMatch) {
      const potentialDomain = domainMatch[1]
      const restPath = domainMatch[2] || ''
      
      // 检查是否是有效的域名格式（包含点号或 localhost）
      if (potentialDomain.includes('.') || potentialDomain.startsWith('localhost')) {
        // 自定义域名，直接使用
        return `https://${potentialDomain}${restPath}`
      } else {
        // 不是域名，是路径，使用默认的 rsshub.app
        return `https://rsshub.app/${path}`
      }
    } else {
      // 空路径或无效格式，使用默认域名
      return `https://rsshub.app/${path}`
    }
  }
  
  // 处理 rsshub: 协议（不带双斜杠）
  if (url.startsWith('rsshub:')) {
    // 移除协议前缀
    let path = url.replace(/^rsshub:/, '')
    // 移除可能的前导斜杠
    path = path.replace(/^\/+/, '')
    
    // 检查是否包含自定义域名
    const domainMatch = path.match(/^([^\/]+?)(\/.*)?$/)
    
    if (domainMatch) {
      const potentialDomain = domainMatch[1]
      const restPath = domainMatch[2] || ''
      
      // 检查是否是有效的域名格式
      if (potentialDomain.includes('.') || potentialDomain.startsWith('localhost')) {
        return `https://${potentialDomain}${restPath}`
      } else {
        return `https://rsshub.app/${path}`
      }
    } else {
      return `https://rsshub.app/${path}`
    }
  }
  
  return url
}

// 客户端 RSS 解析 - 通过服务器 API
export async function parseFeed(url: string, timeout?: number) {
  try {
    // 在 Tauri 环境中，确保服务器已启动
    const { ensureServerReady } = await import('./useServerReady')
    await ensureServerReady()
    
    // 标准化 URL（转换 rsshub:// 协议等）
    const normalizedUrl = normalizeRssUrl(url)
    
    const response = await $fetch('/api/rss/parse', {
      method: 'POST',
      body: { url: normalizedUrl, timeout }
    })
    return (response as any).data
  } catch (error: any) {
    // 检查是否是 429 错误
    const is429 = error.status === 429 || 
                  error.statusCode === 429 ||
                  error.data?.statusCode === 429 ||
                  error.message?.includes('429') ||
                  error.message?.includes('请求频率过高')
    
    if (is429) {
      throw new Error('请求频率过高，服务器暂时限制访问。建议：\n1. 增加刷新间隔时间\n2. 减少并发刷新数量\n3. 稍后再试')
    }
    
    throw new Error(error.data?.message || error.message || '解析 RSS 失败')
  }
}

export async function refreshFeed(url: string, timeout?: number) {
  return await parseFeed(url, timeout)
}

export async function refreshAllFeeds() {
  const { useFeedStore } = await import('~/stores/feed')
  const feedStore = useFeedStore()
  const feeds = await feedStore.getAllFeeds()
  
  const results = []
  for (const feed of feeds) {
    try {
      await feedStore.refreshFeed(feed.id)
      results.push({ id: feed.id, success: true })
    } catch (error: any) {
      results.push({ id: feed.id, success: false, error: error.message })
    }
  }
  
  return results
}

