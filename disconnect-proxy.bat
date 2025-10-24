@echo off
chcp 65001 >nul
title Proxy Devre Disi Birakma

echo =====================================
echo  VeloCache Proxy Devre Disi Birakiliyor...
echo =====================================
echo.

reg add "HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings" /v ProxyEnable /t REG_DWORD /d 0 /f >nul

echo ✅ Windows Proxy devre disi birakildi.
echo    Internet ayarlariniz normale dondu.
echo.
pause