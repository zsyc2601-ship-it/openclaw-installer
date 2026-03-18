#!/usr/bin/env bash
# Download NSSM for Windows service management
set -euo pipefail

TARGET_DIR="apps/installer/src-tauri/resources"
NSSM_URL="https://nssm.cc/release/nssm-2.24.zip"
NSSM_DEST="$TARGET_DIR/nssm.exe"

if [ -f "$NSSM_DEST" ]; then
    echo "[skip] nssm.exe already exists"
    exit 0
fi

mkdir -p "$TARGET_DIR"

TMPDIR=$(mktemp -d)
echo "[download] NSSM 2.24..."
curl -fSL --progress-bar -o "$TMPDIR/nssm.zip" "$NSSM_URL"

echo "[extract] nssm.exe (win64)..."
if command -v unzip &>/dev/null; then
    unzip -q -o "$TMPDIR/nssm.zip" "nssm-2.24/win64/nssm.exe" -d "$TMPDIR"
    cp "$TMPDIR/nssm-2.24/win64/nssm.exe" "$NSSM_DEST"
elif command -v python3 &>/dev/null; then
    python3 -c "
import zipfile, shutil
with zipfile.ZipFile('$TMPDIR/nssm.zip') as z:
    with z.open('nssm-2.24/win64/nssm.exe') as src:
        with open('$NSSM_DEST', 'wb') as dst:
            shutil.copyfileobj(src, dst)
"
else
    echo "Error: need unzip or python3 to extract"
    exit 1
fi

rm -rf "$TMPDIR"
echo "[done] nssm.exe ($(du -h "$NSSM_DEST" | cut -f1))"
