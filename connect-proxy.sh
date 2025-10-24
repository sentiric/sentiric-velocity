#!/bin/bash
# VeloCache Proxy'ye baglanmak icin bu betigi 'source' komutu ile calistirin.
# Ornek: source ./connect-proxy.sh

# !!! BU BÖLÜMÜ KENDİ SUNUCU BİLGİLERİNİZLE DEĞİŞTİRİN !!!
PROXY_IP="127.0.0.1"
PROXY_PORT="3128"
# !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!

export http_proxy="http://${PROXY_IP}:${PROXY_PORT}"
export https_proxy="http://${PROXY_IP}:${PROXY_PORT}"
export HTTP_PROXY="$http_proxy"
export HTTPS_PROXY="$https_proxy"
export NO_PROXY="localhost,127.0.0.1,.local"

echo "✅ VeloCache proxy etkinlestirildi. (Hedef Sunucu: ${PROXY_IP})"
echo "   Bu ayarlar sadece bu terminal oturumu icin gecerlidir."
echo "   Kapatmak icin 'disconnect-proxy' komutunu kullanin veya yeni bir terminal acin."

# Kolaylik olmasi icin bir alias tanimlayalim
alias disconnect-proxy="unset http_proxy https_proxy HTTP_PROXY HTTPS_PROXY NO_PROXY; unalias disconnect-proxy; echo '🗑️ VeloCache proxy devre disi birakildi.'"

echo 'Acquire::http::Proxy "http://127.0.0.1:3128";' | sudo tee /etc/apt/apt.conf.d/99proxy
echo 'Acquire::https::Proxy "http://127.0.0.1:3128";' | sudo tee -a /etc/apt/apt.conf.d/99proxy

# /mnt/c/ -> Windows C: sürücüsüne karşılık gelir
sudo cp /mnt/c/sentiric/sentiric-velocity/certs/ca.crt /usr/local/share/ca-certificates/velocache.crt

sudo update-ca-certificates