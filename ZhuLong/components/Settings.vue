<template>
  <div class="settings">
    <el-card class="settings-card">
      <template #header>
        <div class="card-header">
          <span>设置</span>
        </div>
      </template>

      <el-form :model="settings" label-width="150px" ref="formRef">
        <el-divider>LLM 配置</el-divider>
        
        <el-form-item label="默认 LLM 提供商" required>
          <el-select v-model="settings.defaultLlmProvider" placeholder="选择 LLM 提供商" @change="handleProviderChange">
            <el-option label="OpenAI GPT-5" value="openai/gpt-5" />
            <el-option label="Anthropic Claude Sonnet 4.5" value="anthropic/claude-sonnet-4-5" />
            <el-option label="DeepSeek-V3.2 (Chat)" value="deepseek/deepseek-chat" />
            <el-option label="DeepSeek-V3.2 (Reasoner)" value="deepseek/deepseek-reasoner" />
            <el-option label="本地模型 (Ollama)" value="ollama/llama3" />
            <el-option label="自定义" value="custom" />
          </el-select>
        </el-form-item>

        <el-form-item label="API Key" required>
          <el-input
            v-model="settings.llmApiKey"
            type="password"
            show-password
            placeholder="输入 LLM API Key"
          />
        </el-form-item>

        <el-form-item
          label="API Base URL"
          v-if="settings.defaultLlmProvider.includes('ollama') || settings.defaultLlmProvider === 'custom' || settings.defaultLlmProvider.includes('deepseek')"
        >
          <el-input
            v-model="settings.llmApiBase"
            :placeholder="getApiBasePlaceholder()"
          />
        </el-form-item>

        <el-form-item>
          <el-button type="primary" @click="saveSettings" :loading="saving">保存设置</el-button>
          <el-button @click="loadSettings">重新加载</el-button>
        </el-form-item>
      </el-form>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { ElMessage } from 'element-plus'
import { invoke } from '@tauri-apps/api/core'

const formRef = ref()
const saving = ref(false)

const settings = ref({
  defaultLlmProvider: 'openai/gpt-5',
  llmApiKey: '',
  llmApiBase: '',
})

const handleProviderChange = (value: string) => {
  // 当选择 DeepSeek 时，自动设置 API Base URL
  if (value.includes('deepseek') && !settings.value.llmApiBase) {
    settings.value.llmApiBase = 'https://api.deepseek.com'
  }
  // 当选择其他提供商时，清空自定义的 API Base URL（除了 Ollama 和 custom）
  if (!value.includes('ollama') && value !== 'custom' && !value.includes('deepseek')) {
    settings.value.llmApiBase = ''
  }
}

const getApiBasePlaceholder = () => {
  if (settings.value.defaultLlmProvider.includes('deepseek')) {
    return 'https://api.deepseek.com (DeepSeek API)'
  } else if (settings.value.defaultLlmProvider.includes('ollama')) {
    return 'http://localhost:11434 (Ollama)'
  } else {
    return '输入自定义 API Base URL'
  }
}

const loadSettings = async () => {
  try {
    const allSettings = await invoke<Record<string, string>>('get_all_settings')
    
    if (allSettings['defaultLlmProvider']) {
      settings.value.defaultLlmProvider = allSettings['defaultLlmProvider']
    }
    
    if (allSettings['llmApiKey']) {
      settings.value.llmApiKey = allSettings['llmApiKey']
    }
    
    if (allSettings['llmApiBase']) {
      settings.value.llmApiBase = allSettings['llmApiBase']
    } else {
      // 如果选择 DeepSeek 但没有保存的 API Base URL，设置默认值
      if (settings.value.defaultLlmProvider.includes('deepseek')) {
        settings.value.llmApiBase = 'https://api.deepseek.com'
      }
    }
    
    ElMessage.success('设置已加载')
  } catch (error: any) {
    console.error('加载设置失败:', error)
    ElMessage.warning('加载设置失败，使用默认值')
  }
}

const saveSettings = async () => {
  saving.value = true
  try {
    // 保存所有设置
    await invoke('save_setting', { key: 'defaultLlmProvider', value: settings.value.defaultLlmProvider })
    
    // 直接保存 API Key
    if (settings.value.llmApiKey) {
      await invoke('save_setting', { key: 'llmApiKey', value: settings.value.llmApiKey })
    } else {
      await invoke('save_setting', { key: 'llmApiKey', value: '' })
    }
    
    // 保存 API Base URL
    if (settings.value.llmApiBase) {
      await invoke('save_setting', { key: 'llmApiBase', value: settings.value.llmApiBase })
    } else {
      await invoke('save_setting', { key: 'llmApiBase', value: '' })
    }
    
    ElMessage.success('设置已保存')
  } catch (error: any) {
    console.error('保存设置失败:', error)
    ElMessage.error(`保存设置失败: ${error.message || error}`)
  } finally {
    saving.value = false
  }
}

// 页面加载时自动加载设置
onMounted(() => {
  loadSettings()
})
</script>

<style scoped>
.settings {
  width: 100%;
  height: auto;
}

.card-header {
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary, #000000);
}

.setting-hint {
  margin-left: 12px;
  font-size: 12px;
  color: var(--text-secondary, #666666);
}
</style>
