# 如何查看 Tauri 应用的控制台输出

在 Windows 上，Tauri 应用默认在 release 模式下会隐藏控制台窗口。以下是几种查看控制台输出的方法：

## 方法 1: 从命令行运行（最简单）

直接在命令行（PowerShell 或 CMD）中运行 exe 文件：

```powershell
cd E:\Auto-Crafted\FeedFlow\dist-portable
.\feedflow.exe
```

或者：

```cmd
cd E:\Auto-Crafted\FeedFlow\dist-portable
feedflow.exe
```

这样可以看到所有的 `println!` 和 `eprintln!` 输出。

## 方法 2: 启用控制台窗口（临时调试）

修改 `src-tauri/src/main.rs`，临时注释掉控制台隐藏代码：

```rust
// 临时注释掉这行以显示控制台
// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
```

然后重新构建：

```bash
npm run tauri:build:portable:ps1
```

**注意**：完成后记得恢复这行代码，因为隐藏控制台是生产版本的标准做法。

## 方法 3: 使用日志文件

修改代码将输出写入日志文件。我已经创建了 `main-debug.rs` 作为参考，它包含了启用控制台的代码。

## 方法 4: 使用 Windows 事件查看器

查看 Windows 事件查看器中的应用程序日志（但 Tauri 应用通常不会写入事件日志）。

## 推荐方法

**最简单的方法是从命令行运行**：

1. 打开 PowerShell 或 CMD
2. 导航到便携版目录：
   ```powershell
   cd E:\Auto-Crafted\FeedFlow\dist-portable
   ```
3. 运行应用：
   ```powershell
   .\feedflow.exe
   ```

这样你就可以看到所有的控制台输出，包括：
- 服务器启动信息
- 错误消息
- 调试信息

## 查看服务器启动状态

运行应用后，你应该能看到类似这样的输出：

```
Executable directory: "E:\\Auto-Crafted\\FeedFlow\\dist-portable"
Attempting to start Nitro server...
Starting Nitro server: "E:\\Auto-Crafted\\FeedFlow\\dist-portable\\.output\\server\\index.mjs"
Working directory: "E:\\Auto-Crafted\\FeedFlow\\dist-portable\\.output\\server"
Node command: node.exe
Nitro server started and responding (attempt 3/20)
Nitro server process started successfully
```

如果看到错误，请检查：
1. `.output/server` 目录是否存在
2. Node.js 是否已安装并在 PATH 中
3. 端口 3000 是否被占用

