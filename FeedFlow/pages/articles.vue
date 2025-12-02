<template>
  <div class="articles-page">
    <!-- 中间：新闻列表 -->
    <div class="articles-list">
      <div class="list-header">
        <h2>{{ selectedFeedId ? selectedFeedName : '全部文章' }}</h2>
        <el-button type="primary" @click="refreshAll">刷新</el-button>
      </div>
      
      <div class="articles-scroll-container" ref="scrollContainer">
        <el-empty v-if="articles.length === 0 && !loading" description="暂无文章" />
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
              <div class="article-title">{{ getDisplayTitle(article) }}</div>
              <el-button
                :icon="article.is_favorite ? StarFilled : Star"
                :type="article.is_favorite ? 'warning' : 'default'"
                :plain="!article.is_favorite"
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
      <div v-else class="content-wrapper" ref="contentWrapperRef">
        <div class="content-header">
          <div class="content-title-row">
            <h1>{{ getDisplayTitle(selectedArticle) }}</h1>
            <el-button
              :icon="selectedArticle.is_favorite ? StarFilled : Star"
              :type="selectedArticle.is_favorite ? 'warning' : 'default'"
              :plain="!selectedArticle.is_favorite"
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
        
        <!-- AI 总结 -->
        <div v-if="showAiSummary && aiSummaryEnabled" class="ai-summary-section">
          <div class="ai-summary-header">
            <el-icon class="ai-summary-icon"><MagicStick /></el-icon>
            <span class="ai-summary-title">AI 总结</span>
            <el-button
              v-if="!articleSummary && !loadingSummary"
              size="small"
              type="primary"
              :loading="generatingSummary"
              @click="generateSummary"
            >
              生成总结
            </el-button>
            <el-button
              v-if="!articleSummary && loadingSummary"
              size="small"
              :loading="true"
            >
              生成中...
            </el-button>
          </div>
          <div v-if="articleSummary" class="ai-summary-content">
            {{ articleSummary }}
          </div>
          <div v-else-if="loadingSummary" class="ai-summary-loading">
            <el-icon class="is-loading"><Loading /></el-icon>
            <span>正在生成总结...</span>
          </div>
        </div>
        
        <el-divider v-if="showAiSummary && aiSummaryEnabled" />
        
        <div class="content-body-wrapper">
          <div class="content-body" v-html="getDisplayContent()"></div>
        </div>
        <div v-if="selectedArticle && selectedArticle.link" class="content-footer">
          <el-button type="primary" @click="openLink">查看原文</el-button>
          <el-button 
            type="success" 
            @click="fetchFullContent" 
            :loading="loadingFullContent"
            :disabled="!selectedArticle || !selectedArticle.link"
          >
            {{ hasFullContent ? '已获取全文' : '获取全文' }}
          </el-button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed, watch, onUnmounted, nextTick, inject } from 'vue'
import { useRoute } from 'vue-router'
import { Loading, Star, StarFilled, MagicStick } from '@element-plus/icons-vue'
import { ElMessage } from 'element-plus'
import { useArticleStore } from '~/stores/article'
import { useSettingsStore } from '~/stores/settings'
import { useFeedStore } from '~/stores/feed'

const route = useRoute()
const articleStore = useArticleStore()
const settingsStore = useSettingsStore()
const feedStore = useFeedStore()

// 文章列表相关
const articles = ref<any[]>([])
const selectedArticle = ref<any>(null)
const loading = ref(false)
const loadingMore = ref(false)
const hasMore = ref(true)
const selectedFeedId = ref<number | null>(null)
const selectedFeedName = ref<string>('')

// 全文内容相关
const loadingFullContent = ref(false)
const fullContent = ref<string | null>(null)
const hasFullContent = ref(false)

// 内容区域引用
const contentWrapperRef = ref<HTMLElement | null>(null)

// 计算显示的内容
const displayContent = computed(() => {
  if (fullContent.value && hasFullContent.value) {
    return fullContent.value
  }
  return selectedArticle.value?.content || ''
})

