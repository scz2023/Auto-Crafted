<template>
  <div class="about-page">
    <div class="about-container">
      <div class="about-header">
        <h1>FeedFlow</h1>
        <p class="version">版本 {{ version }}</p>
      </div>

      <el-divider />

      <div class="about-content">
        <div class="section">
          <h2>项目信息</h2>
          <div class="info-item">
            <span class="label">项目名称：</span>
            <span class="value">FeedFlow</span>
          </div>
          <div class="info-item">
            <span class="label">版本号：</span>
            <span class="value">{{ version }}</span>
          </div>
          <div class="info-item">
            <span class="label">描述：</span>
            <span class="value">现代化的跨平台 RSS 阅读器，让信息流更智能、更优雅</span>
          </div>
        </div>

        <el-divider />

        <div class="section">
          <h2>技术栈</h2>
          <div class="tech-stack">
            <div class="tech-category">
              <h3>前端框架</h3>
              <ul>
                <li><strong>Nuxt</strong> - {{ nuxtVersion }}</li>
                <li><strong>Vue</strong> - {{ vueVersion }}</li>
                <li><strong>TypeScript</strong> - {{ typescriptVersion }}</li>
              </ul>
            </div>

            <div class="tech-category">
              <h3>UI 组件库</h3>
              <ul>
                <li><strong>Element Plus</strong> - {{ elementPlusVersion }}</li>
              </ul>
            </div>

            <div class="tech-category">
              <h3>状态管理</h3>
              <ul>
                <li><strong>Pinia</strong> - {{ piniaVersion }}</li>
              </ul>
            </div>

            <div class="tech-category">
              <h3>桌面应用</h3>
              <ul>
                <li><strong>Tauri</strong> - {{ tauriVersion }}</li>
              </ul>
            </div>

            <div class="tech-category">
              <h3>数据库</h3>
              <ul>
                <li><strong>SQLite</strong> (better-sqlite3) - {{ sqliteVersion }}</li>
              </ul>
            </div>

            <div class="tech-category">
              <h3>RSS 解析</h3>
              <ul>
                <li><strong>rss-parser</strong> - {{ rssParserVersion }}</li>
              </ul>
            </div>
          </div>
        </div>

        <el-divider />

        <div class="section">
          <h2>功能特性</h2>
          <ul class="features">
            <li>支持 RSS 订阅源管理</li>
            <li>自动刷新订阅内容</li>
            <li>文章收藏功能</li>
            <li>按订阅源筛选文章</li>
            <li>无限滚动加载</li>
            <li>深色/浅色主题切换</li>
            <li>字体大小自定义</li>
            <li>文章自动清理</li>
            <li>AI 智能总结和翻译</li>
            <li>OPML 导入/导出</li>
            <li>分类管理</li>
          </ul>
        </div>

        <el-divider />

        <div class="section">
          <h2>作者信息</h2>
          <div class="author-info">
            <div class="author-item">
              <span class="label">开发者：</span>
              <span class="value">Auto (AI Assistant)</span>
            </div>
            <div class="author-item">
              <span class="label">开发方式：</span>
              <span class="value">AI 辅助开发</span>
            </div>
            <div class="author-item">
              <span class="label">开发平台：</span>
              <span class="value">Cursor IDE</span>
            </div>
            <div class="author-description">
              <p>FeedFlow 由 AI 助手 Auto 在 Cursor IDE 中全程辅助开发完成。从项目架构设计、功能实现到界面优化，每一个细节都经过精心打磨，致力于打造一个现代化、智能化的 RSS 阅读体验。</p>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'

const version = ref('1.0.0')
const nuxtVersion = ref('4.2.1')
const vueVersion = ref('3.5.23')
const typescriptVersion = ref('5.9.3')
const elementPlusVersion = ref('2.11.8')
const piniaVersion = ref('3.0.4')
const tauriVersion = ref('2.9.0')
const sqliteVersion = ref('12.4.6')
const rssParserVersion = ref('3.13.0')

