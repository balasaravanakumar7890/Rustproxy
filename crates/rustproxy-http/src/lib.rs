use anyhow::{anyhow, Result};
use http_body_util::{BodyExt, Full};
use hyper::header::HeaderName;
use hyper::header::HeaderValue;
use hyper_util::rt::TokioIo;
use rustls::pki_types::ServerName;
use rustproxy_core::{RequestModel, ResponseModel};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Instant;
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;
use tracing::debug;

pub struct HttpClient {
    tls_connector: TlsConnector,
}

impl HttpClient {
    pub fn new() -> Result<Self> {
        let _ = rustls::crypto::ring::default_provider().install_default();

        let root_store = rustls::RootCertStore {
            roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
        };

        let client_config = rustls::ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        Ok(Self {
            tls_connector: TlsConnector::from(Arc::new(client_config)),
        })
    }

    pub fn new_insecure() -> Result<Self> {
        let _ = rustls::crypto::ring::default_provider().install_default();
        // Insecure client for local development / testing with self-signed certs
        #[derive(Debug)]
        struct NoopVerifier;
        impl rustls::client::danger::ServerCertVerifier for NoopVerifier {
            fn verify_server_cert(
                &self,
                _end_entity: &rustls::pki_types::CertificateDer<'_>,
                _intermediates: &[rustls::pki_types::CertificateDer<'_>],
                _server_name: &ServerName<'_>,
                _ocsp_response: &[u8],
                _now: rustls::pki_types::UnixTime,
            ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
                Ok(rustls::client::danger::ServerCertVerified::assertion())
            }
            fn verify_tls12_signature(
                &self,
                _message: &[u8],
                _cert: &rustls::pki_types::CertificateDer<'_>,
                _dss: &rustls::DigitallySignedStruct,
            ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error>
            {
                Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
            }
            fn verify_tls13_signature(
                &self,
                _message: &[u8],
                _cert: &rustls::pki_types::CertificateDer<'_>,
                _dss: &rustls::DigitallySignedStruct,
            ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error>
            {
                Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
            }
            fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
                vec![
                    rustls::SignatureScheme::RSA_PKCS1_SHA256,
                    rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
                    rustls::SignatureScheme::ED25519,
                ]
            }
        }

        let client_config = rustls::ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(NoopVerifier))
            .with_no_client_auth();

        Ok(Self {
            tls_connector: TlsConnector::from(Arc::new(client_config)),
        })
    }

    pub async fn send(&self, req: &RequestModel) -> Result<ResponseModel> {
        let is_https = req.scheme.eq_ignore_ascii_case("https") || req.port == 443;
        let host = req.host.as_str();
        let port = req.port;
        let addr = format!("{}:{}", host, port);

        let start_time = Instant::now();
        let tcp_stream = TcpStream::connect(&addr).await?;

        let uri_to_send = if req.uri.starts_with('/') {
            req.uri.clone()
        } else if let Some(q) = &req.query {
            format!("{}?{}", req.path, q)
        } else {
            req.path.clone()
        };

        let mut hyper_req_builder = hyper::Request::builder()
            .method(req.method.as_str())
            .uri(&uri_to_send);

        for (k, v) in &req.headers {
            if let (Ok(hname), Ok(hval)) = (HeaderName::from_str(k), HeaderValue::from_str(v)) {
                hyper_req_builder = hyper_req_builder.header(hname, hval);
            }
        }

        // Set Host header if not already present
        if req.get_header("host").is_none() {
            hyper_req_builder = hyper_req_builder.header("Host", host);
        }

        let hyper_req = hyper_req_builder.body(Full::new(req.body.clone()))?;

        let (hyper_res, duration_ms) = if is_https {
            let server_name = ServerName::try_from(host.to_string())
                .map_err(|e| anyhow!("Invalid DNS name {}: {:?}", host, e))?;
            let tls_stream = self.tls_connector.connect(server_name, tcp_stream).await?;
            let io = TokioIo::new(tls_stream);

            let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;
            tokio::spawn(async move {
                if let Err(err) = conn.await {
                    debug!("Hyper connection closed with error: {:?}", err);
                }
            });

            let res = sender.send_request(hyper_req).await?;
            let duration = start_time.elapsed().as_millis() as u64;
            (res, duration)
        } else {
            let io = TokioIo::new(tcp_stream);
            let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;
            tokio::spawn(async move {
                if let Err(err) = conn.await {
                    debug!("Hyper connection closed with error: {:?}", err);
                }
            });

            let res = sender.send_request(hyper_req).await?;
            let duration = start_time.elapsed().as_millis() as u64;
            (res, duration)
        };

        let status = hyper_res.status().as_u16();
        let status_text = hyper_res
            .status()
            .canonical_reason()
            .unwrap_or("")
            .to_string();
        let version = format!("{:?}", hyper_res.version());

        let mut res_headers = Vec::new();
        for (name, val) in hyper_res.headers() {
            if let Ok(val_str) = val.to_str() {
                res_headers.push((name.as_str().to_string(), val_str.to_string()));
            }
        }

        let body_bytes = hyper_res.into_body().collect().await?.to_bytes();

        Ok(ResponseModel::new(
            &req.id,
            status,
            status_text,
            version,
            res_headers,
            body_bytes,
            duration_ms,
        ))
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new()
            .unwrap_or_else(|_| Self::new_insecure().expect("Failed to initialize HttpClient"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_init() {
        let client = HttpClient::new_insecure().expect("Insecure client init failed");
        let _ = client;
    }
}
