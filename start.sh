#!/bin/bash
# VeloCache - Başlatma Betiği

cd "$(dirname "$0")"

PROGRAM_PATH="./target/release/velocache"
PID_FILE="./velocache.pid"
LOG_FILE="./velocache.log"

if [ ! -f "$PROGRAM_PATH" ]; then
    echo "❌ HATA: velocache bulunamadı! 'cargo build --release' ile derleyin."
    exit 1
fi

if [ -f "$PID_FILE" ] && ps -p $(cat "$PID_FILE") > /dev/null; then
    echo "🟡 UYARI: VeloCache zaten çalışıyor (PID: $(cat "$PID_FILE"))."
    exit 0
fi

echo "🚀 VeloCache sunucusu arka planda başlatılıyor..."
nohup "$PROGRAM_PATH" run > "$LOG_FILE" 2>&1 &
echo $! > "$PID_FILE"

sleep 1

if ps -p $(cat "$PID_FILE") > /dev/null; then
    echo "✅ VeloCache başlatıldı (PID: $(cat "$PID_FILE")). Log dosyası: $LOG_FILE"
else
    echo "❌ HATA: VeloCache başlatılamadı. Detaylar için '$LOG_FILE' dosyasını kontrol edin."
    rm "$PID_FILE"
    exit 1
fi