#!/bin/bash
# VeloCache proxy'yi etkinleştirir

export HOST_IP=$(grep nameserver /etc/resolv.conf | sed 's/nameserver //')

if [ -z "$HOST_IP" ]; then
    echo "❌ HATA: Windows Host IP adresi /etc/resolv.conf içinde bulunamadı."
    return 1
fi

export http_proxy="http://${HOST_IP}:3128"
export https_proxy="http://${HOST_IP}:3128"
export HTTP_PROXY="$http_proxy"
export HTTPS_PROXY="$https_proxy"
export NO_PROXY="localhost,127.0.0.1"

echo "✅ VeloCache proxy WSL için etkinleştirildi. (Host: ${HOST_IP})"