<template>
  <div class="settings-page">
    <!-- 左侧：设置分类 -->
    <div class="settings-menu">
      <div class="menu-header">
        <h2>设置</h2>
      </div>
      <el-menu
        :default-active="activeSetting"
        class="settings-menu-list"
        @select="handleSettingSelect"
      >
        <el-menu-item index="general">
          <el-icon><Setting /></el-icon>
          <span>常规设置</span>
        </el-menu-item>
        <el-menu-item index="sync">
          <el-icon><Refresh /></el-icon>
          <span>同步设置</span>
        </el-menu-item>
        <el-menu-item index="display">
          <el-icon><View /></el-icon>
          <span>显示设置</span>
        </el-menu-item>
        <el-menu-item index="feeds">
          <el-icon><Document /></el-icon>
          <span>订阅源管理</span>
        </el-menu-item>
        <el-menu-item index="opml">
          <el-icon><Upload /></el-icon>
          <span>OPML 管理</span>
        </el-menu-item>
        <el-menu-item index="ai">
          <el-icon><MagicStick /></el-icon>
          <span>AI接入</span>
        </el-menu-item>
      </el-menu>
    </div>

    <!-- 右侧：设置内容 -->
    <div class="settings-content">
      <div class="content-wrapper">
        <!-- 常规设置 -->
        <div v-if="activeSetting === 'general'" class="setting-section">
          <h3>常规设置</h3>
          <el-form :model="settings" label-width="150px">
            <el-form-item label="自动刷新间隔">
              <el-input-number
                v-model="settings.autoRefreshInterval"
                :min="5"
                :max="1440"
                :step="5"
              />
              <span class="text-hint">分钟</span>
            </el-form-item>
            <el-form-item label="启动时自动刷新">
              <el-switch v-model="settings.autoRefreshOnStart" />
            </el-form-item>
            <el-form-item label="保留文章数量">
              <el-input-number
                v-model="settings.maxArticles"
                :min="100"
                :max="10000"
                :step="100"
              />
              <span class="text-hint">条</span>
            </el-form-item>
            
            <el-divider />
            
            <el-form-item label="AI 总结">
              <el-switch v-model="settings.aiSummary" />
            </el-form-item>
            <el-form-item label="AI 翻译">
              <el-switch v-model="settings.aiTranslation" />
            </el-form-item>
            <el-form-item label="翻译偏好" v-if="settings.aiTranslation">
              <el-radio-group v-model="settings.translationPreference">
                <el-radio value="bilingual">双语对照</el-radio>
                <el-radio value="translated">只显示译文</el-radio>
              </el-radio-group>
            </el-form-item>
            <el-form-item label="AI 输出语言" v-if="settings.aiTranslation">
              <el-select
                v-model="settings.aiOutputLanguage"
                placeholder="选择输出语言"
                style="width: 200px;"
              >
                <el-option label="英语" value="en" />
                <el-option label="日语" value="ja" />
                <el-option label="韩语" value="ko" />
                <el-option label="法语" value="fr" />
                <el-option label="德语" value="de" />
                <el-option label="西班牙语" value="es" />
                <el-option label="俄语" value="ru" />
                <el-option label="意大利语" value="it" />
                <el-option label="葡萄牙语" value="pt" />
                <el-option label="中文" value="zh" />
              </el-select>
            </el-form-item>
            
            <el-form-item>
              <el-button type="primary" @click="saveSettings">保存设置</el-button>
            </el-form-item>
          </el-form>
        </div>

        <!-- 同步设置 -->
        <div v-if="activeSetting === 'sync'" class="setting-section">
          <h3>同步设置</h3>
          <el-form :model="settings" label-width="150px">
            <el-form-item label="后台自动刷新">
              <el-switch v-model="settings.backgroundRefresh" />
            </el-form-item>
            <el-form-item label="刷新超时时间">
              <el-input-number
                v-model="settings.refreshTimeout"
                :min="5"
                :max="60"
                :step="5"
              />
              <span class="text-hint">秒</span>
            </el-form-item>
            <el-form-item label="并发刷新数量">
              <el-input-number
                v-model="settings.concurrentRefresh"
                :min="1"
                :max="10"
              />
            </el-form-item>
            <el-form-item>
              <el-button type="primary" @click="saveSettings">保存设置</el-button>
            </el-form-item>
          </el-form>
        </div>

        <!-- 显示设置 -->
        <div v-if="activeSetting === 'display'" class="setting-section">
          <h3>显示设置</h3>
          <el-form :model="settings" label-width="150px">
            <el-form-item label="主题">
              <el-radio-group v-model="settings.theme">
                <el-radio value="light">浅色</el-radio>
                <el-radio value="dark">深色</el-radio>
                <el-radio value="auto">跟随系统</el-radio>
              </el-radio-group>
            </el-form-item>
            <el-form-item label="字体大小">
              <el-slider
                v-model="settings.fontSize"
                :min="12"
                :max="20"
                :step="1"
                show-stops
              />
              <span class="text-hint">{{ settings.fontSize }}px</span>
            </el-form-item>
            <el-form-item label="每页显示文章数">
              <el-input-number
                v-model="settings.articlesPerPage"
                :min="10"
                :max="100"
                :step="10"
              />
            </el-form-item>
            <el-form-item>
              <el-button type="primary" @click="saveSettings">保存设置</el-button>
            </el-form-item>
          </el-form>
        </div>

        <!-- 订阅源管理 -->
        <div v-if="activeSetting === 'feeds'" class="setting-section feeds-management">
          <div class="feeds-header">
            <h3>订阅源管理</h3>
            <el-button type="primary" @click="showAddDialog = true">添加订阅</el-button>
          </div>

          <!-- 分类筛选 -->
          <div class="feeds-filter" style="margin: 20px 0;">
            <el-select
              v-model="selectedCategoryFilter"
              placeholder="筛选分类"
              clearable
              style="width: 200px;"
              @change="loadFeeds"
            >
              <el-option label="全部" :value="null" />
              <el-option label="未分类" :value="'uncategorized'" />
              <el-option
                v-for="category in categories"
                :key="category.id"
                :label="category.name"
                :value="category.id"
              />
            </el-select>
          </div>

          <el-empty v-if="feeds.length === 0 && !loading && totalFeeds === 0" description="暂无订阅，点击上方按钮添加" />
          <el-skeleton v-else-if="loading" :rows="5" animated />
          
          <div v-else class="feeds-list-container">
            <el-table :data="feeds" stripe style="width: 100%">
              <el-table-column prop="title" label="标题" min-width="200" />
              <el-table-column prop="categoryName" label="分类" width="150">
                <template #default="{ row }">
                  <span v-if="row.categoryName">{{ row.categoryName }}</span>
                  <span v-else class="text-hint">未分类</span>
                </template>
              </el-table-column>
              <el-table-column prop="url" label="URL" min-width="300" show-overflow-tooltip />
              <el-table-column label="订阅状态" width="100" align="center">
                <template #default="{ row }">
                  <el-tag :type="row.is_subscribed ? 'success' : 'info'" size="small">
                    {{ row.is_subscribed ? '已订阅' : '未订阅' }}
                  </el-tag>
                </template>
              </el-table-column>
              <el-table-column prop="last_update" label="最后更新" width="180">
                <template #default="{ row }">
                  {{ formatDate(row.last_update) }}
                </template>
              </el-table-column>
              <el-table-column label="操作" width="280" fixed="right">
                <template #default="{ row }">
                  <el-button
                    size="small"
                    :type="row.is_subscribed ? 'warning' : 'success'"
                    @click="toggleSubscribe(row)"
                    :loading="row._subscribing"
                  >
                    {{ row.is_subscribed ? '取消订阅' : '订阅' }}
                  </el-button>
                  <el-button size="small" type="primary" @click="refreshFeed(row)">刷新</el-button>
                  <el-button size="small" @click="editFeed(row)">编辑</el-button>
                  <el-button size="small" type="danger" @click="deleteFeed(row)">删除</el-button>
                </template>
              </el-table-column>
            </el-table>
            
            <!-- 分页 -->
            <div class="pagination-container" style="margin-top: 20px; display: flex; justify-content: flex-end;">
              <el-pagination
                v-model:current-page="currentPage"
                v-model:page-size="pageSize"
                :page-sizes="[10, 20, 50, 100]"
                :total="totalFeeds"
                layout="total, sizes, prev, pager, next, jumper"
                @size-change="handleSizeChange"
                @current-change="handlePageChange"
              />
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
              <el-form-item label="分类">
                <el-select
                  v-model="newFeed.category_id"
                  placeholder="选择分类"
                  clearable
                  style="width: 100%;"
                >
                  <el-option label="未分类" :value="null" />
                  <el-option
                    v-for="category in categories"
                    :key="category.id"
                    :label="category.name"
                    :value="category.id"
                  />
                </el-select>
              </el-form-item>
            </el-form>
            <template #footer>
              <el-button @click="showAddDialog = false">取消</el-button>
              <el-button type="primary" @click="addFeed" :loading="adding">添加</el-button>
            </template>
          </el-dialog>

          <!-- 编辑订阅对话框 -->
          <el-dialog v-model="showEditDialog" title="编辑订阅" width="500px">
            <el-form v-if="editingFeed" :model="editingFeed" label-width="100px">
              <el-form-item label="标题" required>
                <el-input v-model="editingFeed.title" />
              </el-form-item>
              <el-form-item label="URL" required>
                <el-input v-model="editingFeed.url" disabled />
              </el-form-item>
              <el-form-item label="分类">
                <el-select
                  v-model="editingFeed.category_id"
                  placeholder="选择分类"
                  clearable
                  style="width: 100%;"
                >
                  <el-option label="未分类" :value="null" />
                  <el-option
                    v-for="category in categories"
                    :key="category.id"
                    :label="category.name"
                    :value="category.id"
                  />
                </el-select>
              </el-form-item>
              <el-form-item label="描述">
                <el-input v-model="editingFeed.description" type="textarea" :rows="3" />
              </el-form-item>
            </el-form>
            <template #footer>
              <el-button @click="showEditDialog = false">取消</el-button>
              <el-button type="primary" @click="saveFeed" :loading="saving" :disabled="!editingFeed">保存</el-button>
            </template>
          </el-dialog>
        </div>

        <!-- AI接入设置 -->
        <div v-if="activeSetting === 'ai'" class="setting-section">
          <h3>AI接入设置</h3>
          <el-form :model="aiSettings" label-width="150px">
            <el-form-item label="API Token" required>
              <el-input
                v-model="aiSettings.apiToken"
                type="password"
                placeholder="请输入 SiliconFlow API Token"
                show-password
                clearable
              />
              <div class="text-hint-small">
                在 <a href="https://siliconflow.cn" target="_blank" class="link-primary">SiliconFlow</a> 获取您的 API Token
              </div>
            </el-form-item>
            <el-form-item label="API 地址">
              <el-input
                v-model="aiSettings.apiEndpoint"
                placeholder="https://api.siliconflow.cn/v1/chat/completions"
              />
              <div class="text-hint-small">
                默认使用 SiliconFlow 官方 API 地址
              </div>
            </el-form-item>
            <el-form-item label="模型选择" required>
              <el-select
                v-model="aiSettings.model"
                placeholder="请选择模型"
                filterable
                style="width: 100%;"
              >
                <el-option
                  v-for="model in aiModels"
                  :key="model.value"
                  :label="model.label + (model.recommended ? ` (${model.recommendText || '推荐'})` : '')"
                  :value="model.value"
                >
                  <div style="display: flex; justify-content: space-between; align-items: center; width: 100%;">
                    <span>{{ model.label }}</span>
                    <el-tag v-if="model.recommended" size="small" type="success" effect="plain" style="margin-left: 8px;">
                      {{ model.recommendText || '推荐' }}
                    </el-tag>
                  </div>
                </el-option>
              </el-select>
            </el-form-item>
            <el-form-item label="Temperature">
              <el-slider
                v-model="aiSettings.temperature"
                :min="0"
                :max="2"
                :step="0.1"
                show-stops
                :show-tooltip="true"
              />
              <span class="text-hint">{{ aiSettings.temperature }}</span>
              <div class="text-hint-small">
                控制输出的随机性，值越大越随机（0-2）
              </div>
            </el-form-item>
            <el-form-item label="Max Tokens">
              <el-input-number
                v-model="aiSettings.maxTokens"
                :min="1"
                :max="32768"
                :step="100"
              />
              <span class="text-hint">最大生成token数</span>
            </el-form-item>
            <el-form-item label="Top P">
              <el-slider
                v-model="aiSettings.topP"
                :min="0"
                :max="1"
                :step="0.05"
                show-stops
                :show-tooltip="true"
              />
              <span class="text-hint">{{ aiSettings.topP }}</span>
              <div class="text-hint-small">
                核采样参数，控制输出的多样性（0-1）
              </div>
            </el-form-item>
            <el-form-item>
              <el-button type="primary" @click="saveAiSettings">保存设置</el-button>
              <el-button @click="testAiConnection" :loading="testingConnection">测试连接</el-button>
            </el-form-item>
          </el-form>
        </div>

        <!-- OPML 管理 -->
        <div v-if="activeSetting === 'opml'" class="setting-section opml-management">
          <h3>OPML 管理</h3>
          
          <!-- 导入 OPML -->
          <div class="opml-import-section">
            <h4>导入 OPML</h4>
            
            <!-- URL 导入 -->
            <el-form :model="opmlImport" label-width="100px" style="margin-bottom: 20px;">
              <el-form-item label="OPML URL">
                <el-input 
                  v-model="opmlImport.url" 
                  placeholder="请输入 OPML 文件的 URL 地址"
                  clearable
                >
                  <template #append>
                    <el-button 
                      type="primary" 
                      :loading="importing && importSource === 'url'"
                      :disabled="!opmlImport.url || importing"
                      @click="importOpmlFromUrl"
                    >
                      从 URL 导入
                    </el-button>
                  </template>
                </el-input>
              </el-form-item>
            </el-form>

            <!-- 文件导入 -->
            <div class="file-import-section">
              <el-upload
                ref="uploadRef"
                :auto-upload="false"
                :on-change="handleFileChange"
                :show-file-list="false"
                accept=".opml,.xml"
              >
                <template #trigger>
                  <el-button type="primary">选择 OPML 文件</el-button>
                </template>
              </el-upload>
              <el-button 
                type="success" 
                :disabled="!selectedOpmlFile || importing" 
                :loading="importing && importSource === 'file'"
                @click="importOpmlFromFile"
                style="margin-left: 10px;"
              >
                导入 OPML
              </el-button>
            </div>
            
            <div v-if="selectedOpmlFile" class="selected-file" style="margin-top: 10px;">
              <span>已选择: {{ selectedOpmlFile.name }}</span>
              <el-button size="small" @click="selectedOpmlFile = null">清除</el-button>
            </div>
          </div>

          <el-divider />

          <!-- 分类管理 -->
          <div class="categories-section">
            <div class="categories-header">
              <h4>分类管理</h4>
              <el-button type="primary" size="small" @click="showAddCategoryDialog = true">添加分类</el-button>
            </div>
            
            <el-empty v-if="categories.length === 0" description="暂无分类" />
            <el-table v-else :data="categories" stripe style="width: 100%; margin-top: 20px;">
              <el-table-column prop="name" label="分类名称" min-width="200" />
              <el-table-column prop="description" label="描述" min-width="300" />
              <el-table-column label="操作" width="150" fixed="right">
                <template #default="{ row }">
                  <el-button size="small" @click="editCategory(row)">编辑</el-button>
                  <el-button size="small" type="danger" @click="deleteCategory(row)">删除</el-button>
                </template>
              </el-table-column>
            </el-table>
          </div>

          <!-- 添加分类对话框 -->
          <el-dialog v-model="showAddCategoryDialog" title="添加分类" width="500px">
            <el-form :model="newCategory" label-width="100px">
              <el-form-item label="分类名称" required>
                <el-input v-model="newCategory.name" placeholder="请输入分类名称" />
              </el-form-item>
              <el-form-item label="描述">
                <el-input v-model="newCategory.description" type="textarea" :rows="3" placeholder="可选" />
              </el-form-item>
            </el-form>
            <template #footer>
              <el-button @click="showAddCategoryDialog = false">取消</el-button>
              <el-button type="primary" @click="addCategory" :loading="addingCategory">添加</el-button>
            </template>
          </el-dialog>

          <!-- 编辑分类对话框 -->
          <el-dialog v-model="showEditCategoryDialog" title="编辑分类" width="500px">
            <el-form :model="editingCategory" label-width="100px">
              <el-form-item label="分类名称" required>
                <el-input v-model="editingCategory.name" />
              </el-form-item>
              <el-form-item label="描述">
                <el-input v-model="editingCategory.description" type="textarea" :rows="3" />
              </el-form-item>
            </el-form>
            <template #footer>
              <el-button @click="showEditCategoryDialog = false">取消</el-button>
              <el-button type="primary" @click="saveCategory" :loading="savingCategory">保存</el-button>
            </template>
          </el-dialog>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { Setting, Refresh, View, Document, Upload, MagicStick } from '@element-plus/icons-vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { useSettingsStore } from '~/stores/settings'
