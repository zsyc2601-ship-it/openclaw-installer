# OpenClaw 一键部署器 — 技术方案

## 项目定位

傻瓜式一键安装/卸载 OpenClaw 的跨平台客户端。用户只需点一个按钮，无需终端、无需自己装 Node.js、无需 Docker。

---

## 目标平台

| 平台 | 技术 | 产物 |
|------|------|------|
| Windows | Tauri 2.x | `.msi` / `.exe` (NSIS) |
| macOS | Tauri 2.x | `.dmg` |
| Android | Kotlin + WebView | `.apk` |

---

## OpenClaw 本体信息

- **运行时**: Node.js ≥22（唯一依赖）
- **安装命令**: `npm install -g openclaw@latest`
- **启动命令**: `openclaw up`
- **配置目录**: `~/.openclaw/`
- **Gateway端口**: `localhost:18789`
- **需要**: 至少一个 AI API Key（Claude/OpenAI/Gemini）

---

## 核心架构

```
┌─────────────────────────────────┐
│     Tauri 安装器 (~35MB)         │
│  ┌───────────┐ ┌─────────────┐  │
│  │ React UI  │ │ Rust 后端    │  │
│  │ 一键按钮   │ │ 环境检测     │  │
│  │ 进度展示   │ │ Node.js管理  │  │
│  │ API Key   │ │ 服务注册     │  │
│  │ 卸载页面   │ │ 健康检查     │  │
│  └───────────┘ └─────────────┘  │
│         │              │        │
│         └──── IPC ─────┘        │
└─────────────────────────────────┘
         │
         ▼ 管理
┌─────────────────────────────────┐
│  内嵌 Node.js 22 (Sidecar)      │
│  → npm install -g openclaw      │
│  → openclaw up (系统服务)        │
│  → localhost:18789              │
└─────────────────────────────────┘
```

---

## 用户流程

### 安装（一键）

```
打开安装器
    │
    ▼
┌──────────────────────────┐
│    [ 一键安装 OpenClaw ]   │  ← 用户唯一操作：点这个按钮
└──────────────────────────┘
    │
    ▼  全自动（用户只看进度条）
Step 1: 检测系统环境         (OS/架构/磁盘/RAM)     ~0.5s
Step 2: 释放内嵌 Node.js     (从 sidecar 解压)      ~2s
Step 3: npm install openclaw (全局安装)             ~30-120s
Step 4: 注册系统服务          (开机自启)             ~2s
Step 5: 启动 Gateway         (openclaw up)          ~3s
Step 6: 健康检查             (GET :18789)           ~2s
    │
    ▼
┌──────────────────────────┐
│  请输入 AI API Key:       │  ← 唯一需要用户填写的东西
│  [sk-ant-xxx________]    │
│  (附带获取教程链接)        │
└──────────────────────────┘
    │
    ▼
┌──────────────────────────┐
│  ✅ 安装完成！             │
│  控制台: localhost:18789  │
│  [打开控制台] [一键卸载]   │
└──────────────────────────┘
```

### 卸载（一键）

```
[ 一键卸载 ]
    │
    ▼
  "确定要卸载 OpenClaw 吗？"
  [ ] 同时删除配置和聊天记录
  [取消]  [确定卸载]
    │
    ▼  全自动
  ✓ 停止 OpenClaw 服务
  ✓ 移除系统服务注册
  ✓ npm uninstall -g openclaw
  ✓ 删除 ~/.openclaw/ (如勾选)
  ✓ 清理内嵌 Node.js
  ✓ 完成
```

---

## 持久化方案（系统服务）

### macOS — launchd

```xml
<!-- ~/Library/LaunchAgents/com.openclaw.gateway.plist -->
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.openclaw.gateway</string>
    <key>ProgramArguments</key>
    <array>
        <string>{{NODE_PATH}}</string>
        <string>{{OPENCLAW_BIN_PATH}}</string>
        <string>up</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>{{DATA_DIR}}/logs/gateway.out.log</string>
    <key>StandardErrorPath</key>
    <string>{{DATA_DIR}}/logs/gateway.err.log</string>
    <key>EnvironmentVariables</key>
    <dict>
        <key>PATH</key>
        <string>{{NODE_DIR}}:/usr/local/bin:/usr/bin:/bin</string>
    </dict>
</dict>
</plist>
```

