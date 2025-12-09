<template>
  <div class="scan-history">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>扫描历史</span>
          <div class="header-actions">
            <el-button @click="refreshScans" :loading="loading">
              <el-icon><Refresh /></el-icon>
              刷新
            </el-button>
            <el-button @click="handleClearAll" type="danger" :disabled="scans.length === 0">
              <el-icon><Delete /></el-icon>
              清空历史
            </el-button>
          </div>
        </div>
      </template>

      <el-table :data="scans" v-loading="loading" style="width: 100%">
        <el-table-column prop="runName" label="运行名称" width="200" />
        <el-table-column prop="status" label="状态" width="120">
          <template #default="{ row }">
            <el-tag :type="getStatusType(row.status)">
              {{ getStatusText(row.status) }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="progress" label="进度" width="120">
          <template #default="{ row }">
            <el-progress :percentage="Math.round(row.progress)" :status="getProgressStatus(row.status)" />
          </template>
        </el-table-column>
        <el-table-column prop="message" label="消息" />
        <el-table-column prop="createdAt" label="创建时间" width="180" />
        <el-table-column label="操作" width="420" fixed="right">
          <template #default="{ row }">
            <div class="action-buttons">
              <el-button
                size="small"
                @click="handleViewResult(row.id)"
                :disabled="row.status !== 'completed'"
              >
                查看结果
              </el-button>
              <el-button
                size="small"
                @click="handleViewDetails(row.id)"
                :type="row.status === 'running' ? 'primary' : 'default'"
              >
                {{ row.status === 'running' ? '查看日志' : '查看详情' }}
              </el-button>
              <el-button
                size="small"
                type="danger"
                @click="stopScan(row.id)"
                :disabled="row.status !== 'running'"
              >
                停止
              </el-button>
              <el-button
                size="small"
                type="danger"
                @click="handleDeleteScan(row.id)"
                :disabled="row.status === 'running'"
              >
                删除
              </el-button>
            </div>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <!-- 详情对话框 -->
    <el-dialog
      v-model="showDetailsDialog"
      title="扫描详情"
      width="80%"
      v-if="selectedScanDetails"
    >
      <div class="details-content">
        <el-descriptions :column="1" border>
          <el-descriptions-item label="运行名称">
            {{ selectedScanDetails.runName }}
          </el-descriptions-item>
          <el-descriptions-item label="状态">
            <el-tag :type="getStatusType(selectedScanDetails.status)">
              {{ getStatusText(selectedScanDetails.status) }}
            </el-tag>
          </el-descriptions-item>
          <el-descriptions-item label="进度">
            <el-progress :percentage="Math.round(selectedScanDetails.progress)" :status="getProgressStatus(selectedScanDetails.status)" />
          </el-descriptions-item>
          <el-descriptions-item label="创建时间">
            {{ selectedScanDetails.createdAt }}
          </el-descriptions-item>
          <el-descriptions-item label="消息">
            {{ selectedScanDetails.message || '无' }}
          </el-descriptions-item>
        </el-descriptions>
        
        <el-divider />
        
        <div class="command-section">
          <h3>执行的命令</h3>
          <el-input
            v-model="selectedScanDetails.command"
            type="textarea"
            :rows="6"
            readonly
            class="command-display"
          />
        </div>
        
        <el-divider />
        
        <div class="logs-section">
          <h3>执行日志</h3>
          <div class="logs-container" ref="detailsLogsContainer">
            <div
              v-for="(log, index) in selectedScanDetails.logs"
              :key="index"
              :class="['log-line', `log-${log.level}`]"
            >
              <span class="log-time">{{ log.time }}</span>
              <span class="log-level">{{ log.level.toUpperCase() }}</span>
              <span class="log-message">{{ log.message }}</span>
            </div>
            <div v-if="selectedScanDetails.logs.length === 0" class="no-logs">
              暂无日志
            </div>
          </div>
        </div>
      </div>
    </el-dialog>

    <!-- 结果对话框 -->
    <el-dialog
      v-model="showResultDialog"
      title="扫描结果"
      width="80%"
      v-if="selectedResult"
    >
      <div class="result-content">
        <div class="result-stats">
          <div class="stat-item">
            <div class="stat-label">总漏洞数</div>
            <div class="stat-value">{{ selectedResult.stats.totalVulnerabilities }}</div>
          </div>
          <div class="stat-item">
            <div class="stat-label">严重</div>
            <div class="stat-value critical">{{ selectedResult.stats.critical }}</div>
          </div>
          <div class="stat-item">
            <div class="stat-label">高危</div>
            <div class="stat-value high">{{ selectedResult.stats.high }}</div>
          </div>
          <div class="stat-item">
            <div class="stat-label">中危</div>
            <div class="stat-value medium">{{ selectedResult.stats.medium }}</div>
          </div>
          <div class="stat-item">
            <div class="stat-label">低危</div>
            <div class="stat-value low">{{ selectedResult.stats.low }}</div>
          </div>
        </div>

        <el-divider />

        <div class="vulnerabilities">
          <h3>漏洞列表</h3>
          <el-table :data="selectedResult.vulnerabilities">
            <el-table-column prop="title" label="标题" />
            <el-table-column prop="severity" label="严重程度" width="120">
              <template #default="{ row }">
                <el-tag :type="getSeverityType(row.severity)">
                  {{ row.severity }}
                </el-tag>
              </template>
            </el-table-column>
            <el-table-column prop="description" label="描述" show-overflow-tooltip />
            <el-table-column label="操作" width="120">
              <template #default="{ row }">
                <el-button size="small" @click="viewVulnDetail(row)">详情</el-button>
              </template>
            </el-table-column>
          </el-table>
        </div>
      </div>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick, watch } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Refresh, Delete } from '@element-plus/icons-vue'
