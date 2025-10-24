# VeloCache Pro: Birleşik Hızlandırma & Zeka Katmanı - Teknik Şartname v2.0

**Durum:** Onaylandı
**Sürüm:** 2.0
**Tarih:** 2025-10-25

## 1. Vizyon ve Temel Prensipler

### 1.1. Vizyon
VeloCache, geliştirici ortamlarından üretim altyapılarına kadar uzanan, ağ trafiğini akıllıca yöneten, hızlandıran, gözlemleyen ve otomatize eden merkezi bir katman olacaktır. Sadece HTTP tabanlı varlıkları değil, aynı zamanda DNS sorgularını, paket yöneticisi indirmelerini, Docker imajlarını ve SIP/RTP gibi gerçek zamanlı iletişim protokollerini de anlayan ve optimize eden bir **"ağ beyni"** olarak hizmet verecektir.

### 1.2. Temel Prensipler

- **Şeffaflık (Transparency):** Sistemde "bir kere kur ve unut" felsefesiyle çalışmalı, son kullanıcı veya servis için varlığı hissedilmemelidir.
- **Performans (Performance):** Rust'ın gücünü kullanarak minimum gecikme (latency) ve maksimum verim (throughput) sağlamalıdır. Bellek ve CPU kullanımı verimli olmalıdır.
- **Gözlemlenebilirlik (Observability):** Sistemden geçen her türlü trafiği anlamayı, hata ayıklamayı ve analiz etmeyi kolaylaştıran zengin araçlar sunmalıdır.
- **Kontrol (Control):** Kullanıcıya, ayrıntılı kurallar ve politikalarla sistemin davranışını tam olarak yönetme gücü vermelidir.
- **Genişletilebilirlik (Extensibility):** Çekirdek yapıyı değiştirmeden yeni protokoller ve davranışlar eklemeye olanak tanıyan bir mimariye sahip olmalıdır.

---

## 2. Mimari Sütunlar

Proje, dört ana bileşen üzerinde yükselecektir:

1.  **Evrensel Protokol Motoru:** Farklı ağ protokollerini anlayan ve yönlendiren çekirdek.
2.  **Çok Stratejili Önbellek Katmanı:** İçeriğe duyarlı, kural tabanlı ve verimli önbellekleme beyni.
3.  **Birleşik Kontrol Düzlemi:** Sistemin yönetimi ve gözlemlenmesi için sunulan insan ve makine arayüzleri.
4.  **Dağıtım ve Entegrasyon Altyapısı:** Projenin kolayca kurulmasını ve mevcut sistemlere entegre edilmesini sağlayan araçlar.

---

## 3. Detaylı Özellik Şartnamesi

### Sütun A: Evrensel Protokol Motoru

#### A-1: Gelişmiş HTTP/S Proxy (HTTP/1.1 & HTTP/2)
- **Ne:** Modern web trafiği için tam uyumlu, yüksek performanslı HTTP proxy.
- **Neden:** Web siteleri, REST API'ler, LLM/TTS/STT servisleri gibi günümüzün temel iletişim protokolünü desteklemek. HTTP/2, gRPC gibi teknolojiler için kritik öneme sahiptir.
- **Nasıl:** `hyper` ve `tokio-rustls` kütüphanelerini ALPN (h2, http/1.1) desteğiyle yapılandırmak. MitM (Man-in-the-Middle) için dinamik sertifika üretim mekanizmasını korumak.

#### A-2: Şeffaf DNS Proxy ve Önbelleği
- **Ne:** UDP/53 portunda çalışan, gelen DNS sorgularını önbelleğe alan bir DNS sunucusu.
- **Neden:** Her ağ isteğinin başlangıcı olan DNS çözümlemesini yerelde milisaniyeler içinde gerçekleştirerek tüm sistemin ağ performansını artırmak.
- **Nasıl:** `trust-dns-server` kütüphanesini kullanarak ayrı bir `tokio` görevi olarak DNS sunucusu başlatmak. Sorguları kendi LRU belleğinde TTL değerlerine uygun olarak saklamak ve upstream resolver (örn: 1.1.1.1) olarak yapılandırılmış bir adrese yönlendirmek.