import { useFeedStore } from '~/stores/feed'
import { useCategoryStore } from '~/stores/category'
import { updateAutoRefreshSettings } from '~/composables/useAutoRefresh'
import { useTheme } from '~/composables/useTheme'
import { parseOpml } from '~/composables/useOpml'

const settingsStore = useSettingsStore()
const feedStore = useFeedStore()
const categoryStore = useCategoryStore()
const { setTheme } = useTheme()
const activeSetting = ref('general')
const settings = ref({
  autoRefreshInterval: 30,
  autoRefreshOnStart: true,
  maxArticles: 1000,
  backgroundRefresh: true,
  refreshTimeout: 30,
  concurrentRefresh: 3,
  theme: 'light',
  fontSize: 14,
  articlesPerPage: 20,
  aiSummary: false,
  aiTranslation: false,
  translationPreference: 'bilingual',
  aiOutputLanguage: 'en'
})

// AI接入设置
const aiSettings = ref({
  apiToken: '',
  apiEndpoint: 'https://api.siliconflow.cn/v1/chat/completions',
  model: 'Qwen/Qwen2.5-7B-Instruct',
  temperature: 0.7,
  maxTokens: 2000,
  topP: 0.7
})

const testingConnection = ref(false)

// 常用模型列表
const aiModels = [
  { label: 'Qwen/Qwen2.5-7B-Instruct', value: 'Qwen/Qwen2.5-7B-Instruct', recommended: false },
  { label: 'Qwen/Qwen2.5-14B-Instruct', value: 'Qwen/Qwen2.5-14B-Instruct', recommended: false },
  { label: 'Qwen/Qwen2.5-32B-Instruct', value: 'Qwen/Qwen2.5-32B-Instruct', recommended: true, recommendText: '翻译推荐' },
  { label: 'Qwen/Qwen2.5-72B-Instruct', value: 'Qwen/Qwen2.5-72B-Instruct', recommended: true, recommendText: '翻译最准确' },
  { label: 'Qwen/Qwen2.5-Coder-7B-Instruct', value: 'Qwen/Qwen2.5-Coder-7B-Instruct', recommended: false },
  { label: 'Qwen/Qwen2.5-Coder-32B-Instruct', value: 'Qwen/Qwen2.5-Coder-32B-Instruct', recommended: false },
  { label: 'deepseek-ai/DeepSeek-V3', value: 'deepseek-ai/DeepSeek-V3', recommended: true, recommendText: '推荐' },
  { label: 'deepseek-ai/DeepSeek-R1', value: 'deepseek-ai/DeepSeek-R1', recommended: false },
  { label: 'THUDM/glm-4-9b-chat', value: 'THUDM/glm-4-9b-chat', recommended: false },
  { label: 'moonshotai/Kimi-K2-Thinking', value: 'moonshotai/Kimi-K2-Thinking', recommended: false },
  { label: 'zai-org/GLM-4.5', value: 'zai-org/GLM-4.5', recommended: false },
  { label: 'Qwen/QwQ-32B', value: 'Qwen/QwQ-32B', recommended: false }
]

