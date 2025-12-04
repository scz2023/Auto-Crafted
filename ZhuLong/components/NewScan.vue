<template>
  <div class="new-scan">
    <el-card class="scan-card">
      <template #header>
        <div class="card-header">
          <span>新建安全扫描</span>
        </div>
      </template>

      <el-form :model="scanConfig" label-width="120px" :rules="rules" ref="formRef" class="scan-form">
        <el-form-item label="扫描目标" prop="targets" required>
          <el-input
            v-model="targetInput"
            type="textarea"
            :rows="3"
            placeholder="输入目标 URL、GitHub 仓库、本地路径或 IP 地址，每行一个目标"
            @blur="parseTargets"
          />
          <div class="target-tags" v-if="scanConfig.targets.length > 0">
            <el-tag
              v-for="(target, index) in scanConfig.targets"
              :key="index"
              closable
              @close="removeTarget(index)"
              style="margin: 4px 4px 0 0"
            >
              {{ target }}
            </el-tag>
          </div>
        </el-form-item>

        <el-form-item label="自定义指令" prop="instruction">
          <el-input
            v-model="scanConfig.instruction"
            type="textarea"
            :rows="3"
            placeholder="可选：指定扫描重点，例如：'专注于 IDOR 和 XSS 漏洞' 或 '使用以下凭证进行认证测试：admin:password123'"
          />
        </el-form-item>

        <el-form-item label="运行名称" prop="runName">
          <el-input
            v-model="scanConfig.runName"
            placeholder="可选：自定义扫描运行名称，留空将自动生成"
          />
        </el-form-item>

        <el-alert
          type="info"
          :closable="false"
          show-icon
          style="margin-bottom: 16px"
        >
          <template #default>
            <span>LLM 配置请在 <strong>设置</strong> 页面进行配置和保存。扫描时将自动使用设置中的 LLM 配置。</span>
          </template>
        </el-alert>

        <el-form-item>
          <el-checkbox v-model="scanConfig.nonInteractive">
            非交互模式（后台运行，完成后自动退出）
          </el-checkbox>
        </el-form-item>

        <el-form-item>
          <el-button
            type="primary"
            @click="startScan"
            :loading="scanning"
            :disabled="!canStartScan"
            size="large"
          >
            <el-icon><VideoPlay /></el-icon>
            开始扫描
          </el-button>
          <el-button @click="checkEnvironment" :loading="checking" size="large">
            <el-icon><Tools /></el-icon>
            检查环境
          </el-button>
        </el-form-item>
      </el-form>
    </el-card>

    <!-- 环境检查结果 -->
    <el-card v-if="envCheckResult" class="env-check-card" style="margin-top: 20px">
      <template #header>
        <div class="card-header">
          <span>环境检查结果</span>
          <el-button
            text
            circle
            @click="closeEnvCheck"
            class="close-btn"
            title="关闭"
          >
            <el-icon><Close /></el-icon>
          </el-button>
        </div>
      </template>
      <div class="env-status">
        <div class="env-item">
          <el-icon :class="envCheckResult.installed ? 'success' : 'error'">
            <Check v-if="envCheckResult.installed" />
            <Close v-else />
          </el-icon>
          <span>Docker: {{ envCheckResult.installed ? '已安装' : '未安装' }}</span>
        </div>
        <div class="env-item" v-if="envCheckResult.installed">
          <el-icon :class="envCheckResult.running ? 'success' : 'warning'">
            <Check v-if="envCheckResult.running" />
            <Close v-else />
          </el-icon>
          <span>Docker 运行状态: {{ envCheckResult.running ? '运行中' : '未运行' }}</span>
        </div>
        <div class="env-item" v-if="envCheckResult.installed && envCheckResult.running">
          <el-icon :class="envCheckResult.image_pulled ? 'success' : 'error'">
            <Check v-if="envCheckResult.image_pulled" />
            <Close v-else />
          </el-icon>
          <span>Strix 镜像: {{ envCheckResult.image_pulled ? (envCheckResult.image_name ? `已拉取 (${envCheckResult.image_name})` : '已拉取') : '未拉取' }}</span>
        </div>
      </div>
      <div class="env-description" v-if="envCheckResult.message">
        <el-divider />
        <div class="description-content">
          <p><strong>检查结果：</strong></p>
          <p>{{ envCheckResult.message }}</p>
          <div v-if="!envCheckResult.image_pulled && envCheckResult.installed && envCheckResult.running" class="build-section">
            <p><strong>提示：</strong>请运行以下命令构建 Strix 镜像，或点击下方按钮一键构建：</p>
            <pre class="command-hint">cd strix-0.4.0
