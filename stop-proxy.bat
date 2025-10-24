@echo off
chcp 65001 >nul
title VeloCache - Durduruluyor

:: Yönetici İzni Kontrolü
net session >nul 2>&1
if %errorlevel% neq 0 (
    echo 🟡 Yönetici izni gerekiyor...
    powershell -Command "Start-Process '%~f0' -Verb RunAs"
    exit /b
)

echo =================================
echo  🛑 VeloCache Durduruluyor...
echo =================================

taskkill /F /IM velocache.exe /T >nul 2>&1
if %errorlevel% equ 0 (
    echo ✅ Sunucu başarıyla durduruldu.
) else (
    echo ℹ️ Sunucu zaten çalışmıyor olabilir.
)

echo.
echo 🔥 Windows Güvenlik Duvarı kuralı kaldırılıyor...
powershell -Command "Remove-NetFirewallRule -DisplayName 'VeloCache Proxy' -ErrorAction SilentlyContinue" >nul
echo ✅ Güvenlik duvarı kuralı kaldırıldı.

echo.
echo ⚙️  Windows Proxy ayarları sıfırlanıyor...
reg add "HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings" /v ProxyEnable /t REG_DWORD /d 0 /f >nul
echo ✅ Windows Proxy devre dışı bırakıldı.
echo.

pause