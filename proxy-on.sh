#!/bin/bash
# VeloCache proxy'yi etkinleştirir

# Windows Host IP'sini doğrudan ipconfig.exe'den al ve tüm istenmeyen karakterleri temizle
# tr -d '\r' komutu, Windows'un satır sonu karakterini (CR) siler.
export HOST_IP=$(ipconfig.exe | grep -A 4 "vEthernet (WSL" | grep "IPv4 Address" | sed 's/.*: //' | tr -d '[:space:]')

if [ -z "$HOST_IP" ]; then
    echo "❌ HATA: Windows Host IP adresi bulunamadı. 'ipconfig.exe' çıktısında 'vEthernet (WSL' adaptörü kontrol edin."
    # Yedek yöntem
    echo "ℹ️ Yedek yöntem deneniyor (/etc/resolv.conf)..."
    export HOST_IP=$(grep nameserver /etc/resolv.conf | sed 's/nameserver //' | tr -d '[:space:]')
    if [ -z "$HOST_IP" ]; then
        echo "❌ HATA: Yedek yöntem de başarısız oldu. Lütfen ağ ayarlarınızı kontrol edin."
        return 1
    fi
fi

export http_proxy="http://${HOST_IP}:3128"
export https_proxy="http://${HOST_IP}:3128"
export HTTP_PROXY="$http_proxy"
export HTTPS_PROXY="$https_proxy"
export NO_PROXY="localhost,1227.0.0.1"

echo "✅ VeloCache proxy WSL için etkinleştirildi. (Host: ${HOST_IP})"