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
    console.log('[parseFeed] 开始解析 RSS:', { url, timeout })
    
    // 在 Tauri 环境中，确保服务器已启动
    const { ensureServerReady } = await import('./useServerReady')
    try {
      await ensureServerReady()
      console.log('[parseFeed] 服务器已就绪')
    } catch (serverError: any) {
      console.error('[parseFeed] 服务器启动检查失败:', serverError)
      throw new Error(`服务器未就绪: ${serverError.message || '请检查 Node.js 是否已安装'}`)
    }
    
    // 标准化 URL（转换 rsshub:// 协议等）
    const normalizedUrl = normalizeRssUrl(url)
    console.log('[parseFeed] 标准化后的 URL:', normalizedUrl)
    
    // 如果没有提供 timeout，从数据库读取设置
    let timeoutValue = timeout
    if (!timeoutValue) {
      try {
        const { useSettingsStore } = await import('~/stores/settings')
        const settingsStore = useSettingsStore()
        const settings = await settingsStore.getSettings()
        
        if (settings.refreshTimeout !== undefined) {
          const timeoutSeconds = typeof settings.refreshTimeout === 'string'
            ? JSON.parse(settings.refreshTimeout)
            : settings.refreshTimeout
          timeoutValue = timeoutSeconds * 1000
        } else {
          timeoutValue = 30000 // 默认30秒
        }
      } catch (settingsError) {
        console.warn('[parseFeed] 读取设置失败，使用默认超时:', settingsError)
        timeoutValue = 30000 // 默认30秒
      }
    }
    
    console.log('[parseFeed] 调用 API /api/rss/parse:', { url: normalizedUrl, timeout: timeoutValue })
    
    const response = await $fetch('/api/rss/parse', {
      method: 'POST',
      body: { url: normalizedUrl, timeout: timeoutValue }
    }) as any
    
    console.log('[parseFeed] API 响应:', { 
      hasResponse: !!response, 
      success: response?.success,
      hasData: !!response?.data,
      dataType: typeof response?.data,
      itemsCount: Array.isArray(response?.data?.items) ? response?.data.items.length : 'N/A'
    })
    
    // 确保返回的数据格式正确
    if (!response) {
      throw new Error('API 返回空响应')
    }
    
    if (response.success === false) {
      throw new Error(response.message || '解析 RSS 失败')
    }
    
    const data = response.data
    if (!data || typeof data !== 'object') {
      console.error('[parseFeed] 数据格式无效:', { data, type: typeof data })
      throw new Error('API 返回的数据格式无效')
    }
    
    // 确保 items 是数组
    if (!Array.isArray(data.items)) {
      console.warn('[parseFeed] items 不是数组，转换为数组:', data.items)
      data.items = data.items ? [data.items] : []
    }
    
    console.log('[parseFeed] 解析成功:', { 
      title: data.title, 
      itemsCount: data.items.length 
    })
    
    return data
  } catch (error: any) {
    console.error('[parseFeed] RSS 解析失败:', { 
      url, 
      error: error.message || error.toString(),
      status: error.status || error.statusCode,
      data: error.data 
    })
    
    // 检查是否是 429 错误
    const is429 = error.status === 429 || 
                  error.statusCode === 429 ||
                  error.data?.statusCode === 429 ||
                  error.message?.includes('429') ||
                  error.message?.includes('请求频率过高') ||
                  error.message?.includes('Too Many Requests')
    
    if (is429) {
      throw new Error('请求频率过高，服务器暂时限制访问。建议：\n1. 增加刷新间隔时间\n2. 减少并发刷新数量\n3. 稍后再试')
    }
    
    // 检查是否是网络错误
    const isNetworkError = error.message?.includes('fetch') || 
                          error.message?.includes('network') ||
                          error.status === 0 ||
                          error.statusCode === 0 ||
                          error.name === 'NetworkError'
    
    if (isNetworkError) {
      throw new Error(`网络错误: ${error.message || '无法连接到服务器。请检查网络连接或服务器是否已启动。'}`)
    }
    
    // 检查是否是超时错误
    const isTimeout = error.message?.includes('timeout') || 
                     error.message?.includes('超时') ||
                     error.name === 'TimeoutError'
    
    if (isTimeout) {
      throw new Error(`请求超时: ${error.message || '服务器响应时间过长。请检查网络连接或稍后重试。'}`)
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

