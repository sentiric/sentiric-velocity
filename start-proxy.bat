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
echo 🔥 Windows Güvenlik Duvarı kuralı oluşturuluyor (Tüm Profiller)...
powershell -Command "Remove-NetFirewallRule -DisplayName 'VeloCache Proxy' -ErrorAction SilentlyContinue" >nul
powershell -Command "New-NetFirewallRule -DisplayName 'VeloCache Proxy' -Direction Inbound -Protocol TCP -LocalPort 3128 -Action Allow -Profile 'Public, Private, Domain'" >nul
echo ✅ Güvenlik duvarı kuralı başarıyla eklendi.

echo.
echo ⚙️  Windows Proxy ayarları etkinleştiriliyor...
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
echo 🐧 WSL kullanıyorsanız, lütfen README.md dosyasındaki talimatları izleyin.
pause