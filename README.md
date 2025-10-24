# 🚀 VeloCache - Rust ile Güçlendirilmiş Yüksek Performanslı Cache Proxy

VeloCache, hız, güvenlik ve verimlilik odaklı modern bir HTTP/HTTPS cache proxy sunucusudur. Geliştirme ve dağıtım için platforma özel betiklerle birlikte gelir.

## ✨ Temel Özellikler

- **Tek Binary:** Kolay dağıtım ve yönetim için tek bir çalıştırılabilir dosya.
- **HTTPS Desteği:** `CONNECT` tünelleme ile tam HTTPS uyumluluğu.
- **Yapılandırılabilir Cache:** Hem bellek (LRU) hem de disk tabanlı kalıcı cache desteği.
- **Gerçek Zamanlı Yönetim:** Dahili web arayüzü ile anlık istatistikler ve kontrol.
- **Yapılandırılmış Loglama:** `tracing` ile esnek ve detaylı loglama.

## 📦 Kurulum

1.  **Rust Kurulumu:**
    ```bash
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs/ | sh
    ```

2.  **Projeyi Derleme:**
    Projenin release versiyonunu derlemek için aşağıdaki komutu çalıştırın. Bu komut, `target/release/` dizininde platformunuza uygun bir çalıştırılabilir dosya (`velocache.exe` veya `velocache`) oluşturacaktır.
    ```bash
    cargo build --release
    ```

## 🚀 Kullanım

Proje, hem Windows'ta geliştirme yapmayı kolaylaştıran hem de Linux sunucularında dağıtımı sağlayan betikler içerir.

### 🖥️ Windows'ta Geliştirme Ortamı

Windows'ta geliştirme yaparken, proxy ayarlarınızı ve güvenlik duvarı kurallarınızı otomatik olarak yöneten `start.bat` ve `stop.bat` betiklerini kullanabilirsiniz.

1.  **Yapılandırma:** `config.toml` dosyasını ihtiyaçlarınıza göre düzenleyin.
2.  **Proxy'yi Başlat:** `start.bat` dosyasına sağ tıklayın ve **"Yönetici olarak çalıştır"** seçeneğini seçin. Bu betik:
    *   Gerekli güvenlik duvarı kuralını ekler.
    *   Windows sistem proxy ayarlarını etkinleştirir.
    *   VeloCache sunucusunu yeni bir pencerede başlatır.
3.  **Proxy'yi Durdur:** `stop.bat` dosyasına çift tıklayarak çalıştırın. Bu betik:
    *   VeloCache sunucusunu kapatır.
    *   Windows sistem proxy ayarlarını eski haline getirir.

### 🐧 Linux'ta Sunucu Olarak Çalıştırma

Linux sunucularında VeloCache'i arka planda (daemon olarak) yönetmek için `start.sh` ve `stop.sh` betiklerini kullanın.

1.  **Yapılandırma:** `config.toml` dosyasını sunucu ortamına göre düzenleyin. Özellikle `bind_address` ayarını `0.0.0.0` olarak ayarladığınızdan emin olun.
2.  **Betikleri Çalıştırılabilir Yapma:**
    ```bash
    chmod +x start.sh stop.sh
    ```
3.  **Proxy'yi Başlat:**
    ```bash
    ./start.sh
    ```
    Bu komut, sunucuyu arka planda başlatır ve logları `velocache.log` dosyasına yazar.
4.  **Proxy'yi Durdur:**
    ```bash
    ./stop.sh
    ```
    Bu komut, arka planda çalışan sunucu işlemini güvenli bir şekilde sonlandırır.

### 🌐 Yönetim Arayüzü

Sunucu çalışırken, proxy istatistiklerini görmek ve cache'i yönetmek için tarayıcınızdan aşağıdaki adrese gidin:
**`http://127.0.0.1:8080`**

## CLI Komutları

Betikleri kullanmanın yanı sıra, `velocache` uygulamasını doğrudan da çalıştırabilirsiniz:

- **Sunucuyu Başlat (Ön Planda):** `velocache run`
- **Durumu Kontrol Et:** `velocache status`
- **Sunucuyu Durdur:** `velocache stop`

---
## 👥 Uzak Kullanıcılar İçin Proxy Kullanımı

Bu proxy sunucusunu başka makinelerden kullanmak için, `client-scripts` klasöründeki betikleri kullanabilirsiniz.

**Önemli:** Betikleri kullanmadan önce, içlerindeki `PROXY_IP` değişkenini VeloCache sunucusunun çalıştığı makinenin IP adresi ile değiştirmeniz gerekmektedir.

### Windows İstemcileri İçin

1.  `client-scripts/connect-proxy.bat` dosyasını çalıştırarak sistem proxy ayarlarınızı etkinleştirin.
2.  İşiniz bittiğinde, `client-scripts/disconnect-proxy.bat` dosyasını çalıştırarak ayarları geri alın.

### Linux/macOS İstemcileri İçin

Linux ve macOS'ta proxy ayarları genellikle mevcut terminal oturumu için ayarlanır.

1.  Proxy'yi etkinleştirmek için betiği `source` komutu ile çalıştırın:
    ```bash
    source connect-proxy.sh
    ```
2.  Proxy'yi devre dışı bırakmak için, aynı terminalde aşağıdaki komutu çalıştırın:
    ```bash
    disconnect-proxy
    ```
    (Bu komut, `connect-proxy.sh` tarafından otomatik olarak oluşturulur.)
    Alternatif olarak, yeni bir terminal açarak da proxy'siz bir oturum başlatabilirsiniz.