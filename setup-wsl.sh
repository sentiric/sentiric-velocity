#!/bin/bash
# VeloCache için WSL tarafı kurulum betiği

# Bu betiğin, proje klasörü içinden çalıştırıldığını varsayıyoruz.
if [ ! -f "Cargo.toml" ]; then
    echo "HATA: Lütfen bu betiği VeloCache proje klasörünün içinden çalıştırın."
    exit 1
fi

# Komutun çalıştırıldığı dizini al (zaten WSL formatında)
VELOCACHE_PATH="$(pwd)"
echo "Proje yolu bulundu: ${VELOCACHE_PATH}"

# Hedef shell profil dosyasını belirle
if [ -n "$ZSH_VERSION" ]; then
   PROFILE_FILE=~/.zshrc
elif [ -n "$BASH_VERSION" ]; then
   PROFILE_FILE=~/.bashrc
else
   echo "Desteklenmeyen shell. Lütfen .bashrc veya .zshrc dosyanızı manuel olarak düzenleyin."
   exit 1
fi
echo "Hedef profil dosyası: ${PROFILE_FILE}"

# Mevcut VeloCache ayarlarını temizle
sed -i '/# VeloCache Proxy Kısayolları/,/# Bitiş: VeloCache/d' "${PROFILE_FILE}"
echo "Eski VeloCache ayarları temizlendi (varsa)."

# ~/.bashrc veya ~/.zshrc dosyasına kısayolları ekle
echo '' >> "${PROFILE_FILE}"
echo '# VeloCache Proxy Kısayolları' >> "${PROFILE_FILE}"
echo "alias proxy-on='source \"${VELOCACHE_PATH}/proxy-on.sh\"'" >> "${PROFILE_FILE}"
echo "alias proxy-off='source \"${VELOCACHE_PATH}/proxy-off.sh\"'" >> "${PROFILE_FILE}"
echo '# Bitiş: VeloCache' >> "${PROFILE_FILE}"
echo '' >> "${PROFILE_FILE}"

echo "✅ VeloCache kısayolları başarıyla eklendi."
echo "Değişikliklerin aktif olması için lütfen terminali yeniden başlatın veya 'source ${PROFILE_FILE}' komutunu çalıştırın."