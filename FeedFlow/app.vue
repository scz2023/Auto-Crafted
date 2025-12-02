<template>
  <div class="reader-container">
    <!-- 左侧菜单 -->
    <div class="sidebar">
      <div class="sidebar-header">
        <h1>FeedFlow</h1>
      </div>
      <el-menu
        :default-active="activeMenu"
        class="sidebar-menu"
        @select="handleMenuSelect"
      >
        <el-menu-item index="add-feed">
          <el-icon><Plus /></el-icon>
          <span>添加订阅</span>
        </el-menu-item>
              <el-sub-menu index="articles">
                <template #title>
                  <el-icon><List /></el-icon>
                  <span>全部文章</span>
                </template>
                <!-- 按分类分组显示订阅源 -->
                <template v-for="category in groupedFeeds" :key="category.name || 'uncategorized'">
                  <el-sub-menu v-if="category.name" :index="`category-${category.name}`">
                    <template #title>
                      <span>{{ category.name }}</span>
                    </template>
                    <el-menu-item
                      v-for="feed in category.feeds"
                      :key="feed.id"
                      :index="`feed-${feed.id}`"
                      @click="selectFeed(feed.id)"
                      :class="{ 'is-active': activeMenu === `feed-${feed.id}` }"
                    >
                      <div class="feed-menu-item">
                        <img 
                          v-if="getFeedIcon(feed)" 
                          :src="getFeedIcon(feed)" 
                          :alt="feed.title"
                          class="feed-icon"
                          loading="lazy"
                          @error="handleIconError"
                          @load="(e) => { if ((e.target as HTMLImageElement).naturalWidth === 0) handleIconError(e) }"
                        />
                        <el-icon v-else class="feed-icon-placeholder"><Document /></el-icon>
                        <span>{{ feed.title }}</span>
                      </div>
                    </el-menu-item>
                  </el-sub-menu>
                  <!-- 未分类的订阅源 -->
                  <el-menu-item
                    v-else
                    v-for="feed in category.feeds"
                    :key="feed.id"
                    :index="`feed-${feed.id}`"
                    @click="selectFeed(feed.id)"
                    :class="{ 'is-active': activeMenu === `feed-${feed.id}` }"
                  >
                    <div class="feed-menu-item">
                      <img 
                        v-if="getFeedIcon(feed)" 
                        :src="getFeedIcon(feed)" 
                        :alt="feed.title"
                        class="feed-icon"
                        loading="lazy"
                        @error="handleIconError"
                        @load="(e) => { if ((e.target as HTMLImageElement).naturalWidth === 0) handleIconError(e) }"
                      />
                      <el-icon v-else class="feed-icon-placeholder"><Document /></el-icon>
                      <span>{{ feed.title }}</span>
                    </div>
                  </el-menu-item>
                </template>
              </el-sub-menu>
        <el-menu-item index="favorites">
          <el-icon><Star /></el-icon>
          <span>收藏</span>
        </el-menu-item>
        <el-menu-item index="settings">
          <el-icon><Setting /></el-icon>
          <span>设置</span>
        </el-menu-item>
        <el-menu-item index="about">
          <el-icon><InfoFilled /></el-icon>
          <span>关于</span>
        </el-menu-item>
      </el-menu>
    </div>

    <!-- 右侧内容区域 -->
    <div class="main-content">
      <NuxtPage />
    </div>

    <!-- 添加订阅对话框 -->
    <el-dialog v-model="showAddDialog" title="添加订阅" width="500px">
      <el-form :model="newFeed" label-width="100px">
        <el-form-item label="URL" required>
          <el-input v-model="newFeed.url" placeholder="请输入 RSS 订阅地址" />
        </el-form-item>
        <el-form-item label="标题">
          <el-input v-model="newFeed.title" placeholder="可选，将自动获取" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showAddDialog = false">取消</el-button>
        <el-button type="primary" @click="addFeed" :loading="adding">添加</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { Plus, List, Setting, Document, Star, InfoFilled } from '@element-plus/icons-vue'
import { ref, computed, watch, onMounted, onUnmounted, provide } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import { useFeedStore } from '~/stores/feed'
import { initAutoRefresh, updateAutoRefreshSettings, stopAutoRefresh } from '~/composables/useAutoRefresh'
import { useTheme } from '~/composables/useTheme'
import { useSettingsStore } from '~/stores/settings'

const route = useRoute()
const router = useRouter()
const feedStore = useFeedStore()
const settingsStore = useSettingsStore()
const { initTheme, setTheme } = useTheme()
const activeMenu = ref('articles')

// 订阅源列表
const feeds = ref<any[]>([])
const selectedFeedId = ref<number | null>(null)

