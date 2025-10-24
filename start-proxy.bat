@echo off
chcp 65001 >nul
title VeloCache Sunucu

echo =================================
echo  🚀 VeloCache Başlatılıyor...
echo =================================

cd /d "%~dp0"

REM Derlenmiş dosyanın varlığını kontrol et
if not exist "target\release\velocache.exe" (
    echo ❌ velocache.exe bulunamadı!
    echo ⏳ Lütfen önce 'cargo build --release' ile derleyin.
    pause
    exit /b 1
)

echo ✅ Sunucu başlatılıyor. Kapatmak için 'stop-proxy.bat' kullanın veya bu pencereyi kapatın.
start "VeloCache" /B target\release\velocache.exe run

echo.
echo 🌐 Yönetim Paneli: http://127.0.0.1:8080
echo 📍 Proxy Port: 3128
timeout /t 2 >nul