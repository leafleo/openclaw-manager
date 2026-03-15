#!/bin/bash

set -e

echo "===================================="
echo "Installing OpenClaw Runtime (macOS)"
echo "===================================="
echo ""

# 获取脚本所在目录
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$SCRIPT_DIR/../../"

# 解压 Node.js (x64)
NODE_TAR="$SCRIPT_DIR/node/node-v18.20.4-darwin-x64.tar.gz"
NODE_DIR="$ROOT_DIR/runtime/node"

echo "Extracting Node.js (x64)..."
if [ ! -d "$NODE_DIR" ]; then
    mkdir -p "$NODE_DIR"
fi
tar -xzf "$NODE_TAR" -C "$NODE_DIR"

# 解压 Git
echo "Mounting Git DMG..."
GIT_DMG="$SCRIPT_DIR/git/git-2.33.0-intel-universal-mavericks.dmg"
MOUNT_POINT="/Volumes/Git"

hdiutil mount "$GIT_DMG"

# 安装 Git
echo "Installing Git..."
sudo installer -pkg "$MOUNT_POINT/Git.pkg" -target /

hdiutil unmount "$MOUNT_POINT"

# 配置环境变量
NODE_BIN="$NODE_DIR/node-v18.20.4-darwin-x64/bin"

# 添加到 .zprofile
if [ -f "$HOME/.zprofile" ]; then
    echo "export NODE_HOME=$NODE_DIR/node-v18.20.4-darwin-x64" >> "$HOME/.zprofile"
    echo "export PATH=$NODE_BIN:$PATH" >> "$HOME/.zprofile"
else
    echo "export NODE_HOME=$NODE_DIR/node-v18.20.4-darwin-x64" > "$HOME/.zprofile"
    echo "export PATH=$NODE_BIN:$PATH" >> "$HOME/.zprofile"
fi

# 添加到 .bash_profile
if [ -f "$HOME/.bash_profile" ]; then
    echo "export NODE_HOME=$NODE_DIR/node-v18.20.4-darwin-x64" >> "$HOME/.bash_profile"
    echo "export PATH=$NODE_BIN:$PATH" >> "$HOME/.bash_profile"
fi

echo ""
echo "===================================="
echo "Installation complete!"
echo "===================================="
echo ""
echo "Node.js: $NODE_BIN/node"
echo "Git: $(which git)"
echo ""
echo "You can now run: node --version"
echo "You can now run: git --version"
echo ""
echo "Note: You may need to restart your terminal for environment changes to take effect."
echo ""
