<template>
  <div class="scan-history">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>扫描历史</span>
          <el-button @click="refreshScans" :loading="loading">
            <el-icon><Refresh /></el-icon>
            刷新
          </el-button>
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
        <el-table-column label="操作" width="200" fixed="right">
          <template #default="{ row }">
            <el-button
              size="small"
              @click="handleViewResult(row.id)"
              :disabled="row.status !== 'completed'"
            >
              查看结果
            </el-button>
            <el-button
              size="small"
              @click="() => { emit('scan-selected', row.id) }"
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
          </template>
        </el-table-column>
      </el-table>
    </el-card>

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
import { ref, onMounted, onUnmounted } from 'vue'
import { ElMessage } from 'element-plus'
import { Refresh } from '@element-plus/icons-vue'
import { invoke } from '@tauri-apps/api/core'

const emit = defineEmits(['scan-selected'])

const handleViewResult = async (scanId: number) => {
  await viewResult(scanId)
  emit('scan-selected', scanId)
}

const scans = ref<any[]>([])
const loading = ref(false)
const showResultDialog = ref(false)
const selectedResult = ref<any>(null)

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

// 定时刷新运行中的扫描
let refreshInterval: NodeJS.Timeout | null = null

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
})
</script>

<style scoped>
.scan-history {
  width: 100%;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary, #fff);
}

.result-content {
  color: var(--text-primary, #fff);
}

.result-stats {
  display: flex;
  gap: 20px;
  flex-wrap: wrap;
}

.stat-item {
  text-align: center;
  padding: 16px;
  background-color: var(--bg-secondary, #252525);
  border-radius: 8px;
  min-width: 100px;
}

.stat-label {
  font-size: 14px;
  color: var(--text-secondary, #ccc);
  margin-bottom: 8px;
}

.stat-value {
  font-size: 24px;
  font-weight: 600;
  color: var(--text-primary, #fff);
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
  color: var(--text-primary, #fff);
}
</style>