#### A-3: SIP Sinyalizasyon Proxy'si (Gözlem Modu)
- **Ne:** SIP (UDP/TCP 5060) trafiğini anlayan, deşifre eden (TLS için), loglayan ve yönlendiren bir proxy katmanı.
- **Neden:** AI destekli SIP projesinde çağrı kurulumu (REGISTER, INVITE, BYE vb.) akışlarını izlemek, hata ayıklamak ve performans analizi yapmak. Bu katman, çağrıların neden başarısız olduğunu anlamak için paha biçilmezdir.
- **Nasıl:** `rsip` veya benzeri bir Rust SIP parser kütüphanesi entegre etmek. Gelen SIP mesajlarını parse ederek `Call-ID`, `From`, `To` gibi önemli başlıkları yapılandırılmış loglara (JSON) yazdırmak. Bu modda önbellekleme yapılmaz, sadece gözlem ve yönlendirme yapılır.

#### A-4: Yüksek Performanslı RTP Medya Rölesi (Passthrough)
- **Ne:** Gerçek zamanlı ses ve video akışlarını (RTP/SRTP) taşıyan UDP paketleri için bir röle (relay) görevi görmek.
- **Neden:** Medya trafiğini de tek bir noktadan geçirerek ağ topolojisini basitleştirmek ve paket kaybı, gecikme (jitter) gibi metrikleri izlemek için bir temel oluşturmak.
- **Nasıl:** SIP mesajlarındaki SDP (Session Description Protocol) içeriklerini okuyarak RTP portlarını dinamik olarak öğrenmek. Bu portlara gelen UDP trafiğini, karşı tarafın SDP'de belirttiği adrese şeffaf bir şekilde iletmek.

### Sütun B: Çok Stratejili Önbellek Katmanı

#### B-1: Akıllı Kural ve Politika Motoru
- **Ne:** `rules.toml` dosyasında tanımlanan kurallara göre önbellekleme, tünelleme, atlama veya TTL değiştirme gibi kararlar veren sistem.
- **Neden:** Farklı trafik türleri farklı davranışlar gerektirir. Bir Docker imaj katmanı uzun süre önbellekte kalabilirken, bir LLM yanıtı asla kalmamalıdır. Bu esneklik, projenin en güçlü yanıdır.
- **Nasıl:** Gelişmiş bir kural yapısı tasarlamak. Kurallar; `match_host`, `match_path`, `match_content_type` gibi koşullar ve `action: cache | passthrough | intercept_no_cache | deny`, `ttl_override` gibi eylemler içerecektir.

#### B-2: Akış Tabanlı Kalıcı Önbellek (Streaming to Disk)
- **Ne:** Büyük dosyaları (Docker imajları, OS güncellemeleri, AI modelleri) indirirken veriyi bellekte biriktirmeden, geldiği gibi hem istemciye hem de diske akıtmak.
- **Neden:** RAM kullanımını minimumda tutarak onlarca GB boyutundaki dosyaların bile sorunsuzca önbelleğe alınmasını sağlamak.
- **Nasıl:** Upstream'den gelen `Body` stream'indeki her `chunk`'ı aynı anda iki hedefe (istemciye giden `Body::channel` ve diske yazılan geçici bir dosya) göndermek.

#### B-3: Proaktif Önbellek Doldurma (Cache Warming)
- **Ne:** VeloCache başlatıldığında veya bir API çağrısıyla, önceden tanımlanmış bir liste URL'i indirip önbelleğe alması.
- **Neden:** Sık kullanılan Docker base imajları, `npm` paketleri veya temel AI modellerinin ilk istekten önce önbellekte hazır olmasını sağlamak.
- **Nasıl:** `rules.toml`'a `warmup_urls` listesi eklemek ve başlangıçta bu URL'leri indiren bir arkaplan görevi başlatmak.

### Sütun C: Birleşik Kontrol Düzlemi

