@echo off
title VeloCache Baslatici (Windows Otomatik Proxy)
cd /d "%~dp0"

:: 1. Yonetici Izni Kontrolu
net session >nul 2>&1
if %errorlevel% neq 0 (
    echo YONETICI IZNI GEREKIYOR...
    echo    Guvenlik duvari ve proxy ayarlari icin script yeniden baslatilacak.
    powershell -Command "Start-Process '%~f0' -Verb RunAs"
    exit /b
)

echo =================================
echo  VeloCache Baslatiliyor...
echo =================================

:: 2. Programin varligini kontrol et
set "PROGRAM_PATH=%~dp0target\release\velocache.exe"
if not exist "%PROGRAM_PATH%" (
    echo HATA: velocache.exe bulunamadi!
    echo    Lutfen once 'cargo build --release' ile derleyin.
    pause
    exit /b 1
)

:: 3. Windows Guvenlik Duvari Kurali Ekleme
echo Guvenlik Duvari kurali ekleniyor/guncelleniyor...
powershell -Command "New-NetFirewallRule -DisplayName 'VeloCache Proxy' -Direction Inbound -Protocol TCP -LocalPort 3128 -Action Allow -Profile Any -ErrorAction SilentlyContinue" >nul
echo Guvenlik duvari kurali eklendi.

:: 4. Windows Proxy Ayarlarini Etkinlestirme
echo Windows sistem proxy ayarlari etkinlestiriliyor...
reg add "HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings" /v ProxyEnable /t REG_DWORD /d 1 /f >nul
reg add "HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings" /v ProxyServer /t REG_SZ /d "127.0.0.1:3128" /f >nul
reg add "HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings" /v ProxyOverride /t REG_SZ /d "<local>" /f >nul
echo Windows Proxy etkinlestirildi.

:: 5. Sunucuyu Baslatma
echo.
echo Sunucu yeni bir pencerede baslatiliyor...
set NO_COLOR=1
start "VeloCache Sunucu" "%PROGRAM_PATH%" run

echo.
echo ----------------------------------------------------
echo  Baslatma islemi tamamlandi.
echo  Yonetim Paneli: http://127.0.0.1:8080
echo  Proxy Port: 127.0.0.1:3128
echo ----------------------------------------------------
echo.
echo Bu pencereyi kapatabilirsiniz. Sunucu ayri bir pencerede calismaya devam edecek.
pause