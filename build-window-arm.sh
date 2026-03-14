#!/bin/bash

# 构建目录
BUILD_DIR="src-tauri/target/aarch64-pc-windows-msvc/release"
mkdir -p "$BUILD_DIR"

# 构建 ARM64 版本
echo "构建 Windows 11 ARM 版本..."
export PKG_CONFIG_ALLOW_CROSS=1
npm run tauri build -- --target aarch64-pc-windows-msvc

if [ $? -eq 0 ]; then
    cp "$BUILD_DIR/openclaw-manager.exe" "$BUILD_DIR/OpenClaw-Manager-ARM64.exe"
    echo "✅ 已创建 OpenClaw-Manager-ARM64.exe"
else
    echo "❌ ARM64 构建失败"
    exit 1
fi

# 显示结果
ls -lh "$BUILD_DIR"/*.exe