#!/bin/bash

set -e

echo "===================================="
echo "Installing OpenClaw 2026.3.12"
echo "===================================="
echo ""

# 检查 Node.js 是否安装
if ! command -v node &> /dev/null; then
    echo "ERROR: Node.js is not installed. Please install Node.js first."
    exit 1
fi

echo "✓ Node.js found: $(node --version)"

# 检查 Git 是否安装
if ! command -v git &> /dev/null; then
    echo "ERROR: Git is not installed. Please install Git first."
    exit 1
fi

echo "✓ Git found: $(git --version)"

# 获取脚本所在目录
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
TARBALL="${SCRIPT_DIR}/openclaw-2026.3.12.tgz"

# 本地安装 OpenClaw
echo "Installing OpenClaw from local package..."
cd "${SCRIPT_DIR}"
npm install "${TARBALL}" --save-exact

# 启动 OpenClaw 网关
echo "Starting OpenClaw Gateway..."
openclaw gateway start

# 检查网关状态
echo "Checking Gateway status..."
openclaw gateway status

echo ""
echo "===================================="
echo "Installation complete!"
echo "===================================="
