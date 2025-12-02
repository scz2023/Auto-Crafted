import { useDatabase } from '~/composables/useDatabase'
import { useSettingsStore } from '~/stores/settings'

/**
 * 获取文章摘要（从数据库读取）
 */
export async function getArticleSummary(articleId: number): Promise<string | null> {
  try {
    const db = useDatabase()
    const stmt = db.prepare('SELECT ai_summary FROM articles WHERE id = ?')
    const article = await stmt.get(articleId) as any

    if (!article) {
      throw new Error('Article not found')
    }

    return article.ai_summary || null
  } catch (error: any) {
    console.error('获取摘要失败:', error)
    throw new Error(error.message || 'Failed to get summary')
  }
}

/**
 * 生成文章摘要
 */
export async function generateArticleSummary(articleId: number): Promise<{ success: boolean; summary: string; cached: boolean }> {
  try {
    const db = useDatabase()
    
    // 读取文章信息
    const stmt = db.prepare('SELECT id, title, content, snippet, ai_summary FROM articles WHERE id = ?')
    const article = await stmt.get(articleId) as any

    if (!article) {
      throw new Error('Article not found')
    }

    // 如果已有摘要，直接返回
    if (article.ai_summary) {
      return {
        success: true,
        summary: article.ai_summary,
        cached: true
      }
    }

    // 读取 AI 设置
    const settingsStore = useSettingsStore()
    const allSettings = await settingsStore.getSettings()
    
    const aiSettings = allSettings.aiSettings
    if (!aiSettings || !aiSettings.apiToken || !aiSettings.model) {
      throw new Error('AI settings not configured. Please configure AI settings first.')
    }

    // 检查是否启用 AI 总结
    let aiSummaryEnabled = false
    if (allSettings.aiSummary !== undefined) {
      aiSummaryEnabled = allSettings.aiSummary === true || allSettings.aiSummary === 'true' || allSettings.aiSummary === 1
    }
    if (!aiSummaryEnabled && allSettings.general) {
      const generalSettings = typeof allSettings.general === 'string' 
        ? JSON.parse(allSettings.general) 
        : allSettings.general
      aiSummaryEnabled = generalSettings.aiSummary === true || generalSettings.aiSummary === 'true' || generalSettings.aiSummary === 1
    }

    if (!aiSummaryEnabled) {
      throw new Error('AI summary is not enabled in settings')
    }

    // 准备内容
    const contentToSummarize = article.content || article.snippet || article.title
    if (!contentToSummarize || contentToSummarize.trim().length === 0) {
      throw new Error('Article content is empty')
    }

    const textContent = contentToSummarize
      .replace(/<[^>]+>/g, ' ')
      .replace(/\s+/g, ' ')
      .trim()
      .substring(0, 8000)

    const prompt = `请用中文对以下新闻内容进行简洁总结，总结应该：
1. 突出核心要点
2. 控制在 100-200 字左右
3. 使用简洁明了的语言

新闻标题：${article.title}

新闻内容：
${textContent}`

    // 调用 AI API
    const response = await fetch(aiSettings.apiEndpoint || 'https://api.siliconflow.cn/v1/chat/completions', {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${aiSettings.apiToken}`,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        model: aiSettings.model,
        messages: [
          {
            role: 'user',
            content: prompt
          }
        ],
        max_tokens: aiSettings.maxTokens || 500,
        temperature: aiSettings.temperature || 0.7,
        top_p: aiSettings.topP || 0.7
      })
    })

    if (!response.ok) {
      const errorData = await response.text()
      throw new Error(`AI API error: ${response.status} - ${errorData}`)
    }

    const data = await response.json()
    const summary = data.choices?.[0]?.message?.content?.trim() || ''

    if (!summary) {
      throw new Error('AI API returned empty summary')
    }

    // 保存摘要到数据库
    const updateStmt = db.prepare('UPDATE articles SET ai_summary = ? WHERE id = ?')
    await updateStmt.run(summary, articleId)

    return {
      success: true,
      summary: summary,
      cached: false
    }
  } catch (error: any) {
    console.error('Generate summary error:', error)
    throw new Error(error.message || 'Failed to generate summary')
  }
}

/**
 * 翻译文章
 */
export async function translateArticle(
  articleId: number,
  options: {
    translateTitleOnly?: boolean
    translateContentOnly?: boolean
  } = {}
): Promise<{
  success: boolean
  titleTranslated?: string
  contentTranslated?: string
  detectedLanguage?: string
  cached: boolean
  noTranslationNeeded?: boolean
}> {
  try {
    const db = useDatabase()
    const { translateTitleOnly, translateContentOnly } = options

    // 读取文章信息
    const stmt = db.prepare('SELECT id, title, content, snippet, title_translated, content_translated, detected_language FROM articles WHERE id = ?')
    const article = await stmt.get(articleId) as any

    if (!article) {
      throw new Error('Article not found')
    }

    // 读取 AI 设置
    const settingsStore = useSettingsStore()
    const allSettings = await settingsStore.getSettings()
    
    const aiSettings = allSettings.aiSettings
    if (!aiSettings || !aiSettings.apiToken || !aiSettings.model) {
      throw new Error('AI settings not configured. Please configure AI settings first.')
    }

    // 检查是否启用 AI 翻译
    let aiTranslationEnabled = false
    let targetLanguage = 'en'
    let translationPreference = 'bilingual'

    if (allSettings.aiTranslation !== undefined) {
      aiTranslationEnabled = allSettings.aiTranslation === true || allSettings.aiTranslation === 'true' || allSettings.aiTranslation === 1
    }
    if (allSettings.aiOutputLanguage) {
      targetLanguage = typeof allSettings.aiOutputLanguage === 'string' 
        ? allSettings.aiOutputLanguage 
        : allSettings.aiOutputLanguage
    }
    if (allSettings.translationPreference) {
      translationPreference = typeof allSettings.translationPreference === 'string'
        ? allSettings.translationPreference
        : allSettings.translationPreference
    }

    // 如果根级别没有，检查 general 键下的设置
    if (!aiTranslationEnabled && allSettings.general) {
      const generalSettings = typeof allSettings.general === 'string'
        ? JSON.parse(allSettings.general)
        : allSettings.general
      aiTranslationEnabled = generalSettings.aiTranslation === true || generalSettings.aiTranslation === 'true' || generalSettings.aiTranslation === 1
      if (!targetLanguage || targetLanguage === 'en') {
        targetLanguage = generalSettings.aiOutputLanguage || 'en'
      }
      if (!translationPreference || translationPreference === 'bilingual') {
        translationPreference = generalSettings.translationPreference || 'bilingual'
      }
    }

    if (!aiTranslationEnabled) {
      throw new Error('AI translation is not enabled in settings')
    }

    // 如果只翻译标题，检查标题是否已翻译
    if (translateTitleOnly) {
      if (article.title_translated && article.detected_language) {
        return {
          success: true,
          titleTranslated: article.title_translated,
          detectedLanguage: article.detected_language,
          cached: true
        }
      }
    }

    // 如果只翻译内容，检查内容是否已翻译
    if (translateContentOnly) {
      if (article.content_translated && article.detected_language) {
        return {
          success: true,
          contentTranslated: article.content_translated,
          detectedLanguage: article.detected_language,
          cached: true
        }
      }
    }

    // 如果已有完整翻译且目标语言匹配，直接返回
    if (!translateTitleOnly && !translateContentOnly) {
      if (article.title_translated && article.content_translated && article.detected_language) {
        return {
          success: true,
          titleTranslated: article.title_translated,
          contentTranslated: article.content_translated,
          detectedLanguage: article.detected_language,
          cached: true
        }
      }
    }

    // 准备要翻译的内容
    const titleToTranslate = article.title || ''
    const contentToTranslate = article.content || article.snippet || ''

    if (!titleToTranslate && !contentToTranslate) {
      throw new Error('Article content is empty')
    }

    // 清理 HTML 标签，只保留文本内容
    const cleanTitle = titleToTranslate.replace(/<[^>]+>/g, ' ').replace(/\s+/g, ' ').trim()
    const cleanContent = contentToTranslate
      .replace(/<[^>]+>/g, ' ')
      .replace(/\s+/g, ' ')
      .trim()
      .substring(0, 15000) // 限制长度

    // 语言代码映射
    const languageNames: Record<string, string> = {
      'en': '英语',
      'ja': '日语',
      'ko': '韩语',
      'fr': '法语',
      'de': '德语',
      'es': '西班牙语',
      'ru': '俄语',
      'it': '意大利语',
      'pt': '葡萄牙语',
      'zh': '中文'
    }

    const targetLanguageName = languageNames[targetLanguage] || '英语'

    // 先检测语言
    const detectPrompt = `请检测以下文本的语言，只返回语言代码（如：zh, en, ja, ko, fr, de, es, ru, it, pt）。如果无法确定，返回 "unknown"。

文本：${cleanTitle.substring(0, 500)}${cleanContent.substring(0, 500)}`

    let detectedLanguage = 'unknown'
    try {
      const detectResponse = await fetch(aiSettings.apiEndpoint || 'https://api.siliconflow.cn/v1/chat/completions', {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${aiSettings.apiToken}`,
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({
          model: aiSettings.model,
          messages: [
            {
              role: 'user',
              content: detectPrompt
            }
          ],
          max_tokens: 10,
          temperature: 0.1
        })
      })

      if (detectResponse.ok) {
        const detectData = await detectResponse.json()
        const detected = detectData.choices?.[0]?.message?.content?.trim()?.toLowerCase() || 'unknown'
        // 验证语言代码
        if (['zh', 'en', 'ja', 'ko', 'fr', 'de', 'es', 'ru', 'it', 'pt'].includes(detected)) {
          detectedLanguage = detected
        }
      }
    } catch (error) {
      console.error('Language detection failed:', error)
    }

    // 如果检测到的语言就是目标语言，不需要翻译
    if (detectedLanguage === targetLanguage) {
      // 保存检测到的语言
      const updateStmt = db.prepare('UPDATE articles SET detected_language = ? WHERE id = ?')
      await updateStmt.run(detectedLanguage, articleId)

      const result: any = {
        success: true,
        detectedLanguage: detectedLanguage,
        cached: false,
        noTranslationNeeded: true
      }

      if (translateTitleOnly) {
        result.titleTranslated = article.title
      } else if (translateContentOnly) {
        result.contentTranslated = article.content || article.snippet || ''
      } else {
        result.titleTranslated = article.title
        result.contentTranslated = article.content || article.snippet || ''
      }

      return result
    }

    // 翻译标题（如果需要）
    let titleTranslated = article.title_translated || article.title
    if (cleanTitle && !translateContentOnly) {
      const titlePrompt = `请将以下文本翻译成${targetLanguageName}，只返回翻译结果，不要添加任何解释：

${cleanTitle}`

      try {
        const titleResponse = await fetch(aiSettings.apiEndpoint || 'https://api.siliconflow.cn/v1/chat/completions', {
          method: 'POST',
          headers: {
            'Authorization': `Bearer ${aiSettings.apiToken}`,
            'Content-Type': 'application/json'
          },
          body: JSON.stringify({
            model: aiSettings.model,
            messages: [
              {
                role: 'user',
                content: titlePrompt
              }
            ],
            max_tokens: aiSettings.maxTokens || 500,
            temperature: aiSettings.temperature || 0.7,
            top_p: aiSettings.topP || 0.7
          })
        })

        if (titleResponse.ok) {
          const titleData = await titleResponse.json()
          titleTranslated = titleData.choices?.[0]?.message?.content?.trim() || article.title
        }
      } catch (error) {
        console.error('Title translation failed:', error)
      }
    }

    // 翻译内容（如果需要）
    let contentTranslated = article.content_translated || article.content || article.snippet || ''
    if (cleanContent && !translateTitleOnly) {
      const contentPrompt = `请将以下文本翻译成${targetLanguageName}，保持原有的HTML格式和结构，只翻译文本内容，不要翻译HTML标签：

${cleanContent}`

      try {
        const contentResponse = await fetch(aiSettings.apiEndpoint || 'https://api.siliconflow.cn/v1/chat/completions', {
          method: 'POST',
          headers: {
            'Authorization': `Bearer ${aiSettings.apiToken}`,
            'Content-Type': 'application/json'
          },
          body: JSON.stringify({
            model: aiSettings.model,
            messages: [
              {
                role: 'user',
                content: contentPrompt
              }
            ],
            max_tokens: aiSettings.maxTokens || 2000,
            temperature: aiSettings.temperature || 0.7,
            top_p: aiSettings.topP || 0.7
          })
        })

        if (contentResponse.ok) {
          const contentData = await contentResponse.json()
          contentTranslated = contentData.choices?.[0]?.message?.content?.trim() || contentTranslated
        }
      } catch (error) {
        console.error('Content translation failed:', error)
      }
    }

    // 保存翻译结果到数据库
    if (translateTitleOnly) {
      // 只更新标题翻译
      const updateStmt = db.prepare('UPDATE articles SET title_translated = ?, detected_language = ? WHERE id = ?')
      await updateStmt.run(titleTranslated, detectedLanguage, articleId)

      return {
        success: true,
        titleTranslated: titleTranslated,
        detectedLanguage: detectedLanguage,
        cached: false
      }
    } else if (translateContentOnly) {
      // 只更新内容翻译
      const updateStmt = db.prepare('UPDATE articles SET content_translated = ?, detected_language = ? WHERE id = ?')
      await updateStmt.run(contentTranslated, detectedLanguage, articleId)

      return {
        success: true,
        contentTranslated: contentTranslated,
        detectedLanguage: detectedLanguage,
        cached: false
      }
    } else {
      // 更新完整翻译
      const updateStmt = db.prepare('UPDATE articles SET title_translated = ?, content_translated = ?, detected_language = ? WHERE id = ?')
      await updateStmt.run(titleTranslated, contentTranslated, detectedLanguage, articleId)

      return {
        success: true,
        titleTranslated: titleTranslated,
        contentTranslated: contentTranslated,
        detectedLanguage: detectedLanguage,
        cached: false
      }
    }
  } catch (error: any) {
    console.error('Translate error:', error)
    throw new Error(error.message || 'Failed to translate article')
  }
}

