import { useDatabase } from '~/server/utils/database'

export default defineEventHandler(async (event) => {
  const body = await readBody(event)
  const { articleId } = body

  if (!articleId) {
    throw createError({
      statusCode: 400,
      message: 'Article ID is required'
    })
  }

  try {
    const db = useDatabase()
    const stmt = db.prepare('SELECT id, title, content, snippet, ai_summary FROM articles WHERE id = ?')
    const article = stmt.get(articleId) as any

    if (!article) {
      throw createError({
        statusCode: 404,
        message: 'Article not found'
      })
    }

    if (article.ai_summary) {
      return {
        success: true,
        summary: article.ai_summary,
        cached: true
      }
    }

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

    const generalSettingsRow = settingsStmt.get('general') as any
    const aiSummaryRow = settingsStmt.get('aiSummary') as any
    let aiSummaryEnabled = false

    if (aiSummaryRow) {
      try {
        const value = JSON.parse(aiSummaryRow.value)
        aiSummaryEnabled = value === true || value === 'true' || value === 1
      } catch {
        aiSummaryEnabled = aiSummaryRow.value === 'true' || aiSummaryRow.value === '1'
      }
    }

    if (!aiSummaryEnabled && generalSettingsRow) {
      try {
        const generalSettings = JSON.parse(generalSettingsRow.value)
        aiSummaryEnabled = generalSettings.aiSummary === true || generalSettings.aiSummary === 'true' || generalSettings.aiSummary === 1
      } catch {
        // 解析失败
      }
    }

    if (!aiSummaryEnabled) {
      throw createError({
        statusCode: 400,
        message: 'AI summary is not enabled in settings'
      })
    }

    const contentToSummarize = article.content || article.snippet || article.title
    if (!contentToSummarize || contentToSummarize.trim().length === 0) {
      throw createError({
        statusCode: 400,
        message: 'Article content is empty'
      })
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

    const updateStmt = db.prepare('UPDATE articles SET ai_summary = ? WHERE id = ?')
    updateStmt.run(summary, articleId)

    return {
      success: true,
      summary: summary,
      cached: false
    }
  } catch (error: any) {
    console.error('Generate summary error:', error)
    throw createError({
      statusCode: 500,
      message: error.message || 'Failed to generate summary'
    })
  }
})
