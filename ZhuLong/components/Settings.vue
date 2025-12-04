<template>
  <div class="settings">
    <el-card class="settings-card">
      <template #header>
        <div class="card-header">
          <span>设置</span>
        </div>
      </template>

      <el-form label-width="150px">
        <el-form-item label="默认 LLM 提供商">
          <el-select v-model="settings.defaultLlmProvider">
            <el-option label="OpenAI GPT-5" value="openai/gpt-5" />
            <el-option label="Anthropic Claude Sonnet 4.5" value="anthropic/claude-sonnet-4-5" />
          </el-select>
        </el-form-item>

        <el-form-item label="自动保存 API Key">
          <el-switch v-model="settings.autoSaveApiKey" />
        </el-form-item>

        <el-form-item>
          <el-button type="primary" @click="saveSettings">保存设置</el-button>
        </el-form-item>
      </el-form>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { ElMessage } from 'element-plus'

const settings = ref({
  defaultLlmProvider: 'openai/gpt-5',
  autoSaveApiKey: true,
})

const saveSettings = () => {
  // TODO: 保存到本地存储或数据库
  ElMessage.success('设置已保存')
}
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
</style>

