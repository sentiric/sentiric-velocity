# VeloCache - Sistem Mimarisi ve Tasarım Notları
==================================================

## 1. Genel Bakış

VeloCache, `tokio` asenkron çalışma zamanı üzerine inşa edilmiş, modüler bir Rust uygulamasıdır. Sistem, iki ana eş zamanlı görevden oluşur:
1.  **Proxy Sunucusu:** Gelen HTTP ve HTTPS isteklerini dinleyen ve işleyen ana görev.
2.  **Yönetim Sunucusu:** Web tabanlı arayüz ve API hizmeti sunan ikincil görev.

Bu iki görev, `Arc<T>` (Atomically Reference-Counted) yapısı aracılığıyla paylaşılan `CacheManager` ve `CertificateAuthority` gibi merkezi durum (state) bileşenlerine erişir.

![High Level Architecture Diagram - Textual Representation]
[Client] ---> [Proxy Server (hyper on tokio)] ---> [CacheManager] ---> [Internet]
                |         ^
                |         | (Decrypted Request)
                v         |
           [CONNECT Handler] ---> [CertificateAuthority]

[Admin UI] <---> [Management Server (warp)] ---> [CacheManager]
                                              |
                                              +-> [CertificateAuthority]

## 2. Bileşenlerin Detaylı Açıklaması

### `main.rs`
- **Giriş Noktası:** Uygulamanın başlangıç noktasıdır.
- **Sorumlulukları:**
    - `clap` ile komut satırı argümanlarını ayrıştırma.
    - `config::init()` ile `config.toml` dosyasını yükleme.
    - `tracing_subscriber` ile loglama altyapısını kurma (konsol ve WebSocket için `BroadcastLayer` dahil).
    - `CertificateAuthority` ve `CacheManager` örneklerini (instance) oluşturma.
    - Proxy ve Yönetim sunucularını ayrı `tokio` görevleri olarak başlatma.

### `proxy.rs` - Proxy Çekirdeği
- **Teknoloji:** `hyper`
- **İş Akışı:**
    1. `tokio::net::TcpListener` ile belirtilen portta bir soket açar.
    2. Gelen her bağlantı için bir `hyper::service::service_fn` oluşturulur.
    3. `serve_req` fonksiyonu, isteğin metodunu kontrol eder:
        - **Standart HTTP (örn: GET):** İstek doğrudan `handler::proxy_handler`'a yönlendirilir.
        - **`CONNECT` Metodu (HTTPS Talebi):** Bu, en kritik akıştır:
            a. `hyper::upgrade::on(req)` ile ham TCP bağlantısı (upgraded stream) ele geçirilir.
            b. İsteğin URI'sinden hedef alan adı (örn: `google.com:443`) alınır.
            c. `CertificateAuthority::get_server_config` çağrılarak bu alan adına özel, anlık bir TLS sertifikası ve yapılandırması alınır.
            d. `tokio_rustls::TlsAcceptor` ile istemci ve proxy arasında TLS el sıkışması (handshake) tamamlanır.
            e. Artık şifresi çözülmüş olan bu TLS akışı, **yeni bir `hyper` sunucusuna** verilir. Bu sunucu, gelen şifresiz HTTP isteklerini alır, URI'larını `https://<domain>/path` şeklinde yeniden oluşturur ve `handler::proxy_handler`'a gönderir.

### `certs.rs` - Sertifika Yönetimi
- **Teknoloji:** `rcgen`, `rustls`
- **Sorumlulukları:**
    - **Kök Sertifika (Root CA) Yönetimi:**
        - İlk çalıştırmada, `certs` dizininde bir `ca.crt` (sertifika) ve `ca.key` (özel anahtar) dosyası oluşturur.
        - Sonraki çalıştırmalarda, bu kalıcı dosyaları diskten yükler.
    - **Yaprak Sertifika (Leaf Certificate) Üretimi:**
        - `get_server_config(domain)` metodu, her benzersiz alan adı için çağrılır.
        - Bellekteki bir `HashMap` önbelleğini (`leaf_cache`) kontrol eder.
        - Önbellekte yoksa, `rcgen` kullanarak o alan adına özel yeni bir sertifika oluşturur ve bunu Kök CA ile imzalar.
        - Bu yeni sertifika ve anahtarı kullanarak bir `rustls::ServerConfig` oluşturur. Bu yapılandırma, TLS el sıkışması için gereklidir.
        - Oluşturulan yapılandırmayı ilerideki istekler için bellekteki `HashMap`'e ekler.

### `handler.rs` - İstek İşleyici
- Bu modül, hem standart HTTP isteklerini hem de `proxy.rs` tarafından şifresi çözülmüş HTTPS isteklerini işleyen ortak mantığı barındırır.
- **İş Akışı:**
    1. Gelen isteğin `METHOD::URI` formatında bir önbellek anahtarı (`cache_key`) oluşturur.
    2. `CacheManager::get` ile önbelleği kontrol eder.
        - **Cache Hit:** Yanıtı önbellekten alır, uygun başlıkları (header) ekler ve doğrudan istemciye döndürür.
    3. **Cache Miss:**
        - `hyper::Client` kullanarak isteği orijinal sunucuya iletir.
        - Gelen yanıtı alır.
        - Eğer yanıt başarılı ise (`200 OK` ve `GET` metodu), `CacheManager::put` ile yanıt gövdesini ve başlıklarını önbelleğe yazar.
        - Yanıtı istemciye geri döndürür.

### `cache.rs` - Önbellek Yöneticisi
- **Yapı:**
    - `memory_cache`: `tokio::sync::Mutex` ile korunan bir `lru::LruCache`. Hızlı ve sık erişilen veriler için.
    - `disk_path`: Disk önbelleği için dosya yolu.
- **Mantık:**
    - Anahtar olarak `METHOD::URI` string'inin MD5 hash'i kullanılır. Bu, dosya adlarının geçerli ve tekil olmasını sağlar.
    - `put`: Girdiyi hem `memory_cache`'e yazar hem de `bincode` ile serileştirerek diske kaydeder.
    - `get`: Önce `memory_cache`'e bakar. Bulamazsa, diskten okumayı dener.
    - `CacheStatsInternal`: Tüm istatistikler (`hits`, `misses`, `disk_size` vb.), iş parçacıkları arasında güvenli (thread-safe) ve kilitsiz (lock-free) güncelleme için `std::sync::atomic` türleri kullanılarak tutulur.

### `management.rs` - Yönetim Sunucusu
- **Teknoloji:** `warp`
- **Sorumlulukları:**
    - `config.toml`'da belirtilen portta bir HTTP sunucusu başlatır.
    - Statik `index.html` dosyasını sunar.
    - `/api/*` altındaki tüm RESTful API endpoint'lerini tanımlar.
    - Paylaşılan `CacheManager` ve `CertificateAuthority` durumlarına `warp` filtreleri (`warp::any().map(...)`) aracılığıyla erişir.
    - **Loglama için WebSocket:**
        - `BroadcastLayer` adında özel bir `tracing::Layer` tanımlanmıştır.
        - Bu katman, `INFO` seviyesi ve üzerindeki tüm log olaylarını yakalar.
        - Yakalanan mesajları `tokio::sync::broadcast::Sender` aracılığıyla genel bir kanala gönderir.
        - `/api/logs` endpoint'ine bağlanan her WebSocket istemcisi, bu kanala abone (`subscribe`) olur ve log akışını alır.