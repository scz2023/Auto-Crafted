import { useDatabase } from '~/server/utils/database'

export default defineEventHandler(async (event) => {
  const body = await readBody(event)
  const { articleId, translateTitleOnly, translateContentOnly } = body

  if (!articleId) {
    throw createError({
      statusCode: 400,
      message: 'Article ID is required'
    })
  }

  try {
    const db = useDatabase()
    const stmt = db.prepare('SELECT id, title, content, snippet, title_translated, content_translated, detected_language FROM articles WHERE id = ?')
    const article = stmt.get(articleId) as any

    if (!article) {
      throw createError({
        statusCode: 404,
        message: 'Article not found'
      })
    }

    // 获取 AI 设置
    const settingsStmt = db.prepare('SELECT value FROM settings WHERE key = ?')
    const settingsRow = settingsStmt.get('aiSettings') as any
    let aiSettings: any = null

    if (settingsRow) {
      try {
        aiSettings = JSON.parse(settingsRow.value)
      } catch {
        // 解析失败
      }
    }

    if (!aiSettings || !aiSettings.apiToken || !aiSettings.model) {
      throw createError({
        statusCode: 400,
        message: 'AI settings not configured. Please configure AI settings first.'
      })
    }

    // 获取常规设置，检查是否启用 AI 翻译
    // 设置可能直接保存在根级别，也可能保存在 general 键下
    const generalSettingsRow = settingsStmt.get('general') as any
    const aiTranslationRow = settingsStmt.get('aiTranslation') as any
    const aiOutputLanguageRow = settingsStmt.get('aiOutputLanguage') as any
    const translationPreferenceRow = settingsStmt.get('translationPreference') as any
    
    console.log('翻译 API - 设置读取:', {
      generalSettingsRow: generalSettingsRow ? '存在' : '不存在',
      aiTranslationRow: aiTranslationRow ? `值: ${aiTranslationRow.value}` : '不存在',
      aiOutputLanguageRow: aiOutputLanguageRow ? `值: ${aiOutputLanguageRow.value}` : '不存在',
      translationPreferenceRow: translationPreferenceRow ? `值: ${translationPreferenceRow.value}` : '不存在'
    })
    
    let aiTranslationEnabled = false
    let targetLanguage = 'en'
    let translationPreference = 'bilingual'
    
    // 先检查根级别的设置
    if (aiTranslationRow) {
      try {
        const value = JSON.parse(aiTranslationRow.value)
        aiTranslationEnabled = value === true || value === 'true' || value === 1
      } catch {
        aiTranslationEnabled = aiTranslationRow.value === 'true' || aiTranslationRow.value === '1'
      }
    }

    if (aiOutputLanguageRow) {
      try {
        targetLanguage = JSON.parse(aiOutputLanguageRow.value) || 'en'
      } catch {
        targetLanguage = aiOutputLanguageRow.value || 'en'
      }
    }

    if (translationPreferenceRow) {
      try {
        translationPreference = JSON.parse(translationPreferenceRow.value) || 'bilingual'
      } catch {
        translationPreference = translationPreferenceRow.value || 'bilingual'
      }
    }

    // 如果根级别没有，检查 general 键下的设置
    if (!aiTranslationEnabled && generalSettingsRow) {
      try {
        const generalSettings = JSON.parse(generalSettingsRow.value)
        aiTranslationEnabled = generalSettings.aiTranslation === true || generalSettings.aiTranslation === 'true' || generalSettings.aiTranslation === 1
        if (!targetLanguage || targetLanguage === 'en') {
          targetLanguage = generalSettings.aiOutputLanguage || 'en'
        }
        if (!translationPreference || translationPreference === 'bilingual') {
          translationPreference = generalSettings.translationPreference || 'bilingual'
        }
      } catch (e) {
        // 解析失败
        console.error('解析 general 设置失败:', e)
      }
    }
    
    console.log('翻译 API - 最终设置:', {
      aiTranslationEnabled,
      targetLanguage,
      translationPreference
    })

    if (!aiTranslationEnabled) {
      throw createError({
        statusCode: 400,
        message: 'AI translation is not enabled in settings'
      })
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
      throw createError({
        statusCode: 400,
        message: 'Article content is empty'
      })
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
      updateStmt.run(detectedLanguage, articleId)
      
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
      updateStmt.run(titleTranslated, detectedLanguage, articleId)
      
      return {
        success: true,
        titleTranslated: titleTranslated,
        detectedLanguage: detectedLanguage,
        cached: false
      }
    } else if (translateContentOnly) {
      // 只更新内容翻译
      const updateStmt = db.prepare('UPDATE articles SET content_translated = ?, detected_language = ? WHERE id = ?')
      updateStmt.run(contentTranslated, detectedLanguage, articleId)
      
      return {
        success: true,
        contentTranslated: contentTranslated,
        detectedLanguage: detectedLanguage,
        cached: false
      }
    } else {
      // 更新完整翻译
      const updateStmt = db.prepare('UPDATE articles SET title_translated = ?, content_translated = ?, detected_language = ? WHERE id = ?')
      updateStmt.run(titleTranslated, contentTranslated, detectedLanguage, articleId)
      
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
    throw createError({
      statusCode: 500,
      message: error.message || 'Failed to translate article'
    })
  }
})

