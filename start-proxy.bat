@echo off
chcp 65001 >nul
title VeloCache Başlatıcı

echo =================================
echo  🚀 VeloCache Başlatılıyor...
echo =================================
cd /d "%~dp0"

set "PROGRAM_PATH=%~dp0target\release\velocache.exe"

if not exist "%PROGRAM_PATH%" (
    echo.
    echo ❌ HATA: velocache.exe bulunamadı!
    echo    Lütfen önce 'cargo build --release' ile derleyin.
    echo.
    pause
    exit /b 1
)

echo.
echo ⚙️  Windows Proxy ayarları etkinleştiriliyor (localhost)...
reg add "HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings" /v ProxyEnable /t REG_DWORD /d 1 /f >nul
reg add "HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings" /v ProxyServer /t REG_SZ /d "127.0.0.1:3128" /f >nul
reg add "HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings" /v ProxyOverride /t REG_SZ /d "<local>" /f >nul
echo ✅ Windows Proxy etkinleştirildi.

echo.
echo ✅ Sunucu yeni bir pencerede başlatılıyor...
start "VeloCache Sunucu" "%PROGRAM_PATH%" run

echo.
echo 🌐 Yönetim Paneli: http://127.0.0.1:8080
echo 📍 Proxy Port: 127.0.0.1:3128
echo.
echo ✅ Başlatma işlemi tamamlandı.
echo 🐧 WSL kullanıyorsanız, proxy adresi olarak 127.0.0.1:3128 kullanın.
pause