// 订阅管理相关
const feeds = ref<any[]>([])
const loading = ref(false)
const showAddDialog = ref(false)
const showEditDialog = ref(false)
const adding = ref(false)
const saving = ref(false)
const selectedCategoryFilter = ref<number | string | null>(null)
const newFeed = ref({
  url: '',
  title: '',
  category_id: null as number | null
})
const editingFeed = ref<any>(null)

// 分页相关
const currentPage = ref(1)
const pageSize = ref(20)
const totalFeeds = ref(0)

// OPML 管理相关
const selectedOpmlFile = ref<File | null>(null)
const importing = ref(false)
const importSource = ref<'file' | 'url' | null>(null)
const uploadRef = ref()
const opmlImport = ref({
  url: ''
})

// 分类管理相关
const categories = ref<any[]>([])
const showAddCategoryDialog = ref(false)
const showEditCategoryDialog = ref(false)
const addingCategory = ref(false)
const savingCategory = ref(false)
const newCategory = ref({
  name: '',
  description: ''
})
const editingCategory = ref<any>(null)

onMounted(async () => {
  await loadSettings()
  // 如果当前是订阅管理页面，加载订阅列表
  if (activeSetting.value === 'feeds') {
    await loadFeeds()
  }
})

// 监听设置页面切换，如果是订阅管理或 OPML 管理则加载数据
watch(activeSetting, async (newVal) => {
  if (newVal === 'feeds') {
    currentPage.value = 1 // 重置到第一页
    await loadFeeds()
    await loadCategories() // 加载分类用于筛选
  } else if (newVal === 'opml') {
    await loadCategories()
  }
})

