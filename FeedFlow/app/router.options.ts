import type { RouterConfig } from '@nuxt/schema'

export default <RouterConfig>{
  routes: (_routes) => {
    // 设置默认路由为 articles
    const routes = _routes.map((route) => {
      if (route.path === '/') {
        return {
          ...route,
          redirect: '/articles'
        }
      }
      return route
    })
    return routes
  },
}
