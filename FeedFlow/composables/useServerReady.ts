// 检查服务器是否就绪
async function checkServerReady(): Promise<boolean> {
  try {
    // 先尝试访问根路径，这更简单可靠
    const rootResponse = await fetch('http://127.0.0.1:3000/', {
      method: 'GET',
      signal: AbortSignal.timeout(3000) // 3 秒超时
    })
    
    // 如果能访问根路径，说明服务器已启动
    if (rootResponse.status >= 200 && rootResponse.status < 500) {
      console.log('[checkServerReady] 服务器已就绪（根路径检查），状态码:', rootResponse.status)
      return true
    }
    
    // 如果根路径不行，尝试访问 API 端点
    const response = await fetch('http://127.0.0.1:3000/api/db/query', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({ sql: 'SELECT 1', params: [] }),
      signal: AbortSignal.timeout(3000) // 3 秒超时
    })
    // 200 表示成功，400 表示服务器已启动但请求格式错误（这是正常的）
    // 500 表示服务器已启动但处理出错（也算服务器已启动）
    const isReady = response.ok || response.status === 400 || response.status === 500
    if (isReady) {
      console.log('[checkServerReady] 服务器已就绪（API 检查），状态码:', response.status)
    }
    return isReady
  } catch (error: any) {
    // 网络错误或超时表示服务器未启动
    if (error.name === 'AbortError' || 
        error.name === 'TimeoutError' ||
        error.message?.includes('fetch') ||
        error.message?.includes('network') ||
        error.message?.includes('Failed to fetch') ||
        error.message?.includes('ERR_CONNECTION_REFUSED') ||
        error.message?.includes('ECONNREFUSED')) {
      console.log('[checkServerReady] 服务器未就绪:', error.name || error.message)
      return false
    }
    // 其他错误可能表示服务器已启动但请求失败
    console.log('[checkServerReady] 服务器可能已启动，但请求失败:', error.message)
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

// 检查是否在 Tauri 环境中（使用完善的检测逻辑）
function isTauri(): boolean {
  if (typeof window === 'undefined') {
    return false
  }
  
  // 方法1: 检查 __TAURI__ 对象（Tauri 2.0 标准方式）
  const hasTauriObject = (window as any).__TAURI__ !== undefined
  
  // 方法2: 检查 userAgent（备用检测方式）
  const userAgent = navigator.userAgent || ''
  const hasTauriUserAgent = userAgent.includes('Tauri') || userAgent.includes('tauri')
  
  // 方法3: 检查是否在 Tauri 窗口中
  const hasTauriWindow = (window as any).__TAURI_INTERNALS__ !== undefined
  
  return hasTauriObject || hasTauriWindow || hasTauriUserAgent
}

// 检查是否是开发模式
function isDevMode(): boolean {
  if (typeof window === 'undefined') return false
  
  // 检查 URL 是否包含 localhost 或 127.0.0.1
  const isLocalhost = window.location.href.includes('localhost') || 
                      window.location.href.includes('127.0.0.1')
  
  // 在 Nuxt/Vite 环境中，import.meta.env 总是可用的
  // 检查开发模式标志
  const isDevEnv = import.meta.env?.DEV === true || import.meta.env?.MODE === 'development'
  
  return isLocalhost || isDevEnv
}

// 在 Tauri 环境中等待服务器，在 Web 环境中直接返回
export async function ensureServerReady(): Promise<void> {
  // 只在 Tauri 生产环境中等待服务器
  // 开发模式下，Nuxt 开发服务器已经通过 beforeDevCommand 启动，不需要等待
  if (isTauri() && !isDevMode()) {
    console.log('[ensureServerReady] 生产模式：等待 Nitro 服务器启动...')
    const isReady = await waitForServer(15000) // 最多等待 15 秒
    if (!isReady) {
      throw new Error('服务器启动超时。请检查 Node.js 是否已安装，或查看控制台错误信息。')
    }
  } else if (isTauri() && isDevMode()) {
    console.log('[ensureServerReady] 开发模式：使用 Nuxt 开发服务器（已通过 beforeDevCommand 启动）')
    // 开发模式下，可以快速检查一下服务器是否可用
    try {
      const quickCheck = await fetch('http://localhost:3000/', { 
        signal: AbortSignal.timeout(2000) 
      })
      if (quickCheck.ok) {
        console.log('[ensureServerReady] 开发服务器已就绪')
      }
    } catch (e) {
      console.warn('[ensureServerReady] 开发服务器检查失败，但继续执行:', e)
    }
  }
}

