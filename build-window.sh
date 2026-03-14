#!/bin/bash

# 构建目录
BUILD_DIR="src-tauri/target/x86_64-pc-windows-gnu/release"
mkdir -p "$BUILD_DIR"

# 构建 Windows 7 版本
echo "构建 Windows 7 版本..."
export PKG_CONFIG_ALLOW_CROSS=1
export RUSTFLAGS="-C target-feature=+crt-static -C link-args=-static -C link-args=-static-libgcc -C link-args=-static-libstdc++"
npm run tauri build -- --target x86_64-pc-windows-gnu

if [ $? -eq 0 ]; then
    cp "$BUILD_DIR/openclaw-manager.exe" "$BUILD_DIR/OpenClaw-Manager-Win7.exe"
    echo "✅ 已创建 OpenClaw-Manager-Win7.exe"
else
    echo "❌ Windows 7 构建失败"
    exit 1
fi

# 构建 Windows 10/11 版本
echo "构建 Windows 10/11 版本..."
export PKG_CONFIG_ALLOW_CROSS=1
export RUSTFLAGS="-C link-args=-static -C link-args=-static-libgcc -C link-args=-static-libstdc++"
npm run tauri build -- --target x86_64-pc-windows-gnu

if [ $? -eq 0 ]; then
    cp "$BUILD_DIR/openclaw-manager.exe" "$BUILD_DIR/OpenClaw-Manager-Win10.exe"
    echo "✅ 已创建 OpenClaw-Manager-Win10.exe"
else
    echo "❌ Windows 10/11 构建失败"
    exit 1
fi

# 显示结果
ls -lh "$BUILD_DIR"/*.exe