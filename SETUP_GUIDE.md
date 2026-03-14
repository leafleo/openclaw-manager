# 🚀 OpenClaw Manager — 安装配置指南

## 面向终端用户（下载发行版）

如果您下载了发行版安装包（`.msi`、`.exe`、`.dmg` 或 `.AppImage`），只需**打开应用** — 内置的**安装向导**将自动完成以下操作：

1. ✅ 检测您的操作系统
2. ✅ 检查 **Node.js** (>= 18) 和 **Git** 是否已安装
3. ✅ 一键安装任何缺失的依赖项
4. ✅ 安装 **OpenClaw** 并初始化配置

无需使用终端。

---

## 面向开发者（从源码构建）

### 环境要求

| 依赖项 | 版本要求 | 下载地址 |
|--------|----------|----------|
| **Node.js** | >= 18.0 | [nodejs.org](https://nodejs.org/) |
| **Rust** | >= 1.70 | [rustup.rs](https://rustup.rs/) |
| **Git** | 最新版 | [git-scm.com](https://git-scm.com/) |

> [!TIP]
> 验证安装是否成功：
> ```bash
> node --version    # 应显示 v18.x 或更高版本
> rustc --version   # 应显示 1.70 或更高版本
> git --version
> ```

### 平台特定依赖

<details>
<summary><b>🪟 Windows</b></summary>

- [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) — 选择 **"使用 C++ 的桌面开发"** 工作负载
- [WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) *(Windows 10/11 已预装)*
</details>

<details>
<summary><b>🍎 macOS</b></summary>

```bash
xcode-select --install
```
</details>

<details>
<summary><b>🐧 Linux (Ubuntu/Debian)</b></summary>

```bash
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file \
  libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
```
</details>

<details>
<summary><b>🐧 Linux (Fedora)</b></summary>

```bash
sudo dnf install webkit2gtk4.1-devel openssl-devel curl wget file libxdo-devel
```
</details>

---

### 第一步 — 克隆仓库

```bash
git clone https://github.com/MrFadiAi/openclaw-one-click-installer.git
cd openclaw-one-click-installer
```

### 第二步 — 安装依赖

```bash
npm install
```

此命令将安装所有前端（React、Vite、TailwindCSS）和 Tauri CLI 依赖。

### 第三步 — 运行开发模式

```bash
npm run tauri:dev
```

此命令将：
1. 启动 **Vite** 开发服务器（支持热重载的 React 前端）
2. 编译 **Rust** 后端（首次运行需要 3-5 分钟）
3. 打开原生桌面应用窗口

> [!NOTE]
> 首次构建会编译所有 Rust 依赖，可能需要 **3-5 分钟**。后续运行会快得多，因为会使用缓存。

---

## 常用命令

| 命令 | 说明 |
|------|------|
| `npm run dev` | 仅在浏览器中运行前端（无 Tauri） |
| `npm run build` | 构建前端 |
| `npm run tauri:dev` | 完整的桌面应用，支持热重载 |
| `npm run tauri:build` | 构建发行版安装包（`.msi` / `.exe` / `.dmg`） |
| `cd src-tauri && cargo check` | 检查 Rust 代码错误 |
| `cd src-tauri && cargo test` | 运行 Rust 测试 |

---

## 应用功能概览

应用运行后，您可以在侧边栏找到以下功能模块：

| 页面 | 功能说明 |
|------|----------|
| **概览** | 服务状态仪表板、快捷操作（启动/停止/重启）、系统要求面板 |
| **MCP** | 管理 MCP 服务器 — 添加、编辑、测试、启用/禁用。安装 mcporter。自动同步到 `~/.mcporter/mcporter.json` |
| **技能** | 通过 ClawHub 浏览和安装 OpenClaw 技能 |
| **AI 配置** | 配置 AI 提供商（14+）、设置 API 密钥、选择主模型 |
| **渠道** | 设置消息集成（Telegram、Discord、飞书、Slack 等） |
| **测试** | 运行系统、AI 和渠道连接性诊断 |
| **日志** | 查看结构化应用日志，支持级别过滤和导出 |
| **设置** | 通用应用设置 |

---

## 构建输出

执行 `npm run tauri:build` 后，安装包将位于：

```
src-tauri/target/release/bundle/
├── msi/    → .msi 安装包 (Windows)
├── nsis/   → .exe 安装包 (Windows)
├── dmg/    → .dmg 镜像 (macOS)
├── deb/    → .deb 包 (Linux)
└── appimage/ → .AppImage (Linux)
```

---

## 故障排除

### `npm install` 失败，提示 `ENOENT`
请确保您在正确的项目目录中。

### Tauri 版本不匹配
如果您看到版本不匹配的错误，请运行 `npm install` 更新依赖。

### Rust 编译错误
确保已安装 C++ 构建工具：
- **Windows**：打开 Visual Studio 安装程序 → 选择 **"使用 C++ 的桌面开发"**
- **macOS**：运行 `xcode-select --install`

### WebView2 缺失（Windows）
下载并安装 [WebView2 Evergreen Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/)。

### macOS "已损坏，无法打开"
```bash
xattr -cr /Applications/OpenClaw\ Manager.app
```

或者前往 **系统偏好设置** > **隐私与安全性** → 点击 **仍要打开**。

---

## 项目结构

```
openclaw-manager/
├── src-tauri/                 # Rust 后端
│   ├── src/
│   │   ├── main.rs            # 入口点
│   │   ├── commands/          # 后端逻辑（配置、安装、服务等）
│   │   ├── models/            # 数据结构
│   │   └── utils/             # 工具函数
│   ├── Cargo.toml
│   └── tauri.conf.json
│
├── src/                       # React 前端
│   ├── components/            # UI 组件（仪表板、设置、特定功能）
│   ├── hooks/                 # 自定义 Hooks
│   ├── lib/                   # API 绑定
│   ├── stores/                # 状态管理（Zustand）
│   └── styles/                # Tailwind CSS
│
├── package.json
└── vite.config.ts
```

---

## 技术栈

| 层级 | 技术 | 用途 |
|------|------|------|
| 前端 | React 18 | UI 框架 |
| 前端 | TypeScript | 类型安全 |
| 前端 | Vite | 构建工具 |
| 前端 | TailwindCSS | 样式 |
| 前端 | Zustand | 状态管理 |
| 后端 | Rust | 原生性能 |
| 后端 | Tauri 2.0 | 桌面应用框架 |
| 后端 | Tokio | 异步运行时 |

---

## 更多信息

- 📖 [多代理配置指南](./MULTI_AGENT_GUIDE.md)
- 📦 [发行说明](./RELEASE.md)
- 🐛 [问题反馈](https://github.com/MrFadiAi/openclaw-one-click-installer/issues)

---

<p align="center">用 ❤️ 构建 | OpenClaw Team</p>
