#!/usr/bin/env bash
set -euo pipefail

NODE_VERSION="22.22.1"
TARGET_DIR="apps/installer/src-tauri/resources"

# Auto-select mirror: npmmirror (China) or official
NODE_BASE_URL="${NODE_MIRROR:-}"
if [ -z "$NODE_BASE_URL" ]; then
    # Probe: if npmmirror responds within 3s, use it
    if curl -fsS --connect-timeout 3 --max-time 5 "https://npmmirror.com/mirrors/node/" -o /dev/null 2>/dev/null; then
        NODE_BASE_URL="https://npmmirror.com/mirrors/node"
        echo "使用 npmmirror 镜像 (中国大陆加速)"
    else
        NODE_BASE_URL="https://nodejs.org/dist"
        echo "使用 Node.js 官方源"
    fi
fi

mkdir -p "$TARGET_DIR"

download() {
    local platform="$1"
    local arch="$2"
    local ext="$3"
    local filename="node-v${NODE_VERSION}-${platform}-${arch}.${ext}"
    local url="${NODE_BASE_URL}/v${NODE_VERSION}/${filename}"
    local dest="${TARGET_DIR}/${filename}"

    if [ -f "$dest" ]; then
        echo "[skip] $filename already exists"
        return
    fi

    echo "[download] $url"
    curl -fSL --progress-bar -o "$dest" "$url"
    echo "[done] $filename ($(du -h "$dest" | cut -f1))"
}

echo ""
echo "=== Downloading Node.js v${NODE_VERSION} archives ==="
echo ""

# macOS
download "darwin" "arm64" "tar.xz"
download "darwin" "x64" "tar.xz"

# Linux
download "linux" "x64" "tar.xz"
download "linux" "arm64" "tar.xz"

# Windows
download "win" "x64" "zip"

echo ""
echo "=== All downloads complete ==="
ls -lh "$TARGET_DIR"/node-*
