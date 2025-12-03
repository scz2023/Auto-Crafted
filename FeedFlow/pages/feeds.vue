<template>
  <div class="feeds-page">
    <!-- 左侧：订阅列表 -->
    <div class="feeds-list">
      <div class="list-header">
        <h2>订阅管理</h2>
        <el-button type="primary" @click="showAddDialog = true">添加订阅</el-button>
      </div>
      
      <div class="feeds-scroll-container">
        <el-empty v-if="feeds.length === 0" description="暂无订阅" />
        <div v-else class="feeds-container">
          <div
            v-for="feed in feeds"
            :key="feed.id"
            class="feed-item"
            :class="{ active: selectedFeed?.id === feed.id }"
            @click="selectFeed(feed)"
          >
            <div class="feed-title">{{ feed.title }}</div>
            <div class="feed-url">{{ feed.url }}</div>
            <div class="feed-actions">
              <el-button size="small" type="primary" @click.stop="refreshFeed(feed)">刷新</el-button>
              <el-button size="small" type="danger" @click.stop="deleteFeed(feed)">删除</el-button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 右侧：订阅详情/编辑 -->
    <div class="feed-detail">
      <el-empty v-if="!selectedFeed" description="请选择一个订阅" />
      <div v-else class="detail-wrapper">
        <h2>订阅详情</h2>
        <el-form :model="selectedFeed" label-width="100px">
          <el-form-item label="标题">
            <el-input v-model="selectedFeed.title" />
          </el-form-item>
          <el-form-item label="URL">
            <el-input v-model="selectedFeed.url" />
          </el-form-item>
          <el-form-item label="描述">
            <el-input v-model="selectedFeed.description" type="textarea" :rows="3" />
          </el-form-item>
          <el-form-item label="最后更新">
            <span>{{ formatDate(selectedFeed.lastUpdate) }}</span>
          </el-form-item>
          <el-form-item>
            <el-button type="primary" @click="saveFeed">保存</el-button>
            <el-button @click="cancelEdit">取消</el-button>
          </el-form-item>
        </el-form>
      </div>
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
import { ref, onMounted, watch } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { useRoute, useRouter } from 'vue-router'
import { useFeedStore } from '~/stores/feed'

const route = useRoute()
const router = useRouter()
const feedStore = useFeedStore()
const feeds = ref<any[]>([])
const selectedFeed = ref<any>(null)
const showAddDialog = ref(false)
const adding = ref(false)
const newFeed = ref({
  url: '',
  title: ''
})

onMounted(async () => {
  await loadFeeds()
  // 检查路由参数，如果存在 add=true 则打开添加对话框
  if (route.query.add === 'true') {
    showAddDialog.value = true
  }
})

// 监听路由变化，如果查询参数变化则打开对话框
watch(() => route.query.add, (value) => {
  if (value === 'true') {
    showAddDialog.value = true
  }
})

const loadFeeds = async () => {
  try {
    // 使用包含订阅状态的完整列表（已订阅 + 未订阅）
    feeds.value = await feedStore.getAllFeedsWithStatus()
  } catch (error) {
    console.error('加载订阅失败:', error)
    ElMessage.error('加载订阅失败')
  }
}

const selectFeed = (feed: any) => {
  selectedFeed.value = { ...feed }
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
    // 清除路由查询参数
    if (route.query.add) {
      router.replace({ path: '/feeds', query: {} })
    }
    await loadFeeds()
  } catch (error: any) {
    ElMessage.error(error.message || '添加失败')
  } finally {
    adding.value = false
  }
}

// 监听对话框关闭，清除查询参数
watch(showAddDialog, (value) => {
  if (!value && route.query.add) {
    router.replace({ path: '/feeds', query: {} })
  }
})

const saveFeed = async () => {
  try {
    await feedStore.updateFeed(selectedFeed.value.id, selectedFeed.value)
    ElMessage.success('保存成功')
    await loadFeeds()
  } catch (error: any) {
    ElMessage.error(error.message || '保存失败')
  }
}

const cancelEdit = () => {
  selectedFeed.value = null
}

const refreshFeed = async (feed: any) => {
  try {
    await feedStore.refreshFeed(feed.id)
    ElMessage.success('刷新成功')
    await loadFeeds()
  } catch (error: any) {
    ElMessage.error(error.message || '刷新失败')
  }
}

const deleteFeed = async (feed: any) => {
  try {
    await ElMessageBox.confirm('确定要删除这个订阅吗？', '提示', {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      type: 'warning'
    })
    
    await feedStore.deleteFeed(feed.id)
    ElMessage.success('删除成功')
    if (selectedFeed.value?.id === feed.id) {
      selectedFeed.value = null
    }
    await loadFeeds()
  } catch (error: any) {
    if (error !== 'cancel') {
      ElMessage.error(error.message || '删除失败')
    }
  }
}

const formatDate = (date: string | Date) => {
  if (!date) return '从未更新'
  const d = new Date(date)
  return d.toLocaleString('zh-CN')
}
</script>

<style scoped>
.feeds-page {
  display: flex;
  height: 100%;
  width: 100%;
  overflow: hidden;
}

.feeds-list {
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

.feeds-scroll-container {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
}

.feeds-container {
  padding: 10px;
}

.feed-item {
  padding: 15px;
  margin-bottom: 10px;
  border: 1px solid #e4e7ed;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.3s;
}

.feed-item:hover {
  background-color: #f5f7fa;
  border-color: #409eff;
}

.feed-item.active {
  background-color: #ecf5ff;
  border-color: #409eff;
}

.feed-title {
  font-size: 16px;
  font-weight: 600;
  margin-bottom: 8px;
  color: #303133;
}

.feed-url {
  font-size: 12px;
  color: #909399;
  margin-bottom: 10px;
  word-break: break-all;
}

.feed-actions {
  display: flex;
  gap: 10px;
}

.feed-detail {
  flex: 1;
  min-width: 400px;
  overflow: hidden;
  background-color: #fafafa;
  display: flex;
  flex-direction: column;
}

.detail-wrapper {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
  max-width: 600px;
  margin: 0 auto;
  padding: 40px;
  background-color: #fff;
  width: 100%;
}

.detail-wrapper h2 {
  margin: 0 0 30px 0;
  font-size: 20px;
  color: #303133;
}

/* 深色主题样式 */
html.dark .feeds-list {
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

html.dark .feed-item {
  background-color: #2a2a2a;
  border-color: #333;
  color: #e5e5e5;
}

html.dark .feed-item:hover {
  background-color: #333;
  border-color: #409eff;
}

html.dark .feed-item.active {
  background-color: #2d4a5f;
  border-color: #409eff;
}

html.dark .feed-title {
  color: #e5e5e5;
}

html.dark .feed-url {
  color: #909399;
}

html.dark .feed-detail {
  background-color: #1a1a1a;
}

html.dark .detail-wrapper {
  background-color: #1a1a1a;
  color: #e5e5e5;
}

html.dark .detail-wrapper h2 {
  color: #e5e5e5;
}
</style>

