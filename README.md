# 🚀 VeloCache - Rust ile Güçlendirilmiş Yüksek Performanslı Cache Proxy

VeloCache, hız, güvenlik ve verimlilik odaklı modern bir HTTP/HTTPS cache proxy sunucusudur.

## ✨ Temel Özellikler

- **Tek Binary:** Kolay dağıtım ve yönetim için tek bir çalıştırılabilir dosya.
- **HTTPS Desteği:** `CONNECT` tünelleme ile tam HTTPS uyumluluğu.
- **Yapılandırılabilir Cache:** Hem bellek (LRU) hem de disk tabanlı kalıcı cache desteği.
- **Gerçek Zamanlı Yönetim:** Dahili web arayüzü ile anlık istatistikler ve kontrol.
- **Yapılandırılmış Loglama:** `tracing` ile esnek ve detaylı loglama.
- **Whitelist Desteği:** Sadece izin verilen alan adlarına erişim.

## 📦 Kurulum

1.  **Rust Kurulumu:**
    ```bash
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs/ | sh
    ```

2.  **Projeyi Derleme:**
    ```bash
    cargo build --release
    ```
    Derleme sonrası `target/release/` altında `velocache.exe` (Windows) veya `velocache` (Linux/macOS) dosyası oluşacaktır.

## 🚀 Kullanım

1.  **Yapılandırma:** `config.toml` dosyasını ihtiyaçlarınıza göre düzenleyin.
2.  **Proxy'yi Başlatma:**
    ```bash
    ./target/release/velocache run
    ```
3.  **Web Arayüzü:**
    Tarayıcınızdan `http://127.0.0.1:8080` adresine gidin.

## CLI Komutları

- **Sunucuyu Başlat:** `velocache run`
- **Durumu Kontrol Et:** `velocache status`
- **Sunucuyu Durdur:** `velocache stop`

---

## 🐧 WSL Entegrasyonu

VeloCache, WSL (Windows Subsystem for Linux) içindeki `apt`, `wget`, `curl` gibi komutların trafiğini de cache'leyebilir. Entegrasyon için aşağıdaki **tek seferlik kurulumu** yapmanız yeterlidir.

### Tek Seferlik WSL Kurulumu

1.  Proje klasörünüzde bir WSL terminali açın. (`/mnt/c/sentiric/sentiric-velocity` gibi)
2.  `dos2unix` aracının yüklü olduğundan emin olun. Değilse, yükleyin:
    ```bash
    sudo apt update && sudo apt install dos2unix
    ```
3.  Proje ile gelen `.sh` betiklerini çalıştırılabilir yapın ve kurulumu başlatın:
    ```bash
    dos2unix *.sh
    chmod +x *.sh
    ./setup-wsl.sh
    ```
4.  Kurulum betiği `~/.bashrc` veya `~/.zshrc` dosyanıza gerekli kısayolları ekleyecektir. Değişikliklerin aktif olması için terminalinizi yeniden başlatın veya şu komutu çalıştırın:
    ```bash
    source ~/.bashrc  # veya 'source ~/.zshrc'
    ```

### WSL'de Proxy Kullanımı

Kurulumu tamamladıktan sonra, WSL terminalinizde proxy'yi anında yönetmek için şu basit komutları kullanabilirsiniz:

-   **Proxy'yi Etkinleştir:** `proxy-on`
-   **Proxy'yi Devre Dışı Bırak:** `proxy-off`

**Önemli:** Bu komutların çalışması için Windows tarafında `start-proxy.bat` ile VeloCache sunucusunun çalışıyor olması gerekir.