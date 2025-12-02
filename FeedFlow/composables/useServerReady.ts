// 检查服务器是否就绪
async function checkServerReady(): Promise<boolean> {
  try {
    // 尝试访问一个简单的 API 端点
    const response = await fetch('http://127.0.0.1:3000/api/db/query', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({ sql: 'SELECT 1', params: [] }),
      signal: AbortSignal.timeout(2000)
    })
    return response.ok || response.status === 400 // 400 表示服务器已启动但请求格式错误（这是正常的）
  } catch (error: any) {
    // 网络错误或超时表示服务器未启动
    if (error.name === 'AbortError' || error.message?.includes('fetch')) {
      return false
    }
    // 其他错误可能表示服务器已启动但请求失败
    return true
  }
}

// 等待服务器就绪，最多等待指定时间
export async function waitForServer(maxWaitMs: number = 10000): Promise<boolean> {
  const startTime = Date.now()
  const checkInterval = 500 // 每 500ms 检查一次
  
  while (Date.now() - startTime < maxWaitMs) {
    if (await checkServerReady()) {
      return true
    }
    await new Promise(resolve => setTimeout(resolve, checkInterval))
  }
  
  return false
}

// 检查是否在 Tauri 环境中
function isTauri(): boolean {
  return typeof window !== 'undefined' && 
         (window as any).__TAURI__ !== undefined
}

// 在 Tauri 环境中等待服务器，在 Web 环境中直接返回
export async function ensureServerReady(): Promise<void> {
  // 只在 Tauri 环境中等待服务器
  if (isTauri()) {
    const isReady = await waitForServer(15000) // 最多等待 15 秒
    if (!isReady) {
      throw new Error('服务器启动超时。请检查 Node.js 是否已安装，或查看控制台错误信息。')
    }
  }
}

