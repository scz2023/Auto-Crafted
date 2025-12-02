// 检查是否在 Tauri 环境中
function isTauri(): boolean {
  return typeof window !== 'undefined' && 
         (window as any).__TAURI__ !== undefined
}

// 客户端数据库操作 - 通过 API 调用或 Tauri 命令
export async function dbQuery(sql: string, params: any[] = []) {
  // 如果在 Tauri 环境中，使用 Tauri 命令
  if (isTauri()) {
    try {
      // Tauri 2 使用 @tauri-apps/api/core
      const { invoke } = await import('@tauri-apps/api/core')
      const response = await invoke('db_query', {
        query: {
          sql,
          params: params.map(p => {
            // 将参数转换为 JSON 兼容格式
            if (p === null || p === undefined) return null
            if (typeof p === 'string' || typeof p === 'number' || typeof p === 'boolean') {
              return p
            }
            // 处理 Date 对象
            if (p instanceof Date) {
              return p.toISOString()
            }
            return JSON.parse(JSON.stringify(p))
          })
        }
      }) as any
      
      if (!response || !response.success) {
        throw new Error(response?.error || 'Tauri 命令执行失败')
      }
      
      return response.data
    } catch (error: any) {
      console.error('Tauri 数据库查询失败:', { sql, params, error })
      throw new Error(error.message || '数据库查询失败')
    }
  }
  
  // 在 Web 环境中，使用 API 调用
  // 在 Tauri 环境中，如果 Tauri 命令失败，也尝试使用 API（作为后备）
  try {
    // 在 Tauri 环境中，确保服务器已启动（如果使用 API）
    if (isTauri()) {
      const { ensureServerReady } = await import('./useServerReady')
      await ensureServerReady().catch(() => {
        // 如果服务器未启动，继续尝试（可能使用 Tauri 命令）
      })
    }
    
    const response = await $fetch('/api/db/query', {
      method: 'POST',
      body: { sql, params }
    })
    
    if (!response) {
      throw new Error('API 返回空响应，服务器端可能未运行')
    }
    
    const data = (response as any)?.data
    if (data === undefined) {
      console.error('API 响应格式错误:', response)
      throw new Error('API 响应格式错误，服务器端可能未正确配置')
    }
    
    return data
  } catch (error: any) {
    console.error('数据库查询失败:', { sql, params, error })
    
    // 检查是否是网络错误（API 不可用）
    if (error.message?.includes('fetch') || 
        error.message?.includes('network') ||
        error.status === 0 ||
        error.statusCode === 0) {
      throw new Error('无法连接到服务器端 API。请确保应用正确启动，服务器端代码已加载。')
    }
    
    // 检查是否是 503 错误（服务不可用）
    if (error.status === 503 || error.statusCode === 503) {
      throw new Error('数据库服务不可用：' + (error.data?.message || error.message || '未知错误'))
    }
    
    // 其他错误
    throw new Error(error.data?.message || error.message || '数据库查询失败')
  }
}

// 为了兼容性，提供一个类似 better-sqlite3 的接口
export function useDatabase() {
  return {
    prepare(sql: string) {
      return {
        all: async (...params: any[]) => {
          return await dbQuery(sql, params)
        },
        get: async (...params: any[]) => {
          const results = await dbQuery(sql, params)
          if (!results || !Array.isArray(results)) {
            console.error('数据库查询返回非数组结果:', results)
            return null
          }
          return results[0] || null
        },
        run: async (...params: any[]) => {
          const result = await dbQuery(sql, params)
          return {
            lastInsertRowid: (result as any).lastInsertRowid || null,
            changes: (result as any).changes || 0
          }
        }
      }
    },
    exec: async (sql: string) => {
      await dbQuery(sql)
    },
    transaction: (fn: (items: any[]) => void) => {
      return async (items: any[]) => {
        // 在客户端，事务需要特殊处理
        // 这里简化处理，直接执行
        fn(items)
      }
    }
  }
}

