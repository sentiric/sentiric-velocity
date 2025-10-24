@echo off
chcp 65001 >nul
title Proxy Etkinlestirici

:: !!! BU BÖLÜMÜ KENDİ SUNUCU BİLGİLERİNİZLE DEĞİŞTİRİN !!!
set PROXY_IP=127.0.0.1
set PROXY_PORT=3128
:: !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!

echo =================================
echo  VeloCache Proxy Etkinlestiriliyor...
echo =================================
echo.
echo  Hedef Sunucu: %PROXY_IP%:%PROXY_PORT%
echo.

reg add "HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings" /v ProxyEnable /t REG_DWORD /d 1 /f >nul
reg add "HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings" /v ProxyServer /t REG_SZ /d "%PROXY_IP%:%PROXY_PORT%" /f >nul
reg add "HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings" /v ProxyOverride /t REG_SZ /d "<local>" /f >nul

echo ✅ Windows Proxy etkinlestirildi.
echo    Artik internet trafiginiz VeloCache uzerinden gececek.
echo.
pause