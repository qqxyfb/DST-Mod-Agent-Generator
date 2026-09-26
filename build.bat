@echo off
setlocal EnableExtensions
REM =====================================================================
REM  DST-Mod-Agent-Generator one-click build script (Windows)
REM  Usage:
REM    build.bat                    Full bundle (NSIS + MSI)
REM    build.bat -NoBundle          App exe only (fast debug)
REM    build.bat -Bundles nsis      NSIS installer only
REM    build.bat -Bundles msi       MSI installer only (WiX auto-download on first run)
REM    build.bat -SkipNpmInstall    Skip npm install and icon generation
REM  Optional env vars:
REM    DST_NO_ELEVATE=1             Skip UAC elevation (CI / non-interactive)
REM    npm_config_registry          Override npm registry (default npmmirror)
REM    TAURI_BUNDLER_TOOLS_GITHUB_MIRROR  Override bundler tool mirror
REM =====================================================================
cd /d "%~dp0"

set "ARGS=%*"

REM ---- Self-elevate to admin unless DST_NO_ELEVATE=1 ----
if "%DST_NO_ELEVATE%"=="1" goto :check_env
net session >nul 2>&1
if errorlevel 1 (
    echo [build.bat] Admin rights needed for bundler tools dir. Requesting elevation...
    if "%ARGS%"=="" (
        powershell -NoProfile -Command "Start-Process -FilePath '%~f0' -Verb RunAs"
    ) else (
        powershell -NoProfile -Command "Start-Process -FilePath '%~f0' -ArgumentList '%ARGS%' -Verb RunAs"
    )
    exit /b 0
)

:check_env
REM ---- Defaults: workspace npm cache + CN mirrors ----
if not defined npm_config_cache set "npm_config_cache=%CD%\cache\npm-cache"
if not defined npm_config_registry set "npm_config_registry=https://registry.npmmirror.com"
if not defined TAURI_BUNDLER_TOOLS_GITHUB_MIRROR set "TAURI_BUNDLER_TOOLS_GITHUB_MIRROR=https://gh-proxy.com"
echo [build.bat] npm cache   : %npm_config_cache%
echo [build.bat] npm registry: %npm_config_registry%
echo [build.bat] tools mirror: %TAURI_BUNDLER_TOOLS_GITHUB_MIRROR%
echo.

REM ---- Delegate to scripts\build.ps1 ----
powershell -NoProfile -ExecutionPolicy Bypass -File "scripts\build.ps1" %ARGS%
set "RC=%errorlevel%"

echo.
if "%RC%"=="0" (
    echo [build.bat] Build OK ^^! See artifact paths above.
) else (
    echo [build.bat] Build FAILED with code %RC%. See error log above.
)
echo.
pause
exit /b %RC%