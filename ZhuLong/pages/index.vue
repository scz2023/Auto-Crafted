<template>
  <div class="scan-page">
    <el-container class="main-container">
      <!-- 左侧边栏 -->
      <el-aside width="280px" class="sidebar">
        <div class="sidebar-header">
          <h1>🕯️ 烛龙</h1>
          <p class="subtitle">AI 驱动的渗透测试工具</p>
        </div>
        
        <el-menu
          :default-active="activeMenu"
          class="sidebar-menu"
          @select="handleMenuSelect"
        >
          <el-menu-item index="new-scan">
            <el-icon><Plus /></el-icon>
            <span>新建扫描</span>
          </el-menu-item>
          <el-menu-item index="scans">
            <el-icon><List /></el-icon>
            <span>扫描历史</span>
          </el-menu-item>
          <el-menu-item index="settings">
            <el-icon><Setting /></el-icon>
            <span>设置</span>
          </el-menu-item>
        </el-menu>
      </el-aside>

      <!-- 主内容区 -->
      <el-main class="main-content">
        <div v-if="activeMenu === 'new-scan'" class="content-section">
          <NewScan @scan-started="handleScanStarted" />
        </div>
        <div v-else-if="activeMenu === 'scans'" class="content-section">
          <el-row :gutter="20">
            <el-col :span="16">
              <ScanHistory @scan-selected="handleScanSelected" />
            </el-col>
            <el-col :span="8">
              <ScanLogs :scan-id="selectedScanId" />
            </el-col>
          </el-row>
        </div>
        <div v-else-if="activeMenu === 'settings'" class="content-section">
          <Settings />
        </div>
      </el-main>
    </el-container>
  </div>
</template>

<script setup lang="ts">
import { Plus, List, Setting } from '@element-plus/icons-vue'
import { ref } from 'vue'

const activeMenu = ref('new-scan')
const selectedScanId = ref<number | null>(null)

const handleMenuSelect = (index: string) => {
  activeMenu.value = index
}

const handleScanStarted = (scanId: number) => {
  activeMenu.value = 'scans'
  selectedScanId.value = scanId
  // 可以在这里触发扫描历史页面的刷新
}

const handleScanSelected = (scanId: number) => {
  selectedScanId.value = scanId
  console.log('Selected scan:', scanId)
}
</script>

<style scoped>
.scan-page {
  height: 100%;
  width: 100%;
  display: flex;
  flex-direction: column;
}

.main-container {
  height: 100%;
  width: 100%;
}

.sidebar {
  background-color: var(--bg-primary, #1e1e1e);
  border-right: 1px solid var(--border-primary, #333);
  display: flex;
  flex-direction: column;
}

.sidebar-header {
  padding: 20px;
  border-bottom: 1px solid var(--border-primary, #333);
  background-color: var(--bg-secondary, #252525);
}

.sidebar-header h1 {
  margin: 0;
  font-size: 24px;
  font-weight: 600;
  color: var(--text-primary, #fff);
  margin-bottom: 8px;
}

.subtitle {
  margin: 0;
  font-size: 12px;
  color: var(--text-secondary, #999);
}

.sidebar-menu {
  flex: 1;
  border-right: none;
  background-color: var(--bg-primary, #1e1e1e);
}

.main-content {
  background-color: var(--bg-primary, #1e1e1e);
  padding: 20px;
  overflow-y: auto;
}

.content-section {
  width: 100%;
}
</style>

