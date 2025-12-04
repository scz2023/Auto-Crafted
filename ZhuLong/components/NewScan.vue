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

        <el-divider>LLM 配置</el-divider>

        <el-form-item label="LLM 提供商" prop="llmProvider" required>
          <el-select v-model="scanConfig.llmProvider" placeholder="选择 LLM 提供商">
            <el-option label="OpenAI GPT-5" value="openai/gpt-5" />
            <el-option label="Anthropic Claude Sonnet 4.5" value="anthropic/claude-sonnet-4-5" />
            <el-option label="本地模型 (Ollama)" value="ollama/llama3" />
            <el-option label="自定义" value="custom" />
          </el-select>
        </el-form-item>

        <el-form-item label="API Key" prop="llmApiKey" required>
          <el-input
            v-model="scanConfig.llmApiKey"
            type="password"
            show-password
            placeholder="输入 LLM API Key"
          />
        </el-form-item>

        <el-form-item
          label="API Base URL"
          prop="llmApiBase"
          v-if="scanConfig.llmProvider.includes('ollama') || scanConfig.llmProvider === 'custom'"
        >
          <el-input
            v-model="scanConfig.llmApiBase"
            placeholder="例如：http://localhost:11434 (Ollama)"
          />
        </el-form-item>

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
          <el-icon :class="envCheckResult.docker ? 'success' : 'error'">
            <Check v-if="envCheckResult.docker" />
            <Close v-else />
          </el-icon>
          <span>Docker: {{ envCheckResult.docker ? '已安装' : '未安装' }}</span>
        </div>
        <div class="env-item">
          <el-icon :class="envCheckResult.python ? 'success' : 'error'">
            <Check v-if="envCheckResult.python" />
            <Close v-else />
          </el-icon>
          <span>系统 Python: {{ envCheckResult.python ? '已安装' : '未安装' }}</span>
        </div>
        <div class="env-item">
          <el-icon :class="envCheckResult.embeddedPython ? 'success' : 'error'">
            <Check v-if="envCheckResult.embeddedPython" />
            <Close v-else />
          </el-icon>
          <span>内嵌 Python: {{ envCheckResult.embeddedPython ? '可用' : '不可用' }}</span>
        </div>
      </div>
      <div class="env-description">
        <el-divider />
        <div class="description-content">
          <p><strong>环境说明：</strong></p>
          <ul>
            <li><strong>Docker：</strong>Strix 扫描工具需要 Docker 环境来运行扫描容器。如果未安装，请访问 <a href="https://www.docker.com/get-started" target="_blank">Docker 官网</a> 下载安装。</li>
            <li><strong>系统 Python：</strong>如果内嵌 Python 不可用，应用会回退使用系统 Python。建议安装 Python 3.8+ 作为备用方案。</li>
            <li><strong>内嵌 Python：</strong>应用内置的 Python 解释器，优先使用。如果不可用，会自动回退到系统 Python。内嵌 Python 的优势是不依赖系统环境，更加稳定可靠。</li>
          </ul>
        </div>
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { VideoPlay, Tools, Check, Close } from '@element-plus/icons-vue'
import { invoke } from '@tauri-apps/api/core'

const emit = defineEmits(['scan-started'])

const formRef = ref()
const scanning = ref(false)
const checking = ref(false)
const targetInput = ref('')
const envCheckResult = ref<{ docker: boolean; python: boolean; embeddedPython: boolean } | null>(null)

const scanConfig = ref({
  targets: [] as string[],
  instruction: '',
  runName: '',
  llmProvider: 'openai/gpt-5',
  llmApiKey: '',
  llmApiBase: '',
  nonInteractive: false,
})

const rules = {
  targets: [{ required: true, message: '请至少输入一个扫描目标', trigger: 'blur' }],
  llmProvider: [{ required: true, message: '请选择 LLM 提供商', trigger: 'change' }],
  llmApiKey: [{ required: true, message: '请输入 API Key', trigger: 'blur' }],
}

const canStartScan = computed(() => {
  return (
    scanConfig.value.targets.length > 0 &&
    scanConfig.value.llmProvider &&
    scanConfig.value.llmApiKey
  )
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

const checkEnvironment = async () => {
  checking.value = true
  envCheckResult.value = null
  
  try {
    // 分别检查每个环境，即使某个失败也继续检查其他的
    const results = {
      docker: false,
      python: false,
      embeddedPython: false,
    }
    
    // 检查 Docker
    try {
      results.docker = await invoke<boolean>('check_docker')
    } catch (error: any) {
      console.error('Docker 检查失败:', error)
      results.docker = false
    }
    
    // 检查系统 Python
    try {
      results.python = await invoke<boolean>('check_python')
    } catch (error: any) {
      console.error('系统 Python 检查失败:', error)
      results.python = false
    }
    
    // 检查内嵌 Python
    try {
      results.embeddedPython = await invoke<boolean>('check_embedded_python')
    } catch (error: any) {
      console.error('内嵌 Python 检查失败:', error)
      results.embeddedPython = false
    }
    
    envCheckResult.value = results
    
    // 显示检查结果提示
    const messages: string[] = []
    if (!results.docker) {
      messages.push('Docker 未安装，Strix 需要 Docker 环境')
    }
    if (!results.python && !results.embeddedPython) {
      messages.push('系统 Python 和内嵌 Python 都不可用，扫描可能无法运行')
    } else if (!results.embeddedPython && results.python) {
      messages.push('内嵌 Python 不可用，将使用系统 Python')
    } else if (results.embeddedPython) {
      messages.push('内嵌 Python 可用，将优先使用')
    }
    
    if (messages.length > 0) {
      messages.forEach(msg => ElMessage.warning(msg))
    } else {
      ElMessage.success('所有环境检查通过')
    }
  } catch (error: any) {
    const errorMessage = error?.message || error?.toString() || '未知错误'
    console.error('环境检查失败:', error)
    ElMessage.error(`环境检查失败: ${errorMessage}`)
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

  scanning.value = true
  try {
    const scanId = await invoke<number>('start_scan', {
      config: {
        targets: scanConfig.value.targets,
        instruction: scanConfig.value.instruction || null,
        runName: scanConfig.value.runName || null,
        llmProvider: scanConfig.value.llmProvider,
        llmApiKey: scanConfig.value.llmApiKey,
        llmApiBase: scanConfig.value.llmApiBase || null,
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
    ElMessage.error(`启动扫描失败: ${error.message}`)
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