// 获取显示的标题（根据翻译设置）
const getDisplayTitle = (article: any) => {
  if (!article) return ''
  
  if (!aiTranslationEnabled.value) {
    return article.title || ''
  }

  // 如果已有翻译（包括检测到已经是目标语言的情况）
  if (article.title_translated && article.detected_language) {
    // 如果检测到的语言就是目标语言，不需要显示双语
    if (article.detected_language === targetLanguage.value) {
      return article.title || ''
    }
    
    if (translationPreference.value === 'bilingual') {
      // 双语对照：显示原文和译文（用分隔符）
      return `${article.title}\n${article.title_translated}`
    } else {
      // 只显示译文
      return article.title_translated
    }
  }

  // 如果没有翻译，显示原文（翻译可能正在进行中）
  return article.title || ''
}

// 获取显示的内容（根据翻译设置）
const getDisplayContent = () => {
  if (!selectedArticle.value) return ''
  
  const content = displayContent.value
  
  if (!aiTranslationEnabled.value) {
    return content
  }

  // 如果已有翻译（包括检测到已经是目标语言的情况）
  if (selectedArticle.value.content_translated) {
    // 如果检测到的语言就是目标语言，不需要显示双语
    if (selectedArticle.value.detected_language === targetLanguage.value) {
      return content
    }
    
    if (translationPreference.value === 'bilingual') {
      // 双语对照：显示原文和译文
      return `<div class="translation-bilingual">
        <div class="original-content">
          <h3>原文</h3>
          ${content}
        </div>
        <div class="translated-content">
          <h3>译文</h3>
          ${selectedArticle.value.content_translated}
        </div>
      </div>`
    } else {
      // 只显示译文
      return selectedArticle.value.content_translated
    }
  }

  // 如果没有翻译，显示原文（翻译可能正在进行中）
  if (translating.value) {
    return content + '<div style="text-align: center; padding: 20px; color: var(--text-tertiary);">正在翻译中...</div>'
  }
  
  return content
}

// 获取设置
const settings = ref({
  fontSize: 14,
  articlesPerPage: 20,
  aiSummary: false,
  aiTranslation: false,
  translationPreference: 'bilingual',
  aiOutputLanguage: 'en'
})

// AI 总结相关
const articleSummary = ref<string | null>(null)
const loadingSummary = ref(false)
const generatingSummary = ref(false)
const aiSummaryEnabled = computed(() => settings.value.aiSummary === true)
const showAiSummary = computed(() => selectedArticle.value !== null)

// AI 翻译相关
const aiTranslationEnabled = computed(() => settings.value.aiTranslation === true)
const translationPreference = computed(() => settings.value.translationPreference || 'bilingual')
const targetLanguage = computed(() => settings.value.aiOutputLanguage || 'en')
const translating = ref(false)

// 每页加载数量
const pageSize = computed(() => settings.value.articlesPerPage || 20)