import { invoke } from '@tauri-apps/api/core'

const emit = defineEmits(['scan-selected'])

const scans = ref<any[]>([])
const loading = ref(false)
const showResultDialog = ref(false)
const selectedResult = ref<any>(null)
const showDetailsDialog = ref(false)
const selectedScanDetails = ref<{
  runName: string;
  status: string;
  progress: number;
  message: string;
  createdAt: string;
  command: string;
  logs: Array<{ time: string; level: string; message: string }>;
} | null>(null)
const detailsLogsContainer = ref<HTMLElement | null>(null)

let detailsLogsUnsubscribe: (() => void) | null = null
let detailsStatusInterval: ReturnType<typeof setInterval> | null = null

const handleViewResult = async (scanId: number) => {
  await viewResult(scanId)
  emit('scan-selected', scanId)
}

const handleViewDetails = async (scanId: number) => {
  loading.value = true
  try {
    const details = await invoke<any>('get_scan_details', { scanId })
    selectedScanDetails.value = {
      runName: details.runName || '',
      status: details.status || '',
      progress: details.progress || 0,
      message: details.message || '',
      createdAt: details.createdAt || '',
      command: details.command || '无命令信息',
      logs: details.logs || []
    }
    showDetailsDialog.value = true
    
    // 如果是运行中的扫描，也触发日志查看并监听实时日志
    if (details.status === 'running') {
      emit('scan-selected', scanId)
      
      // 监听实时日志
      try {
        const { listen } = await import('@tauri-apps/api/event')
        detailsLogsUnsubscribe = await listen('scan-log', (event: any) => {
          const payload = event.payload as { scan_id: number; level: string; message: string }
          if (payload.scan_id === scanId && selectedScanDetails.value) {
            selectedScanDetails.value.logs.push({
              time: new Date().toLocaleTimeString(),
              level: payload.level,
              message: payload.message
            })
            nextTick(() => {
              if (detailsLogsContainer.value) {
                detailsLogsContainer.value.scrollTop = detailsLogsContainer.value.scrollHeight
              }
            })
          }
        })
      } catch (error) {
        console.error('监听日志事件失败:', error)
      }
      
      // 定期刷新状态和日志
      detailsStatusInterval = setInterval(async () => {
        if (selectedScanDetails.value) {
          try {
            const status = await invoke<any>('get_scan_status', { scanId })
            if (selectedScanDetails.value) {
              selectedScanDetails.value.status = status.status
              selectedScanDetails.value.progress = status.progress
              selectedScanDetails.value.message = status.message
              
              // 如果扫描已完成或失败，停止刷新
              if (status.status === 'completed' || status.status === 'failed' || status.status === 'stopped') {
                if (detailsStatusInterval) {
                  clearInterval(detailsStatusInterval)
                  detailsStatusInterval = null
                }
              }
            }
          } catch (error) {
            console.error('刷新状态失败:', error)
          }
        }
      }, 2000)
    }
    
    // 滚动到底部
    await nextTick()
    if (detailsLogsContainer.value) {
      detailsLogsContainer.value.scrollTop = detailsLogsContainer.value.scrollHeight
    }
  } catch (error: any) {
    ElMessage.error(`获取扫描详情失败: ${error.message || error}`)
  } finally {
    loading.value = false
  }
}