#### C-1: VeloCache Companion (Masaüstü Yardımcı Uygulaması)
- **Ne:** Sistem tepsisinde çalışan, Windows, macOS ve Linux destekli, `Tauri` ile geliştirilmiş bir uygulama.
- **Neden:** Sertifika kurulumu, sistem proxy ve DNS ayarlarının yönetimi gibi karmaşık işlemleri tek tıklamaya indirgeyerek "şeffaflık" ilkesini hayata geçirmek.
- **Nasıl:** `Tauri`'nin Rust backend'ini kullanarak platforma özel sistem komutlarını (sertifika ekleme, ağ ayarlarını değiştirme) güvenli bir şekilde çalıştırmak.

#### C-2: Gerçek Zamanlı Gözlem Arayüzü (Web UI v2.0)
- **Ne:** SvelteKit veya Vue ile geliştirilmiş, reaktif, modern bir web paneli.
- **Neden:** Geliştiricilere ve sistem yöneticilerine, ağda neler olup bittiğini anlamaları için güçlü bir görsel araç sunmak.
- **Nasıl:** WebSocket üzerinden anlık veri akışı sağlayarak Ağ Akışı Monitörü, Detaylı İnceleyici, Görsel Analitik ve İnteraktif Kural Düzenleyici gibi bileşenler geliştirmek.

#### C-3: Otomasyon Odaklı Komut Satırı Arayüzü (CLI)
- **Ne:** `clap` ile geliştirilmiş, script'lerde ve otomasyonlarda kullanılabilecek zengin bir CLI.
- **Neden:** VeloCache'i CI/CD pipeline'larına veya sunucu otomasyon betiklerine entegre etmek.
- **Nasıl:** `stats`, `cache`, `rules`, `service` gibi alt komutlarla tüm temel yönetim işlevlerini komut satırına taşımak.

### Sütun D: Dağıtım ve Entegrasyon Altyapısı

#### D-1: Platforma Özel Kurulum Paketleri
- **Ne:** MSI (Windows), DEB/RPM (Linux), Homebrew (macOS) gibi standart formatlarda kurulum paketleri.
- **Neden:** Kullanıcıların `cargo build` gibi adımlarla uğraşmadan, VeloCache'i bir ürün gibi kolayca kurabilmelerini sağlamak.
- **Nasıl:** GitHub Actions CI/CD pipeline'ı kullanarak her release için bu paketleri otomatik olarak oluşturmak ve yayınlamak.

#### D-2: Eklenti (Plugin) Mimarisi (Gelecek Vizyonu)
- **Ne:** Üçüncü partilerin VeloCache için yeni protokoller (örn: FTP, SOCKS5) veya özel davranışlar eklemesine olanak tanıyan bir sistem.
- **Neden:** Projenin çekirdeğini yalın tutarken, topluluğun ve özel ihtiyaçların projeyi sonsuz şekilde genişletmesine imkan tanımak.
- **Nasıl:** WASM (WebAssembly) tabanlı bir eklenti sistemi tasarlamak. VeloCache, belirli olaylarda (`on_dns_query`, `on_sip_invite`) `.wasm` eklentilerini güvenli bir sanal ortamda (`Wasmer` ile) çalıştırabilir.

---

## 4. Aşamalı Uygulama Yol Haritası

- **Faz 1: Temel Deneyim ve Sağlamlaştırma (İlk 3-6 Ay)**
  - **Odak:** Kullanıcı deneyimini kökten iyileştirmek ve en yaygın senaryoları desteklemek.
  - **Hedefler:** A-1 (HTTP/2), A-2 (DNS Proxy), B-1 (Temel Kural Motoru), C-1 (Companion App), C-3 (Temel CLI).

- **Faz 2: Güç ve Zeka (Sonraki 6-9 Ay)**
  - **Odak:** Gelişmiş önbellekleme stratejileri ve gözlemlenebilirlik araçları.
  - **Hedefler:** B-2 (Streaming Cache), B-3 (Cache Warming), C-2 (Web UI v2.0 - Network Inspector ile), A-3 (SIP Gözlem Modu).

- **Faz 3: Ekosistem ve Profesyonelleşme (Uzun Vade)**
  - **Odak:** Projeyi paketlenmiş bir ürün haline getirmek ve genişletilebilirliği sağlamak.
  - **Hedefler:** D-1 (Kurulum Paketleri), A-4 (RTP Relay), C-2 (UI Kural Düzenleyici), D-2 (Plugin Mimarisi Ar-Ge).