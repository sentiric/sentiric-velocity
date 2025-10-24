@echo off
chcp 65001 >nul
title VeloCache Başlatıcı

:: ==================================================================
:: Yönetici İzni Kontrolü ve Otomatik Yükseltme
:: ==================================================================
net session >nul 2>&1
if %errorlevel% neq 0 (
    echo 🟡 Yönetici izni gerekiyor...
    echo    Güvenlik duvarı kuralı eklemek için script yeniden başlatılacak.
    powershell -Command "Start-Process '%~f0' -Verb RunAs"
    exit /b
)

echo =================================
echo  🚀 VeloCache Başlatılıyor...
echo =================================
cd /d "%~dp0"

REM Derlenmiş dosyanın tam yolunu al
set "PROGRAM_PATH=%~dp0target\release\velocache.exe"

REM Derlenmiş dosyanın varlığını kontrol et
if not exist "%PROGRAM_PATH%" (
    echo.
    echo ❌ HATA: velocache.exe bulunamadı!
    echo    Lütfen önce 'cargo build --release' ile derleyin.
    echo.
    pause
    exit /b 1
)

echo.
echo 🔥 Windows Güvenlik Duvarı kuralı oluşturuluyor (WSL erişimi için)...
powershell -Command "Remove-NetFirewallRule -DisplayName 'VeloCache Proxy' -ErrorAction SilentlyContinue"
powershell -Command "New-NetFirewallRule -DisplayName 'VeloCache Proxy' -Direction Inbound -Protocol TCP -LocalPort 3128 -Program '%PROGRAM_PATH%' -Action Allow"
echo ✅ Güvenlik duvarı kuralı başarıyla eklendi.

echo.
echo ⚙️  Windows Proxy ayarları etkinleştiriliyor...
reg add "HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings" /v ProxyEnable /t REG_DWORD /d 1 /f >nul
reg add "HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings" /v ProxyServer /t REG_SZ /d "127.0.0.1:3128" /f >nul
reg add "HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings" /v ProxyOverride /t REG_SZ /d "<local>" /f >nul
echo ✅ Windows Proxy etkinleştirildi.

echo.
echo 🐧 WSL için proxy betikleri oluşturuluyor...
if exist wsl-proxy.sh del wsl-proxy.sh
if exist wsl-proxy-off.sh del wsl-proxy-off.sh
(echo #!/bin/bash & echo # Bu dosya VeloCache tarafından otomatik oluşturulmuştur. & echo export HOST_IP=$(grep nameserver /etc/resolv.conf ^| sed 's/nameserver //') & echo export http_proxy="http://$HOST_IP:3128" & echo export https_proxy="http://$HOST_IP:3128" & echo export HTTP_PROXY="$http_proxy" & echo export HTTPS_PROXY="$https_proxy" & echo export NO_PROXY="localhost,127.0.0.1" & echo echo "✅ VeloCache proxy WSL için etkinleştirildi. (Host: $HOST_IP)") > wsl-proxy.sh
(echo #!/bin/bash & echo # Bu dosya VeloCache tarafından otomatik oluşturulmuştur. & echo unset http_proxy & echo unset https_proxy & echo unset HTTP_PROXY & echo unset HTTPS_PROXY & echo unset NO_PROXY & echo echo "🗑️ VeloCache proxy WSL için devre dışı bırakıldı.") > wsl-proxy-off.sh
wsl dos2unix wsl-proxy.sh >nul 2>&1
wsl dos2unix wsl-proxy-off.sh >nul 2>&1
echo ✅ WSL betikleri kullanıma hazır.

echo.
echo ✅ Sunucu yeni bir pencerede başlatılıyor...
start "VeloCache Sunucu" "%PROGRAM_PATH%" run

echo.
echo 🌐 Yönetim Paneli: http://127.0.0.1:8080
echo 📍 Proxy Port: 3128
echo.
echo ✅ Başlatma işlemi tamamlandı.
pause