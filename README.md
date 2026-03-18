# OpenClaw Installer

OpenClaw AI Gateway 的一键安装器。用户只需点一个按钮（或一行命令），无需终端、无需自己装 Node.js、无需 Docker。

## 支持平台

| 平台 | 安装包 | 服务管理 | 状态 |
|------|--------|---------|------|
| macOS ARM (Apple Silicon) | `.dmg` | launchd | ✅ |
| macOS Intel | `.dmg` | launchd | ✅ |
| Windows x64 | `.msi` / `.exe` | NSSM | ✅ |
| Linux x64 | `.deb` / `.AppImage` | systemd --user | ✅ |
| Linux ARM64 | `.deb` / `.AppImage` | systemd --user | ✅ |
| Android（遥控器） | `.apk` | — | ✅ |
| Android（本机 Server） | `.apk` | Foreground Service | ✅ |

## 用户一键安装

### macOS / Linux

```bash
curl -fsSL https://github.com/zsyc2601-ship-it/openclaw-installer/raw/main/scripts/install.sh | bash
```

### Windows (PowerShell)

```powershell
irm https://github.com/zsyc2601-ship-it/openclaw-installer/raw/main/scripts/install.ps1 | iex
```

### 中国大陆用户

脚本会自动检测网络环境，如果 GitHub 不通则自动切换到 Gitee 镜像。npm 安装也会自动使用 npmmirror 加速。

如果自动检测失败，可手动使用 Gitee 源：

```bash
# macOS / Linux
curl -fsSL https://gitee.com/zsyc2601-ship-it/openclaw-installer/raw/main/scripts/install.sh | bash

# Windows
irm https://gitee.com/zsyc2601-ship-it/openclaw-installer/raw/main/scripts/install.ps1 | iex
```

### Android

从 [Releases](https://github.com/zsyc2601-ship-it/openclaw-installer/releases) 下载：
- `openclaw-companion.apk` — 遥控器，扫码连接电脑上的 Gateway
- `openclaw-server.apk` — 在手机上直接运行 OpenClaw Gateway

---

## 安装流程（用户视角）

```
执行一键命令 / 下载安装包
         │
         ▼
┌──────────────────┐
│  [ 一键安装 ]     │  ← 用户唯一操作：点这个按钮
└──────────────────┘
         │  全自动
         ▼
Step 1: 检测系统环境        ~0.5s
Step 2: 释放内嵌 Node.js    ~2s
Step 3: npm install openclaw ~30-120s
Step 4: 注册系统服务         ~2s
Step 5: 启动 Gateway        ~3s
Step 6: 健康检查            ~2s
         │
         ▼
┌──────────────────┐
│  输入 API Key     │  ← 唯一需要填写的东西
└──────────────────┘
         │
         ▼
┌──────────────────┐
│  ✅ 安装完成       │
│  Gateway 运行中   │
│  localhost:18789  │
└──────────────────┘
```

安装完成后 OpenClaw 作为系统服务常驻后台，重启电脑自动恢复。

---

## 开发者指南

### 项目结构

```
├── apps/
│   ├── installer/              # Tauri 2.x 桌面安装器 (macOS/Windows/Linux)
│   │   ├── src/                # React + TypeScript 前端
│   │   └── src-tauri/          # Rust 后端
│   ├── android/                # Android 遥控器 (Kotlin + Compose)
│   └── android-server/         # Android Server (内嵌 Node.js)
├── scripts/
│   ├── install.sh              # macOS/Linux 一键安装脚本
│   ├── install.ps1             # Windows 一键安装脚本
│   ├── download-node.sh        # 下载各平台 Node.js 归档
│   ├── download-node-android.sh # 下载 Android Node.js
│   └── download-nssm.sh        # 下载 NSSM (Windows 服务管理)
├── .github/workflows/
│   ├── build.yml               # CI: 构建全平台桌面安装包
│   ├── android.yml             # CI: 构建 Android APK
│   └── sync-gitee.yml          # 自动同步到 Gitee
├── Dockerfile                  # Linux 容器构建
└── docker-compose.yml
```

### 前置依赖

- Node.js ≥ 22
- pnpm ≥ 9
- Rust (stable)
- `cargo install tauri-cli --version "^2"`

### 构建桌面安装包

```bash
# 1. 下载 Node.js 归档到 resources/
bash scripts/download-node.sh

# 2. 安装前端依赖
cd apps/installer && pnpm install

# 3. 构建（产物在 src-tauri/target/release/bundle/）
npx tauri build
```

### 构建 Linux（Docker）

```bash
# 下载 Node.js
make download-node

# 完整 Linux 构建
make build-linux
```

### 构建 Android

```bash
# 遥控器
cd apps/android && ./gradlew assembleRelease

# Server（需先下载 Android Node.js）
bash scripts/download-node-android.sh
cd apps/android-server && ./gradlew assembleRelease
```

### CI 自动构建

推送 tag 即触发全平台构建：

```bash
git tag v0.1.0
git push origin v0.1.0
```

GitHub Actions 会自动构建：
- macOS ARM `.dmg`
- macOS Intel `.dmg`
- Windows `.msi` + `.exe`
- Linux `.deb` + `.AppImage`
- Android Companion `.apk`
- Android Server `.apk`

构建产物发布到 [Releases](https://github.com/zsyc2601-ship-it/openclaw-installer/releases)。

---

## 技术架构

| 组件 | 技术 |
|------|------|
| 桌面框架 | Tauri 2.x (Rust + WebView) |
| 前端 | React 18 + TypeScript + Vite |
| 状态管理 | Zustand |
| 进度推送 | `tauri::ipc::Channel<T>` |
| macOS 服务 | launchd (用户态) |
| Windows 服务 | NSSM |
| Linux 服务 | systemd --user |
| Android UI | Kotlin + Jetpack Compose |
| Android 扫码 | CameraX + ML Kit |
| 中国加速 | npmmirror + Gitee 自动切换 |
