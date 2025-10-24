@echo off
chcp 65001 >nul
title VeloCache Başlatıcı

echo =================================
echo  🚀 VeloCache Başlatılıyor...
echo =================================
cd /d "%~dp0"

REM Zaten çalışıp çalışmadığını kontrol et
tasklist /FI "IMAGENAME eq velocache.exe" | find "velocache.exe" >nul
if %errorlevel% equ 0 (
    echo.
    echo 🟡 UYARI: VeloCache zaten çalışıyor.
    echo    Durdurmak için 'stop-proxy.bat' kullanın.
    echo.
    pause
    exit /b 1
)

REM Derlenmiş dosyanın varlığını kontrol et
if not exist "target\release\velocache.exe" (
    echo.
    echo ❌ HATA: velocache.exe bulunamadı!
    echo    Lütfen önce 'cargo build --release' ile derleyin.
    echo.
    pause
    exit /b 1
)

echo.
echo ✅ Sunucu yeni bir pencerede başlatılıyor...
start "VeloCache Sunucu" target\release\velocache.exe run

echo.
echo 🌐 Yönetim Paneli: http://127.0.0.1:8080
echo 📍 Proxy Port: 3128
echo.
echo 🕒 Sunucunun başlaması için birkaç saniye bekleyin...
timeout /t 3 /nobreak >nul
echo.
echo ✅ Başlatma işlemi tamamlandı. Logları diğer pencereden takip edebilirsiniz.
pause