@echo off
setlocal

:: Check for Administrator privileges
net session >nul 2>&1
if %errorlevel% neq 0 (
    echo [mikey] Administrator privileges required to configure audio devices.
    echo [mikey] Requesting UAC elevation...
    powershell.exe -NoProfile -ExecutionPolicy Bypass -Command "Start-Process cmd -ArgumentList '/c `\"%~f0`\"' -Verb RunAs"
    exit /b
)

title Mikey Mic Setup
echo ==============================================
echo   Setting up Mikey Mic Audio Device...
echo ==============================================
powershell.exe -ExecutionPolicy Bypass -NoProfile -File "%~dp0installer\setup-audio-device.ps1"
echo.
echo ==============================================
echo   Done! You can close this window.
echo ==============================================
pause >nul
