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
echo ⚙️  Windows Proxy ayarları etkinleştiriliyor...
reg add "HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings" /v ProxyEnable /t REG_DWORD /d 1 /f >nul
reg add "HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings" /v ProxyServer /t REG_SZ /d "127.0.0.1:3128" /f >nul
reg add "HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings" /v ProxyOverride /t REG_SZ /d "<local>" /f >nul
echo ✅ Windows Proxy etkinleştirildi.

echo.
echo 🐧 WSL için proxy betikleri oluşturuluyor...

REM Önce eski dosyaları sil (varsa)
if exist wsl-proxy.sh del wsl-proxy.sh
if exist wsl-proxy-off.sh del wsl-proxy-off.sh

REM wsl-proxy.sh dosyasını satır satır oluştur
echo #!/bin/bash > wsl-proxy.sh
echo # Bu dosya VeloCache tarafından otomatik oluşturulmuştur. >> wsl-proxy.sh
echo export HOST_IP=$(grep nameserver /etc/resolv.conf ^| sed 's/nameserver //') >> wsl-proxy.sh
echo export http_proxy="http://$HOST_IP:3128" >> wsl-proxy.sh
echo export https_proxy="http://$HOST_IP:3128" >> wsl-proxy.sh
echo export HTTP_PROXY="$http_proxy" >> wsl-proxy.sh
echo export HTTPS_PROXY="$https_proxy" >> wsl-proxy.sh
echo export NO_PROXY="localhost,127.0.0.1" >> wsl-proxy.sh
echo echo "✅ VeloCache proxy WSL için etkinleştirildi. (Host: $HOST_IP)" >> wsl-proxy.sh

REM wsl-proxy-off.sh dosyasını satır satır oluştur
echo #!/bin/bash > wsl-proxy-off.sh
echo # Bu dosya VeloCache tarafından otomatik oluşturulmuştur. >> wsl-proxy-off.sh
echo unset http_proxy >> wsl-proxy-off.sh
echo unset https_proxy >> wsl-proxy-off.sh
echo unset HTTP_PROXY >> wsl-proxy-off.sh
echo unset HTTPS_PROXY >> wsl-proxy-off.sh
echo unset NO_PROXY >> wsl-proxy-off.sh
echo echo "🗑️ VeloCache proxy WSL için devre dışı bırakıldı." >> wsl-proxy-off.sh

echo ✅ WSL betikleri oluşturuldu.

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
echo ✅ Başlatma işlemi tamamlandı.
pause