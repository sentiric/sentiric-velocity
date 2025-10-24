#!/bin/bash 
# Bu dosya VeloCache tarafından otomatik oluşturulmuştur. 
export HOST_IP=$(grep nameserver /etc/resolv.conf | sed 's/nameserver //') 
export http_proxy="http://$HOST_IP:3128" 
export https_proxy="http://$HOST_IP:3128" 
export HTTP_PROXY="$http_proxy" 
export HTTPS_PROXY="$https_proxy" 
export NO_PROXY="localhost,127.0.0.1" 
echo "✅ VeloCache proxy WSL için etkinleştirildi. (Host: $HOST_IP)" 
