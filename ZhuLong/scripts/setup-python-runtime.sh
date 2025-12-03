#!/bin/bash
# 烛龙 - Python 运行时打包脚本 (Linux/macOS)
# 此脚本用于下载并打包 Python 解释器到应用中

set -e

PYTHON_VERSION="${1:-3.13.0}"
PLATFORM="${2:-linux-x64}"

echo "🕯️  烛龙 - Python 运行时打包工具"
echo "================================="

# 配置
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
RUNTIME_DIR="$PROJECT_ROOT/python-runtime"
PYTHON_DIR="$RUNTIME_DIR/python"

# 检测平台
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    PLATFORM="linux-x64"
    PYTHON_URL="https://www.python.org/ftp/python/${PYTHON_VERSION}/Python-${PYTHON_VERSION}.tgz"
    PYTHON_EXE="python3"
elif [[ "$OSTYPE" == "darwin"* ]]; then
    PLATFORM="macos-x64"
    PYTHON_URL="https://www.python.org/ftp/python/${PYTHON_VERSION}/Python-${PYTHON_VERSION}.tgz"
    PYTHON_EXE="python3"
else
    echo "❌ 不支持的平台: $OSTYPE"
    exit 1
fi

# 创建目录
echo ""
echo "📁 创建目录结构..."
if [ -d "$RUNTIME_DIR" ]; then
    echo "   目录已存在，将清理: $RUNTIME_DIR"
    rm -rf "$RUNTIME_DIR"
fi
mkdir -p "$PYTHON_DIR"

# 下载 Python
ARCHIVE_FILE="$RUNTIME_DIR/python.tar.gz"
echo ""
echo "⬇️  下载 Python $PYTHON_VERSION ($PLATFORM)..."
echo "   来源: $PYTHON_URL"

if command -v curl &> /dev/null; then
    curl -L -o "$ARCHIVE_FILE" "$PYTHON_URL"
elif command -v wget &> /dev/null; then
    wget -O "$ARCHIVE_FILE" "$PYTHON_URL"
else
    echo "❌ 需要 curl 或 wget 来下载文件"
    exit 1
fi

# 解压
echo ""
echo "📦 解压 Python..."
tar -xzf "$ARCHIVE_FILE" -C "$PYTHON_DIR" --strip-components=1 || {
    echo "❌ 解压失败"
    exit 1
}

# 清理
rm -f "$ARCHIVE_FILE"

# 编译 Python（如果需要）
echo ""
echo "🔨 编译 Python..."
cd "$PYTHON_DIR"
./configure --prefix="$PYTHON_DIR" --enable-optimizations || {
    echo "⚠️  配置失败，尝试使用系统 Python"
    # 如果编译失败，可以链接到系统 Python
    if command -v python3 &> /dev/null; then
        PYTHON_SYSTEM=$(which python3)
        ln -sf "$PYTHON_SYSTEM" "$PYTHON_DIR/$PYTHON_EXE"
        echo "   ✅ 链接到系统 Python: $PYTHON_SYSTEM"
    else
        echo "❌ 无法找到系统 Python"
        exit 1
    fi
}

# 创建说明文件
README_PATH="$RUNTIME_DIR/README.md"
cat > "$README_PATH" << EOF
# Python 运行时

此目录包含打包的 Python $PYTHON_VERSION 解释器。

## 目录结构

- \`python/\` - Python 解释器文件
  - \`python3\` - Python 可执行文件
  - \`lib/\` - Python 库文件
  - \`include/\` - Python 头文件

## 安装 pip（可选）

如果需要使用 pip 安装包，可以运行：

\`\`\`bash
python-runtime/python/python3 -m ensurepip --upgrade
\`\`\`

## 版本信息

- Python 版本: $PYTHON_VERSION
- 平台: $PLATFORM
- 打包时间: $(date '+%Y-%m-%d %H:%M:%S')
EOF

echo ""
echo "✅ Python 运行时打包完成！"
echo ""
echo "📝 下一步："
echo "   1. 检查 python-runtime/python/$PYTHON_EXE 是否存在"
echo "   2. 运行应用，Python 将自动使用打包的解释器"

