import { useDatabase } from '~/server/utils/database'

export default defineEventHandler(async (event) => {
  try {
    const body = await readBody(event)
    const { sql, params = [] } = body

    if (!sql) {
      throw createError({
        statusCode: 400,
        message: 'SQL query is required'
      })
    }

    // 尝试获取数据库连接
    let db
    try {
      db = useDatabase()
    } catch (dbError: any) {
      console.error('Database connection failed:', dbError)
      throw createError({
        statusCode: 503,
        message: `Database service unavailable: ${dbError.message || 'Failed to connect to database'}`
      })
    }

    if (!db) {
      throw createError({
        statusCode: 503,
        message: 'Database instance is null'
      })
    }
    
    try {
      if (sql.trim().toUpperCase().startsWith('SELECT')) {
        const stmt = db.prepare(sql)
        const result = stmt.all(...params)
        return { success: true, data: result }
      } else {
        const stmt = db.prepare(sql)
        const result = stmt.run(...params)
        return { 
          success: true, 
          data: {
            lastInsertRowid: result.lastInsertRowid,
            changes: result.changes
          }
        }
      }
    } catch (queryError: any) {
      console.error('Database query error:', queryError)
      console.error('SQL:', sql)
      console.error('Params:', params)
      throw createError({
        statusCode: 500,
        message: `Database query failed: ${queryError.message || 'Unknown error'}`
      })
    }
  } catch (error: any) {
    // 如果已经是 createError，直接抛出
    if (error.statusCode) {
      throw error
    }
    
    // 否则包装为 500 错误
    console.error('Unexpected error in /api/db/query:', error)
    throw createError({
      statusCode: error.statusCode || 500,
      message: error.message || 'Internal server error'
    })
  }
})

