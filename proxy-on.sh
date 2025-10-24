#!/bin/bash
# VeloCache proxy'yi etkinleştirir

# Windows Host IP'sini doğrudan ipconfig.exe'den almanın en sağlam yolu
export HOST_IP=$(ipconfig.exe | grep -A 4 "vEthernet (WSL" | grep "IPv4 Address" | sed 's/.*: //')

if [ -z "$HOST_IP" ]; then
    echo "❌ HATA: Windows Host IP adresi bulunamadı. 'ipconfig.exe' çıktısında 'vEthernet (WSL' adaptörü kontrol edin."
    # Yedek yöntem
    echo "ℹ️ Yedek yöntem deneniyor (/etc/resolv.conf)..."
    export HOST_IP=$(grep nameserver /etc/resolv.conf | sed 's/nameserver //')
    if [ -z "$HOST_IP" ]; then
        echo "❌ HATA: Yedek yöntem de başarısız oldu. Lütfen ağ ayarlarınızı kontrol edin."
        return 1
    fi
fi

export http_proxy="http://${HOST_IP}:3128"
export https_proxy="http://${HOST_IP}:3128"
export HTTP_PROXY="$http_proxy"
export HTTPS_PROXY="$https_proxy"
export NO_PROXY="localhost,127.0.0.1"

echo "✅ VeloCache proxy WSL için etkinleştirildi. (Host: ${HOST_IP})"