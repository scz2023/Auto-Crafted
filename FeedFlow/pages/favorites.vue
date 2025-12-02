<template>
  <div class="articles-page">
    <!-- 中间：收藏文章列表 -->
    <div class="articles-list">
      <div class="list-header">
        <h2>收藏的文章</h2>
      </div>
      
      <div class="articles-scroll-container" ref="scrollContainer">
        <el-empty v-if="articles.length === 0 && !loading" description="暂无收藏文章" />
        <el-skeleton v-else-if="loading" :rows="10" animated />
        <div v-else class="articles-container">
          <div
            v-for="article in articles"
            :key="article.id"
            class="article-item"
            :class="{ active: selectedArticle?.id === article.id }"
            @click="selectArticle(article)"
          >
            <div class="article-item-header">
              <div class="article-title">{{ article.title }}</div>
              <el-button
                :icon="StarFilled"
                type="warning"
                circle
                size="small"
                @click.stop="toggleFavorite(article)"
                class="favorite-btn"
              />
            </div>
            <div class="article-meta">
              <span class="feed-name">{{ article.feedName }}</span>
              <span class="article-date">{{ formatDate(article.pub_date) }}</span>
            </div>
            <div class="article-snippet">{{ article.snippet }}</div>
          </div>
          
          <!-- 加载更多提示 -->
          <div v-if="loadingMore" class="loading-more">
            <el-icon class="is-loading"><Loading /></el-icon>
            <span>加载中...</span>
          </div>
          <div v-else-if="!hasMore && articles.length > 0" class="no-more">
            没有更多文章了
          </div>
        </div>
      </div>
    </div>

    <!-- 右边：新闻内容 -->
    <div class="article-content">
      <el-empty v-if="!selectedArticle" description="请选择一篇文章" />
      <div v-else class="content-wrapper">
        <div class="content-header">
          <div class="content-title-row">
            <h1>{{ selectedArticle.title }}</h1>
            <el-button
              :icon="StarFilled"
              type="warning"
              circle
              @click="toggleFavorite(selectedArticle)"
              class="favorite-btn"
            />
          </div>
          <div class="content-meta">
            <span>{{ selectedArticle.feedName }}</span>
            <span>{{ formatDate(selectedArticle.pub_date) }}</span>
          </div>
        </div>
        <el-divider />
        <div class="content-body" v-html="selectedArticle.content"></div>
        <div v-if="selectedArticle.link" class="content-footer">
          <el-button type="primary" @click="openLink">查看原文</el-button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed, watch, onUnmounted, nextTick } from 'vue'
import { Loading, StarFilled } from '@element-plus/icons-vue'
import { ElMessage } from 'element-plus'
import { useArticleStore } from '~/stores/article'
import { useSettingsStore } from '~/stores/settings'

const articleStore = useArticleStore()
const settingsStore = useSettingsStore()
const articles = ref<any[]>([])
const selectedArticle = ref<any>(null)
const loading = ref(false)
const loadingMore = ref(false)
const hasMore = ref(true)

// 获取设置
const settings = ref({
  fontSize: 14,
  articlesPerPage: 20
})

// 每页加载数量
const pageSize = computed(() => settings.value.articlesPerPage || 20)

onMounted(async () => {
  await loadSettings()
  await loadArticles(true)
  
  // 监听滚动事件
  nextTick(() => {
    const container = document.querySelector('.articles-scroll-container')
    if (container) {
      scrollContainer.value = container as HTMLElement
      container.addEventListener('scroll', handleScroll)
    }
  })
})

onUnmounted(() => {
  if (scrollContainer.value) {
    scrollContainer.value.removeEventListener('scroll', handleScroll)
  }
})

const loadSettings = async () => {
  try {
    const saved = await settingsStore.getSettings()
    if (saved) {
      settings.value = { ...settings.value, ...saved }
      applyFontSize(settings.value.fontSize)
    }
  } catch (error) {
    console.error('加载设置失败:', error)
  }
}

