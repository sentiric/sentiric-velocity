# 🚀 VeloCache Pro - Birleşik Hızlandırma & Zeka Katmanı

VeloCache, sadece bir HTTP/S cache proxy'si değil; geliştiriciler ve sistemler için tasarlanmış, ağ trafiğini akıllıca yöneten, hızlandıran ve gözlemleyen **evrensel bir ağ beynidir**. Amacı, geliştirme döngülerini kısaltmak, tekrarlayan indirmeleri ortadan kaldırmak ve karmaşık sistemlerdeki ağ etkileşimlerini şeffaf bir şekilde optimize etmektir.

"Bir kere kur ve unut" felsefesiyle, VeloCache makineniz ile internet arasına yerleşerek size daha hızlı ve verimli bir çalışma ortamı sunar.

## ✨ Temel Özellikler

VeloCache, modern geliştirme ve altyapı ihtiyaçlarına cevap vermek için devrim yaratacak bir dizi özellik sunar:

- **Evrensel Protokol Desteği:**
  - **Gelişmiş HTTP/S Proxy:** HTTP/1.1, HTTP/2 ve gRPC trafiği için tam MitM (Man-in-the-Middle) önbellekleme.
  - **Şeffaf DNS Proxy:** Tüm DNS sorgularını yerel olarak önbelleğe alarak her ağ isteğini milisaniyelerce hızlandırma.
  - **SIP & RTP Gözlemlenebilirliği:** VoIP ve telekomünikasyon sistemlerindeki sinyal ve medya akışlarını izleme ve hata ayıklama yeteneği.

- **Akıllı ve Güçlü Önbellekleme:**
  - **Kural Tabanlı Yönetim:** Hangi içeriğin, ne zaman ve nasıl önbelleğe alınacağını (`rules.toml` ile) tam olarak siz kontrol edin.
  - **Devasa Dosya Desteği:** Akış tabanlı disk yazma (Streaming to Disk) ile onlarca GB boyutundaki Docker imajlarını, AI modellerini veya videoları bile RAM'inizi doldurmadan önbelleğe alın.
  - **Proaktif Doldurma (Cache Warming):** Sık kullandığınız araçları ve imajları siz istemeden önce önbelleğe alarak ortam kurulum sürelerini sıfıra indirin.

- **Zahmetsiz Kullanıcı Deneyimi:**
  - **VeloCache Companion:** Sistem tepsisinde çalışan yardımcı uygulama ile tek tıkla sertifika kurun, proxy ve DNS ayarlarınızı yönetin.
  - **Modern Yönetim Paneli:** Gerçek zamanlı ağ akışını izleyin, önbelleği yönetin ve istatistikleri canlı grafiklerle görün.
  - **Kapsamlı CLI:** Tüm yönetim işlevlerini otomasyon ve script'lerinizde kullanın.

- **Profesyonel ve Genişletilebilir:**
  - **Tek Binary Dağıtım:** Kolay kurulum ve yönetim.
  - **Platforma Özel Kurulumcular:** MSI, DEB, RPM ve Homebrew ile zahmetsiz kurulum.
  - **Eklenti Mimarisi (Gelecek):** WASM tabanlı eklentilerle VeloCache'e yeni yetenekler kazandırın.

---

## 🏛️ Proje Mimarisi ve Teknik Detaylar

Projenin vizyonunu, hedeflerini ve teknik altyapısını anlamak için aşağıdaki dokümanları inceleyebilirsiniz:

- **[Teknik Şartname v2.0](./docs/SPECIFICATION_V2.md):** Projenin hedefleri, tüm özellikleri, fonksiyonel gereksinimleri ve API tanımları.
- **[Sistem Mimarisi v2.0](./docs/ARCHITECTURE_v2.md):** Projenin iç yapısı, bileşenlerin çalışması, veri akışları ve temel tasarım kararları.

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