// 按分类分组订阅源
const groupedFeeds = computed(() => {
  const groups: Map<string | null, any[]> = new Map()
  
  feeds.value.forEach(feed => {
    const categoryName = feed.categoryName || null
    if (!groups.has(categoryName)) {
      groups.set(categoryName, [])
    }
    groups.get(categoryName)!.push(feed)
  })
  
  // 转换为数组，未分类的放在最后
  const result: Array<{ name: string | null, feeds: any[] }> = []
  const uncategorized: any[] = []
  
  groups.forEach((feeds, name) => {
    if (name) {
      result.push({ name, feeds })
    } else {
      uncategorized.push(...feeds)
    }
  })
  
  // 按分类名称排序
  result.sort((a, b) => (a.name || '').localeCompare(b.name || ''))
  
  // 如果有未分类的，添加到末尾
  if (uncategorized.length > 0) {
    result.push({ name: null, feeds: uncategorized })
  }
  
  return result
})

// 添加订阅相关
const showAddDialog = ref(false)
const adding = ref(false)
const newFeed = ref({
  url: '',
  title: ''
})

// 根据路由设置激活的菜单项
watch(() => route.path, (path) => {
  if (path === '/settings') {
    activeMenu.value = 'settings'
  } else if (path === '/favorites') {
    activeMenu.value = 'favorites'
  } else if (path === '/about') {
    activeMenu.value = 'about'
  } else if (path === '/articles') {
    activeMenu.value = 'articles'
    // 如果有 feedId 参数，设置对应的菜单项为激活状态
    const feedId = route.query.feedId
    if (feedId) {
      activeMenu.value = `feed-${feedId}`
      selectedFeedId.value = parseInt(feedId as string)
    } else if (feeds.value.length > 0 && !selectedFeedId.value) {
      // 如果没有 feedId 参数但有订阅源，默认选中第一个
      const firstFeed = feeds.value[0]
      selectFeed(firstFeed.id)
    }
  }
}, { immediate: true })

const handleMenuSelect = (index: string) => {
  if (index === 'add-feed') {
    // 直接打开添加订阅对话框，不跳转路由
    showAddDialog.value = true
  } else if (index === 'settings') {
    router.push('/settings')
  } else if (index === 'favorites') {
    router.push('/favorites')
  } else if (index === 'about') {
    router.push('/about')
  } else if (index.startsWith('feed-')) {
    // 文章相关，已经在 selectFeed 中处理路由
    router.push('/articles')
  } else {
    router.push('/articles')
  }
}

const selectFeed = (feedId: number | null) => {
  selectedFeedId.value = feedId
  // 通过路由参数传递
  if (feedId) {
    activeMenu.value = `feed-${feedId}`
    router.push({ path: '/articles', query: { feedId: feedId.toString() } })
  } else {
    router.push({ path: '/articles' })
  }
}

// 存储已失败的 favicon URL，避免重复尝试
const failedFavicons = new Set<string>()

// 获取订阅源图标
const getFeedIcon = (feed: any) => {
  // 优先使用存储的favicon
  if (feed.favicon) {
    // 检查是否之前加载失败过
    if (failedFavicons.has(feed.favicon)) {
      return null
    }
    return feed.favicon
  }
  // 如果没有，尝试从URL获取favicon
  if (feed.url) {
    try {
      const url = new URL(feed.url)
      const faviconUrl = `${url.protocol}//${url.host}/favicon.ico`
      // 检查是否之前加载失败过
      if (failedFavicons.has(faviconUrl)) {
        return null
      }
      return faviconUrl
    } catch {
      return null
    }
  }
  return null
}

// 处理图标加载错误
const handleIconError = (event: Event) => {
  const img = event.target as HTMLImageElement
  if (img && img.src) {
    // 记录失败的 favicon URL
    failedFavicons.add(img.src)
    // 隐藏图片
    img.style.display = 'none'
    // 阻止错误冒泡
    event.preventDefault()
    event.stopPropagation()
    // 阻止默认行为
    return false
  }
}

// 直接加载订阅源，参考订阅源管理页面的方式，确保立即显示
const loadFeeds = async () => {
  try {
    // 直接调用，不使用复杂缓存逻辑
    const loadedFeeds = await feedStore.getAllFeeds()
    feeds.value = loadedFeeds
    
    // 如果订阅源列表加载完成且当前没有选中订阅源，默认选中第一个
    if (loadedFeeds.length > 0 && !selectedFeedId.value && route.path === '/articles') {
      const firstFeed = loadedFeeds[0]
      selectFeed(firstFeed.id)
    }
  } catch (error) {
    console.error('加载订阅源失败:', error)
    feeds.value = []
  }
}

