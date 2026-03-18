#!/usr/bin/env bash
# OpenClaw one-click install script (macOS / Linux)
# Auto-detects network, uses Gitee mirror for China mainland.
#
# Usage:
#   curl -fsSL https://github.com/zsyc2601-ship-it/openclaw-installer/raw/main/scripts/install.sh | bash
#   curl -fsSL https://gitee.com/zsyc2601-ship-it/openclaw-installer/raw/main/scripts/install.sh | bash
set -euo pipefail

GITHUB_REPO="zsyc2601-ship-it/openclaw-installer"
GITEE_REPO="zsyc2601-ship-it/openclaw-installer"
APP_NAME="OpenClaw Installer"

echo "============================================"
echo "  OpenClaw Installer"
echo "============================================"
echo ""

# ─── Detect platform ────────────────────────────
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Darwin)
        case "$ARCH" in
            arm64)  MATCH_PATTERN="aarch64.dmg" ;;
            x86_64) MATCH_PATTERN="x64.dmg" ;;
            *) echo "Unsupported arch: $ARCH"; exit 1 ;;
        esac
        EXT="dmg"
        ;;
    Linux)
        case "$ARCH" in
            x86_64)  MATCH_PATTERN="amd64.AppImage" ;;
            aarch64) MATCH_PATTERN="aarch64.AppImage" ;;
            *) echo "Unsupported arch: $ARCH"; exit 1 ;;
        esac
        EXT="AppImage"
        ;;
    *)
        echo "Unsupported OS: $OS"
        echo "Windows users, run in PowerShell:"
        echo "  irm https://github.com/$GITHUB_REPO/raw/main/scripts/install.ps1 | iex"
        exit 1
        ;;
esac

echo "Platform: $OS $ARCH"

# ─── Detect network: China or Global ────────────
USE_CHINA=false

detect_network() {
    if curl -fsS --connect-timeout 3 --max-time 5 "https://gitee.com" -o /dev/null 2>/dev/null; then
        if ! curl -fsS --connect-timeout 3 --max-time 5 "https://github.com" -o /dev/null 2>/dev/null; then
            USE_CHINA=true
        fi
    fi
}

echo "Detecting network..."
detect_network

if [ "$USE_CHINA" = true ]; then
    echo "China mainland detected, using Gitee mirror"
    SOURCE="gitee"
else
    echo "Using GitHub"
    SOURCE="github"
fi
echo ""

# ─── Get download URL ───────────────────────────
get_download_url() {
    if [ "$SOURCE" = "gitee" ]; then
        local api="https://gitee.com/api/v5/repos/$GITEE_REPO/releases/latest"
        local json
        json=$(curl -fsSL "$api") || {
            echo "Gitee API failed, falling back to GitHub..."
            SOURCE="github"
            get_download_url
            return
        }
        DOWNLOAD_URL=$(echo "$json" | grep -o "\"browser_download_url\":\"[^\"]*${MATCH_PATTERN}[^\"]*\"" | head -1 | cut -d'"' -f4)
    else
        local api="https://api.github.com/repos/$GITHUB_REPO/releases/latest"
        local json
        json=$(curl -fsSL "$api") || {
            echo "Error: cannot access GitHub API"
            echo ""
            echo "China mainland users try:"
            echo "  curl -fsSL https://gitee.com/$GITEE_REPO/raw/main/scripts/install.sh | bash"
            exit 1
        }
        DOWNLOAD_URL=$(echo "$json" | grep "browser_download_url" | grep "$MATCH_PATTERN" | head -1 | cut -d '"' -f 4)
    fi
}

echo "Fetching latest release..."
get_download_url

if [ -z "${DOWNLOAD_URL:-}" ]; then
    echo "Error: no installer found for $OS $ARCH (pattern: $MATCH_PATTERN)"
    echo ""
    echo "Manual download:"
    echo "  GitHub: https://github.com/$GITHUB_REPO/releases"
    echo "  Gitee:  https://gitee.com/$GITEE_REPO/releases"
    exit 1
fi

FILENAME=$(basename "$DOWNLOAD_URL")
TMPDIR=$(mktemp -d)
TMPFILE="$TMPDIR/$FILENAME"

echo "Downloading: $FILENAME"
curl -fSL --progress-bar -o "$TMPFILE" "$DOWNLOAD_URL"
echo ""

# ─── Install ────────────────────────────────────
case "$EXT" in
    dmg)
        echo "Mounting DMG..."
        MOUNT_POINT=$(hdiutil attach "$TMPFILE" -nobrowse | tail -1 | awk '{for(i=3;i<=NF;i++) printf "%s ", $i; print ""}' | sed 's/ *$//')
        echo "Installing to /Applications..."
        rm -rf "/Applications/$APP_NAME.app" 2>/dev/null || true
        cp -R "$MOUNT_POINT/$APP_NAME.app" /Applications/
        hdiutil detach "$MOUNT_POINT" -quiet
        echo "Launching installer..."
        open "/Applications/$APP_NAME.app"
        ;;
    AppImage)
        INSTALL_DIR="$HOME/.local/bin"
        mkdir -p "$INSTALL_DIR"
        DEST="$INSTALL_DIR/openclaw-installer"
        cp "$TMPFILE" "$DEST"
        chmod +x "$DEST"
        echo "Installed to: $DEST"
        echo "Launching installer..."
        "$DEST" &
        disown
        ;;
esac

rm -rf "$TMPDIR"

echo ""
echo "============================================"
echo "  Installer launched!"
echo "  Click 'One-Click Install' to deploy"
echo "============================================"