// 应用字体大小
const applyFontSize = (size: number) => {
  if (typeof document !== 'undefined') {
    document.documentElement.style.setProperty('--article-font-size', `${size}px`)
  }
}

// 监听设置变化
watch(() => settings.value.fontSize, (newSize) => {
  applyFontSize(newSize)
})

const loadArticles = async (reset = false) => {
  if (reset) {
    loading.value = true
    articles.value = []
    hasMore.value = true
  } else {
    loadingMore.value = true
  }

  try {
    const offset = reset ? 0 : articles.value.length
    const newArticles = await articleStore.getFavoriteArticles(
      pageSize.value, 
      offset
    )
    
    if (reset) {
      articles.value = newArticles
    } else {
      articles.value = [...articles.value, ...newArticles]
    }

    // 检查是否还有更多文章
    hasMore.value = newArticles.length === pageSize.value
  } catch (error) {
    console.error('加载文章失败:', error)
  } finally {
    loading.value = false
    loadingMore.value = false
  }
}

// 滚动加载更多
const scrollContainer = ref<HTMLElement | null>(null)
const handleScroll = () => {
  if (!scrollContainer.value || loadingMore.value || !hasMore.value) return

  const container = scrollContainer.value
  const scrollTop = container.scrollTop
  const scrollHeight = container.scrollHeight
  const clientHeight = container.clientHeight

  // 当滚动到距离底部 100px 时加载更多
  if (scrollHeight - scrollTop - clientHeight < 100) {
    loadArticles(false)
  }
}

const selectArticle = (article: any) => {
  selectedArticle.value = article
}

const formatDate = (date: string | Date) => {
  if (!date) return ''
  const d = new Date(date)
  return d.toLocaleString('zh-CN')
}

const openLink = async () => {
  if (!selectedArticle.value?.link) {
    return
  }

  const url = selectedArticle.value.link
  
  try {
    // 检查是否在 Tauri 环境中
    if (typeof window !== 'undefined' && (window as any).__TAURI_INTERNALS__) {
      // 使用 Tauri 的 shell plugin 打开外部链接
      const { open } = await import('@tauri-apps/plugin-shell')
      await open(url)
    } else {
      // 在浏览器环境中使用 window.open
      window.open(url, '_blank', 'noopener,noreferrer')
    }
  } catch (error: any) {
    console.error('打开链接失败:', error)
    // 降级到 window.open
    try {
      window.open(url, '_blank', 'noopener,noreferrer')
    } catch (e) {
      console.error('使用 window.open 也失败:', e)
      ElMessage.error('无法打开链接，请手动复制链接地址')
    }
  }
}

const toggleFavorite = async (article: any) => {
  try {
    const newFavorite = await articleStore.toggleFavorite(article.id)
    // 更新本地状态
    article.is_favorite = newFavorite ? 1 : 0
    
    // 如果当前选中的文章，也要更新
    if (selectedArticle.value?.id === article.id) {
      selectedArticle.value.is_favorite = article.is_favorite
    }
    
    // 如果取消收藏，从列表中移除
    if (!newFavorite) {
      articles.value = articles.value.filter(a => a.id !== article.id)
      // 如果当前选中的文章被取消收藏，清空选中
      if (selectedArticle.value?.id === article.id) {
        selectedArticle.value = null
      }
    }
    
    ElMessage.success(newFavorite ? '已收藏' : '已取消收藏')
  } catch (error: any) {
    ElMessage.error(error.message || '操作失败')
  }
}
</script>

<style scoped>
.articles-page {
  display: flex;
  height: 100%;
  width: 100%;
  overflow: hidden;
}

.articles-list {
  width: 400px;
  min-width: 350px;
  max-width: 500px;
  border-right: 1px solid #e4e7ed;
  display: flex;
  flex-direction: column;
  background-color: #fff;
  overflow: hidden;
  height: 100%;
}

.list-header {
  padding: 20px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  border-bottom: 1px solid #e4e7ed;
  flex-shrink: 0;
}

.list-header h2 {
  margin: 0;
  font-size: 18px;
}

