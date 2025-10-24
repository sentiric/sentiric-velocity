#!/bin/bash
# VeloCache Proxy baglantisini kesmek icin bu betigi 'source' komutu ile calistirin.
# Ornek: source ./disconnect-proxy.sh

unset http_proxy
unset https_proxy
unset HTTP_PROXY
unset HTTPS_PROXY
unset NO_PROXY

echo "🗑️ VeloCache proxy bu terminal oturumu icin devre disi birakildi."