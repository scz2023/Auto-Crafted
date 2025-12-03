// 检查是否在 Tauri 环境中
function isTauri(): boolean {
  if (typeof window === 'undefined') {
    return false
  }
  
  // 方法1: 检查 __TAURI__ 对象（Tauri 2.0 标准方式）
  const hasTauriObject = (window as any).__TAURI__ !== undefined
  
  // 方法2: 检查 userAgent（备用检测方式）
  const userAgent = navigator.userAgent || ''
  const hasTauriUserAgent = userAgent.includes('Tauri') || userAgent.includes('tauri')
  
  // 方法3: 检查是否在 Tauri 窗口中（通过 window.name 或其他特征）
  const hasTauriWindow = (window as any).__TAURI_INTERNALS__ !== undefined
  
  const result = hasTauriObject || hasTauriWindow || hasTauriUserAgent
  
  // 只在首次调用或调试时输出日志
  if (!(window as any).__TAURI_ENV_CHECKED__) {
    console.log('[useDatabase] Tauri 环境检测:', {
      hasTauriObject,
      hasTauriWindow,
      hasTauriUserAgent,
      result,
      userAgent: userAgent.substring(0, 100),
      tauriObject: (window as any).__TAURI__ ? '存在' : '不存在',
      tauriInternals: (window as any).__TAURI_INTERNALS__ ? '存在' : '不存在'
    })
    ;(window as any).__TAURI_ENV_CHECKED__ = true
  }
  
  return result
}

// 客户端数据库操作 - 通过 API 调用或 Tauri 命令
export async function dbQuery(sql: string, params: any[] = []) {
  const isTauriEnv = isTauri()
  console.log('[dbQuery] 开始执行，Tauri 环境:', isTauriEnv)
  
  // 如果在 Tauri 环境中，使用 Tauri 命令（直接操作数据库）
  if (isTauriEnv) {
    try {
      // Tauri 2 使用 @tauri-apps/api/core
      let invoke: any
      try {
        const apiModule = await import('@tauri-apps/api/core')
        invoke = apiModule.invoke
        if (!invoke) {
          throw new Error('invoke 方法未找到，@tauri-apps/api/core 可能未正确加载')
        }
      } catch (importError: any) {
        console.error('导入 @tauri-apps/api/core 失败:', importError)
        throw new Error(`无法加载 Tauri API: ${importError.message || importError.toString()}`)
      }
      
      console.log('调用 Tauri 命令 db_query:', { sql: sql.substring(0, 50) + '...', paramsCount: params.length })
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
      
      if (!response) {
        console.error('Tauri 命令返回空响应:', { sql, params })
        throw new Error('Tauri 命令返回空响应，可能是命令未正确注册')
      }
      
      if (!response.success) {
        const errorMsg = response?.error || 'Tauri 命令执行失败'
        console.error('Tauri 命令执行失败:', { sql, params, error: errorMsg })
        throw new Error(errorMsg)
      }
      
      const data = response.data
      console.log('[dbQuery] Tauri 命令返回数据:', { 
        sql: sql.substring(0, 50), 
        paramsCount: params.length,
        dataType: Array.isArray(data) ? 'array' : typeof data,
        dataLength: Array.isArray(data) ? data.length : 'N/A'
      })
      
      return data
    } catch (error: any) {
      console.error('Tauri 数据库查询失败:', { sql, params, error })
      
      // 在 Tauri 环境中，如果命令失败，直接抛出错误，不要回退到 API
      // 因为如果 Tauri 命令不可用，说明应用本身有问题
      const errorMessage = error.message || error.toString() || '数据库查询失败'
      
      // 检查是否是命令未找到的错误
      if (errorMessage.includes('command') && errorMessage.includes('not found')) {
        throw new Error('数据库命令未找到。请确保应用已正确构建，Tauri 命令已注册。')
      }
      
      // 检查是否是数据库连接错误
      if (errorMessage.includes('数据库连接失败') || errorMessage.includes('connection')) {
        throw new Error('数据库连接失败：' + errorMessage)
      }
      
      throw new Error('数据库操作失败：' + errorMessage)
    }
  }
  
  // 在 Web 环境中，使用 API 调用
  // 注意：如果 isTauri() 返回 false 但实际在 Tauri 环境中，这里会执行 API 调用
  // 这通常不应该发生，但如果发生了，说明 Tauri 环境检测有问题
  console.warn('[dbQuery] 警告：在非 Tauri 环境中使用 API 调用，这可能是环境检测问题')
  try {
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
          const results = await dbQuery(sql, params)
          // 确保返回的是数组
          if (!results) {
            console.warn('[useDatabase] 查询返回空结果:', { sql: sql.substring(0, 50), params })
            return []
          }
          if (Array.isArray(results)) {
            return results
          }
          // 如果不是数组，尝试包装成数组
          console.warn('[useDatabase] 查询返回非数组结果，尝试转换:', { sql: sql.substring(0, 50), params, results })
          return [results]
        },
        get: async (...params: any[]) => {
          const results = await dbQuery(sql, params)
          if (!results) {
            return null
          }
          if (Array.isArray(results)) {
            return results[0] || null
          }
          // 如果不是数组，直接返回结果（可能是单个对象）
          console.warn('[useDatabase] get 查询返回非数组结果:', { sql: sql.substring(0, 50), params, results })
          return results
        },
        run: async (...params: any[]) => {
          const result = await dbQuery(sql, params)
          // 对于非 SELECT 语句，result 应该是一个对象，包含 lastInsertRowid 和 changes
          if (result && typeof result === 'object' && !Array.isArray(result)) {
            return {
              lastInsertRowid: (result as any).lastInsertRowid || null,
              changes: (result as any).changes || 0
            }
          }
          // 如果返回的是数组或其他格式，返回默认值
          console.warn('[useDatabase] run 查询返回意外格式:', { sql: sql.substring(0, 50), params, result })
          return {
            lastInsertRowid: null,
            changes: 0
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

