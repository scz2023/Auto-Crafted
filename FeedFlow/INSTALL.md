# 安装和运行指南

## 前置要求

- Node.js 18+ 
- npm 或 yarn
- Rust (用于 Tauri，如果使用 Tauri 功能)

## 安装步骤

1. 安装依赖：
```bash
npm install
```

2. 开发模式运行（Web）：
```bash
npm run dev
```

应用将在 http://localhost:3000 运行

3. Tauri 开发模式：
```bash
npm run tauri:dev
```

4. 构建生产版本：
```bash
npm run build
```

5. 构建 Tauri 应用：
```bash
npm run tauri:build
```

## 使用说明

1. **添加订阅**：点击左侧菜单的"订阅"，然后点击"添加订阅"按钮，输入 RSS 订阅地址
2. **查看文章**：点击"全部文章"查看所有订阅的文章列表
3. **阅读文章**：在文章列表中点击文章，右侧会显示文章内容
4. **设置**：点击"设置"可以配置应用的各种选项

## 注意事项

- 数据库文件 `rss-reader.db` 会在首次运行时自动创建
- 在 Tauri 环境中，数据库文件会保存在应用数据目录
- 在开发环境中，数据库文件会保存在项目根目录