// 版本信息（从 package.json 同步）
onMounted(() => {
  // 版本信息已从 package.json 同步设置
  version.value = '1.0.0'
  nuxtVersion.value = '4.2.1'
  vueVersion.value = '3.5.23'
  typescriptVersion.value = '5.9.3'
  elementPlusVersion.value = '2.11.8'
  piniaVersion.value = '3.0.4'
  tauriVersion.value = '2.9.0'
  sqliteVersion.value = '12.4.6'
  rssParserVersion.value = '3.13.0'
})
</script>

<style scoped>
.about-page {
  height: 100%;
  width: 100%;
  overflow-y: auto;
  background-color: var(--bg-tertiary);
  padding: 40px 20px;
  transition: background-color 0.3s ease;
}

.about-container {
  max-width: 900px;
  margin: 0 auto;
  background-color: var(--bg-primary);
  border-radius: 8px;
  padding: 40px;
  box-shadow: var(--shadow-md);
  transition: background-color 0.3s ease, box-shadow 0.3s ease;
}

.about-header {
  text-align: center;
  margin-bottom: 20px;
}

.about-header h1 {
  margin: 0 0 10px 0;
  font-size: 32px;
  color: var(--text-primary);
  font-weight: 600;
  transition: color 0.3s ease;
}

.version {
  margin: 0;
  font-size: 16px;
  color: var(--text-tertiary);
  transition: color 0.3s ease;
}

.about-content {
  margin-top: 30px;
}

.section {
  margin-bottom: 30px;
}

.section h2 {
  margin: 0 0 20px 0;
  font-size: 20px;
  color: var(--text-primary);
  font-weight: 600;
  transition: color 0.3s ease;
}

.section h3 {
  margin: 0 0 12px 0;
  font-size: 16px;
  color: var(--text-secondary);
  font-weight: 500;
  transition: color 0.3s ease;
}

.info-item {
  margin-bottom: 12px;
  font-size: 14px;
  line-height: 1.8;
}

.info-item .label {
  color: var(--text-secondary);
  font-weight: 500;
  margin-right: 8px;
  transition: color 0.3s ease;
}

.info-item .value {
  color: var(--text-primary);
  transition: color 0.3s ease;
}

.tech-stack {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: 24px;
  margin-top: 20px;
}

.tech-category {
  background-color: var(--bg-secondary);
  padding: 20px;
  border-radius: 6px;
  border: 1px solid var(--border-primary);
  transition: background-color 0.3s ease, border-color 0.3s ease;
}

.tech-category ul {
  margin: 0;
  padding-left: 20px;
  list-style-type: disc;
}

.tech-category li {
  margin-bottom: 8px;
  font-size: 14px;
  color: var(--text-secondary);
  line-height: 1.6;
  transition: color 0.3s ease;
}

.tech-category li strong {
  color: var(--text-primary);
  font-weight: 600;
  transition: color 0.3s ease;
}

.features {
  margin: 0;
  padding-left: 20px;
  list-style-type: disc;
}

.features li {
  margin-bottom: 10px;
  font-size: 14px;
  color: var(--text-secondary);
  line-height: 1.8;
  transition: color 0.3s ease;
}

/* 作者信息样式 */
.author-info {
  margin-top: 20px;
}

.author-item {
  margin-bottom: 12px;
  font-size: 14px;
  line-height: 1.8;
}

.author-item .label {
  color: var(--text-secondary);
  font-weight: 500;
  margin-right: 8px;
  transition: color 0.3s ease;
}

.author-item .value {
  color: var(--text-primary);
  transition: color 0.3s ease;
}

.author-description {
  margin-top: 20px;
  padding: 20px;
  background-color: var(--bg-secondary);
  border-radius: 6px;
  border-left: 4px solid var(--color-primary);
  transition: background-color 0.3s ease, border-color 0.3s ease;
}

.author-description p {
  margin: 0;
  font-size: 14px;
  line-height: 1.8;
  color: var(--text-secondary);
  transition: color 0.3s ease;
}
</style>

