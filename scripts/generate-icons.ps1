#Requires -Version 5.1
# =============================================================================
# DST-Mod-Agent-Generator 应用图标生成脚本（Windows）
# 1) 用 System.Drawing 画一张 1024x1024 源图（深蓝底 + DST 字样）；
# 2) 调用 @tauri-apps/cli 的 `tauri icon` 生成全尺寸图标与 .ico 到 src-tauri/icons。
# Tauri 打包 Windows 安装包（NSIS/MSI）与 exe 资源图标都需要这些文件。
# =============================================================================
$ErrorActionPreference = "Stop"
$ScriptsDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$Root = Split-Path -Parent $ScriptsDir
$SrcPng = Join-Path $Root "app-icon.png"

Add-Type -AssemblyName System.Drawing
$bmp = New-Object System.Drawing.Bitmap 1024, 1024
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
$g.TextRenderingHint = [System.Drawing.Text.TextRenderingHint]::AntiAliasGridFit

# 背景 + 中央圆形 + 文字（简单品牌图标，后续可替换为正式美术资源）
$bg = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(255, 77, 140, 255))
$g.FillRectangle($bg, 0, 0, 1024, 1024)
$dark = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(255, 23, 26, 33))
$g.FillEllipse($dark, 96, 96, 832, 832)
$white = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::White)
$font = New-Object System.Drawing.Font("Segoe UI", 300, [System.Drawing.FontStyle]::Bold)
$sf = New-Object System.Drawing.StringFormat
$sf.Alignment = [System.Drawing.StringAlignment]::Center
$sf.LineAlignment = [System.Drawing.StringAlignment]::Center
$rect = New-Object System.Drawing.RectangleF(0, 0, 1024, 1024)
$g.DrawString("DST", $font, $white, $rect, $sf)

$bmp.Save($SrcPng, [System.Drawing.Imaging.ImageFormat]::Png)
$g.Dispose()
$bmp.Dispose()
Write-Host "已生成源图: $SrcPng"

# 调用 tauri icon（需要先 npm install 安装 @tauri-apps/cli）
Push-Location $Root
try {
    npx tauri icon $SrcPng
    if ($LASTEXITCODE -ne 0) { throw "tauri icon 生成失败（退出码 $LASTEXITCODE）" }
    Write-Host "图标已生成到 src-tauri/icons"
} finally {
    Pop-Location
}
