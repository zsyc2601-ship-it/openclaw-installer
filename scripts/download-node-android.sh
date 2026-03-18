#!/usr/bin/env bash
# Download Node.js for Android (arm64) — Termux prebuilt binary
#
# Standard Node.js linux-arm64 binaries don't work on Android because
# Android uses Bionic libc, not glibc. We use Termux's build which is
# compiled against Bionic.
#
# Source: https://packages.termux.dev/apt/termux-main/
set -euo pipefail

NODE_VERSION="22.14.0"
TARGET_DIR="apps/android-server/app/src/main/assets"
OUTPUT="$TARGET_DIR/node-arm64.tar.xz"

if [ -f "$OUTPUT" ]; then
    echo "[skip] $OUTPUT already exists"
    exit 0
fi

mkdir -p "$TARGET_DIR"

echo "=== Downloading Node.js $NODE_VERSION for Android arm64 ==="
echo ""
echo "NOTE: Android requires Node.js built against Bionic libc."
echo "Options:"
echo "  1. Termux package (recommended): https://packages.termux.dev"
echo "  2. nodejs-mobile: https://github.com/nicedoc/node-prebuilt-android"
echo "  3. Build from source with Android NDK"
echo ""

# Try to download from Termux mirror
# Termux packages use .deb format, we need to extract the node binary
TERMUX_MIRROR="https://packages-cf.termux.dev/apt/termux-main/pool/main/n/nodejs"
DEB_FILE="nodejs_${NODE_VERSION}_aarch64.deb"
DEB_URL="$TERMUX_MIRROR/$DEB_FILE"

echo "Trying Termux package: $DEB_URL"

TMPDIR=$(mktemp -d)
TMPFILE="$TMPDIR/$DEB_FILE"

if curl -fSL --progress-bar -o "$TMPFILE" "$DEB_URL" 2>/dev/null; then
    echo "Downloaded Termux package, extracting..."
    cd "$TMPDIR"
    ar x "$DEB_FILE"
    # data.tar.xz contains the files
    if [ -f data.tar.xz ]; then
        cp data.tar.xz "$OLDPWD/$OUTPUT"
        echo "[done] Extracted to $OUTPUT"
    else
        echo "Warning: data.tar.xz not found in .deb, trying data.tar.gz..."
        cp data.tar.* "$OLDPWD/$OUTPUT" 2>/dev/null || true
    fi
    cd "$OLDPWD"
else
    echo ""
    echo "Termux package not found for this exact version."
    echo ""
    echo "Manual steps:"
    echo "  1. Download Node.js Android arm64 binary from one of the sources above"
    echo "  2. Create a tar.xz archive with bin/node, bin/npm, lib/node_modules/npm/..."
    echo "  3. Place it at: $OUTPUT"
    echo ""
    echo "Or use nodejs-mobile:"
    echo "  git clone https://github.com/nicedoc/node-prebuilt-android"
    echo "  # Follow their build instructions"
fi

rm -rf "$TMPDIR"

if [ -f "$OUTPUT" ]; then
    echo ""
    echo "=== Done ==="
    ls -lh "$OUTPUT"
else
    echo ""
    echo "=== Node.js for Android not downloaded ==="
    echo "The server APK will not work without this file."
    exit 1
fi