**管理命令:**
```bash
launchctl load   ~/Library/LaunchAgents/com.openclaw.gateway.plist  # 注册+启动
launchctl unload ~/Library/LaunchAgents/com.openclaw.gateway.plist  # 停止+注销
launchctl list | grep openclaw                                      # 查状态
```

### Windows — NSSM (Non-Sucking Service Manager)

NSSM 是一个轻量 exe (~300KB)，内嵌到安装器中。

```powershell
# 安装服务
nssm install OpenClaw "{{NODE_PATH}}" "{{OPENCLAW_BIN_PATH}} up"
nssm set OpenClaw AppDirectory "{{DATA_DIR}}"
nssm set OpenClaw DisplayName "OpenClaw Gateway"
nssm set OpenClaw Description "OpenClaw AI Assistant Gateway Service"
nssm set OpenClaw Start SERVICE_AUTO_START
nssm set OpenClaw AppStdout "{{DATA_DIR}}\logs\gateway.out.log"
nssm set OpenClaw AppStderr "{{DATA_DIR}}\logs\gateway.err.log"

# 启动
nssm start OpenClaw

# 停止+删除（卸载时）
nssm stop OpenClaw
nssm remove OpenClaw confirm
```

### Android — 不适用

Android 端是「遥控器」，不运行 OpenClaw 本体，不需要系统服务。
通过 WebSocket 连接到桌面/服务器上运行的 Gateway。

---

## 数据目录规划

### Windows
```
%APPDATA%\OpenClawDeploy\
├── node\                    # 内嵌 Node.js 运行时
│   ├── node.exe
│   └── npm.cmd
├── logs\                    # 服务日志
│   ├── gateway.out.log
│   └── gateway.err.log
├── nssm.exe                 # 服务管理工具
└── state.json               # 安装器状态（版本、安装时间等）

%APPDATA%\npm\               # npm 全局包目录（openclaw 装这里）
%USERPROFILE%\.openclaw\     # OpenClaw 自身配置
```

### macOS
```
~/Library/Application Support/OpenClawDeploy/
├── node/                    # 内嵌 Node.js 运行时
│   ├── bin/node
│   └── bin/npm
├── logs/                    # 服务日志
│   ├── gateway.out.log
│   └── gateway.err.log
└── state.json               # 安装器状态

~/Library/LaunchAgents/
└── com.openclaw.gateway.plist  # launchd 服务配置

~/.openclaw/                 # OpenClaw 自身配置
```

---

## 项目结构