// 监听分类筛选变化，重置分页
watch(selectedCategoryFilter, () => {
  currentPage.value = 1 // 筛选时重置到第一页
  loadFeeds()
})

const loadSettings = async () => {
  try {
    const saved = await settingsStore.getSettings()
    if (saved) {
      settings.value = { ...settings.value, ...saved }
      // 加载AI设置
      if (saved.aiSettings) {
        aiSettings.value = { ...aiSettings.value, ...saved.aiSettings }
      }
    }
  } catch (error) {
    console.error('加载设置失败:', error)
  }
}

const saveSettings = async () => {
  try {
    await settingsStore.saveSettings(settings.value)
    
    // 应用设置
    // 更新自动刷新
    updateAutoRefreshSettings()
    
    // 应用主题
    if (settings.value.theme) {
      await setTheme(settings.value.theme)
    }
    
    // 应用字体大小
    if (settings.value.fontSize) {
      document.documentElement.style.setProperty('--article-font-size', `${settings.value.fontSize}px`)
    }
    
    ElMessage.success('设置已保存')
  } catch (error: any) {
    ElMessage.error(error.message || '保存失败')
  }
}

const handleSettingSelect = (index: string) => {
  activeSetting.value = index
}

// 订阅管理相关方法
const loadFeeds = async () => {
  loading.value = true
  try {
    console.log('[设置页面] 开始加载订阅源...', { 
      categoryFilter: selectedCategoryFilter.value,
      page: currentPage.value,
      pageSize: pageSize.value 
    })
    
    // 根据筛选条件加载订阅源
    let categoryId: number | null | undefined = undefined
    if (selectedCategoryFilter.value === 'uncategorized') {
      categoryId = null
    } else if (selectedCategoryFilter.value !== null) {
      categoryId = selectedCategoryFilter.value as number
    }
    
    // 获取总数
    totalFeeds.value = await feedStore.getFeedsCount(categoryId)
    console.log('[设置页面] 订阅源总数:', totalFeeds.value)
    
    // 计算偏移量
    const offset = (currentPage.value - 1) * pageSize.value
    
    // 加载分页数据
    const result = await feedStore.getAllFeedsForManagement(categoryId, pageSize.value, offset)
    console.log('[设置页面] 订阅源加载成功:', result)
    feeds.value = result || []
    if (feeds.value.length === 0 && totalFeeds.value > 0) {
      console.warn('[设置页面] 订阅源列表为空，但总数不为0，可能是分页问题')
    }
  } catch (error: any) {
    console.error('加载订阅失败:', error)
    const errorMsg = error?.message || error?.toString() || '未知错误'
    ElMessage.error(`加载订阅失败: ${errorMsg}`)
    feeds.value = [] // 确保设置为空数组，避免显示错误
    totalFeeds.value = 0
  } finally {
    loading.value = false
  }
}

