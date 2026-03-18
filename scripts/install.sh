#!/usr/bin/env bash
# OpenClaw 一键安装脚本 (macOS / Linux)
# 自动检测网络环境，中国大陆优先走 Gitee 镜像
#
# 用法:
#   curl -fsSL https://openclaw.dev/install.sh | bash
#   curl -fsSL https://gitee.com/zsyc2601-ship-it/openclaw-installer/raw/main/scripts/install.sh | bash
set -euo pipefail

GITHUB_REPO="zsyc2601-ship-it/openclaw-installer"
GITEE_REPO="zsyc2601-ship-it/openclaw-installer"
APP_NAME="OpenClaw Installer"

echo "============================================"
echo "  OpenClaw 一键安装器"
echo "============================================"
echo ""

# ─── Detect platform ────────────────────────────
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Darwin)
        case "$ARCH" in
            arm64)  TARGET="aarch64-apple-darwin" ;;
            x86_64) TARGET="x86_64-apple-darwin" ;;
            *) echo "不支持的架构: $ARCH"; exit 1 ;;
        esac
        EXT="dmg"
        ;;
    Linux)
        case "$ARCH" in
            x86_64)  TARGET="x86_64-unknown-linux-gnu" ;;
            aarch64) TARGET="aarch64-unknown-linux-gnu" ;;
            *) echo "不支持的架构: $ARCH"; exit 1 ;;
        esac
        EXT="AppImage"
        ;;
    *)
        echo "不支持的操作系统: $OS"
        echo "Windows 用户请使用 PowerShell:"
        echo "  irm https://gitee.com/$GITEE_REPO/raw/main/scripts/install.ps1 | iex"
        exit 1
        ;;
esac

echo "系统: $OS $ARCH → $TARGET"

# ─── Detect network: China or Global ────────────
USE_CHINA=false

detect_network() {
    # Try to reach Gitee (China) and GitHub (Global) — whoever responds first wins
    if curl -fsS --connect-timeout 3 --max-time 5 "https://gitee.com" -o /dev/null 2>/dev/null; then
        # Gitee reachable, now check if GitHub is slow
        if ! curl -fsS --connect-timeout 3 --max-time 5 "https://github.com" -o /dev/null 2>/dev/null; then
            USE_CHINA=true
        fi
    fi
}

echo "检测网络环境..."
detect_network

if [ "$USE_CHINA" = true ]; then
    echo "检测到中国大陆网络，使用 Gitee 镜像"
    SOURCE="gitee"
else
    echo "使用 GitHub 下载"
    SOURCE="github"
fi
echo ""

# ─── Get download URL ───────────────────────────
get_download_url() {
    if [ "$SOURCE" = "gitee" ]; then
        # Gitee Releases API
        local api="https://gitee.com/api/v5/repos/$GITEE_REPO/releases/latest"
        local json
        json=$(curl -fsSL "$api") || {
            echo "Gitee API 失败，回退到 GitHub..."
            SOURCE="github"
            get_download_url
            return
        }
        # Parse with grep/sed (avoid jq dependency)
        DOWNLOAD_URL=$(echo "$json" | grep -o "\"browser_download_url\":\"[^\"]*${TARGET}[^\"]*\"" | head -1 | cut -d'"' -f4)
    else
        local api="https://api.github.com/repos/$GITHUB_REPO/releases/latest"
        local json
        json=$(curl -fsSL "$api") || {
            echo "错误: 无法访问 GitHub API"
            echo ""
            echo "中国大陆用户请尝试:"
            echo "  curl -fsSL https://gitee.com/$GITEE_REPO/raw/main/scripts/install.sh | bash"
            exit 1
        }
        DOWNLOAD_URL=$(echo "$json" | grep "browser_download_url" | grep "$TARGET" | head -1 | cut -d '"' -f 4)
    fi
}

echo "获取最新版本..."
get_download_url

if [ -z "${DOWNLOAD_URL:-}" ]; then
    echo "错误: 找不到适合 $TARGET 的安装包"
    echo ""
    echo "手动下载:"
    echo "  GitHub: https://github.com/$GITHUB_REPO/releases"
    echo "  Gitee:  https://gitee.com/$GITEE_REPO/releases"
    exit 1
fi

FILENAME=$(basename "$DOWNLOAD_URL")
TMPDIR=$(mktemp -d)
TMPFILE="$TMPDIR/$FILENAME"

echo "下载: $FILENAME"
curl -fSL --progress-bar -o "$TMPFILE" "$DOWNLOAD_URL"
echo ""

# ─── Install ────────────────────────────────────
case "$EXT" in
    dmg)
        echo "挂载 DMG..."
        MOUNT_POINT=$(hdiutil attach "$TMPFILE" -nobrowse | tail -1 | awk '{for(i=3;i<=NF;i++) printf "%s ", $i; print ""}' | sed 's/ *$//')
        echo "安装到 /Applications..."
        rm -rf "/Applications/$APP_NAME.app" 2>/dev/null || true
        cp -R "$MOUNT_POINT/$APP_NAME.app" /Applications/
        hdiutil detach "$MOUNT_POINT" -quiet
        echo "启动安装器..."
        open "/Applications/$APP_NAME.app"
        ;;
    AppImage)
        INSTALL_DIR="$HOME/.local/bin"
        mkdir -p "$INSTALL_DIR"
        DEST="$INSTALL_DIR/openclaw-installer"
        cp "$TMPFILE" "$DEST"
        chmod +x "$DEST"
        echo "已安装到: $DEST"
        echo "启动安装器..."
        "$DEST" &
        disown
        ;;
esac

rm -rf "$TMPDIR"

echo ""
echo "============================================"
echo "  安装器已启动"
echo "  点击「一键安装」完成 OpenClaw 部署"
echo "============================================"
