#!/bin/bash
# VeloCache - Durdurma Betiği

cd "$(dirname "$0")"
PID_FILE="./velocache.pid"

echo "🛑 VeloCache sunucusu durduruluyor..."

if [ ! -f "$PID_FILE" ]; then
    echo "ℹ️ PID dosyası bulunamadı. Sunucu çalışmıyor olabilir."
    exit 0
fi

PID=$(cat "$PID_FILE")
if [ -z "$PID" ] || ! ps -p $PID > /dev/null; then
    echo "ℹ️ Sunucu zaten çalışmıyor."
    rm -f "$PID_FILE"
    exit 0
fi

kill $PID
sleep 1

if ps -p $PID > /dev/null; then
    echo "🟡 Zorla sonlandırma deneniyor (kill -9)..."
    kill -9 $PID
fi

echo "✅ Sunucu durduruldu."
rm -f "$PID_FILE"