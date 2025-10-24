# 🚀 VeloCache - Rust ile Güçlendirilmiş Yüksek Performanslı Cache Proxy

VeloCache, hız, güvenlik ve verimlilik odaklı modern bir HTTP/HTTPS cache proxy sunucusudur. Geliştirme ve dağıtım için platforma özel betiklerle birlikte gelir.

## ✨ Temel Özellikler

- **Tek Binary:** Kolay dağıtım ve yönetim için tek bir çalıştırılabilir dosya.
- **Tam HTTPS Desteği:** Dinamik sertifika üretimi ile tam HTTPS trafiği önbelleğe alma (interception).
- **Yapılandırılabilir Cache:** Hem bellek (LRU) hem de disk tabanlı kalıcı cache desteği.
- **Gelişmiş Yönetim Arayüzü:** Dahili web arayüzü ile anlık istatistikler, canlı log akışı ve detaylı cache kontrolü.
- **Platforma Özel Betikler:** Windows ve Linux için otomatik kurulum ve yönetim betikleri.
- **Yapılandırılmış Loglama:** `tracing` ile esnek ve detaylı loglama.

---

## 🏛️ Proje Mimarisi ve Teknik Detaylar

Projenin ne yaptığını, hangi özellikleri kapsadığını ve nasıl çalıştığını anlamak için aşağıdaki dökümanları inceleyebilirsiniz:

- **[Teknik Şartname (SPECIFICATION.md)](./SPECIFICATION.md):** Projenin hedefleri, özellikleri, fonksiyonel gereksinimleri ve API tanımları.
- **[Sistem Mimarisi (ARCHITECTURE.md)](./ARCHITECTURE.md):** Projenin iç yapısı, bileşenlerin çalışması, veri akışları ve temel tasarım kararları.

---

## ⚠️ Önemli Kurulum Adımı: HTTPS Desteği ve Sertifika Kurulumu

VeloCache'in HTTPS trafiğini (örneğin, `https://google.com`) önbelleğe alabilmesi için, trafiği deşifre etmesi gerekir. Bu işlem için VeloCache, bir "Kök Güven Sertifikası" (Root CA) kullanır. Bu sertifikayı bilgisayarınıza **sadece bir kereliğine** yüklemeniz gerekmektedir.

1.  VeloCache sunucusunu `start.bat` veya `start.sh` ile başlatın.
2.  Tarayıcınızdan yönetim arayüzüne gidin: **`http://127.0.0.1:8080`**
3.  Arayüzdeki **"Güven Sertifikasını İndir (.crt)"** butonuna tıklayarak `VeloCache_CA.crt` dosyasını indirin.
4.  İndirdiğiniz dosyaya çift tıklayın ve açılan pencerede şu adımları izleyin:
    *   "Sertifika Yükle..." butonuna tıklayın.
    *   Depolama Konumu olarak **"Yerel Makine"** seçeneğini seçin ve "İleri" deyin.
    *   **"Tüm sertifikaları aşağıdaki depolama alanına yerleştir"** seçeneğini işaretleyin.
    *   "Gözat..." butonuna tıklayın ve listeden **"Güvenilen Kök Sertifika Yetkilileri"** klasörünü seçip "Tamam" deyin.
    *   "İleri" ve ardından "Son" butonuna basarak kurulumu tamamlayın.

Bu işlemden sonra tarayıcınız VeloCache üzerinden geçen HTTPS sitelerine güvecektir.

---

## 📦 Kurulum ve Derleme

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

1.  **Yapılandırma:** `config.toml` dosyasını ihtiyaçlarınıza göre düzenleyin.
2.  **Proxy'yi Başlat:** `start.bat` dosyasına sağ tıklayın ve **"Yönetici olarak çalıştır"** seçeneğini seçin. Bu betik; güvenlik duvarı kuralı ekler, sistem proxy ayarlarını yapar ve sunucuyu başlatır.
3.  **Proxy'yi Durdur:** `stop.bat` dosyasına çift tıklayarak çalıştırın. Bu betik; sunucuyu kapatır ve proxy ayarlarını geri alır.

### 🐧 Linux'ta Sunucu Olarak Çalıştırma

1.  **Yapılandırma:** `config.toml` dosyasını sunucu ortamına göre düzenleyin (`bind_address = "0.0.0.0"`).
2.  **Betikleri Çalıştırılabilir Yapma:**
    ```bash
    chmod +x start.sh stop.sh
    ```
3.  **Proxy'yi Başlat:** `./start.sh` (Arka planda başlatır)
4.  **Proxy'yi Durdur:** `./stop.sh`

---

## 🌐 Yönetim Arayüzü

Sunucu çalışırken, proxy'yi yönetmek için tarayıcınızdan aşağıdaki adrese gidin:
**`http://127.0.0.1:8080`**

Arayüz üzerinden yapabilecekleriniz:
-   **Anlık İstatistikler:** Hit oranı, toplam istek sayısı, cache boyutu ve cache'den sağlanan veri kazancı gibi metrikleri izleyin.
-   **Canlı Log Akışı:** Sunucuda gerçekleşen olayları gerçek zamanlı olarak takip edin.
-   **Cache Yönetimi:** Önbelleğe alınmış tüm girdileri (URL, boyut, tarih vb.) listeleyin ve istediğiniz girdiyi tek tıkla silin.
-   **Sertifika İndirme:** HTTPS desteği için gereken Kök Sertifikayı indirin.

---
## 👥 Uzak Kullanıcılar İçin Proxy Kullanımı

Bu proxy sunucusunu başka makinelerden kullanmak için `connect-proxy.bat` ve `connect-proxy.sh` betiklerini kullanabilirsiniz.

**Önemli:** Betikleri kullanmadan önce, içlerindeki `PROXY_IP` değişkenini VeloCache sunucusunun çalıştığı makinenin IP adresi ile değiştirmeniz gerekmektedir.