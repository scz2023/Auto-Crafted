// 客户端数据库操作 - 通过 API 调用
export async function dbQuery(sql: string, params: any[] = []) {
  const response = await $fetch('/api/db/query', {
    method: 'POST',
    body: { sql, params }
  })
  return (response as any).data
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

