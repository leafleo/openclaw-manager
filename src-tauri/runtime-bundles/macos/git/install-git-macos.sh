#!/bin/bash
# Git Installation Script for macOS
# This script installs Git using Homebrew

echo "Installing Git on macOS..."

# 检查 Homebrew 是否安装
if ! command -v brew &> /dev/null; then
    echo "Homebrew is not installed. Installing Homebrew..."
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    
    # 配置 Homebrew 环境变量
    export PATH="/opt/homebrew/bin:$PATH"
    export PATH="/usr/local/bin:$PATH"
fi

# 安装 Git
echo "Installing Git..."
brew install git

# 验证安装
echo "Verifying Git installation..."
git --version

echo "Git installation complete!"