const handleSizeChange = (size: number) => {
  pageSize.value = size
  currentPage.value = 1 // 重置到第一页
  loadFeeds()
}

const handlePageChange = (page: number) => {
  currentPage.value = page
  loadFeeds()
}

const toggleSubscribe = async (feed: any) => {
  feed._subscribing = true
  try {
    const isSubscribed = await feedStore.toggleSubscribe(feed.id)
    feed.is_subscribed = isSubscribed ? 1 : 0
    ElMessage.success(isSubscribed ? '已订阅' : '已取消订阅')
    
    // 无论订阅还是取消订阅，都刷新左侧菜单（因为全部文章只显示已订阅的）
    const event = new CustomEvent('feeds-updated')
    window.dispatchEvent(event)
  } catch (error: any) {
    ElMessage.error(error.message || '操作失败')
  } finally {
    feed._subscribing = false
  }
}

const loadCategories = async () => {
  try {
    console.log('[设置页面] 开始加载分类...')
    const result = await categoryStore.getAllCategories()
    console.log('[设置页面] 分类加载成功:', result)
    categories.value = result || []
    if (categories.value.length === 0) {
      console.warn('[设置页面] 分类列表为空')
    }
  } catch (error: any) {
    console.error('加载分类失败:', error)
    const errorMsg = error?.message || error?.toString() || '未知错误'
    ElMessage.error(`加载分类失败: ${errorMsg}`)
    categories.value = [] // 确保设置为空数组，避免显示错误
  }
}

