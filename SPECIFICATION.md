# VeloCache - Teknik Şartname
============================================

## 1. Proje Özeti

VeloCache, Rust dili ile geliştirilmiş, yüksek performanslı bir HTTP ve HTTPS önbellekleme (caching) proxy sunucusudur. Temel amacı, tekrar eden web isteklerini (web siteleri, API'ler, dosya indirmeleri) yerel bir önbellekte saklayarak ağ trafiğini azaltmak, bant genişliğinden tasarruf sağlamak ve erişim hızını artırmaktır.

## 2. Temel Özellikler

- **HTTP/1.1 Proxy:** Standart HTTP isteklerini vekil sunucu üzerinden yönlendirme.
- **HTTPS Interception (MitM):** `CONNECT` metodu ile kurulan TLS tünellerini sonlandırarak şifreli HTTPS trafiğini deşifre etme, inceleme ve önbelleğe alma.
- **Çift Katmanlı Önbellek (Caching):**
    - **Bellek Önbelleği:** Sık erişilen nesneler için LRU (Least Recently Used) algoritmasıyla çalışan, yüksek hızlı bellek içi önbellek.
    - **Disk Önbelleği:** Daha büyük nesneler ve kalıcılık için dosya sistemi tabanlı disk önbelleği.
- **Dinamik Sertifika Üretimi:** Her HTTPS alan adı için anlık (on-the-fly) olarak yaprak (leaf) sertifika üretimi ve kök sertifika (Root CA) ile imzalama.
- **Web Tabanlı Yönetim Arayüzü:**
    - Gerçek zamanlı istatistik takibi (Hit/Miss oranı, toplam istek, veri kazancı vb.).
    - Canlı log akışının WebSocket üzerinden izlenmesi.
    - Önbellekteki tüm girdilerin listelenmesi ve tekil olarak silinebilmesi.
    - Kök Sertifika (Root CA) dosyasının indirilmesi için arayüz.
- **Yapılandırma:** `config.toml` dosyası üzerinden tüm temel ayarların (portlar, cache limitleri, log seviyesi) yönetimi.
- **Platforma Özel Yönetim Betikleri:** Windows (`.bat`) ve Linux (`.sh`) için sunucuyu başlatma, durdurma ve sistem proxy ayarlarını otomatik yönetme.

## 3. Fonksiyonel Gereksinimler

### 3.1. Proxy Çekirdeği
- Sunucu, `config.toml` dosyasında belirtilen `proxy.bind_address` ve `proxy.port` üzerinde gelen TCP bağlantılarını dinlemelidir.
- Gelen standart HTTP istekleri (`GET`, `POST` vb.), `handler` modülü tarafından işlenmelidir.
- Gelen `CONNECT` istekleri, HTTPS Interception akışını tetiklemelidir. Hedef alan adı (hostname) istekten ayrıştırılmalı ve TLS sonlandırma işlemi için kullanılmalıdır.

### 3.2. Önbellek Sistemi
- Sadece `GET` metodu ile yapılan ve `200 OK` durum kodu ile dönen başarılı yanıtlar önbelleğe alınmalıdır.
- Her önbellek girdisi, `config.toml`'daki `cache.ttl_seconds` ile belirtilen yaşam süresine (Time-to-Live) sahip olmalıdır. Süresi dolan girdiler geçersiz sayılmalıdır.
- Bir istek geldiğinde, öncelikle bellek önbelleği kontrol edilmelidir.
- Bellekte bulunamazsa, disk önbelleği kontrol edilmelidir. Diskte bulunursa, nesne belleğe de alınmalı (LRU politikasına göre) ve istemciye sunulmalıdır.
- Hiçbir önbellekte bulunamazsa (cache miss), istek orijinal sunucuya iletilmeli ve dönen yanıt önbelleğe yazılmalıdır.

### 3.3. Yönetim Arayüzü API (`/api`)
- **`GET /api/stats`**: JSON formatında güncel sistem istatistiklerini döndürmelidir.
- **`POST /api/clear`**: Hem bellek hem de disk önbelleğini tamamen temizlemelidir.
- **`GET /api/entries`**: Disk önbelleğindeki tüm girdileri JSON formatında bir liste olarak döndürmelidir.
- **`DELETE /api/entries/:hash`**: Belirtilen hash'e sahip tek bir önbellek girdisini silmelidir.
- **`GET /api/ca.crt`**: Kök Sertifika dosyasını (`ca.crt`) `application/x-x509-ca-cert` içerik tipiyle indirme amaçlı sunmalıdır.
- **`GET /api/logs` (WebSocket)**: Sunucu loglarını gerçek zamanlı olarak yayınlayan bir WebSocket bağlantısı kurmalıdır.

## 4. Güvenlik Hususları
- HTTPS Interception, doğası gereği bir Man-in-the-Middle (MitM) saldırısı tekniğidir. Bu özellik, yalnızca geliştirme ve güvenilen ağ ortamlarında kullanılmalıdır.
- Üretilen Kök Sertifika (Root CA) özel anahtarı (`ca.key`) güvenli bir şekilde saklanmalı ve yetkisiz kişilerle paylaşılmamalıdır. Bu anahtara sahip olan biri, bu proxy'yi kullanan istemcilerin tüm şifreli trafiğini okuyabilir.
- Yönetim arayüzü, varsayılan olarak yalnızca `127.0.0.1` (localhost) üzerinden erişilebilir şekilde yapılandırılmıştır. Dış ağa açılması gerekiyorsa, güvenlik duvarı ve kimlik doğrulama gibi ek önlemler alınmalıdır.