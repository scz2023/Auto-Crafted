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
    const stmt = db.prepare('SELECT ai_summary FROM articles WHERE id = ?')
    const article = stmt.get(articleId) as any

    if (!article) {
      throw createError({
        statusCode: 404,
        message: 'Article not found'
      })
    }

    return {
      success: true,
      summary: article.ai_summary || null
    }
  } catch (error: any) {
    throw createError({
      statusCode: 500,
      message: error.message || 'Failed to get summary'
    })
  }
})