const addFeed = async () => {
  if (!newFeed.value.url) {
    ElMessage.warning('请输入订阅地址')
    return
  }

  // 确保分类列表已加载
  if (categories.value.length === 0) {
    await loadCategories()
  }

  adding.value = true
  try {
    // 使用 addFeedWithCategory 方法，支持分类
    await feedStore.addFeedWithCategory(
      newFeed.value.url,
      newFeed.value.title,
      newFeed.value.category_id || null
    )
    ElMessage.success('添加成功')
    showAddDialog.value = false
    newFeed.value = { url: '', title: '', category_id: null }
    await loadFeeds()
    
    // 刷新左侧菜单（因为可能添加了新的订阅源）
    const event = new CustomEvent('feeds-updated')
    window.dispatchEvent(event)
  } catch (error: any) {
    ElMessage.error(error.message || '添加失败')
  } finally {
    adding.value = false
  }
}

const editFeed = async (feed: any) => {
  // 确保分类列表已加载
  if (categories.value.length === 0) {
    await loadCategories()
  }
  editingFeed.value = { ...feed }
  showEditDialog.value = true
}

// OPML 导入相关
const handleFileChange = (file: any) => {
  selectedOpmlFile.value = file.raw
}

// 从 URL 导入 OPML
const importOpmlFromUrl = async () => {
  if (!opmlImport.value.url) {
    ElMessage.warning('请输入 OPML URL')
    return
  }

  importing.value = true
  importSource.value = 'url'
  try {
    // 通过服务器端 API 获取 OPML 内容（避免 CORS 问题）
    const response = await $fetch('/api/opml/fetch', {
      method: 'POST',
      body: {
        url: opmlImport.value.url
      }
    })
    
    const text = (response as any).data
    await processOpmlImport(text)
    opmlImport.value.url = '' // 清空 URL
  } catch (error: any) {
    console.error('从 URL 导入 OPML 失败:', error)
    ElMessage.error(`导入失败: ${error.data?.message || error.message || '未知错误'}`)
  } finally {
    importing.value = false
    importSource.value = null
  }
}

// 从文件导入 OPML
const importOpmlFromFile = async () => {
  if (!selectedOpmlFile.value) {
    ElMessage.warning('请先选择 OPML 文件')
    return
  }

  importing.value = true
  importSource.value = 'file'
  try {
    const text = await selectedOpmlFile.value.text()
    await processOpmlImport(text)
    selectedOpmlFile.value = null
    uploadRef.value?.clearFiles()
  } catch (error: any) {
    console.error('从文件导入 OPML 失败:', error)
    ElMessage.error(`导入失败: ${error.message || '未知错误'}`)
  } finally {
    importing.value = false
    importSource.value = null
  }
}

// 处理 OPML 导入的通用逻辑
const processOpmlImport = async (opmlText: string) => {
  const opmlData = parseOpml(opmlText)

  if (opmlData.feeds.length === 0) {
    ElMessage.warning('OPML 文件中没有找到订阅源')
    return
  }

  let successCount = 0
  let failCount = 0
  const categoryMap = new Map<string, number>()

  // 导入订阅源
  const failedFeeds: Array<{ title: string, url: string, error: string }> = []
  
  for (const feed of opmlData.feeds) {
    try {
      // 处理分类
      let categoryId: number | null = null
      if (feed.category) {
        // 检查分类是否已存在
        if (!categoryMap.has(feed.category)) {
          const categoryId_ = await categoryStore.createCategory(feed.category)
          categoryMap.set(feed.category, categoryId_)
          console.log(`创建分类: ${feed.category} (ID: ${categoryId_})`)
        }
        categoryId = categoryMap.get(feed.category)!
        console.log(`订阅源 "${feed.title}" 将放入分类: ${feed.category} (ID: ${categoryId})`)
      } else {
        console.log(`订阅源 "${feed.title}" 没有分类，将作为未分类`)
      }

      // 尝试添加订阅源（带分类）
      try {
        const feedId = await feedStore.addFeedWithCategory(feed.url, feed.title, categoryId, feed.description)
        console.log(`成功添加订阅源 "${feed.title}" (ID: ${feedId}) 到分类 ID: ${categoryId}`)
        successCount++
      } catch (parseError: any) {
        // 如果 RSS 解析失败，尝试仅保存订阅源信息（不解析内容）
        try {
          const feedId = await feedStore.addFeedWithCategory(feed.url, feed.title, categoryId, feed.description, true)
          console.log(`订阅源 "${feed.title}" 解析失败，但已保存基本信息 (ID: ${feedId}) 到分类 ID: ${categoryId}`)
          successCount++
        } catch (saveError: any) {
          // 如果保存也失败，记录错误
          throw saveError
        }
      }
    } catch (error: any) {
      console.error(`导入订阅源失败: ${feed.title} (${feed.url})`, error)
      failCount++
      failedFeeds.push({
        title: feed.title || '未命名',
        url: feed.url,
        error: error.message || '未知错误'
      })
    }
  }

  // 显示导入结果
  if (failCount === 0) {
    ElMessage.success(`导入完成：成功 ${successCount} 个`)
  } else if (successCount === 0) {
    ElMessage.error(`导入失败：所有 ${failCount} 个订阅源都导入失败`)
  } else {
    ElMessage.warning(`导入完成：成功 ${successCount} 个，失败 ${failCount} 个`)
    // 如果有失败的，在控制台显示详细信息
    if (failedFeeds.length > 0) {
      console.group('导入失败的订阅源：')
      failedFeeds.forEach(feed => {
        console.warn(`${feed.title} (${feed.url}): ${feed.error}`)
      })
      console.groupEnd()
    }
  }
  
  // 刷新分类列表
  await loadCategories()
  
  // 如果当前在订阅源管理页面，刷新订阅列表
  if (activeSetting.value === 'feeds') {
    await loadFeeds()
  }
}