// 监听订阅状态变化，刷新订阅源列表
if (typeof window !== 'undefined') {
  window.addEventListener('feeds-updated', loadFeeds)
}

const addFeed = async () => {
  if (!newFeed.value.url) {
    ElMessage.warning('请输入订阅地址')
    return
  }

  adding.value = true
  try {
    await feedStore.addFeed(newFeed.value.url, newFeed.value.title)
    ElMessage.success('添加成功')
    showAddDialog.value = false
    newFeed.value = { url: '', title: '' }
    // 刷新订阅源列表
    await loadFeeds()
  } catch (error: any) {
    ElMessage.error(error.message || '添加失败')
  } finally {
    adding.value = false
  }
}

// 初始化字体大小
const initFontSize = async () => {
  try {
    const settings = await settingsStore.getSettings()
    if (settings.fontSize && typeof document !== 'undefined') {
      document.documentElement.style.setProperty('--article-font-size', `${settings.fontSize}px`)
    }
  } catch (error) {
    console.error('初始化字体大小失败:', error)
  }
}

// 初始化应用
onMounted(async () => {
  // 立即加载订阅源列表，确保菜单快速显示
  loadFeeds()
  
  // 并行执行其他初始化任务
  Promise.all([
    initTheme(),
    initAutoRefresh(),
    initFontSize()
  ]).catch(error => {
    console.error('初始化失败:', error)
  })
})

// 提供选中的订阅源ID给子组件
provide('selectedFeedId', selectedFeedId)

// 清理
onUnmounted(() => {
  stopAutoRefresh()
  if (typeof window !== 'undefined') {
    window.removeEventListener('feeds-updated', loadFeeds)
  }
})
</script>

<style scoped>
.reader-container {
  display: flex;
  height: 100%;
  width: 100%;
  overflow: hidden;
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
}

.sidebar {
  width: 240px;
  min-width: 240px;
  background-color: var(--bg-primary);
  border-right: 1px solid var(--border-primary);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  transition: background-color 0.3s ease, border-color 0.3s ease;
}

.sidebar-header {
  padding: 20px;
  border-bottom: 1px solid var(--border-primary);
  background-color: var(--bg-secondary);
  transition: background-color 0.3s ease, border-color 0.3s ease;
}

.sidebar-header h1 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary);
  transition: color 0.3s ease;
}

.sidebar-menu {
  border-right: none;
  flex: 1;
  overflow-y: auto;
  background-color: var(--bg-primary);
}

/* 一级菜单项样式 - 靠左对齐 */
.sidebar-menu :deep(.el-menu-item),
.sidebar-menu :deep(.el-sub-menu__title) {
  height: 48px;
  line-height: 48px;
  padding-left: 20px !important;
  text-align: left;
  justify-content: flex-start;
  color: var(--text-primary);
  transition: background-color 0.3s ease, color 0.3s ease;
}

/* 一级菜单项之间的分隔 */
.sidebar-menu :deep(.el-menu-item:not(:last-child)),
.sidebar-menu :deep(.el-sub-menu:not(:last-child)) {
  border-bottom: 1px solid var(--border-tertiary);
}

.sidebar-menu :deep(.el-sub-menu) {
  border-bottom: 1px solid var(--border-tertiary);
}

/* 子菜单项样式 */
.sidebar-menu :deep(.el-sub-menu .el-menu-item) {
  padding-left: 50px !important;
  height: 40px;
  line-height: 40px;
  border-bottom: none;
  color: var(--text-primary);
  transition: background-color 0.3s ease, color 0.3s ease;
}

/* 子菜单标题样式 */
.sidebar-menu :deep(.el-sub-menu__title) {
  color: var(--text-primary);
}

/* 子菜单展开后的菜单项 */
.sidebar-menu :deep(.el-sub-menu .el-menu) {
  background-color: var(--bg-secondary);
}

.sidebar-menu :deep(.el-sub-menu .el-menu-item) {
  background-color: transparent;
}

.sidebar-menu :deep(.el-sub-menu .el-menu-item:hover) {
  background-color: var(--bg-hover) !important;
  color: var(--color-primary) !important;
}

.sidebar-menu :deep(.el-sub-menu .el-menu-item.is-active) {
  background-color: var(--bg-active) !important;
  color: var(--color-primary) !important;
}

/* 订阅源菜单项样式 */
.feed-menu-item {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
}

.feed-icon {
  width: 16px;
  height: 16px;
  object-fit: contain;
  flex-shrink: 0;
}

.feed-icon-placeholder {
  width: 16px;
  height: 16px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-tertiary);
  transition: color 0.3s ease;
}

