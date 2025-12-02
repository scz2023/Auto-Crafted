// OPML 解析工具
export interface OpmlFeed {
  title: string
  url: string
  description?: string
  category?: string
  htmlUrl?: string
}

export interface OpmlData {
  title?: string
  feeds: OpmlFeed[]
}

export function parseOpml(opmlText: string): OpmlData {
  const parser = new DOMParser()
  const doc = parser.parseFromString(opmlText, 'text/xml')
  
  // 检查解析错误
  const parserError = doc.querySelector('parsererror')
  if (parserError) {
    throw new Error('OPML 文件格式错误')
  }

  const opmlElement = doc.querySelector('opml')
  if (!opmlElement) {
    throw new Error('不是有效的 OPML 文件')
  }

  const head = opmlElement.querySelector('head')
  const title = head?.querySelector('title')?.textContent || ''

  const body = opmlElement.querySelector('body')
  if (!body) {
    throw new Error('OPML 文件缺少 body 元素')
  }

  const feeds: OpmlFeed[] = []
  const categories: Map<string, string> = new Map()

  // 递归遍历 outline 元素
  const traverseOutlines = (parent: Element, categoryPath: string[] = []) => {
    const outlines = parent.querySelectorAll(':scope > outline')
    
    outlines.forEach((outline) => {
      const type = outline.getAttribute('type')
      const xmlUrl = outline.getAttribute('xmlUrl')
      const text = outline.getAttribute('text') || outline.getAttribute('title') || ''
      const htmlUrl = outline.getAttribute('htmlUrl')
      const description = outline.getAttribute('description') || ''
      const categoryAttr = outline.getAttribute('category') || ''

      // 如果有 xmlUrl，说明这是一个 RSS feed
      if (xmlUrl && (type === 'rss' || type === 'atom' || !type)) {
        // 解析分类：category 属性可能是逗号分隔的多个类别
        let finalCategory: string | undefined = undefined
        
        if (categoryAttr) {
          // 按逗号分割，去除空白，过滤掉 "all"
          const categories = categoryAttr
            .split(',')
            .map(cat => cat.trim())
            .filter(cat => cat && cat.toLowerCase() !== 'all')
          
          // 使用第一个有效的分类
          if (categories.length > 0) {
            finalCategory = categories[0]
          }
        }
        
        // 如果没有从 category 属性获取到分类，使用父级路径
        if (!finalCategory && categoryPath.length > 0) {
          finalCategory = categoryPath.join(' / ')
        }

        feeds.push({
          title: text || '未命名订阅',
          url: xmlUrl,
          description: description,
          category: finalCategory,
          htmlUrl: htmlUrl || undefined
        })
      } else if (text) {
        // 这是一个分类文件夹
        const newCategoryPath = [...categoryPath, text]
        // 递归处理子元素
        traverseOutlines(outline, newCategoryPath)
      }
    })
  }

  traverseOutlines(body)

  return {
    title,
    feeds
  }
}