docker build -f containers/Dockerfile -t strix:0.4.0 .</pre>
            <el-button
              type="primary"
              @click="showBuildDialog = true"
              :loading="building"
              style="margin-top: 12px"
            >
              <el-icon><Tools /></el-icon>
              一键构建镜像
            </el-button>
          </div>
        </div>
      </div>
    </el-card>

    <!-- Docker 构建对话框 -->
    <el-dialog
      v-model="showBuildDialog"
      title="构建 Strix Docker 镜像"
      width="80%"
      :close-on-click-modal="false"
      :close-on-press-escape="false"
      :show-close="!building"
    >
      <div class="build-dialog">
        <el-alert
          type="info"
          :closable="false"
          style="margin-bottom: 16px"
        >
          <template #default>
            <p>正在构建 Docker 镜像，这可能需要几分钟时间，请耐心等待...</p>
            <p style="margin-top: 8px; font-size: 12px; color: #666;">
              构建过程会实时显示在下方，构建完成后会自动关闭此对话框。
            </p>
          </template>
        </el-alert>
        
        <div class="build-logs" ref="buildLogsContainer">
          <div
            v-for="(log, index) in buildLogs"
            :key="index"
            :class="['build-log-line', `log-${log.level}`]"
          >
            <span class="log-time">{{ log.time }}</span>
            <span class="log-message">{{ log.message }}</span>
          </div>
          <div v-if="buildLogs.length === 0" class="build-log-placeholder">
            等待构建输出...
          </div>
        </div>
      </div>
      
      <template #footer>
        <el-button @click="cancelBuild" :disabled="building">取消</el-button>
        <el-button type="primary" @click="closeBuildDialog" :disabled="building">关闭</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { VideoPlay, Tools, Check, Close } from '@element-plus/icons-vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

const emit = defineEmits(['scan-started'])

const formRef = ref()
const scanning = ref(false)
const checking = ref(false)
const building = ref(false)
const showBuildDialog = ref(false)
const buildLogs = ref<Array<{ time: string; level: string; message: string }>>([])
const buildLogsContainer = ref<HTMLElement | null>(null)
let buildLogUnsubscribe: (() => void) | null = null

const targetInput = ref('')
const envCheckResult = ref<{
  installed: boolean;
  running?: boolean;
  image_pulled: boolean;
  image_name?: string;
  message?: string;
} | null>(null)

const scanConfig = ref({
  targets: [] as string[],
  instruction: '',
  runName: '',
  nonInteractive: false,
})


const rules = {
  targets: [{ required: true, message: '请至少输入一个扫描目标', trigger: 'blur' }],
}

const canStartScan = computed(() => {
  return scanConfig.value.targets.length > 0
})

const parseTargets = () => {
  const lines = targetInput.value
    .split('\n')
    .map(line => line.trim())
    .filter(line => line.length > 0)
  scanConfig.value.targets = [...new Set(lines)]
}

const removeTarget = (index: number) => {
  scanConfig.value.targets.splice(index, 1)
}

const closeEnvCheck = () => {
  envCheckResult.value = null
}


const scrollBuildLogsToBottom = () => {
  nextTick(() => {
    if (buildLogsContainer.value) {
      buildLogsContainer.value.scrollTop = buildLogsContainer.value.scrollHeight
    }
  })
}

