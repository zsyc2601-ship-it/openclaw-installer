# OpenClaw Installer

One-click installer for the OpenClaw AI Gateway. No terminal, no manual Node.js setup, no Docker required.

## Supported Platforms

| Platform | Package | Service Manager |
|----------|---------|----------------|
| macOS ARM (Apple Silicon) | `.dmg` | launchd |
| macOS Intel | `.dmg` | launchd |
| Windows x64 | `.msi` / `.exe` | NSSM |
| Linux x64 | `.deb` / `.AppImage` | systemd --user |
| Linux ARM64 | `.deb` / `.AppImage` | systemd --user |

## Quick Install

### macOS / Linux

```bash
curl -fsSL https://github.com/zsyc2601-ship-it/openclaw-installer/raw/main/scripts/install.sh | bash
```

### Windows (PowerShell)

```powershell
irm https://github.com/zsyc2601-ship-it/openclaw-installer/raw/main/scripts/install.ps1 | iex
```

### China Mainland Users

The scripts auto-detect network environment and fall back to Gitee mirror + npmmirror npm registry.

```bash
# macOS / Linux
curl -fsSL https://gitee.com/zsyc2601-ship-it/openclaw-installer/raw/main/scripts/install.sh | bash

# Windows
irm https://gitee.com/zsyc2601-ship-it/openclaw-installer/raw/main/scripts/install.ps1 | iex
```

---

## How It Works

```
Run one-click command / Download installer
         │
         ▼
┌──────────────────┐
│  [ One-Click      │
│    Install ]      │  ← Click this button
└──────────────────┘
         │  Fully automatic
         ▼
Step 1: Detect system environment    ~0.5s
Step 2: Extract bundled Node.js      ~2s
Step 3: npm install openclaw         ~30-120s
Step 4: Register system service      ~2s
Step 5: Start Gateway                ~3s
Step 6: Health check                 ~2s
         │
         ▼
┌──────────────────┐
│  Enter API Key    │  ← The only thing to fill in
└──────────────────┘
         │
         ▼
┌──────────────────┐
│  Done!            │
│  Gateway running  │
│  localhost:18789  │
└──────────────────┘
```

After installation, OpenClaw runs as a system service in the background. It auto-restarts on reboot.

---

## Developer Guide

### Project Structure

```
├── apps/installer/                 # Tauri 2.x desktop installer
│   ├── src/                        # React + TypeScript frontend
│   └── src-tauri/                  # Rust backend
├── scripts/
│   ├── install.sh                  # macOS/Linux one-click install
│   ├── install.ps1                 # Windows one-click install
│   ├── download-node.sh            # Download Node.js archives
│   └── download-nssm.sh            # Download NSSM (Windows)
├── .github/workflows/
│   ├── build.yml                   # CI: Build all platforms
│   └── sync-gitee.yml              # Gitee sync (disabled by default)
├── Dockerfile                      # Linux container build
└── docker-compose.yml
```

### Prerequisites

- Node.js >= 22
- pnpm >= 9
- Rust (stable)
- `cargo install tauri-cli --version "^2"`

### Build

```bash
# 1. Download Node.js archives
bash scripts/download-node.sh

# 2. Install frontend deps
cd apps/installer && pnpm install

# 3. Build
npx tauri build
```

### CI Auto-Build

Push a tag to trigger builds for all platforms:

```bash
git tag v0.2.0
git push origin v0.2.0
```

Output:
- macOS ARM `.dmg` + macOS Intel `.dmg`
- Windows `.msi` + `.exe`
- Linux `.deb` + `.AppImage`

Published to [Releases](https://github.com/zsyc2601-ship-it/openclaw-installer/releases).

---

## Architecture

| Component | Technology |
|-----------|-----------|
| Desktop framework | Tauri 2.x (Rust + WebView) |
| Frontend | React 18 + TypeScript + Vite |
| State management | Zustand |
| Progress streaming | `tauri::ipc::Channel<T>` |
| macOS service | launchd (user-level) |
| Windows service | NSSM |
| Linux service | systemd --user |
| China acceleration | npmmirror + Gitee auto-switch |
