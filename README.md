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