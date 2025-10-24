#!/bin/bash
# VeloCache proxy'yi devre dışı bırakır

unset http_proxy
unset https_proxy
unset HTTP_PROXY
unset HTTPS_PROXY
unset NO_PROXY

echo "🗑️ VeloCache proxy WSL için devre dışı bırakıldı."