const startBuildImage = async () => {
  building.value = true
  buildLogs.value = []
  showBuildDialog.value = true
  
  // 先添加初始消息
  const now = new Date()
  const timeStr = `${now.getHours().toString().padStart(2, '0')}:${now.getMinutes().toString().padStart(2, '0')}:${now.getSeconds().toString().padStart(2, '0')}`
  buildLogs.value.push({
    time: timeStr,
    level: 'info',
    message: '正在初始化构建...'
  })
  
  // 监听构建日志事件（必须在调用 invoke 之前设置）
  try {
    buildLogUnsubscribe = await listen('docker-build-log', (event: any) => {
      try {
        const payload = event.payload as { level: string; message: string }
        
        if (!payload || !payload.message) {
          console.warn('无效的事件 payload:', payload)
          return
        }
        
        const now = new Date()
        const timeStr = `${now.getHours().toString().padStart(2, '0')}:${now.getMinutes().toString().padStart(2, '0')}:${now.getSeconds().toString().padStart(2, '0')}`
        
        // 立即添加到日志列表
        buildLogs.value.push({
          time: timeStr,
          level: payload.level || 'info',
          message: payload.message || ''
        })
        
        // 立即滚动到底部
        scrollBuildLogsToBottom()
        
        // 如果构建成功或失败，更新状态
        if (payload.level === 'success') {
          building.value = false
          ElMessage.success('Docker 镜像构建成功！')
          // 自动刷新环境检查
          setTimeout(() => {
            checkEnvironment()
            // 3秒后自动关闭对话框
            setTimeout(() => {
              closeBuildDialog()
            }, 3000)
          }, 1000)
        } else if (payload.level === 'error' && payload.message.includes('构建失败')) {
          building.value = false
          ElMessage.error('Docker 镜像构建失败，请查看日志')
        }
      } catch (error) {
        console.error('处理构建日志事件失败:', error)
      }
    })
    
    console.log('✅ 已设置构建日志监听器')
    
    // 等待一小段时间确保监听器已注册
    await new Promise(resolve => setTimeout(resolve, 100))
  } catch (error) {
    console.error('❌ 监听构建日志失败:', error)
    ElMessage.error('无法监听构建日志')
    building.value = false
    return
  }
  
  // 开始构建（异步执行，不等待完成）
  invoke('build_strix_image').then(() => {
    console.log('构建命令执行完成')
  }).catch((error: any) => {
    const errorMessage = error?.message || error?.toString() || '未知错误'
    console.error('构建镜像失败:', error)
    ElMessage.error(`构建镜像失败: ${errorMessage}`)
    building.value = false
    
    const now = new Date()
    const timeStr = `${now.getHours().toString().padStart(2, '0')}:${now.getMinutes().toString().padStart(2, '0')}:${now.getSeconds().toString().padStart(2, '0')}`
    buildLogs.value.push({
      time: timeStr,
      level: 'error',
      message: `构建失败: ${errorMessage}`
    })
  })
}

const cancelBuild = () => {
  if (building.value) {
    ElMessage.warning('构建正在进行中，无法取消。请等待构建完成。')
  } else {
    closeBuildDialog()
  }
}

const closeBuildDialog = () => {
  if (buildLogUnsubscribe) {
    buildLogUnsubscribe()
    buildLogUnsubscribe = null
  }
  showBuildDialog.value = false
  buildLogs.value = []
}

onMounted(() => {
  // 组件挂载时的初始化
})

onUnmounted(() => {
  // 组件卸载时清理
  if (buildLogUnsubscribe) {
    buildLogUnsubscribe()
    buildLogUnsubscribe = null
  }
})

const checkEnvironment = async () => {
  checking.value = true
  envCheckResult.value = null
  
  try {
    // 检查 Docker 和镜像
    const result = await invoke<any>('check_docker')
    envCheckResult.value = result
    
    // 显示检查结果提示
    if (!result.installed) {
      ElMessage.error('Docker 未安装，请访问 Docker 官网下载安装')
    } else if (!result.running) {
      ElMessage.warning('Docker 已安装但未运行，请启动 Docker Desktop')
    } else if (!result.image_pulled) {
      ElMessage.warning('Docker 环境就绪，但未找到 Strix 镜像，请构建镜像')
    } else {
      ElMessage.success('Docker 环境检查通过，可以开始扫描')
    }
  } catch (error: any) {
    const errorMessage = error?.message || error?.toString() || '未知错误'
    console.error('环境检查失败:', error)
    ElMessage.error(`环境检查失败: ${errorMessage}`)
    envCheckResult.value = {
      installed: false,
      image_pulled: false,
      message: errorMessage
    }
  } finally {
    checking.value = false
  }
}

