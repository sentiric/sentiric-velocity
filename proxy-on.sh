#!/bin/bash
# VeloCache proxy'yi etkinleştirir (Mirrored Mode)

export http_proxy="http://127.0.0.1:3128"
export https_proxy="http://127.0.0.1:3128"
export HTTP_PROXY="$http_proxy"
export HTTPS_PROXY="$https_proxy"
export NO_PROXY="localhost,127.0.0.1"

echo "✅ VeloCache proxy WSL için etkinleştirildi. (Host: 127.0.0.1)"