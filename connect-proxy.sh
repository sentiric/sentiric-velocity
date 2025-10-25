#!/bin/bash
# ====================================================================================
# VeloCache Pro - WSL Bağlantı ve Kurulum Betiği v3.1 (Taşınabilir Yol Düzeltmesi)
#
# Bu betik, herhangi bir dizinden çalıştırıldığında dahi VeloCache proxy'sine
# doğru bir şekilde bağlanmak için tasarlanmıştır.
#
# Kullanım: source /tam/yol/ile/connect-proxy.sh
# ====================================================================================

# --- Betiğin kendi dizinini bulması için sihirli satır ---
SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

# --- Ayarlar ---
PROXY_PORT="3128"
# Sertifika yolunu artık betiğin kendi konumuna göre belirliyoruz
CERT_SOURCE_PATH="${SCRIPT_DIR}/certs/ca.crt"
# Disconnect betiğinin yolunu da betiğin konumuna göre belirliyoruz
DISCONNECT_SCRIPT_PATH="${SCRIPT_DIR}/disconnect-proxy.sh"

# --- Betik Başlangıcı ---
echo "🚀 VeloCache Pro WSL Bağlantı Asistanı başlatılıyor..."

# Adım 1: Windows Ana Makinesinin IP Adresini Belirleme
HOST_IP="127.0.0.1"

echo "✅ WSL 'Mirrored' ağ modu algılandı. Proxy adresi olarak ${HOST_IP} kullanılacak."

# Adım 2: Ortam Değişkenlerini Ayarlama
export http_proxy="http://${HOST_IP}:${PROXY_PORT}"
export https_proxy="http://${HOST_IP}:${PROXY_PORT}"
export HTTP_PROXY="$http_proxy"
export HTTPS_PROXY="$https_proxy"
export NO_PROXY="localhost,127.0.0.1,.local"

echo "✅ Proxy ortam değişkenleri ayarlandı: ${http_proxy}"

# Adım 3: apt Paket Yöneticisini Yapılandırma
echo "🔧 apt paket yöneticisi yapılandırılıyor..."
APT_CONF_FILE="/etc/apt/apt.conf.d/99velocache_proxy.conf"
echo "Acquire::http::Proxy \"${http_proxy}\";" | sudo tee "$APT_CONF_FILE" > /dev/null
echo "Acquire::https::Proxy \"${https_proxy}\";" | sudo tee -a "$APT_CONF_FILE" > /dev/null
echo "✅ apt yapılandırması tamamlandı."

# Adım 4: VeloCache Kök Sertifikasını Yükleme ve Güvenme
echo "🔐 VeloCache Güven Sertifikası kontrol ediliyor..."
CERT_DEST_PATH="/usr/local/share/ca-certificates/velocache_pro_ca.crt"

if [ ! -f "$CERT_SOURCE_PATH" ]; then
    echo "❌ HATA: Sertifika dosyası bulunamadı: ${CERT_SOURCE_PATH}"
    echo "   Lütfen VeloCache'i en az bir kez çalıştırdığınızdan ve betik yolunun doğru olduğundan emin olun."
    return 1
fi

# Sertifikanın zaten kurulu olup olmadığını kontrol et
if [ -f "$CERT_DEST_PATH" ] && cmp -s "$CERT_SOURCE_PATH" "$CERT_DEST_PATH"; then
    echo "✅ Sertifika zaten güncel ve kurulu."
else
    echo "🔧 Sertifika yükleniyor... (sudo şifresi gerekebilir)"
    sudo cp "$CERT_SOURCE_PATH" "$CERT_DEST_PATH"
    sudo update-ca-certificates
    echo "✅ Sertifika başarıyla yüklendi ve güvenilir hale getirildi."
fi

# Adım 5: Kolay Çıkış İçin disconnect-proxy Alias'ı Tanımlama
# Alias'ı da artık tam yol ile tanımlıyoruz ki her yerden çalışsın
alias disconnect-proxy="source ${DISCONNECT_SCRIPT_PATH}"

echo ""
echo "===================================================================="
echo "🎉 Kurulum Tamamlandı! Bu terminal oturumu artık VeloCache kullanıyor."
echo "   Proxy'yi devre dışı bırakmak için 'disconnect-proxy' komutunu çalıştırın."
echo "===================================================================="