/* 图标和文字对齐 */
.sidebar-menu :deep(.el-menu-item .el-icon),
.sidebar-menu :deep(.el-sub-menu__title .el-icon) {
  margin-right: 8px;
  width: 16px;
  text-align: center;
  color: inherit;
}

/* 菜单项悬停和激活状态 */
.sidebar-menu :deep(.el-menu-item:hover),
.sidebar-menu :deep(.el-sub-menu__title:hover) {
  background-color: var(--bg-hover) !important;
  color: var(--color-primary) !important;
}

.sidebar-menu :deep(.el-menu-item.is-active) {
  background-color: var(--bg-active) !important;
  color: var(--color-primary) !important;
}

.sidebar-menu :deep(.el-menu-item.is-active .el-icon),
.sidebar-menu :deep(.el-menu-item.is-active span) {
  color: var(--color-primary) !important;
}

.main-content {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  background-color: var(--bg-primary);
  transition: background-color 0.3s ease;
}

/* Element Plus 对话框颜色适配 */
:deep(.el-dialog) {
  background-color: var(--bg-primary) !important;
  color: var(--text-primary) !important;
  transition: background-color 0.3s ease, color 0.3s ease;
}

:deep(.el-dialog__header) {
  background-color: var(--bg-primary) !important;
  border-bottom-color: var(--border-primary) !important;
  padding: 20px 20px 10px !important;
}

:deep(.el-dialog__title) {
  color: var(--text-primary) !important;
  font-size: 18px;
  font-weight: 600;
  transition: color 0.3s ease;
}

:deep(.el-dialog__headerbtn) {
  top: 20px;
  right: 20px;
}

:deep(.el-dialog__close) {
  color: var(--text-tertiary) !important;
  font-size: 18px;
  transition: color 0.3s ease;
}

:deep(.el-dialog__close:hover) {
  color: var(--text-primary) !important;
}

:deep(.el-dialog__body) {
  background-color: var(--bg-primary) !important;
  color: var(--text-primary) !important;
  padding: 20px !important;
  transition: background-color 0.3s ease, color 0.3s ease;
}

:deep(.el-dialog__footer) {
  background-color: var(--bg-primary) !important;
  border-top-color: var(--border-primary) !important;
  padding: 10px 20px 20px !important;
  transition: background-color 0.3s ease, border-color 0.3s ease;
}

/* Element Plus 表单标签颜色 */
:deep(.el-form-item__label) {
  color: var(--text-primary) !important;
  transition: color 0.3s ease;
}

/* Element Plus 输入框包装器颜色（补充全局样式） */
:deep(.el-input__wrapper) {
  background-color: var(--bg-primary) !important;
  box-shadow: 0 0 0 1px var(--border-primary) inset !important;
}

:deep(.el-input__wrapper:hover) {
  box-shadow: 0 0 0 1px var(--border-secondary) inset !important;
}

:deep(.el-input.is-focus .el-input__wrapper),
:deep(.el-input__wrapper.is-focus) {
  box-shadow: 0 0 0 1px var(--color-primary) inset !important;
}

/* Element Plus 输入框内部文字颜色 */
:deep(.el-input__inner) {
  background-color: transparent !important;
  color: var(--text-primary) !important;
}

:deep(.el-input__inner::placeholder) {
  color: var(--text-tertiary) !important;
}

/* Element Plus 文本域 */
:deep(.el-textarea__inner) {
  background-color: var(--bg-primary) !important;
  color: var(--text-primary) !important;
  border-color: var(--border-primary) !important;
}

:deep(.el-textarea__inner:focus) {
  border-color: var(--color-primary) !important;
}

:deep(.el-textarea__inner::placeholder) {
  color: var(--text-tertiary) !important;
}

/* Element Plus 选择器输入框包装器 */
:deep(.el-select .el-input__wrapper) {
  background-color: var(--bg-primary) !important;
}

/* 占位符文本颜色 */
:deep(.el-input__inner::placeholder),
:deep(.el-textarea__inner::placeholder) {
  color: var(--text-tertiary) !important;
}

/* 必填标记颜色 */
:deep(.el-form-item.is-required .el-form-item__label::before) {
  color: var(--color-primary) !important;
}

/* Element Plus 选择器下拉菜单 */
:deep(.el-select-dropdown) {
  background-color: var(--bg-primary) !important;
  border-color: var(--border-primary) !important;
}

:deep(.el-select-dropdown__item) {
  color: var(--text-primary) !important;
  background-color: var(--bg-primary) !important;
}

:deep(.el-select-dropdown__item:hover) {
  background-color: var(--bg-hover) !important;
}

:deep(.el-select-dropdown__item.selected) {
  background-color: var(--bg-active) !important;
  color: var(--color-primary) !important;
}
</style>

