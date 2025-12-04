<template>
  <el-card class="logs-card">
    <template #header>
      <div class="card-header">
        <span>扫描日志</span>
        <el-button size="small" @click="clearLogs">清空</el-button>
      </div>
    </template>
    
    <div class="logs-container" ref="logsContainer">
      <div
        v-for="(log, index) in logs"
        :key="index"
        :class="['log-line', `log-${log.level}`]"
      >
        <span class="log-time">{{ log.time }}</span>
        <span class="log-level">{{ log.level.toUpperCase() }}</span>
        <span class="log-message">{{ log.message }}</span>
      </div>
      <div v-if="logs.length === 0" class="no-logs">
        暂无日志
      </div>
    </div>
  </el-card>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, nextTick } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'

const props = defineProps<{
  scanId: number | null
}>()

const logs = ref<Array<{ time: string; level: string; message: string }>>([])
const logsContainer = ref<HTMLElement | null>(null)
let unsubscribe: (() => void) | null = null

const clearLogs = () => {
  logs.value = []
}

const scrollToBottom = () => {
  nextTick(() => {
    if (logsContainer.value) {
      logsContainer.value.scrollTop = logsContainer.value.scrollHeight
    }
  })
}

const loadLogs = async () => {
  if (!props.scanId) return
  
  try {
    const logLines = await invoke<string[]>('get_scan_logs', { scanId: props.scanId })
    logs.value = logLines.map(line => {
      // 解析日志格式: [时间] [级别] 消息
      const match = line.match(/\[([^\]]+)\] \[([^\]]+)\] (.+)/)
      if (match) {
        return {
          time: match[1],
          level: match[2].toLowerCase(),
          message: match[3]
        }
      }
      return {
        time: new Date().toLocaleTimeString(),
        level: 'info',
        message: line
      }
    })
    scrollToBottom()
  } catch (error: any) {
    console.error('加载日志失败:', error)
  }
}

// 监听实时日志事件
onMounted(async () => {
  if (props.scanId) {
    await loadLogs()
  }
  
  // 监听 Tauri 事件
  try {
    unsubscribe = await listen('scan-log', (event: any) => {
      const payload = event.payload as { scan_id: number; level: string; message: string }
      
      // 只显示当前扫描的日志
      if (props.scanId && payload.scan_id === props.scanId) {
        logs.value.push({
          time: new Date().toLocaleTimeString(),
          level: payload.level,
          message: payload.message
        })
        scrollToBottom()
      }
    })
  } catch (error) {
    console.error('监听日志事件失败:', error)
  }
})

onUnmounted(() => {
  if (unsubscribe) {
    unsubscribe()
  }
})

// 当 scanId 变化时重新加载日志
watch(() => props.scanId, async (newId) => {
  if (newId) {
    await loadLogs()
  } else {
    logs.value = []
  }
})
</script>

<style scoped>
.logs-card {
  background-color: var(--bg-secondary, #f5f5f5);
  border: 1px solid var(--border-primary, #d4d4d4);
  height: 100%;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.logs-card :deep(.el-card__body) {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  min-height: 0;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary, #000000);
}

.logs-container {
  flex: 1;
  overflow-y: auto;
  font-family: 'Courier New', monospace;
  font-size: 12px;
  padding: 8px;
  background-color: var(--bg-primary, #ffffff);
  color: var(--text-primary, #000000);
  min-height: 0;
}

.log-line {
  display: flex;
  gap: 8px;
  padding: 4px 0;
  border-bottom: 1px solid var(--border-tertiary, #e0e0e0);
}

.log-time {
  color: var(--text-tertiary, #666666);
  min-width: 80px;
}

.log-level {
  min-width: 60px;
  font-weight: 600;
}

.log-level.info {
  color: #409eff;
}

.log-level.error {
  color: #f56c6c;
}

.log-level.warning {
  color: #e6a23c;
}

.log-level.success {
  color: #67c23a;
}

.log-message {
  flex: 1;
  word-break: break-all;
}

.log-info {
  color: var(--text-primary, #000000);
}

.log-error {
  color: #f56c6c;
}

.log-warning {
  color: #e6a23c;
}

.no-logs {
  text-align: center;
  color: var(--text-tertiary, #666);
  padding: 20px;
}
</style>

