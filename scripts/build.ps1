#Requires -Version 5.1
# =============================================================================
# DST-Mod-Agent-Generator 打包部署脚本（Windows 优先）
#
# 用法：
#   .\scripts\build.ps1                          完整打包（NSIS + MSI 安装包）
#   .\scripts\build.ps1 -NoBundle                只编译应用 exe（快速调试，跳过安装包）
#   .\scripts\build.ps1 -Bundles nsis            只生成 NSIS 安装包
#   .\scripts\build.ps1 -SkipNpmInstall          跳过 npm install 与图标生成
#
# 产物：
#   应用 exe： src-tauri\target\release\dst-mod-agent.exe
#   安装包：   src-tauri\target\release\bundle\nsis\*.exe（NSIS）
#              src-tauri\target\release\bundle\msi\*.msi（MSI，需要 WiX，首次自动下载）
# =============================================================================
param(
    [switch]$NoBundle,
    [string]$Bundles = "",
    [switch]$SkipNpmInstall
)

$ErrorActionPreference = "Stop"
$ScriptsDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$Root = Split-Path -Parent $ScriptsDir
Push-Location $Root
try {
    # ---------- 1) 前置检查 ----------
    foreach ($c in @("node", "npm", "cargo", "rustc")) {
        if (-not (Get-Command $c -ErrorAction SilentlyContinue)) {
            throw "缺少依赖：$c 未安装（需要 Node.js >= 18 与 Rust stable/MSVC）"
        }
    }
    Write-Host "[1/4] 前置检查通过（node/npm/cargo/rustc）"

    # ---------- 2) 前端依赖与图标 ----------
    if (-not $SkipNpmInstall) {
        Write-Host "[2/4] npm install ..."
        & npm install
        if ($LASTEXITCODE -ne 0) { throw "npm install 失败（退出码 $LASTEXITCODE）" }

        if (-not (Test-Path "src-tauri\icons\icon.ico")) {
            Write-Host "      生成应用图标 ..."
            & "$ScriptsDir\generate-icons.ps1"
            if ($LASTEXITCODE -ne 0) { throw "图标生成失败（退出码 $LASTEXITCODE）" }
        }
    }

    # ---------- 3) tauri build ----------
    $cliArgs = @("run", "tauri", "build", "--")
    if ($NoBundle) {
        $cliArgs += "--no-bundle"
        $mode = "仅应用 exe（跳过安装包）"
    } elseif ($Bundles) {
        $cliArgs += "--bundles", $Bundles
        $mode = "安装包：$Bundles"
    } else {
        $mode = "完整打包（NSIS + MSI）"
    }
    Write-Host "[3/4] tauri build（$mode）"
    & npm @cliArgs
    if ($LASTEXITCODE -ne 0) { throw "tauri build 失败（退出码 $LASTEXITCODE），请查看上方错误日志" }

    # ---------- 4) 产物汇总 ----------
    Write-Host "[4/4] 构建成功，产物如下："
    $appExe = "src-tauri\target\release\dst-mod-agent.exe"
    if (Test-Path $appExe) { Write-Host "  - 应用 exe : $(Resolve-Path $appExe)" }
    $bundleDir = "src-tauri\target\release\bundle"
    if (Test-Path $bundleDir) {
        Get-ChildItem $bundleDir -Recurse -Include *.exe, *.msi | ForEach-Object {
            Write-Host "  - 安装包   : $($_.FullName)"
        }
    }
} finally {
    Pop-Location
}
