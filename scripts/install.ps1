# OpenClaw one-click install script (Windows)
# Auto-detects network, uses Gitee mirror for China mainland.
#
# Usage:
#   irm https://github.com/zsyc2601-ship-it/openclaw-installer/raw/main/scripts/install.ps1 | iex
#   irm https://gitee.com/zsyc2601-ship-it/openclaw-installer/raw/main/scripts/install.ps1 | iex
$ErrorActionPreference = "Stop"

$GitHubRepo = "zsyc2601-ship-it/openclaw-installer"
$GiteeRepo = "zsyc2601-ship-it/openclaw-installer"

Write-Host "============================================" -ForegroundColor Cyan
Write-Host "  OpenClaw Installer" -ForegroundColor Cyan
Write-Host "============================================" -ForegroundColor Cyan
Write-Host ""

# ─── Detect network ─────────────────────────────
$UseChina = $false
Write-Host "Detecting network..."

try {
    $null = Invoke-WebRequest -Uri "https://gitee.com" -TimeoutSec 3 -UseBasicParsing -ErrorAction Stop
    try {
        $null = Invoke-WebRequest -Uri "https://github.com" -TimeoutSec 3 -UseBasicParsing -ErrorAction Stop
    } catch {
        $UseChina = $true
    }
} catch {}

if ($UseChina) {
    Write-Host "China mainland detected, using Gitee mirror" -ForegroundColor Yellow
} else {
    Write-Host "Using GitHub"
}
Write-Host ""

# ─── Get download URL ───────────────────────────
$DownloadUrl = ""

if ($UseChina) {
    try {
        $Api = "https://gitee.com/api/v5/repos/$GiteeRepo/releases/latest"
        $Release = Invoke-RestMethod -Uri $Api -UseBasicParsing
        # Match .msi first, then .exe
        $Asset = $Release.assets | Where-Object { $_.name -match 'x64_en-US\.msi$' } | Select-Object -First 1
        if (-not $Asset) {
            $Asset = $Release.assets | Where-Object { $_.name -match 'x64-setup\.exe$' } | Select-Object -First 1
        }
        if ($Asset) { $DownloadUrl = $Asset.browser_download_url }
    } catch {
        Write-Host "Gitee API failed, falling back to GitHub..." -ForegroundColor Yellow
        $UseChina = $false
    }
}

if (-not $DownloadUrl) {
    try {
        $Api = "https://api.github.com/repos/$GitHubRepo/releases/latest"
        $Release = Invoke-RestMethod -Uri $Api -UseBasicParsing
        $Asset = $Release.assets | Where-Object { $_.name -match 'x64_en-US\.msi$' } | Select-Object -First 1
        if (-not $Asset) {
            $Asset = $Release.assets | Where-Object { $_.name -match 'x64-setup\.exe$' } | Select-Object -First 1
        }
        if ($Asset) { $DownloadUrl = $Asset.browser_download_url }
    } catch {
        Write-Host "Error: cannot access download source" -ForegroundColor Red
        Write-Host ""
        Write-Host "Manual download:"
        Write-Host "  GitHub: https://github.com/$GitHubRepo/releases"
        Write-Host "  Gitee:  https://gitee.com/$GiteeRepo/releases"
        exit 1
    }
}

if (-not $DownloadUrl) {
    Write-Host "Error: no installer found for Windows x64" -ForegroundColor Red
    exit 1
}

$FileName = [System.IO.Path]::GetFileName($DownloadUrl)
$TmpDir = Join-Path $env:TEMP "openclaw-install"
$TmpFile = Join-Path $TmpDir $FileName

New-Item -ItemType Directory -Force -Path $TmpDir | Out-Null

Write-Host "Downloading: $FileName"
Invoke-WebRequest -Uri $DownloadUrl -OutFile $TmpFile -UseBasicParsing
Write-Host ""

Write-Host "Installing..."
if ($FileName -match '\.msi$') {
    Start-Process msiexec.exe -ArgumentList "/i `"$TmpFile`" /qr" -Wait
} else {
    Start-Process $TmpFile -Wait
}

Remove-Item -Recurse -Force $TmpDir -ErrorAction SilentlyContinue

Write-Host ""
Write-Host "============================================" -ForegroundColor Green
Write-Host "  Installer launched!" -ForegroundColor Green
Write-Host "  Click 'One-Click Install' to deploy" -ForegroundColor Green
Write-Host "============================================" -ForegroundColor Green