```
OpenClawDeploy/
├── PLAN.md                          # 本文件
├── package.json                     # 前端依赖管理
├── pnpm-workspace.yaml
├── tsconfig.json
│
├── apps/
│   ├── installer/                   # Tauri 桌面安装器 (Win + Mac)
│   │   ├── src-tauri/
│   │   │   ├── Cargo.toml
│   │   │   ├── tauri.conf.json
│   │   │   ├── build.rs
│   │   │   ├── sidecars/           # 打包时放入 Node.js 二进制
│   │   │   │   ├── README.md       # 说明如何下载 Node.js 二进制
│   │   │   │   └── .gitkeep
│   │   │   └── src/
│   │   │       ├── main.rs         # Tauri 入口
│   │   │       ├── lib.rs          # 命令注册
│   │   │       └── commands/
│   │   │           ├── mod.rs
│   │   │           ├── env_detect.rs    # 系统环境检测
│   │   │           ├── node_setup.rs    # Node.js 释放/管理
│   │   │           ├── openclaw.rs      # 安装/卸载 openclaw
│   │   │           ├── service.rs       # 系统服务管理(持久化核心)
│   │   │           ├── health.rs        # 健康检查
│   │   │           └── config.rs        # API Key 配置
│   │   ├── src/                    # React 前端
│   │   │   ├── main.tsx
│   │   │   ├── App.tsx
│   │   │   ├── pages/
│   │   │   │   ├── Install.tsx     # 一键安装（大按钮）
│   │   │   │   ├── Progress.tsx    # 实时进度
│   │   │   │   ├── ApiKey.tsx      # API Key 配置
│   │   │   │   ├── Dashboard.tsx   # 安装完成/状态面板
│   │   │   │   └── Uninstall.tsx   # 一键卸载
│   │   │   ├── hooks/
│   │   │   │   └── useInstaller.ts # 安装状态机
│   │   │   └── styles/
│   │   │       └── global.css
│   │   ├── index.html
│   │   ├── package.json
│   │   └── vite.config.ts
│   │
│   └── android/                    # Android 伴侣 App
│       ├── app/
│       │   ├── src/main/
│       │   │   ├── kotlin/.../openclaw/
│       │   │   │   ├── MainActivity.kt
│       │   │   │   ├── PairActivity.kt       # 扫码配对
│       │   │   │   ├── WebViewActivity.kt     # 远程控制台
│       │   │   │   └── CloudDeployActivity.kt # 云部署
│       │   │   ├── res/
│       │   │   └── AndroidManifest.xml
│       │   └── build.gradle.kts
│       ├── build.gradle.kts
│       └── settings.gradle.kts
│
└── scripts/
    ├── download-node.sh            # CI: 下载各平台 Node.js 二进制到 sidecars/
    └── build-all.sh                # CI: 构建所有平台安装包
```

---

## 技术选型明细

| 组件 | 选型 | 版本 | 理由 |
|------|------|------|------|
| 桌面框架 | Tauri | 2.x | 体积小(~8MB)、原生跨平台、Rust安全 |
| 前端 | React + TypeScript | 18.x | 生态丰富、团队熟悉 |
| 构建 | Vite | 5.x | 快、Tauri 官方推荐 |
| 内嵌运行时 | Node.js 官方二进制 | 22.x LTS | OpenClaw 最低要求 |
| Win服务 | NSSM | 2.24 | 稳定、轻量(300KB)、无依赖 |
| Mac服务 | launchd | 系统内置 | macOS 原生、最可靠 |
| Android | Kotlin + Jetpack Compose | 最新 | 原生体验、WebSocket 支持好 |
| 状态管理 | zustand | 4.x | 轻量、适合安装器场景 |

---

## 安装包体积预估

| 组件 | 大小 |
|------|------|
| Tauri 壳 + Rust 二进制 | ~8 MB |
| Node.js 22 二进制 | ~25 MB |
| React 前端资源 | ~2 MB |
| NSSM (仅Windows) | ~0.3 MB |
| **总计** | **~35 MB** |

---

## 开发阶段

### Phase 1: 桌面安装器骨架
- [ ] Tauri 2.x 项目初始化
- [ ] React 前端页面骨架（Install / Progress / Done / Uninstall）
- [ ] Rust 命令桩（env_detect / node_setup / openclaw / service / health）

### Phase 2: 核心安装流程
- [ ] Node.js sidecar 释放机制
- [ ] npm install openclaw 自动化
- [ ] 安装进度 event 流（Tauri event → React 状态更新）
- [ ] API Key 配置写入

### Phase 3: 持久化（系统服务）
- [ ] macOS launchd plist 生成与管理
- [ ] Windows NSSM 服务注册与管理
- [ ] 健康检查 + 自动重启
- [ ] 日志管理

### Phase 4: 一键卸载
- [ ] 停止服务 + 注销
- [ ] npm uninstall + 清理
- [ ] 可选删除用户数据
- [ ] 卸载确认 UI

### Phase 5: Android 伴侣
- [ ] Kotlin 项目初始化
- [ ] WebSocket 连接 Gateway
- [ ] 扫码配对流程
- [ ] 远程控制台 WebView

### Phase 6: 打包发布
- [ ] CI/CD 自动构建 (GitHub Actions)
- [ ] Windows 代码签名
- [ ] macOS 签名 + 公证
- [ ] 自动更新机制 (Tauri updater)
