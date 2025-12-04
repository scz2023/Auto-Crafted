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
        
        <div class="sidebar-footer">
          <div class="tool-info">
            <p class="tool-title">基于 Strix</p>
            <p class="tool-desc">
              本项目使用 <a href="https://github.com/usestrix/strix" target="_blank" rel="noopener noreferrer">Strix</a> 
              开源 AI 渗透测试工具，提供自动化安全扫描和漏洞检测能力。
            </p>
          </div>
        </div>
      </el-aside>

      <!-- 主内容区 -->
      <el-main class="main-content">
        <div v-if="activeMenu === 'new-scan'" class="content-section">
          <NewScan @scan-started="handleScanStarted" />
        </div>
        <div v-else-if="activeMenu === 'scans'" class="content-section">
          <div class="scans-layout">
            <div class="scan-history-section">
              <ScanHistory @scan-selected="handleScanSelected" />
            </div>
            <div class="scan-logs-section">
              <ScanLogs :scan-id="selectedScanId" />
            </div>
          </div>
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
  background-color: var(--bg-primary, #ffffff);
  border-right: 1px solid var(--border-primary, #d4d4d4);
  display: flex;
  flex-direction: column;
}

.sidebar-header {
  padding: 20px;
  border-bottom: 1px solid var(--border-primary, #d4d4d4);
  background-color: var(--bg-secondary, #f5f5f5);
}

.sidebar-header h1 {
  margin: 0;
  font-size: 24px;
  font-weight: 600;
  color: var(--text-primary, #000000);
  margin-bottom: 8px;
}

.subtitle {
  margin: 0;
  font-size: 12px;
  color: var(--text-secondary, #333333);
}

.sidebar-menu {
  flex: 1;
  border-right: none;
  background-color: var(--bg-primary, #ffffff);
}

.sidebar-footer {
  padding: 16px 20px;
  border-top: 1px solid var(--border-primary, #d4d4d4);
  background-color: var(--bg-secondary, #f5f5f5);
}

.tool-info {
  font-size: 12px;
  line-height: 1.5;
}

.tool-title {
  margin: 0 0 8px 0;
  font-weight: 600;
  color: var(--text-primary, #000000);
  font-size: 13px;
}

.tool-desc {
  margin: 0;
  color: var(--text-secondary, #333333);
  font-size: 11px;
}

.tool-desc a {
  color: var(--color-primary, #409eff);
  text-decoration: none;
}

.tool-desc a:hover {
  text-decoration: underline;
}

.main-content {
  background-color: var(--bg-primary, #ffffff);
  padding: 20px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  height: 100%;
}

.content-section {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.scans-layout {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  gap: 20px;
  overflow: hidden;
}

.scan-history-section {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.scan-logs-section {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}
</style>

