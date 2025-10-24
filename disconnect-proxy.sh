#!/bin/bash
# ==============================================================================
# VeloCache Pro - WSL Bağlantı Kesme Betiği v2.1 (Hata Önleyici)
#
# BU BETİK 'SOURCE' KOMUTU İLE ÇALIŞTIRILMALIDIR!
# Örnek: source ./disconnect-proxy.sh
# VEYA 'connect-proxy.sh' tarafından oluşturulan 'disconnect-proxy' alias'ını kullanın.
# ==============================================================================

# Betiğin 'source' ile mi yoksa doğrudan mı çalıştırıldığını kontrol et
# BASH_SOURCE, betiğin yolunu verir. $0 ise çalıştırılan komutu.
# Eğer source ile çalıştırılmazsa genellikle bunlar farklı olur.
# Bu kontrol %100 kusursuz olmasa da çoğu senaryoda işe yarar.
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    echo "❌ HATA: Bu betik mevcut oturumunuzu değiştirmek için 'source' komutu ile çalıştırılmalıdır."
    echo "   Doğru kullanım: source ./disconnect-proxy.sh"
    echo "   VEYA 'connect-proxy.sh' çalıştırdıktan sonra sadece 'disconnect-proxy' komutunu kullanın."
    return 1
fi

echo "🗑️ VeloCache proxy ayarları kaldırılıyor..."

# Ortam değişkenlerini temizle
unset http_proxy
unset https_proxy
unset HTTP_PROXY
unset HTTPS_PROXY
unset NO_PROXY
echo "✅ Ortam değişkenleri temizlendi."

# apt yapılandırmasını temizle
APT_CONF_FILE="/etc/apt/apt.conf.d/99velocache_proxy.conf"
if [ -f "$APT_CONF_FILE" ]; then
    echo "🔧 apt yapılandırması kaldırılıyor... (sudo şifresi gerekebilir)"
    sudo rm "$APT_CONF_FILE"
    echo "✅ apt yapılandırması kaldırıldı."
fi

# Alias'ı kaldır
unalias disconnect-proxy 2>/dev/null

echo "🎉 Bu terminal oturumu artık VeloCache kullanmıyor."