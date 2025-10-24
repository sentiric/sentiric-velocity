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

## 🐧 WSL Entegrasyonu (Otomatik Kurulum)

VeloCache, Windows'un yanı sıra WSL (Windows Subsystem for Linux) içindeki `apt`, `wget`, `curl` gibi komutların trafiğini de cache'leyebilir.

`start-proxy.bat` betiği, projenizin klasöründe `wsl-proxy.sh` ve `wsl-proxy-off.sh` adında iki betik oluşturur. Bu betikleri WSL terminalinizde kolayca kullanmak için aşağıdaki **tek seferlik kurulumu** yapmanız yeterlidir:

1.  Proje klasörünüzde bir WSL terminali açın.
2.  Aşağıdaki komutu çalıştırarak `~/.bashrc` (veya kullandığınız shell'e göre `~/.zshrc`) dosyasına gerekli kısayolları ekleyin.

    ```bash
    # Proje klasörünün WSL yolunu al
    VELOCACHE_PATH=$(wslpath -a "$(pwd)")

    # ~/.bashrc dosyasına kısayolları ekle
    echo '' >> ~/.bashrc
    echo '# VeloCache Proxy Kısayolları' >> ~/.bashrc
    echo "alias proxy-on='source \"${VELOCACHE_PATH}/wsl-proxy.sh\"'" >> ~/.bashrc
    echo "alias proxy-off='source \"${VELOCACHE_PATH}/wsl-proxy-off.sh\"'" >> ~/.bashrc
    echo '' >> ~/.bashrc
    ```
3.  Terminalinizi yeniden başlatın veya `source ~/.bashrc` komutunu çalıştırın.

Artık WSL terminalinizde proxy'yi anında etkinleştirmek ve devre dışı bırakmak için şu komutları kullanabilirsiniz:

-   **Proxy'yi Etkinleştir:** `proxy-on`
-   **Proxy'yi Devre Dışı Bırak:** `proxy-off`