.articles-scroll-container {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
}

.articles-container {
  padding: 10px;
}

.article-item {
  padding: 15px;
  margin-bottom: 10px;
  border: 1px solid #e4e7ed;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.3s;
}

.article-item:hover {
  background-color: #f5f7fa;
  border-color: #409eff;
}

.article-item.active {
  background-color: #ecf5ff;
  border-color: #409eff;
}

.article-item-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 8px;
  margin-bottom: 8px;
}

.article-title {
  font-size: 16px;
  font-weight: 600;
  color: #303133;
  overflow: hidden;
  text-overflow: ellipsis;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  line-clamp: 2;
  -webkit-box-orient: vertical;
  flex: 1;
  margin: 0;
}

.favorite-btn {
  flex-shrink: 0;
  margin-top: 2px;
}

.article-meta {
  display: flex;
  justify-content: space-between;
  font-size: 12px;
  color: #909399;
  margin-bottom: 8px;
}

.feed-name {
  color: #409eff;
}

.article-snippet {
  font-size: 14px;
  color: #606266;
  overflow: hidden;
  text-overflow: ellipsis;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  line-clamp: 2;
  -webkit-box-orient: vertical;
}

.article-content {
  flex: 1;
  min-width: 400px;
  overflow: hidden;
  background-color: #fafafa;
  display: flex;
  flex-direction: column;
}

.content-wrapper {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
  padding: 40px;
  background-color: #fff;
}

.content-title-row {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 10px;
}

.content-header h1 {
  margin: 0;
  font-size: 24px;
  color: #303133;
  flex: 1;
}

.content-header .favorite-btn {
  flex-shrink: 0;
  margin-top: 4px;
}

.content-meta {
  display: flex;
  gap: 20px;
  font-size: 14px;
  color: #909399;
  margin-bottom: 20px;
}

.content-body {
  font-size: var(--article-font-size, 16px);
  line-height: 1.8;
  color: #606266;
}

.loading-more,
.no-more {
  padding: 20px;
  text-align: center;
  color: #909399;
  font-size: 14px;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
}

.loading-more .el-icon {
  font-size: 16px;
}

.content-body :deep(img) {
  max-width: 100%;
  height: auto;
}

.content-body :deep(a) {
  color: #409eff;
  text-decoration: none;
}

.content-body :deep(a:hover) {
  text-decoration: underline;
}

.content-footer {
  margin-top: 30px;
  padding-top: 20px;
  border-top: 1px solid #e4e7ed;
}

/* 深色主题样式 */
html.dark .articles-list {
  background-color: #1a1a1a;
  border-right-color: #333;
}

html.dark .list-header {
  background-color: #1a1a1a;
  border-bottom-color: #333;
  color: #e5e5e5;
}

html.dark .list-header h2 {
  color: #e5e5e5;
}

html.dark .article-item {
  background-color: #2a2a2a;
  border-color: #333;
  color: #e5e5e5;
}

html.dark .article-item:hover {
  background-color: #333;
  border-color: #409eff;
}

html.dark .article-item.active {
  background-color: #2d4a5f;
  border-color: #409eff;
}

html.dark .article-title {
  color: #e5e5e5;
}

html.dark .article-meta {
  color: #909399;
}

html.dark .feed-name {
  color: #66b1ff;
}

html.dark .article-snippet {
  color: #b0b0b0;
}

html.dark .article-content {
  background-color: #1a1a1a;
}

html.dark .content-wrapper {
  background-color: #1a1a1a;
  color: #e5e5e5;
}

html.dark .content-header h1 {
  color: #e5e5e5;
}

html.dark .content-meta {
  color: #909399;
}

html.dark .content-body {
  color: #d0d0d0;
}

html.dark .content-body :deep(a) {
  color: #66b1ff;
}

html.dark .content-body :deep(a:hover) {
  color: #85c1ff;
}

html.dark .content-footer {
  border-top-color: #333;
}

html.dark .loading-more,
html.dark .no-more {
  color: #909399;
}
</style>