const startScan = async () => {
  if (!formRef.value) return
  
  try {
    await formRef.value.validate()
  } catch {
    return
  }

  if (scanConfig.value.targets.length === 0) {
    ElMessage.warning('请至少输入一个扫描目标')
    return
  }

  // 从设置中读取 LLM 配置
  let llmProvider = 'openai/gpt-5'
  let llmApiKey = ''
  let llmApiBase: string | null = null

  try {
    const allSettings = await invoke<Record<string, string>>('get_all_settings')
    
    // 防御性检查：确保 allSettings 存在
    if (!allSettings || typeof allSettings !== 'object') {
      ElMessage.warning('无法读取设置，请在设置页面配置 LLM')
      return
    }
    
    // 读取 LLM 提供商
    if (allSettings['defaultLlmProvider'] && typeof allSettings['defaultLlmProvider'] === 'string') {
      llmProvider = allSettings['defaultLlmProvider']
    } else {
      ElMessage.warning('未配置 LLM 提供商，请在设置页面配置')
      return
    }
    
    // 读取 API Key
    if (allSettings['llmApiKey'] && typeof allSettings['llmApiKey'] === 'string') {
      llmApiKey = allSettings['llmApiKey']
    } else {
      ElMessage.warning('未配置 API Key，请在设置页面配置')
      return
    }
    
    // 读取 API Base URL
    if (allSettings['llmApiBase'] && typeof allSettings['llmApiBase'] === 'string') {
      llmApiBase = allSettings['llmApiBase']
    } else if (llmProvider && typeof llmProvider === 'string' && llmProvider.includes('deepseek')) {
      // 如果选择 DeepSeek 但没有保存的 API Base URL，设置默认值
      llmApiBase = 'https://api.deepseek.com'
    }
  } catch (error: any) {
    console.error('读取 LLM 设置失败:', error)
    ElMessage.error(`读取 LLM 设置失败: ${error?.message || error || '未知错误'}`)
    return
  }

  scanning.value = true
  try {
    const scanId = await invoke<number>('start_scan', {
      config: {
        targets: scanConfig.value.targets,
        instruction: scanConfig.value.instruction || null,
        runName: scanConfig.value.runName || null,
        llmProvider,
        llmApiKey,
        llmApiBase,
        nonInteractive: scanConfig.value.nonInteractive,
      },
    })

    ElMessage.success(`扫描已启动，ID: ${scanId}`)
    emit('scan-started', scanId)
    
    // 重置表单
    targetInput.value = ''
    scanConfig.value.targets = []
    scanConfig.value.instruction = ''
    scanConfig.value.runName = ''
  } catch (error: any) {
    const errorMessage = error?.message || error?.toString() || String(error) || '未知错误'
    console.error('启动扫描失败:', error)
    ElMessage.error(`启动扫描失败: ${errorMessage}`)
  } finally {
    scanning.value = false
  }
}
</script>

<style scoped>
.new-scan {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow-y: auto;
}

.scan-card {
  background-color: var(--bg-secondary, #f5f5f5);
  border: 1px solid var(--border-primary, #d4d4d4);
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.scan-card :deep(.el-card__body) {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow-y: auto;
  min-height: 0;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary, #000000);
}

.close-btn {
  padding: 6px;
  color: var(--text-secondary, #666666);
  font-size: 18px;
  transition: all 0.3s;
}

.close-btn:hover {
  color: #f56c6c;
  background-color: rgba(245, 108, 108, 0.1);
  transform: scale(1.1);
}

.target-tags {
  margin-top: 8px;
}

.env-check-card {
  background-color: var(--bg-secondary, #f5f5f5);
  border: 1px solid var(--border-primary, #d4d4d4);
}

.env-status {
  display: flex;
  gap: 20px;
}

.env-item {
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--text-primary, #000000);
}

.env-item .success {
  color: #67c23a;
}

.env-item .error {
  color: #f56c6c;
}

.env-description {
  margin-top: 16px;
}

.command-hint {
  background-color: rgba(0, 0, 0, 0.05);
  padding: 12px;
  border-radius: 4px;
  font-family: 'Courier New', monospace;
  font-size: 12px;
  white-space: pre-wrap;
  word-break: break-all;
  margin: 8px 0;
  overflow-x: auto;
}

.build-section {
  margin-top: 16px;
}

.build-dialog {
  max-height: 60vh;
  display: flex;
  flex-direction: column;
}

.build-logs {
  background-color: #1e1e1e;
  color: #d4d4d4;
  padding: 16px;
  border-radius: 4px;
  font-family: 'Courier New', monospace;
  font-size: 12px;
  max-height: 400px;
  overflow-y: auto;
  line-height: 1.6;
}

.build-log-line {
  display: flex;
  margin-bottom: 4px;
  word-break: break-all;
}

.log-time {
  color: #858585;
  margin-right: 8px;
  flex-shrink: 0;
  min-width: 80px;
}

.log-message {
  flex: 1;
}

.log-info {
  color: #d4d4d4;
}

.log-error {
  color: #f48771;
}

.log-success {
  color: #89d185;
  font-weight: bold;
}

.build-log-placeholder {
  color: #858585;
  font-style: italic;
  text-align: center;
  padding: 20px;
}

.description-content {
  font-size: 14px;
  line-height: 1.6;
  color: var(--text-secondary, #333333);
}

.description-content p {
  margin-bottom: 8px;
}

.description-content ul {
  margin: 8px 0;
  padding-left: 24px;
}

.description-content li {
  margin-bottom: 8px;
}

.description-content a {
  color: var(--color-primary, #409eff);
  text-decoration: none;
}

.description-content a:hover {
  text-decoration: underline;
}

.scan-form {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
}
</style>

