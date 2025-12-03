import Parser from 'rss-parser'

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

export default defineEventHandler(async (event) => {
  const body = await readBody(event)
  let { url, timeout } = body

  if (!url) {
    throw createError({
      statusCode: 400,
      message: 'RSS URL is required'
    })
  }

  // 标准化 URL（转换 rsshub:// 协议等）
  url = normalizeRssUrl(url)

  // 获取超时设置（秒转毫秒）
  // 如果客户端没有提供 timeout，使用默认值
  // 客户端应该已经从数据库读取设置并传递过来
  let timeoutValue = timeout || 30000 // 默认30秒

  const parser = new Parser({
    timeout: timeoutValue,
    headers: {
      'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36',
      'Accept': 'application/rss+xml, application/xml, text/xml, */*',
      'Accept-Language': 'en-US,en;q=0.9',
      'Accept-Encoding': 'gzip, deflate, br',
      'Connection': 'keep-alive',
      'Upgrade-Insecure-Requests': '1'
    },
    // 自定义解析器选项，更宽松的解析
    customFields: {
      item: ['media:content', 'media:thumbnail']
    }
  })

  // 重试函数，带指数退避
  const retryWithBackoff = async (fn: () => Promise<any>, maxRetries = 3, baseDelay = 1000): Promise<any> => {
    for (let attempt = 0; attempt < maxRetries; attempt++) {
      try {
        return await fn()
      } catch (error: any) {
        // 检查是否是 429 错误（多种可能的错误格式）
        const is429 = error.status === 429 || 
                     error.statusCode === 429 ||
                     error.code === 429 ||
                     error.message?.includes('429') || 
                     error.message?.includes('Too Many Requests') ||
                     error.message?.includes('rate limit') ||
                     error.message?.includes('Rate limit')
        
        if (is429 && attempt < maxRetries - 1) {
          // 尝试从错误中获取 Retry-After 信息
          let retryAfter = baseDelay * Math.pow(2, attempt) // 指数退避：1s, 2s, 4s
          
          // 如果错误对象中有 retryAfter 信息，使用它
          if (error.retryAfter) {
            retryAfter = error.retryAfter * 1000 // 转换为毫秒
          }
          
          // 限制最大等待时间为 60 秒
          retryAfter = Math.min(retryAfter, 60000)
          
          console.log(`遇到 429 错误，等待 ${retryAfter}ms 后重试 (尝试 ${attempt + 1}/${maxRetries})`)
          await new Promise(resolve => setTimeout(resolve, retryAfter))
          continue
        }
        
        // 如果不是 429 错误，或者已经达到最大重试次数，抛出错误
        throw error
      }
    }
  }

  try {
    // 先尝试直接解析（带重试机制）
    const feed = await retryWithBackoff(async () => {
      try {
        return await parser.parseURL(url)
      } catch (parseError: any) {
        // 检查 rss-parser 返回的错误中是否包含 429
        if (parseError.code === 'ECONNRESET' || 
            parseError.message?.includes('429') || 
            parseError.message?.includes('Too Many Requests') ||
            parseError.statusCode === 429) {
          const error: any = new Error(`HTTP 429: Too Many Requests`)
          error.status = 429
          error.statusCode = 429
          throw error
        }
        throw parseError
      }
    })
    
    return {
      success: true,
      data: {
        title: feed.title || '',
        description: feed.description || '',
        link: feed.link || '',
        items: feed.items || []
      }
    }
  } catch (error: any) {
    // 如果解析失败，尝试手动获取并清理内容
    try {
      const response = await retryWithBackoff(async () => {
        const res = await fetch(url, {
          headers: {
            'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36',
            'Accept': 'application/rss+xml, application/xml, text/xml, */*'
          },
          signal: AbortSignal.timeout(timeoutValue)
        })

        // 检查 429 错误
        if (res.status === 429) {
          // 尝试从响应头获取 Retry-After
          const retryAfter = res.headers.get('Retry-After')
          const error: any = new Error(`HTTP 429: Too Many Requests`)
          error.status = 429
          if (retryAfter) {
            error.retryAfter = parseInt(retryAfter, 10)
          }
          throw error
        }

        if (!res.ok) {
          throw new Error(`HTTP ${res.status}: ${res.statusText}`)
        }

        // 检查 Content-Type，确保是 XML 或文本类型
        const contentType = res.headers.get('content-type') || ''
        if (!contentType.includes('xml') && 
            !contentType.includes('text') && 
            !contentType.includes('application/rss') &&
            !contentType.includes('application/atom')) {
          console.warn(`警告: 响应的 Content-Type 不是 XML 类型: ${contentType}`)
        }

        return res
      })

      let text = await response.text()
      
      // 检查响应是否为空
      if (!text || text.length === 0) {
        throw new Error('RSS 响应为空')
      }
      
      // 更彻底地清理可能的 BOM 和非 XML 字符
      // 1. 移除所有类型的 BOM
      // UTF-8 BOM: EF BB BF
      if (text.charCodeAt(0) === 0xFEFF) {
        text = text.slice(1)
      }
      // UTF-16 BE BOM: FE FF
      if (text.length >= 2 && text.charCodeAt(0) === 0xFE && text.charCodeAt(1) === 0xFF) {
        text = text.slice(2)
      }
      // UTF-16 LE BOM: FF FE
      if (text.length >= 2 && text.charCodeAt(0) === 0xFF && text.charCodeAt(1) === 0xFE) {
        text = text.slice(2)
      }
      
      // 2. 移除所有控制字符和不可见字符（保留换行符和制表符）
      text = text.replace(/[\x00-\x08\x0B\x0C\x0E-\x1F\x7F]/g, '')
      
      // 3. 移除 XML 声明之前的所有字符（包括空白字符）
      const xmlDeclarationMatch = text.match(/<\?xml[^>]*>/i)
      if (xmlDeclarationMatch) {
        const xmlStartIndex = text.indexOf(xmlDeclarationMatch[0])
        if (xmlStartIndex > 0) {
          text = text.substring(xmlStartIndex)
        }
      }
      
      // 4. 如果还没有找到 XML 声明，尝试查找第一个标签
      if (!text.startsWith('<?xml') && !text.startsWith('<rss') && !text.startsWith('<feed') && !text.startsWith('<RDF')) {
        // 尝试查找第一个 XML 标签（包括注释）
        const firstTagMatch = text.match(/<[?!]?[a-zA-Z][^>]*>/)
        if (firstTagMatch) {
          const tagStartIndex = text.indexOf(firstTagMatch[0])
          if (tagStartIndex > 0) {
            text = text.substring(tagStartIndex)
          }
        } else {
          // 如果找不到任何标签，尝试移除所有前导空白字符后再查找
          const trimmedText = text.trimStart()
          const trimmedTagMatch = trimmedText.match(/<[?!]?[a-zA-Z][^>]*>/)
          if (trimmedTagMatch) {
            text = trimmedText
          } else {
            throw new Error('响应不是有效的 RSS/XML 格式：找不到 XML 标签')
          }
        }
      }
      
      // 5. 确保文本以有效的 XML 开始
      text = text.trimStart()
      
      // 6. 验证是否是有效的 XML/RSS 格式
      if (!text.startsWith('<?xml') && 
          !text.startsWith('<rss') && 
          !text.startsWith('<feed') && 
          !text.startsWith('<RDF') &&
          !text.startsWith('<!--')) {
        throw new Error('响应不是有效的 RSS/XML 格式：必须以 XML 声明、RSS 标签或注释开始')
      }

      // 使用清理后的文本重新解析
      try {
        const feed = await parser.parseString(text)
        return {
          success: true,
          data: {
            title: feed.title || '',
            description: feed.description || '',
            link: feed.link || '',
            items: feed.items || []
          }
        }
      } catch (parseError: any) {
        // 如果解析仍然失败，提供更详细的错误信息
        const errorMessage = parseError.message || '未知错误'
        const firstChars = text.substring(0, 100).replace(/[\x00-\x1F\x7F]/g, (char) => {
          return `[\\x${char.charCodeAt(0).toString(16).padStart(2, '0')}]`
        })
        
        console.error('RSS 解析失败详情:', {
          error: errorMessage,
          firstChars: firstChars,
          textLength: text.length,
          startsWith: text.substring(0, 50)
        })
        
        throw new Error(`解析 RSS 失败: ${errorMessage}。响应前100个字符: ${firstChars}`)
      }
    } catch (fallbackError: any) {
      // 检查是否是 429 错误（检查所有可能的错误来源）
      const is429 = fallbackError.status === 429 || 
                   fallbackError.statusCode === 429 ||
                   fallbackError.code === 429 ||
                   fallbackError.message?.includes('429') || 
                   fallbackError.message?.includes('Too Many Requests') ||
                   fallbackError.message?.includes('rate limit') ||
                   error?.status === 429 ||
                   error?.statusCode === 429 ||
                   error?.code === 429 ||
                   error?.message?.includes('429') ||
                   error?.message?.includes('Too Many Requests') ||
                   error?.message?.includes('rate limit')
      
      if (is429) {
        // 获取 Retry-After 信息（如果有）
        const retryAfter = fallbackError.retryAfter || error?.retryAfter
        const message = retryAfter 
          ? `请求频率过高，请等待 ${retryAfter} 秒后重试。建议增加刷新间隔时间。`
          : '请求频率过高，请稍后再试。建议增加刷新间隔时间。'
        
        throw createError({
          statusCode: 429,
          statusMessage: 'Too Many Requests',
          message: message,
          data: {
            retryAfter: retryAfter || null
          }
        })
      }
      
      // 如果所有尝试都失败，返回原始错误
      throw createError({
        statusCode: 500,
        message: `解析 RSS 失败: ${error?.message || fallbackError?.message || '未知错误'}`
      })
    }
  }
})