onMounted(async () => {
  await loadSettings()
  await initFeedFromRoute()
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

// 从路由参数初始化选中的订阅源
const initFeedFromRoute = async () => {
  const feedIdParam = route.query.feedId
  if (feedIdParam) {
    const feedId = parseInt(feedIdParam as string)
    if (!isNaN(feedId)) {
      selectedFeedId.value = feedId
      // 获取订阅源名称
      try {
        const feeds = await feedStore.getAllFeeds()
        const feed = feeds.find(f => f.id === feedId)
        selectedFeedName.value = feed ? feed.title : ''
      } catch (error) {
        console.error('加载订阅源失败:', error)
      }
    }
  } else {
    selectedFeedId.value = null
    selectedFeedName.value = ''
  }
}

// 监听路由变化
watch(() => route.query.feedId, async (newFeedId) => {
  await initFeedFromRoute()
  await loadArticles(true)
  if (scrollContainer.value) {
    scrollContainer.value.scrollTop = 0
  }
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
      // 合并所有设置，优先使用直接保存的设置
      settings.value = { 
        ...settings.value, 
        ...saved,
        // 如果存在单独的常规设置键，也合并进来
        ...(saved.general || {})
      }
      applyFontSize(settings.value.fontSize)
      
      console.log('loadSettings: 设置已加载', {
        aiTranslation: settings.value.aiTranslation,
        translationPreference: settings.value.translationPreference,
        aiOutputLanguage: settings.value.aiOutputLanguage
      })
      
      // 如果启用翻译，为当前选中的文章加载翻译
      if (aiTranslationEnabled.value && selectedArticle.value) {
        await loadTranslation()
      }
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

// 监听 AI 总结设置变化
watch(() => settings.value.aiSummary, async (enabled) => {
  if (enabled && selectedArticle.value) {
    await loadArticleSummary()
  } else {
    articleSummary.value = null
  }
})

// 监听选中文章变化，加载总结
watch(() => selectedArticle.value?.id, async (newId) => {
  if (newId && aiSummaryEnabled.value) {
    await loadArticleSummary()
  }
  if (newId && aiTranslationEnabled.value) {
    await loadTranslation()
  }
})

// 监听翻译设置变化
watch(() => settings.value.aiTranslation, async (enabled) => {
  if (enabled && selectedArticle.value) {
    await loadTranslation()
  } else if (!enabled) {
    // 如果关闭翻译，清除翻译数据
    if (selectedArticle.value) {
      // 不清除数据库中的翻译，只是不显示
    }
  }
})

watch(() => settings.value.translationPreference, () => {
  // 翻译偏好改变时，重新渲染内容（不需要重新翻译）
})

watch(() => settings.value.aiOutputLanguage, async () => {
  // 目标语言改变时，清除旧的翻译并重新翻译
  if (aiTranslationEnabled.value && selectedArticle.value) {
    // 清除旧的翻译，强制重新翻译
    selectedArticle.value.title_translated = null
    selectedArticle.value.content_translated = null
    selectedArticle.value.detected_language = null
    await loadTranslation()
  }
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
    const newArticles = await articleStore.getAllArticles(
      pageSize.value, 
      offset, 
      selectedFeedId.value || undefined
    )
    
    if (reset) {
      articles.value = newArticles
    } else {
      articles.value = [...articles.value, ...newArticles]
    }

    // 如果启用翻译，为文章列表加载标题翻译（批量处理，避免过多请求）
    if (aiTranslationEnabled.value && newArticles.length > 0) {
      // 为所有新加载的文章翻译标题
      for (const article of newArticles) {
        if (!article.title_translated || !article.detected_language) {
          // 异步翻译，不阻塞列表显示
          loadArticleTranslation(article).catch(error => {
            // 静默失败，不影响列表显示
            console.error('标题翻译失败:', error)
          })
        }
      }
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

// 为文章列表中的文章加载翻译（仅标题）
const loadArticleTranslation = async (article: any) => {
  if (!article || !aiTranslationEnabled.value) {
    return
  }

  // 如果已有标题翻译，直接返回
  if (article.title_translated && article.detected_language) {
    return
  }

  try {
    const response = await $fetch('/api/article/translate', {
      method: 'POST',
      body: {
        articleId: article.id,
        translateTitleOnly: true  // 只翻译标题
      }
    })

    if ((response as any).success) {
      // 更新文章对象
      if ((response as any).titleTranslated) {
        article.title_translated = (response as any).titleTranslated
      }
      if ((response as any).detectedLanguage) {
        article.detected_language = (response as any).detectedLanguage
      }
      
      // 如果不需要翻译（已经是目标语言）
      if ((response as any).noTranslationNeeded) {
        article.detected_language = (response as any).detectedLanguage
        article.title_translated = article.title
      }
    }
  } catch (error) {
    // 静默失败，不影响列表显示
    console.error('文章标题翻译失败:', error)
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

const selectArticle = async (article: any) => {
  selectedArticle.value = article
  // 重置全文内容状态
  if (fullContent.value) {
    fullContent.value = null
  }
  if (hasFullContent.value) {
    hasFullContent.value = false
  }
  // 加载 AI 总结
  await loadArticleSummary()
  
  // 加载翻译（如果启用）
  if (aiTranslationEnabled.value) {
    await loadTranslation()
  }
  
  // 滚动内容区域到顶部
  await nextTick()
  if (contentWrapperRef.value) {
    contentWrapperRef.value.scrollTop = 0
  }
}

// 加载文章内容翻译（标题已在列表中翻译，这里只翻译内容）
const loadTranslation = async () => {
  if (!selectedArticle.value) {
    console.log('loadTranslation: 没有选中的文章')
    return
  }
  
  if (!aiTranslationEnabled.value) {
    console.log('loadTranslation: 翻译未启用')
    return
  }

  console.log('loadTranslation: 开始加载内容翻译，文章ID:', selectedArticle.value.id)

  // 如果内容已有翻译，直接使用（不需要重新翻译）
  if (selectedArticle.value.content_translated && selectedArticle.value.detected_language) {
    console.log('loadTranslation: 使用已有内容翻译')
    return
  }

  // 尝试从数据库加载或生成内容翻译
  translating.value = true
  try {
    console.log('loadTranslation: 调用内容翻译 API')
    const response = await $fetch('/api/article/translate', {
      method: 'POST',
      body: {
        articleId: selectedArticle.value.id,
        translateContentOnly: true  // 只翻译内容
      }
    })

    console.log('loadTranslation: API 响应:', response)

    if ((response as any).success) {
      // 更新本地文章对象
      if ((response as any).contentTranslated) {
        selectedArticle.value.content_translated = (response as any).contentTranslated
      }
      if ((response as any).detectedLanguage) {
        selectedArticle.value.detected_language = (response as any).detectedLanguage
      }
      
      // 如果不需要翻译（已经是目标语言），也更新检测到的语言
      if ((response as any).noTranslationNeeded) {
        selectedArticle.value.detected_language = (response as any).detectedLanguage
        // 将原文作为翻译结果（因为已经是目标语言）
        selectedArticle.value.content_translated = selectedArticle.value.content || selectedArticle.value.snippet || ''
        console.log('loadTranslation: 文章已经是目标语言，无需翻译')
      } else {
        console.log('loadTranslation: 内容翻译成功')
      }
    }
  } catch (error: any) {
    console.error('加载内容翻译失败:', error)
    // 显示错误提示
    if (error.data?.message) {
      console.error('翻译错误:', error.data.message)
      ElMessage.error(`内容翻译失败: ${error.data.message}`)
    } else if (error.message) {
      console.error('翻译错误:', error.message)
      ElMessage.error(`内容翻译失败: ${error.message}`)
    }
  } finally {
    translating.value = false
  }
}

// 加载文章 AI 总结
const loadArticleSummary = async () => {
  if (!selectedArticle.value || !aiSummaryEnabled.value) {
    articleSummary.value = null
    return
  }

  // 如果文章已有总结字段，直接使用
  if (selectedArticle.value.ai_summary) {
    articleSummary.value = selectedArticle.value.ai_summary
    return
  }

  // 尝试从数据库加载
  loadingSummary.value = true
  try {
    const response = await $fetch('/api/article/get-summary', {
      method: 'POST',
      body: {
        articleId: selectedArticle.value.id
      }
    })

    if ((response as any).success && (response as any).summary) {
      articleSummary.value = (response as any).summary
      // 更新本地文章对象
      selectedArticle.value.ai_summary = articleSummary.value
    } else {
      articleSummary.value = null
    }
  } catch (error: any) {
    console.error('加载总结失败:', error)
    articleSummary.value = null
  } finally {
    loadingSummary.value = false
  }
}

// 生成 AI 总结
const generateSummary = async () => {
  if (!selectedArticle.value) {
    return
  }

  generatingSummary.value = true
  loadingSummary.value = true
  try {
    const response = await $fetch('/api/article/generate-summary', {
      method: 'POST',
      body: {
        articleId: selectedArticle.value.id
      }
    })

    if ((response as any).success && (response as any).summary) {
      articleSummary.value = (response as any).summary
      // 更新本地文章对象
      selectedArticle.value.ai_summary = articleSummary.value
      ElMessage.success('总结生成成功')
    } else {
      ElMessage.error('生成总结失败')
    }
  } catch (error: any) {
    console.error('生成总结失败:', error)
    ElMessage.error(`生成总结失败: ${error.data?.message || error.message || '未知错误'}`)
  } finally {
    generatingSummary.value = false
    loadingSummary.value = false
  }
}

const refreshAll = async () => {
  loading.value = true
  try {
    await articleStore.refreshAllFeeds()
    await loadArticles(true) // 重置并重新加载
  } catch (error) {
    console.error('刷新失败:', error)
  } finally {
    loading.value = false
  }
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
    ElMessage.success(newFavorite ? '已收藏' : '已取消收藏')
  } catch (error: any) {
    ElMessage.error(error.message || '操作失败')
  }
}

const fetchFullContent = async () => {
  if (!selectedArticle.value?.link) {
    ElMessage.warning('文章链接不存在')
    return
  }

  loadingFullContent.value = true
  try {
    const response = await $fetch('/api/article/fetch-full', {
      method: 'POST',
      body: {
        url: selectedArticle.value.link
      }
    })

    if ((response as any).success && (response as any).content) {
      fullContent.value = (response as any).content
      hasFullContent.value = true
      ElMessage.success('已获取全文内容')
    } else {
      ElMessage.warning('未能获取到完整内容')
    }
  } catch (error: any) {
    console.error('获取全文失败:', error)
    ElMessage.error(`获取全文失败: ${error.data?.message || error.message || '未知错误'}`)
  } finally {
    loadingFullContent.value = false
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
  border-right: 1px solid var(--border-primary);
  display: flex;
  flex-direction: column;
  background-color: var(--bg-primary);
  overflow: hidden;
  height: 100%;
  transition: background-color 0.3s ease, border-color 0.3s ease;
}

.list-header {
  padding: 20px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  border-bottom: 1px solid var(--border-primary);
  flex-shrink: 0;
  background-color: var(--bg-primary);
  transition: background-color 0.3s ease, border-color 0.3s ease;
}

.list-header h2 {
  margin: 0;
  font-size: 18px;
  color: var(--text-primary);
  transition: color 0.3s ease;
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
  border: 1px solid var(--border-primary);
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.3s;
  background-color: var(--bg-card);
  color: var(--text-primary);
}

.article-item:hover {
  background-color: var(--bg-hover);
  border-color: var(--color-primary);
}

.article-item.active {
  background-color: var(--bg-active);
  border-color: var(--color-primary);
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
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  display: -webkit-box;
  -webkit-line-clamp: 4;  /* 增加行数，为双语标题提供足够空间 */
  line-clamp: 4;
  -webkit-box-orient: vertical;
  flex: 1;
  margin: 0;
  line-height: 1.5;  /* 增加行高，提高可读性 */
  min-height: 60px;  /* 最小高度，确保双语标题有足够空间 */
  transition: color 0.3s ease;
}

.favorite-btn {
  flex-shrink: 0;
  margin-top: 2px;
}

.article-meta {
  display: flex;
  justify-content: space-between;
  font-size: 12px;
  color: var(--text-tertiary);
  margin-bottom: 8px;
  transition: color 0.3s ease;
}

.feed-name {
  color: var(--color-primary);
  transition: color 0.3s ease;
}

.article-snippet {
  font-size: 14px;
  color: var(--text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  line-clamp: 2;
  -webkit-box-orient: vertical;
  transition: color 0.3s ease;
}

.article-content {
  flex: 1;
  min-width: 400px;
  overflow: hidden;
  background-color: var(--bg-tertiary);
  display: flex;
  flex-direction: column;
  transition: background-color 0.3s ease;
}

.content-wrapper {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
  padding: 40px;
  background-color: var(--bg-primary);
  /* width: 100%; */
  /* margin: 0 auto; */
  transition: background-color 0.3s ease, color 0.3s ease;
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
  color: var(--text-primary);
  flex: 1;
  transition: color 0.3s ease;
}

.content-header .favorite-btn {
  flex-shrink: 0;
  margin-top: 4px;
}

.content-meta {
  display: flex;
  gap: 20px;
  font-size: 14px;
  color: var(--text-tertiary);
  margin-bottom: 20px;
  transition: color 0.3s ease;
}

.content-body-wrapper {
  position: relative;
  isolation: isolate;
  contain: layout style paint;
  overflow: hidden;
  /* 创建新的层叠上下文，隔离内部样式 */
  z-index: 0;
}

.content-body {
  font-size: var(--article-font-size, 16px);
  line-height: 1.8;
  color: var(--text-secondary);
  position: relative;
  /* 重置可能造成问题的样式 */
  overflow-wrap: break-word;
  word-wrap: break-word;
  transition: color 0.3s ease;
}

/* 隔离文章内容样式，防止污染系统 */
/* 重置所有绝对定位和固定定位为相对定位 */
.content-body :deep([style*="position: absolute"]),
.content-body :deep([style*="position:fixed"]),
.content-body :deep([style*="position:absolute"]),
.content-body :deep([style*="position: fixed"]) {
  position: relative !important;
}

/* 重置可能造成问题的定位样式 */
.content-body :deep(div[style*="position"]),
.content-body :deep(span[style*="position"]),
.content-body :deep(p[style*="position"]),
.content-body :deep(img[style*="position"]),
.content-body :deep(a[style*="position"]),
.content-body :deep(section[style*="position"]),
.content-body :deep(article[style*="position"]),
.content-body :deep(header[style*="position"]),
.content-body :deep(footer[style*="position"]),
.content-body :deep(nav[style*="position"]) {
  position: relative !important;
}

/* 限制固定定位的元素 */
.content-body :deep([style*="fixed"]) {
  position: relative !important;
}

/* 防止溢出和定位问题 */
.content-body :deep(*) {
  box-sizing: border-box;
  max-width: 100%;
}

/* 重置 z-index，防止遮挡系统元素 */
.content-body :deep(*) {
  z-index: auto !important;
}

/* 确保图片不会超出容器 */
.content-body :deep(img) {
  max-width: 100% !important;
  height: auto !important;
  position: relative !important;
  display: block;
}

/* 防止iframe等嵌入内容造成问题 */
.content-body :deep(iframe),
.content-body :deep(embed),
.content-body :deep(object),
.content-body :deep(video) {
  max-width: 100% !important;
  position: relative !important;
  display: block;
}

/* 重置可能造成问题的浮动 */
.content-body :deep([style*="float"]) {
  float: none !important;
}

/* 防止固定定位的广告或弹窗 */
.content-body :deep(.fixed),
.content-body :deep([class*="fixed"]),
.content-body :deep([id*="fixed"]) {
  position: relative !important;
}

/* 限制绝对定位的容器 */
.content-body :deep([class*="absolute"]),
.content-body :deep([id*="absolute"]) {
  position: relative !important;
}

.loading-more,
.no-more {
  padding: 20px;
  text-align: center;
  color: var(--text-tertiary);
  font-size: 14px;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  transition: color 0.3s ease;
}

.loading-more .el-icon {
  font-size: 16px;
}

.content-body :deep(img) {
  max-width: 100%;
  height: auto;
}

.content-body :deep(a) {
  color: var(--color-primary);
  text-decoration: none;
  transition: color 0.3s ease;
}

.content-body :deep(a:hover) {
  color: var(--color-primary-hover);
  text-decoration: underline;
}

.content-footer {
  margin-top: 30px;
  padding-top: 20px;
  border-top: 1px solid var(--border-primary);
  transition: border-color 0.3s ease;
}

.ai-summary-section {
  margin-bottom: 30px;
  padding: 20px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border-radius: 8px;
  color: #fff;
}

.ai-summary-header {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 15px;
}

.ai-summary-icon {
  font-size: 20px;
  color: #fff;
}

.ai-summary-title {
  font-size: 18px;
  font-weight: 600;
  flex: 1;
}

.ai-summary-content {
  font-size: 15px;
  line-height: 1.8;
  color: rgba(255, 255, 255, 0.95);
}

.ai-summary-loading {
  display: flex;
  align-items: center;
  gap: 10px;
  color: rgba(255, 255, 255, 0.9);
  font-size: 14px;
}

.ai-summary-loading .el-icon {
  font-size: 16px;
}

/* 翻译双语对照样式 */
.translation-bilingual {
  display: flex;
  flex-direction: column;
  gap: 30px;
}

.translation-bilingual .original-content,
.translation-bilingual .translated-content {
  padding: 20px;
  border-radius: 8px;
}

.translation-bilingual .original-content {
  background-color: var(--bg-secondary);
  border: 1px solid var(--border-primary);
  transition: background-color 0.3s ease, border-color 0.3s ease;
}

.translation-bilingual .translated-content {
  background-color: var(--bg-active);
  border: 1px solid var(--color-primary);
  transition: background-color 0.3s ease, border-color 0.3s ease;
}

.translation-bilingual h3 {
  margin: 0 0 15px 0;
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
  padding-bottom: 10px;
  border-bottom: 2px solid var(--color-primary);
  transition: color 0.3s ease, border-color 0.3s ease;
}

.translation-bilingual .original-content h3 {
  color: var(--text-secondary);
  border-bottom-color: var(--text-tertiary);
}

.article-title {
  white-space: normal;
  word-break: break-word;
}

/* 深色主题样式已通过 CSS 变量自动适配，无需额外样式 */
</style>

