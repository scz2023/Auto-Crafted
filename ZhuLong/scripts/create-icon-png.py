#!/usr/bin/env python3
"""
将 SVG 图标转换为 PNG 格式
需要安装: pip install cairosvg pillow
"""

import os
import sys

try:
    import cairosvg
    from PIL import Image
except ImportError:
    print("❌ 缺少依赖，请安装: pip install cairosvg pillow")
    sys.exit(1)

def svg_to_png(svg_path, png_path, size=1024):
    """将 SVG 转换为 PNG"""
    try:
        # 使用 cairosvg 转换
        cairosvg.svg2png(
            url=svg_path,
            write_to=png_path,
            output_width=size,
            output_height=size
        )
        print(f"✅ 成功生成 {png_path} ({size}x{size})")
        return True
    except Exception as e:
        print(f"❌ 转换失败: {e}")
        return False

if __name__ == "__main__":
    script_dir = os.path.dirname(os.path.abspath(__file__))
    project_root = os.path.dirname(script_dir)
    
    svg_path = os.path.join(project_root, "src-tauri", "icons", "icon.svg")
    png_path = os.path.join(project_root, "src-tauri", "icons", "icon-1024.png")
    
    if not os.path.exists(svg_path):
        print(f"❌ SVG 文件不存在: {svg_path}")
        sys.exit(1)
    
    print("🕯️ 开始转换图标...")
    if svg_to_png(svg_path, png_path, 1024):
        print("✅ 转换完成！现在可以运行 generate-icons.ps1 生成所有格式的图标")
    else:
        print("❌ 转换失败")

