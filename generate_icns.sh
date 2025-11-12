#!/bin/bash

# 脚本用于从 PNG 文件生成 macOS .icns 图标文件
# 使用方法: ./generate_icns.sh

set -e

LOGO_PNG="assets/logo-black.png"
ICONSET_DIR="assets/logo.iconset"
OUTPUT_ICNS="assets/logo.icns"

echo "🎨 开始生成 macOS .icns 图标文件..."

# 检查源文件是否存在
if [ ! -f "$LOGO_PNG" ]; then
    echo "❌ 错误：找不到源图标文件 $LOGO_PNG"
    exit 1
fi

# 创建 iconset 目录
mkdir -p "$ICONSET_DIR"

echo "📐 生成不同尺寸的图标..."

# 生成所有需要的尺寸
sips -z 16 16     "$LOGO_PNG" --out "$ICONSET_DIR/icon_16x16.png" > /dev/null
sips -z 32 32     "$LOGO_PNG" --out "$ICONSET_DIR/icon_16x16@2x.png" > /dev/null
sips -z 32 32     "$LOGO_PNG" --out "$ICONSET_DIR/icon_32x32.png" > /dev/null
sips -z 64 64     "$LOGO_PNG" --out "$ICONSET_DIR/icon_32x32@2x.png" > /dev/null
sips -z 128 128   "$LOGO_PNG" --out "$ICONSET_DIR/icon_128x128.png" > /dev/null
sips -z 256 256   "$LOGO_PNG" --out "$ICONSET_DIR/icon_128x128@2x.png" > /dev/null
sips -z 256 256   "$LOGO_PNG" --out "$ICONSET_DIR/icon_256x256.png" > /dev/null
sips -z 512 512   "$LOGO_PNG" --out "$ICONSET_DIR/icon_256x256@2x.png" > /dev/null
sips -z 512 512   "$LOGO_PNG" --out "$ICONSET_DIR/icon_512x512.png" > /dev/null
sips -z 1024 1024 "$LOGO_PNG" --out "$ICONSET_DIR/icon_512x512@2x.png" > /dev/null

echo "🔧 生成 .icns 文件..."

# 从 iconset 生成 .icns 文件
iconutil -c icns "$ICONSET_DIR" -o "$OUTPUT_ICNS"

# 清理临时文件
rm -rf "$ICONSET_DIR"

echo "✅ 成功生成 $OUTPUT_ICNS"
echo "📦 文件大小: $(du -h "$OUTPUT_ICNS" | cut -f1)"
