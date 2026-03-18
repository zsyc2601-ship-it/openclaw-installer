# OpenClaw Installer

One-click installer for the OpenClaw AI Gateway. No terminal, no manual Node.js setup, no Docker required.

## Supported Platforms

| Platform | Package | Service Manager | Status |
|----------|---------|----------------|--------|
| macOS ARM (Apple Silicon) | `.dmg` | launchd | ✅ |
| macOS Intel | `.dmg` | launchd | ✅ |
| Windows x64 | `.msi` / `.exe` | NSSM | ✅ |
| Linux x64 | `.deb` / `.AppImage` | systemd --user | ✅ |
| Linux ARM64 | `.deb` / `.AppImage` | systemd --user | ✅ |
| Android (Remote) | `.apk` | — | ✅ |
| Android (Server) | `.apk` | Foreground Service | ✅ |

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

Manual Gitee source:

```bash
# macOS / Linux
curl -fsSL https://gitee.com/zsyc2601-ship-it/openclaw-installer/raw/main/scripts/install.sh | bash

# Windows
irm https://gitee.com/zsyc2601-ship-it/openclaw-installer/raw/main/scripts/install.ps1 | iex
```

### Android

Download from [Releases](https://github.com/zsyc2601-ship-it/openclaw-installer/releases):
- **openclaw-companion.apk** — Remote control, scan QR to connect to desktop Gateway
- **openclaw-server.apk** — Run OpenClaw Gateway directly on your phone

---

## How It Works

```
Run one-click command / Download installer
         │
         ▼
┌──────────────────┐
│  [ One-Click      │
│    Install ]      │  ← The only thing the user does: click this button
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
│  ✅ Done!         │
│  Gateway running  │
│  localhost:18789  │
└──────────────────┘
```

After installation, OpenClaw runs as a system service in the background. It auto-restarts on reboot.

---

## Developer Guide

### Project Structure

```
├── apps/
│   ├── installer/              # Tauri 2.x desktop installer (macOS/Windows/Linux)
│   │   ├── src/                # React + TypeScript frontend
│   │   └── src-tauri/          # Rust backend
│   ├── android/                # Android companion (Kotlin + Compose)
│   └── android-server/         # Android server (embedded Node.js)
├── scripts/
│   ├── install.sh              # macOS/Linux one-click install script
│   ├── install.ps1             # Windows one-click install script
│   ├── download-node.sh        # Download Node.js archives for all platforms
│   ├── download-node-android.sh # Download Node.js for Android
│   └── download-nssm.sh        # Download NSSM (Windows service manager)
├── .github/workflows/
│   ├── build.yml               # CI: Build desktop installers for all platforms
│   ├── android.yml             # CI: Build Android APKs
│   └── sync-gitee.yml          # Auto-sync to Gitee (disabled by default)
├── Dockerfile                  # Linux container build
└── docker-compose.yml
```

### Prerequisites

- Node.js ≥ 22
- pnpm ≥ 9
- Rust (stable)
- `cargo install tauri-cli --version "^2"`

### Build Desktop Installer

```bash
# 1. Download Node.js archives to resources/
bash scripts/download-node.sh

# 2. Install frontend dependencies
cd apps/installer && pnpm install

# 3. Build (output in src-tauri/target/release/bundle/)
npx tauri build
```

### Build Linux (Docker)

```bash
make download-node
make build-linux
```

### Build Android

```bash
# Companion app
cd apps/android && ./gradlew assembleRelease

# Server app (download Android Node.js first)
bash scripts/download-node-android.sh
cd apps/android-server && ./gradlew assembleRelease
```

### CI Auto-Build

Push a tag to trigger full cross-platform builds:

```bash
git tag v0.1.0
git push origin v0.1.0
```

GitHub Actions will build:
- macOS ARM `.dmg` + macOS Intel `.dmg`
- Windows `.msi` + `.exe`
- Linux `.deb` + `.AppImage`
- Android Companion `.apk` + Android Server `.apk`

Artifacts are published to [Releases](https://github.com/zsyc2601-ship-it/openclaw-installer/releases).

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
| Android UI | Kotlin + Jetpack Compose |
| Android QR scan | CameraX + ML Kit |
| China acceleration | npmmirror + Gitee auto-switch |
