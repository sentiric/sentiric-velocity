@echo off
title VeloCache - Durduruluyor

echo =================================
echo  VeloCache Durduruluyor...
echo =================================

:: 1. Sunucu Islemini Sonlandirma
taskkill /F /IM velocache.exe /T >nul 2>&1
if %errorlevel% equ 0 (
    echo Sunucu islemi durduruldu.
) else (
    echo Sunucu zaten calismiyor olabilir.
)

:: 2. Windows Proxy Ayarlarini Devre Disi Birakma
echo Windows sistem proxy ayarlari sifirlaniyor...
reg add "HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings" /v ProxyEnable /t REG_DWORD /d 0 /f >nul
echo Windows Proxy devre disi birakildi.

echo.
echo ----------------------------------------------------
echo  Durdurma islemi tamamlandi. Internet ayarlariniz normale dondu.
echo ----------------------------------------------------
echo.
@REM pause