# OpenClaw 一键安装脚本 (Windows)
# 自动检测网络环境，中国大陆优先走 Gitee 镜像
#
# 用法:
#   irm https://openclaw.dev/install.ps1 | iex
#   irm https://gitee.com/zsyc2601-ship-it/openclaw-installer/raw/main/scripts/install.ps1 | iex
$ErrorActionPreference = "Stop"

$GitHubRepo = "zsyc2601-ship-it/openclaw-installer"
$GiteeRepo = "zsyc2601-ship-it/openclaw-installer"
$Target = "x86_64-pc-windows-msvc"

Write-Host "============================================" -ForegroundColor Cyan
Write-Host "  OpenClaw 一键安装器" -ForegroundColor Cyan
Write-Host "============================================" -ForegroundColor Cyan
Write-Host ""

# ─── Detect network ─────────────────────────────
$UseChina = $false
Write-Host "检测网络环境..."

try {
    $null = Invoke-WebRequest -Uri "https://gitee.com" -TimeoutSec 3 -UseBasicParsing -ErrorAction Stop
    try {
        $null = Invoke-WebRequest -Uri "https://github.com" -TimeoutSec 3 -UseBasicParsing -ErrorAction Stop
    } catch {
        $UseChina = $true
    }
} catch {
    # Gitee also unreachable, use GitHub
}

if ($UseChina) {
    Write-Host "检测到中国大陆网络，使用 Gitee 镜像" -ForegroundColor Yellow
} else {
    Write-Host "使用 GitHub 下载"
}
Write-Host ""

# ─── Get download URL ───────────────────────────
$DownloadUrl = ""

if ($UseChina) {
    try {
        $Api = "https://gitee.com/api/v5/repos/$GiteeRepo/releases/latest"
        $Release = Invoke-RestMethod -Uri $Api -UseBasicParsing
        $Asset = $Release.assets | Where-Object { $_.name -match $Target -and ($_.name -match '\.msi$' -or $_.name -match '\.exe$') } | Select-Object -First 1
        if ($Asset) { $DownloadUrl = $Asset.browser_download_url }
    } catch {
        Write-Host "Gitee API 失败，回退到 GitHub..." -ForegroundColor Yellow
        $UseChina = $false
    }
}

if (-not $DownloadUrl) {
    try {
        $Api = "https://api.github.com/repos/$GitHubRepo/releases/latest"
        $Release = Invoke-RestMethod -Uri $Api -UseBasicParsing
        $Asset = $Release.assets | Where-Object { $_.name -match $Target -and $_.name -match '\.msi$' } | Select-Object -First 1
        if (-not $Asset) {
            $Asset = $Release.assets | Where-Object { $_.name -match $Target -and $_.name -match '\.exe$' } | Select-Object -First 1
        }
        if ($Asset) { $DownloadUrl = $Asset.browser_download_url }
    } catch {
        Write-Host "错误: 无法访问下载源" -ForegroundColor Red
        Write-Host ""
        Write-Host "请手动下载:"
        Write-Host "  GitHub: https://github.com/$GitHubRepo/releases"
        Write-Host "  Gitee:  https://gitee.com/$GiteeRepo/releases"
        exit 1
    }
}

if (-not $DownloadUrl) {
    Write-Host "错误: 找不到适合 Windows 的安装包" -ForegroundColor Red
    exit 1
}

$FileName = [System.IO.Path]::GetFileName($DownloadUrl)
$TmpDir = Join-Path $env:TEMP "openclaw-install"
$TmpFile = Join-Path $TmpDir $FileName

New-Item -ItemType Directory -Force -Path $TmpDir | Out-Null

Write-Host "下载: $FileName"
Invoke-WebRequest -Uri $DownloadUrl -OutFile $TmpFile -UseBasicParsing
Write-Host ""

Write-Host "正在安装..."
if ($FileName -match '\.msi$') {
    Start-Process msiexec.exe -ArgumentList "/i `"$TmpFile`" /qr" -Wait
} else {
    Start-Process $TmpFile -Wait
}

Remove-Item -Recurse -Force $TmpDir -ErrorAction SilentlyContinue

Write-Host ""
Write-Host "============================================" -ForegroundColor Green
Write-Host "  安装器已启动" -ForegroundColor Green
Write-Host "  点击「一键安装」完成 OpenClaw 部署" -ForegroundColor Green
Write-Host "============================================" -ForegroundColor Green