// 分类管理相关
const addCategory = async () => {
  if (!newCategory.value.name) {
    ElMessage.warning('请输入分类名称')
    return
  }

  addingCategory.value = true
  try {
    await categoryStore.createCategory(newCategory.value.name, newCategory.value.description)
    ElMessage.success('添加成功')
    showAddCategoryDialog.value = false
    newCategory.value = { name: '', description: '' }
    await loadCategories()
  } catch (error: any) {
    ElMessage.error(error.message || '添加失败')
  } finally {
    addingCategory.value = false
  }
}

const editCategory = (category: any) => {
  editingCategory.value = { ...category }
  showEditCategoryDialog.value = true
}

const saveCategory = async () => {
  if (!editingCategory.value) return

  savingCategory.value = true
  try {
    await categoryStore.updateCategory(
      editingCategory.value.id,
      editingCategory.value.name,
      editingCategory.value.description
    )
    ElMessage.success('保存成功')
    showEditCategoryDialog.value = false
    editingCategory.value = null
    await loadCategories()
  } catch (error: any) {
    ElMessage.error(error.message || '保存失败')
  } finally {
    savingCategory.value = false
  }
}

const deleteCategory = async (category: any) => {
  try {
    await ElMessageBox.confirm(
      `确定要删除分类 "${category.name}" 吗？该分类下的订阅源将变为未分类。`,
      '确认删除',
      {
        confirmButtonText: '确定',
        cancelButtonText: '取消',
        type: 'warning'
      }
    )
    await categoryStore.deleteCategory(category.id)
    ElMessage.success('删除成功')
    await loadCategories()
  } catch (error: any) {
    if (error !== 'cancel') {
      ElMessage.error(error.message || '删除失败')
    }
  }
}

