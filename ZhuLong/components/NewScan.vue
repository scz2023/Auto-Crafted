<template>
  <div class="new-scan">
    <el-card class="scan-card">
      <template #header>
        <div class="card-header">
          <span>新建安全扫描</span>
        </div>
      </template>

      <el-form :model="scanConfig" label-width="120px" :rules="rules" ref="formRef">
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
          <el-button @click="checkEnvironment" :loading="checking">
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
          <span>Python: {{ envCheckResult.python ? '已安装' : '未安装' }}</span>
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
const envCheckResult = ref<{ docker: boolean; python: boolean } | null>(null)

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

const checkEnvironment = async () => {
  checking.value = true
  try {
    const [docker, python] = await Promise.all([
      invoke<boolean>('check_docker'),
      invoke<boolean>('check_python'),
    ])
    
    envCheckResult.value = { docker, python }
    
    if (!docker) {
      ElMessage.warning('Docker 未安装，Strix 需要 Docker 环境')
    }
    if (!python) {
      ElMessage.warning('Python 未安装，但应用已内置 Python 运行时')
    }
  } catch (error: any) {
    ElMessage.error(`环境检查失败: ${error.message}`)
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
  max-width: 900px;
  margin: 0 auto;
}

.scan-card {
  background-color: var(--bg-secondary, #252525);
  border: 1px solid var(--border-primary, #333);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary, #fff);
}

.target-tags {
  margin-top: 8px;
}

.env-check-card {
  background-color: var(--bg-secondary, #252525);
  border: 1px solid var(--border-primary, #333);
}

.env-status {
  display: flex;
  gap: 20px;
}

.env-item {
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--text-primary, #fff);
}

.env-item .success {
  color: #67c23a;
}

.env-item .error {
  color: #f56c6c;
}
</style>

