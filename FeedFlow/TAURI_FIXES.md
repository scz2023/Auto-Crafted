# Tauri 配置修复记录

## 修复的问题

### 1. Tauri 配置文件格式错误
**错误**: `Additional properties are not allowed ('devPath', 'distDir' were unexpected)`

**修复**:
- 将 `devPath` 改为 `devUrl`
- 将 `distDir` 改为 `frontendDist`
- 添加了 `$schema` 用于配置验证

### 2. Cargo.toml 版本格式错误
**错误**: `Failed to parse version '2.1' for crate 'tauri'`

**修复**:
- 将版本号从 `"2.1"` 改为 `"2"`（自动使用最新 2.x 版本）

### 3. 不存在的功能
**错误**: `package 'rss-reader' depends on 'tauri' with feature 'shell-open' but 'tauri' does not have that feature`

**修复**:
- 从 `tauri` 依赖中移除了 `shell-open` 功能（Tauri 2.x 中已不存在）
- 保留了 `tauri-plugin-shell` 插件（如果需要 shell 功能）

### 4. 缺少图标文件
**错误**: `` `icons/icon.ico` not found; required for generating a Windows Resource file ``

**修复**:
- 创建了 `app-icon.svg` 源图标文件
- 使用 Tauri CLI 生成了所有必需的图标文件：
  - `icon.ico` (Windows)
  - `icon.icns` (macOS)
  - `32x32.png`, `128x128.png`, `128x128@2x.png` (其他平台)
  - 以及 Android 和 iOS 所需的图标

## 当前配置

### tauri.conf.json
```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "RSS Reader",
  "version": "1.0.0",
  "identifier": "com.rssreader.app",
  "build": {
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build",
    "devUrl": "http://localhost:3000",
    "frontendDist": "../.output/public"
  },
  ...
}
```

### Cargo.toml
```toml
[dependencies]
tauri = { version = "2", features = [] }
tauri-plugin-shell = { version = "2", features = [] }
```

### main.rs
```rust
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

## 生成图标文件

使用以下命令生成图标：
```bash
npx @tauri-apps/cli icon src-tauri/app-icon.svg -o src-tauri/icons
```

生成的图标文件包括：
- Windows: `icon.ico`
- macOS: `icon.icns`
- Linux: `32x32.png`, `128x128.png`, `128x128@2x.png`
- Android/iOS: 完整的图标集

## 下一步

现在可以正常运行：
```bash
npm run tauri:dev
```

如果不需要 shell 功能，可以：
1. 从 `Cargo.toml` 中移除 `tauri-plugin-shell`
2. 从 `main.rs` 中移除 `.plugin(tauri_plugin_shell::init())`