const saveFeed = async () => {
  if (!editingFeed.value) return

  saving.value = true
  try {
    await feedStore.updateFeed(editingFeed.value.id, {
      title: editingFeed.value.title,
      description: editingFeed.value.description,
      category_id: editingFeed.value.category_id
    })
    ElMessage.success('保存成功')
    showEditDialog.value = false
    editingFeed.value = null
    await loadFeeds()
    
    // 刷新左侧菜单（因为分类可能改变了）
    const event = new CustomEvent('feeds-updated')
    window.dispatchEvent(event)
  } catch (error: any) {
    ElMessage.error(error.message || '保存失败')
  } finally {
    saving.value = false
  }
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
    
    // 检查删除后当前页是否还有数据
    const remainingCount = totalFeeds.value - 1
    const maxPage = Math.ceil(remainingCount / pageSize.value) || 1
    if (currentPage.value > maxPage) {
      currentPage.value = maxPage
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

// AI设置相关方法
const saveAiSettings = async () => {
  if (!aiSettings.value.apiToken) {
    ElMessage.warning('请输入 API Token')
    return
  }
  if (!aiSettings.value.model) {
    ElMessage.warning('请选择模型')
    return
  }

  try {
    await settingsStore.saveSettings({
      aiSettings: aiSettings.value
    })
    ElMessage.success('AI设置已保存')
  } catch (error: any) {
    ElMessage.error(error.message || '保存失败')
  }
}

const testAiConnection = async () => {
  if (!aiSettings.value.apiToken) {
    ElMessage.warning('请输入 API Token')
    return
  }
  if (!aiSettings.value.model) {
    ElMessage.warning('请选择模型')
    return
  }

  testingConnection.value = true
  try {
    const response = await $fetch(aiSettings.value.apiEndpoint, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${aiSettings.value.apiToken}`,
        'Content-Type': 'application/json'
      },
      body: {
        model: aiSettings.value.model,
        messages: [
          {
            role: 'user',
            content: '你好'
          }
        ],
        max_tokens: 50,
        temperature: aiSettings.value.temperature,
        top_p: aiSettings.value.topP
      }
    })

    if (response && (response as any).choices) {
      ElMessage.success('连接测试成功！')
    } else {
      ElMessage.warning('连接测试完成，但响应格式异常')
    }
  } catch (error: any) {
    console.error('AI连接测试失败:', error)
    ElMessage.error(`连接测试失败: ${error.data?.message || error.message || '未知错误'}`)
  } finally {
    testingConnection.value = false
  }
}
</script>

<style scoped>
.settings-page {
  display: flex;
  height: 100%;
  width: 100%;
  overflow: hidden;
}

.settings-menu {
  width: 250px;
  border-right: 1px solid var(--border-primary);
  background-color: var(--bg-primary);
  display: flex;
  flex-direction: column;
  transition: background-color 0.3s ease, border-color 0.3s ease;
}

.menu-header {
  padding: 20px;
  border-bottom: 1px solid var(--border-primary);
  background-color: var(--bg-primary);
  transition: background-color 0.3s ease, border-color 0.3s ease;
}

.menu-header h2 {
  margin: 0;
  font-size: 18px;
  color: var(--text-primary);
  transition: color 0.3s ease;
}

.settings-menu-list {
  border-right: none;
  flex: 1;
  background-color: var(--bg-primary);
}

.settings-content {
  flex: 1;
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
  color: var(--text-primary);
  transition: background-color 0.3s ease, color 0.3s ease;
}

.setting-section h3 {
  margin: 0 0 30px 0;
  font-size: 20px;
  color: var(--text-primary);
  transition: color 0.3s ease;
}

.feeds-management {
  width: 100%;
}

.feeds-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.feeds-header h3 {
  margin: 0;
  color: var(--text-primary);
  transition: color 0.3s ease;
}

.feeds-list-container {
  margin-top: 20px;
}

.opml-import-section {
  margin-bottom: 30px;
}

.opml-import-section h4 {
  margin-bottom: 15px;
  font-size: 16px;
  color: var(--text-primary);
  transition: color 0.3s ease;
}

.file-import-section {
  display: flex;
  align-items: center;
  gap: 10px;
}

.selected-file {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px;
  background-color: var(--bg-secondary);
  border-radius: 4px;
  font-size: 14px;
  color: var(--text-tertiary);
  transition: background-color 0.3s ease, color 0.3s ease;
}

/* 文本提示样式 */
.text-hint {
  margin-left: 10px;
  color: var(--text-tertiary);
  transition: color 0.3s ease;
}

.text-hint-small {
  margin-top: 5px;
  color: var(--text-tertiary);
  font-size: 12px;
  transition: color 0.3s ease;
}

.link-primary {
  color: var(--color-primary);
  text-decoration: none;
  transition: color 0.3s ease;
}

.link-primary:hover {
  color: var(--color-primary-hover);
  text-decoration: underline;
}

/* Element Plus 菜单主题样式（使用 CSS 变量） */
.settings-menu-list {
  background-color: var(--bg-primary) !important;
}

.settings-menu-list :deep(.el-menu-item) {
  color: var(--text-primary) !important;
  background-color: transparent !important;
  transition: background-color 0.3s ease, color 0.3s ease;
}

.settings-menu-list :deep(.el-menu-item:hover) {
  background-color: var(--bg-hover) !important;
  color: var(--color-primary) !important;
}

.settings-menu-list :deep(.el-menu-item.is-active) {
  background-color: var(--bg-active) !important;
  color: var(--color-primary) !important;
}

.settings-menu-list :deep(.el-menu-item .el-icon),
.settings-menu-list :deep(.el-menu-item span) {
  color: inherit !important;
}

.settings-menu-list :deep(.el-menu-item.is-active .el-icon),
.settings-menu-list :deep(.el-menu-item.is-active span),
.settings-menu-list :deep(.el-menu-item:hover .el-icon),
.settings-menu-list :deep(.el-menu-item:hover span) {
  color: var(--color-primary) !important;
}

/* Element Plus 表单标签颜色 */
.setting-section :deep(.el-form-item__label) {
  color: var(--text-primary) !important;
  transition: color 0.3s ease;
}

/* Element Plus 对话框颜色 */
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

/* 对话框内的占位符文本 */
:deep(.el-input__inner::placeholder),
:deep(.el-textarea__inner::placeholder) {
  color: var(--text-tertiary) !important;
}

/* 对话框内的必填标记 */
:deep(.el-form-item.is-required .el-form-item__label::before) {
  color: var(--color-primary) !important;
}

/* Element Plus 表格颜色 */
:deep(.el-table) {
  background-color: var(--bg-primary) !important;
  color: var(--text-primary) !important;
}

:deep(.el-table th) {
  background-color: var(--bg-secondary) !important;
  color: var(--text-primary) !important;
  border-bottom-color: var(--border-primary) !important;
}

:deep(.el-table td) {
  background-color: var(--bg-primary) !important;
  color: var(--text-primary) !important;
  border-bottom-color: var(--border-primary) !important;
}

:deep(.el-table tr:hover > td) {
  background-color: var(--bg-hover) !important;
}

:deep(.el-table--striped .el-table__body tr.el-table__row--striped td) {
  background-color: var(--bg-secondary) !important;
}

:deep(.el-table--striped .el-table__body tr.el-table__row--striped:hover > td) {
  background-color: var(--bg-hover) !important;
}

/* Element Plus 标签颜色 */
:deep(.el-tag) {
  background-color: var(--bg-secondary) !important;
  border-color: var(--border-primary) !important;
  color: var(--text-primary) !important;
}

:deep(.el-tag.el-tag--success) {
  background-color: var(--bg-active) !important;
  border-color: var(--color-primary) !important;
  color: var(--color-primary) !important;
}

/* Element Plus 空状态颜色 */
:deep(.el-empty) {
  color: var(--text-tertiary) !important;
}

:deep(.el-empty__description) {
  color: var(--text-tertiary) !important;
}

/* Element Plus 分页颜色 */
:deep(.el-pagination) {
  color: var(--text-primary) !important;
}

:deep(.el-pagination .el-pagination__total),
:deep(.el-pagination .el-pagination__jump),
:deep(.el-pagination button),
:deep(.el-pagination .number) {
  color: var(--text-primary) !important;
}

:deep(.el-pagination button:disabled) {
  color: var(--text-disabled) !important;
}

:deep(.el-pagination .number.is-active) {
  background-color: var(--color-primary) !important;
  color: #fff !important;
}

:deep(.el-pagination .number:hover) {
  color: var(--color-primary) !important;
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

