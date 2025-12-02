export default defineEventHandler(async (event) => {
  const query = getQuery(event)
  const { url } = query

  if (!url || typeof url !== 'string') {
    throw createError({
      statusCode: 400,
      message: 'URL is required'
    })
  }

  try {
    // 验证 URL 格式
    const urlObj = new URL(url)
    if (!['http:', 'https:'].includes(urlObj.protocol)) {
      throw new Error('Only HTTP and HTTPS URLs are allowed')
    }

    // 构建 favicon URL
    const faviconUrl = `${urlObj.protocol}//${urlObj.host}/favicon.ico`

    // 尝试获取 favicon
    const response = await fetch(faviconUrl, {
      headers: {
        'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36'
      },
      // 设置较短的超时时间，避免长时间等待
      signal: AbortSignal.timeout(5000)
    })

    if (!response.ok) {
      // 如果 favicon.ico 不存在，返回 null 而不是错误
      if (response.status === 404 || response.status === 502) {
        return {
          success: false,
          favicon: null
        }
      }
      throw new Error(`HTTP ${response.status}: ${response.statusText}`)
    }

    // 检查内容类型
    const contentType = response.headers.get('content-type')
    if (!contentType || !contentType.startsWith('image/')) {
      return {
        success: false,
        favicon: null
      }
    }

    // 返回 favicon URL（让浏览器直接加载，而不是通过代理）
    // 这样可以减少服务器负担
    return {
      success: true,
      favicon: faviconUrl
    }
  } catch (error: any) {
    // 静默处理错误，不抛出异常
    // 这样前端可以优雅地处理失败情况
    if (error.name === 'AbortError' || error.message?.includes('timeout')) {
      return {
        success: false,
        favicon: null,
        error: 'timeout'
      }
    }
    
    return {
      success: false,
      favicon: null,
      error: error.message || 'unknown'
    }
  }
})

