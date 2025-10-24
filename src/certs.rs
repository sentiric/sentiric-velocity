use anyhow::{Context, Result};
use rcgen::{Certificate, CertificateParams, DistinguishedName, DnType, SanType, PKCS_ECDSA_P256_SHA256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tokio_rustls::rustls::{self, sign::SigningKey, PrivateKey, ServerConfig};
use tracing::info;

pub struct CertificateAuthority {
    ca_cert: Certificate,
    ca_key: Arc<dyn SigningKey>,
    cert_path: PathBuf,
    leaf_cache: Mutex<HashMap<String, Arc<ServerConfig>>>,
}

impl CertificateAuthority {
    pub fn new() -> Result<Self> {
        let config = crate::config::get();
        let cert_path = Path::new(&config.certs.path);
        fs::create_dir_all(cert_path).context("Sertifika dizini oluşturulamadı")?;

        let ca_cert_path = cert_path.join("ca.crt");
        let ca_key_path = cert_path.join("ca.key");

        let cert: Certificate;
        if ca_cert_path.exists() && ca_key_path.exists() {
            info!("Mevcut Kök Sertifika (CA) yükleniyor...");
            let cert_pem = fs::read_to_string(ca_cert_path)?;
            cert = Certificate::from_pem(&cert_pem)?;
        } else {
            info!("Yeni Kök Sertifika (CA) oluşturuluyor...");
            let mut params = CertificateParams::new(vec![]);
            params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
            params.distinguished_name = DistinguishedName::new();
            params.distinguished_name.push(DnType::OrganizationName, "VeloCache");
            params.distinguished_name.push(DnType::CommonName, "VeloCache Root CA (Development)");
            params.alg = &PKCS_ECDSA_P256_SHA256;

            cert = Certificate::from_params(params)?;
            let cert_pem = cert.serialize_pem()?;
            let key_pem = cert.serialize_private_key_pem();
            fs::write(&ca_cert_path, &cert_pem)?;
            fs::write(&ca_key_path, &key_pem)?;
        }
        
        let private_key_der = cert.get_key_pair().serialize_der();
        let private_key = PrivateKey(private_key_der);
        let ca_key: Arc<dyn SigningKey> = Arc::new(rustls::sign::any_ecdsa_type(&private_key)?);

        Ok(Self {
            ca_cert: cert,
            ca_key,
            cert_path: cert_path.to_path_buf(),
            leaf_cache: Mutex::new(HashMap::new()),
        })
    }
    
    pub fn get_server_config(&self, domain: &str) -> Result<Arc<ServerConfig>> {
        let mut cache = self.leaf_cache.lock().unwrap();
        if let Some(config) = cache.get(domain) {
            return Ok(config.clone());
        }

        let mut params = CertificateParams::new(vec![domain.to_string()]);
        params.subject_alt_names.push(SanType::DnsName(domain.to_string()));
        params.alg = &PKCS_ECDSA_P256_SHA256;
        
        let cert = Certificate::from_params(params)?;
        let cert_pem = cert.serialize_pem_with_signer(&self.ca_cert, &*self.ca_key)?;
        let key_pem = cert.serialize_private_key_pem();
        
        let mut cert_chain_bytes = cert_pem.as_bytes().to_vec();
        cert_chain_bytes.extend_from_slice(b"\n");
        cert_chain_bytes.extend_from_slice(self.ca_cert.serialize_pem()?.as_bytes());

        let cert_chain = rustls_pemfile::certs(&mut &*cert_chain_bytes)?
            .into_iter()
            .map(rustls::Certificate)
            .collect();
            
        let key_der = rustls_pemfile::pkcs8_private_keys(&mut key_pem.as_bytes())?.remove(0);
        let key = PrivateKey(key_der);

        let mut config = ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(cert_chain, key)?;
        
        config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

        let arc_config = Arc::new(config);
        cache.insert(domain.to_string(), arc_config.clone());
        
        Ok(arc_config)
    }

    pub fn get_ca_cert_path(&self) -> PathBuf {
        self.cert_path.join("ca.crt")
    }
}