const getStatusType = (status: string) => {
  const map: Record<string, string> = {
    running: 'primary',
    completed: 'success',
    failed: 'danger',
    stopped: 'warning',
  }
  return map[status] || 'info'
}

const getStatusText = (status: string) => {
  const map: Record<string, string> = {
    running: '运行中',
    completed: '已完成',
    failed: '失败',
    stopped: '已停止',
  }
  return map[status] || status
}

const getProgressStatus = (status: string) => {
  if (status === 'completed') return 'success'
  if (status === 'failed') return 'exception'
  return undefined
}

const getSeverityType = (severity: string) => {
  const map: Record<string, string> = {
    critical: 'danger',
    high: 'warning',
    medium: 'info',
    low: '',
  }
  return map[severity] || ''
}

const refreshScans = async () => {
  loading.value = true
  try {
    scans.value = await invoke<any[]>('list_scans')
  } catch (error: any) {
    ElMessage.error(`获取扫描列表失败: ${error.message}`)
  } finally {
    loading.value = false
  }
}

const viewResult = async (scanId: number) => {
  try {
    selectedResult.value = await invoke<any>('get_scan_result', { scanId })
    showResultDialog.value = true
  } catch (error: any) {
    ElMessage.error(`获取扫描结果失败: ${error.message}`)
  }
}

const stopScan = async (scanId: number) => {
  try {
    await invoke('stop_scan', { scanId })
    ElMessage.success('扫描已停止')
    await refreshScans()
  } catch (error: any) {
    ElMessage.error(`停止扫描失败: ${error.message}`)
  }
}

const viewVulnDetail = (vuln: any) => {
  ElMessageBox.alert(
    `
      <div>
        <h4>${vuln.title}</h4>
        <p><strong>严重程度:</strong> ${vuln.severity}</p>
        <p><strong>描述:</strong> ${vuln.description}</p>
        ${vuln.poc ? `<p><strong>PoC:</strong><pre>${vuln.poc}</pre></p>` : ''}
        ${vuln.remediation ? `<p><strong>修复建议:</strong> ${vuln.remediation}</p>` : ''}
      </div>
    `,
    '漏洞详情',
    {
      dangerouslyUseHTMLString: true,
    }
  )
}

const handleDeleteScan = async (scanId: number) => {
  try {
    await ElMessageBox.confirm(
      '确定要删除这条扫描记录吗？删除后将无法恢复。',
      '确认删除',
      {
        confirmButtonText: '确定',
        cancelButtonText: '取消',
        type: 'warning',
      }
    )
    
    await invoke('delete_scan', { scanId })
    ElMessage.success('删除成功')
    await refreshScans()
  } catch (error: any) {
    if (error !== 'cancel') {
      ElMessage.error(`删除失败: ${error.message || error}`)
    }
  }
}

