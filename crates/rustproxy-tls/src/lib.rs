use anyhow::Result;
use dashmap::DashMap;
use rcgen::{
    BasicConstraints, CertificateParams, DistinguishedName, DnType, IsCa, KeyPair, KeyUsagePurpose,
    SanType,
};
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use std::sync::Arc;
use tracing::info;

pub struct CertificateAuthority {
    pub ca_cert_der: Vec<u8>,
    pub ca_cert_pem: String,
    pub ca_key_pem: String,
    ca_cert: rcgen::Certificate,
    ca_key_pair: KeyPair,
    cert_cache: DashMap<String, Arc<rustls::sign::CertifiedKey>>,
}

impl std::fmt::Debug for CertificateAuthority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CertificateAuthority")
            .field("ca_cert_pem", &self.ca_cert_pem)
            .finish()
    }
}

impl CertificateAuthority {
    pub fn generate() -> Result<Self> {
        let key_pair = KeyPair::generate_for(&rcgen::PKCS_ECDSA_P256_SHA256)?;

        let mut params = CertificateParams::default();
        params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);

        let mut dn = DistinguishedName::new();
        dn.push(DnType::CommonName, "RustProxy Root CA");
        dn.push(DnType::OrganizationName, "RustProxy Defensive Security");
        dn.push(DnType::OrganizationalUnitName, "Security Auditing");
        params.distinguished_name = dn;

        params.key_usages = vec![
            KeyUsagePurpose::KeyCertSign,
            KeyUsagePurpose::CrlSign,
            KeyUsagePurpose::DigitalSignature,
        ];

        let cert = params.self_signed(&key_pair)?;
        let ca_cert_pem = cert.pem();
        let ca_cert_der = cert.der().to_vec();
        let ca_key_pem = key_pair.serialize_pem();

        info!("Generated new RustProxy Root CA successfully");

        Ok(Self {
            ca_cert_der,
            ca_cert_pem,
            ca_key_pem,
            ca_cert: cert,
            ca_key_pair: key_pair,
            cert_cache: DashMap::new(),
        })
    }

    pub fn get_or_create_leaf_cert(&self, host: &str) -> Result<Arc<rustls::sign::CertifiedKey>> {
        if let Some(cached) = self.cert_cache.get(host) {
            return Ok(cached.clone());
        }

        let leaf_key_pair = KeyPair::generate_for(&rcgen::PKCS_ECDSA_P256_SHA256)?;
        let mut params = CertificateParams::default();

        let mut dn = DistinguishedName::new();
        dn.push(DnType::CommonName, host);
        params.distinguished_name = dn;

        if let Ok(ip) = host.parse::<std::net::IpAddr>() {
            params.subject_alt_names.push(SanType::IpAddress(ip));
        } else {
            params
                .subject_alt_names
                .push(SanType::DnsName(host.try_into()?));
            if host.eq_ignore_ascii_case("localhost") {
                if let Ok(ip4) = "127.0.0.1".parse() {
                    params.subject_alt_names.push(SanType::IpAddress(ip4));
                }
                if let Ok(ip6) = "::1".parse() {
                    params.subject_alt_names.push(SanType::IpAddress(ip6));
                }
            }
        }

        let leaf_cert = params.signed_by(&leaf_key_pair, &self.ca_cert, &self.ca_key_pair)?;

        let cert_der = CertificateDer::from(leaf_cert.der().to_vec());
        let key_der = PrivateKeyDer::Pkcs8(leaf_key_pair.serialize_der().into());

        let signing_key = rustls::crypto::ring::sign::any_ecdsa_type(&key_der)
            .map_err(|e| anyhow::anyhow!("Failed to create signing key: {:?}", e))?;

        let certified_key = rustls::sign::CertifiedKey::new(
            vec![cert_der, CertificateDer::from(self.ca_cert_der.clone())],
            signing_key,
        );

        let arc_key = Arc::new(certified_key);
        self.cert_cache.insert(host.to_string(), arc_key.clone());

        Ok(arc_key)
    }
}

#[derive(Debug)]
pub struct DynamicCertResolver {
    ca: Arc<CertificateAuthority>,
}

impl DynamicCertResolver {
    pub fn new(ca: Arc<CertificateAuthority>) -> Self {
        Self { ca }
    }
}

impl rustls::server::ResolvesServerCert for DynamicCertResolver {
    fn resolve(
        &self,
        client_hello: rustls::server::ClientHello,
    ) -> Option<Arc<rustls::sign::CertifiedKey>> {
        let host = client_hello.server_name().unwrap_or("localhost");
        match self.ca.get_or_create_leaf_cert(host) {
            Ok(key) => Some(key),
            Err(e) => {
                tracing::error!("Failed to generate leaf certificate for {}: {:?}", host, e);
                None
            }
        }
    }
}

pub fn create_mitm_server_config(
    ca: Arc<CertificateAuthority>,
) -> Result<Arc<rustls::ServerConfig>> {
    let resolver = Arc::new(DynamicCertResolver::new(ca));
    let mut config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_cert_resolver(resolver);

    config.alpn_protocols = vec![b"http/1.1".to_vec()];

    Ok(Arc::new(config))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ca_generation_and_leaf_synthesis() {
        let ca = CertificateAuthority::generate().expect("CA generation failed");
        assert!(!ca.ca_cert_pem.is_empty());
        assert!(!ca.ca_key_pem.is_empty());

        let cert_key = ca
            .get_or_create_leaf_cert("test.local")
            .expect("Leaf synthesis failed");
        assert_eq!(cert_key.cert.len(), 2); // Leaf + CA cert chain

        // Check caching
        let cert_key_2 = ca
            .get_or_create_leaf_cert("test.local")
            .expect("Leaf fetch failed");
        assert!(Arc::ptr_eq(&cert_key, &cert_key_2));
    }
}