const handleClearAll = async () => {
  try {
    await ElMessageBox.confirm(
      '确定要清空所有扫描历史吗？此操作不可恢复。',
      '确认清空',
      {
        confirmButtonText: '确定',
        cancelButtonText: '取消',
        type: 'warning',
      }
    )
    
    await invoke('clear_all_scans')
    ElMessage.success('已清空所有扫描历史')
    await refreshScans()
  } catch (error: any) {
    if (error !== 'cancel') {
      ElMessage.error(`清空失败: ${error.message || error}`)
    }
  }
}

// 监听对话框关闭，清理资源
watch(() => showDetailsDialog.value, (isOpen) => {
  if (!isOpen) {
    if (detailsLogsUnsubscribe) {
      detailsLogsUnsubscribe()
      detailsLogsUnsubscribe = null
    }
    if (detailsStatusInterval) {
      clearInterval(detailsStatusInterval)
      detailsStatusInterval = null
    }
  }
})

// 定时刷新运行中的扫描
let refreshInterval: ReturnType<typeof setInterval> | null = null

onMounted(() => {
  refreshScans()
  refreshInterval = setInterval(() => {
    const hasRunning = scans.value.some(s => s.status === 'running')
    if (hasRunning) {
      refreshScans()
    }
  }, 3000)
})

onUnmounted(() => {
  if (refreshInterval) {
    clearInterval(refreshInterval)
  }
  // 清理详情对话框的资源
  if (detailsLogsUnsubscribe) {
    detailsLogsUnsubscribe()
    detailsLogsUnsubscribe = null
  }
  if (detailsStatusInterval) {
    clearInterval(detailsStatusInterval)
    detailsStatusInterval = null
  }
})
</script>

<style scoped>
.scan-history {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.scan-history :deep(.el-card) {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.scan-history :deep(.el-card__body) {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  min-height: 0;
}

.scan-history :deep(.el-table) {
  flex: 1;
  overflow: auto;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary, #000000);
}

.header-actions {
  display: flex;
  gap: 8px;
}

.result-content {
  color: var(--text-primary, #000000);
}

.result-stats {
  display: flex;
  gap: 20px;
  flex-wrap: wrap;
}

.stat-item {
  text-align: center;
  padding: 16px;
  background-color: var(--bg-secondary, #f5f5f5);
  border-radius: 8px;
  min-width: 100px;
}

.stat-label {
  font-size: 14px;
  color: var(--text-secondary, #333333);
  margin-bottom: 8px;
}

.stat-value {
  font-size: 24px;
  font-weight: 600;
  color: var(--text-primary, #000000);
}

.stat-value.critical {
  color: #f56c6c;
}

.stat-value.high {
  color: #e6a23c;
}

.stat-value.medium {
  color: #409eff;
}

.stat-value.low {
  color: #67c23a;
}

.vulnerabilities {
  margin-top: 20px;
}

.vulnerabilities h3 {
  margin-bottom: 16px;
  color: var(--text-primary, #000000);
}

.action-buttons {
  display: flex;
  gap: 8px;
  flex-wrap: nowrap;
  align-items: center;
}

.details-content {
  color: var(--text-primary, #000000);
}

.command-section {
  margin-top: 20px;
}

.command-section h3 {
  margin-bottom: 12px;
  color: var(--text-primary, #000000);
}

.command-display {
  font-family: 'Courier New', monospace;
  font-size: 12px;
}

.logs-section {
  margin-top: 20px;
}

.logs-section h3 {
  margin-bottom: 12px;
  color: var(--text-primary, #000000);
}

.logs-container {
  max-height: 400px;
  overflow-y: auto;
  font-family: 'Courier New', monospace;
  font-size: 12px;
  padding: 8px;
  background-color: var(--bg-secondary, #f5f5f5);
  border-radius: 4px;
  border: 1px solid var(--border-primary, #d4d4d4);
}

.log-line {
  display: flex;
  gap: 8px;
  padding: 4px 0;
  border-bottom: 1px solid var(--border-tertiary, #e0e0e0);
}

.log-time {
  color: var(--text-tertiary, #666666);
  min-width: 100px;
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

