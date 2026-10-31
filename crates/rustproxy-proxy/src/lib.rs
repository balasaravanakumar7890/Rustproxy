use anyhow::Result;
use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper::header::{HeaderName, HeaderValue};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use rustproxy_core::{FlowRecord, FlowSummary, RequestModel, SecurityFinding};
use rustproxy_http::HttpClient;
use rustproxy_intercept::{InterceptDecision, InterceptManager};
use rustproxy_sitemap::SiteMap;
use rustproxy_storage::StorageEngine;
use rustproxy_tls::{create_mitm_server_config, CertificateAuthority};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, RwLock};
use tokio_rustls::TlsAcceptor;
use tracing::{debug, error, info};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProxyEvent {
    FlowRecorded(FlowSummary),
    FindingDetected(SecurityFinding),
    RequestIntercepted(RequestModel),
}

#[derive(Debug, Clone)]
pub struct ProxyConfig {
    pub bind_addr: SocketAddr,
    pub upstream_insecure: bool,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            bind_addr: "127.0.0.1:8080".parse().unwrap(),
            upstream_insecure: true,
        }
    }
}

pub struct ProxyServer {
    config: ProxyConfig,
    ca: Arc<CertificateAuthority>,
    storage: Arc<StorageEngine>,
    interceptor: Arc<InterceptManager>,
    sitemap: Arc<RwLock<SiteMap>>,
    scanner: Arc<rustproxy_scanner::ScannerEngine>,
    http_client: Arc<HttpClient>,
    event_tx: broadcast::Sender<ProxyEvent>,
}

impl ProxyServer {
    pub fn new(
        config: ProxyConfig,
        ca: Arc<CertificateAuthority>,
        storage: Arc<StorageEngine>,
        interceptor: Arc<InterceptManager>,
        sitemap: Arc<RwLock<SiteMap>>,
        scanner: Arc<rustproxy_scanner::ScannerEngine>,
    ) -> Result<Self> {
        let (event_tx, _) = broadcast::channel(1000);
        let http_client = Arc::new(if config.upstream_insecure {
            HttpClient::new_insecure()?
        } else {
            HttpClient::new()?
        });

        Ok(Self {
            config,
            ca,
            storage,
            interceptor,
            sitemap,
            scanner,
            http_client,
            event_tx,
        })
    }

    pub fn subscribe_events(&self) -> broadcast::Receiver<ProxyEvent> {
        self.event_tx.subscribe()
    }

    pub async fn run(&self) -> Result<()> {
        let listener = TcpListener::bind(self.config.bind_addr).await?;
        info!("RustProxy listening on http://{}", self.config.bind_addr);

        let tls_server_config = create_mitm_server_config(self.ca.clone())?;
        let tls_acceptor = TlsAcceptor::from(tls_server_config);

        loop {
            match listener.accept().await {
                Ok((stream, client_addr)) => {
                    let _ca = self.ca.clone();
                    let storage = self.storage.clone();
                    let interceptor = self.interceptor.clone();
                    let sitemap = self.sitemap.clone();
                    let scanner = self.scanner.clone();
                    let http_client = self.http_client.clone();
                    let tls_acceptor = tls_acceptor.clone();
                    let event_tx = self.event_tx.clone();

                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_connection(
                            stream,
                            client_addr,
                            tls_acceptor,
                            storage,
                            interceptor,
                            sitemap,
                            scanner,
                            http_client,
                            event_tx,
                        )
                        .await
                        {
                            debug!("Connection from {} closed: {:?}", client_addr, e);
                        }
                    });
                }
                Err(e) => {
                    error!("TCP Accept error: {:?}", e);
                }
            }
        }
    }

    async fn handle_connection(
        mut stream: TcpStream,
        client_addr: SocketAddr,
        tls_acceptor: TlsAcceptor,
        storage: Arc<StorageEngine>,
        interceptor: Arc<InterceptManager>,
        sitemap: Arc<RwLock<SiteMap>>,
        scanner: Arc<rustproxy_scanner::ScannerEngine>,
        http_client: Arc<HttpClient>,
        event_tx: broadcast::Sender<ProxyEvent>,
    ) -> Result<()> {
        let mut peek_buf = [0u8; 1024];
        let n = stream.peek(&mut peek_buf).await?;
        if n == 0 {
            return Ok(());
        }

        let peeked = String::from_utf8_lossy(&peek_buf[..n]);
        if peeked.starts_with("CONNECT ") {
            // Read the full CONNECT request
            let mut connect_req = Vec::new();
            let mut buf = [0u8; 512];
            loop {
                let bytes_read = stream.read(&mut buf).await?;
                if bytes_read == 0 {
                    return Ok(());
                }
                connect_req.extend_from_slice(&buf[..bytes_read]);
                if connect_req.windows(4).any(|w| w == b"\r\n\r\n") {
                    break;
                }
            }

            let req_str = String::from_utf8_lossy(&connect_req);
            let first_line = req_str.lines().next().unwrap_or("");
            let target = first_line.split_whitespace().nth(1).unwrap_or("");
            let (target_host, target_port) = match target.split_once(':') {
                Some((h, p)) => (h, p.parse::<u16>().unwrap_or(443)),
                None => (target, 443),
            };

            // Reply with 200 Connection Established
            stream
                .write_all(b"HTTP/1.1 200 Connection Established\r\nProxy-Agent: RustProxy/0.1.0\r\n\r\n")
                .await?;

            // Upgrade to TLS stream
            let tls_stream = tls_acceptor.accept(stream).await?;
            let io = TokioIo::new(tls_stream);

            Self::serve_http(
                io,
                "https",
                target_host.to_string(),
                target_port,
                client_addr,
                storage,
                interceptor,
                sitemap,
                scanner,
                http_client,
                event_tx,
            )
            .await?;
        } else {
            // Plain HTTP proxying
            let io = TokioIo::new(stream);
            Self::serve_http(
                io,
                "http",
                "".to_string(),
                80,
                client_addr,
                storage,
                interceptor,
                sitemap,
                scanner,
                http_client,
                event_tx,
            )
            .await?;
        }

        Ok(())
    }

    async fn serve_http<I>(
        io: TokioIo<I>,
        scheme: &'static str,
        default_host: String,
        default_port: u16,
        client_addr: SocketAddr,
        storage: Arc<StorageEngine>,
        interceptor: Arc<InterceptManager>,
        sitemap: Arc<RwLock<SiteMap>>,
        scanner: Arc<rustproxy_scanner::ScannerEngine>,
        http_client: Arc<HttpClient>,
        event_tx: broadcast::Sender<ProxyEvent>,
    ) -> Result<()>
    where
        I: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    {
        let service = service_fn(move |hyper_req: hyper::Request<Incoming>| {
            let storage = storage.clone();
            let interceptor = interceptor.clone();
            let sitemap = sitemap.clone();
            let scanner = scanner.clone();
            let http_client = http_client.clone();
            let event_tx = event_tx.clone();
            let default_host = default_host.clone();

            async move {
                Self::process_request(
                    hyper_req,
                    scheme,
                    &default_host,
                    default_port,
                    client_addr,
                    storage,
                    interceptor,
                    sitemap,
                    scanner,
                    http_client,
                    event_tx,
                )
                .await
            }
        });

        let conn = http1::Builder::new().serve_connection(io, service);
        if let Err(e) = conn.await {
            debug!("HTTP connection finished with info: {:?}", e);
        }

        Ok(())
    }

    async fn process_request(
        req: hyper::Request<Incoming>,
        scheme: &str,
        default_host: &str,
        default_port: u16,
        client_addr: SocketAddr,
        storage: Arc<StorageEngine>,
        interceptor: Arc<InterceptManager>,
        sitemap: Arc<RwLock<SiteMap>>,
        scanner: Arc<rustproxy_scanner::ScannerEngine>,
        http_client: Arc<HttpClient>,
        event_tx: broadcast::Sender<ProxyEvent>,
    ) -> Result<hyper::Response<Full<Bytes>>, hyper::Error> {
        let flow_id = Uuid::new_v4().to_string();
        let method = req.method().to_string();
        let uri = req.uri().to_string();
        let version = format!("{:?}", req.version());

        let mut headers = Vec::new();
        for (k, v) in req.headers() {
            if let Ok(v_str) = v.to_str() {
                headers.push((k.as_str().to_string(), v_str.to_string()));
            }
        }

        let body_bytes = match req.into_body().collect().await {
            Ok(c) => c.to_bytes(),
            Err(_) => Bytes::new(),
        };

        let mut req_model = RequestModel::new(
            &flow_id,
            method,
            uri,
            version,
            headers,
            body_bytes,
        );

        if !default_host.is_empty() {
            req_model.host = default_host.to_string();
            req_model.port = default_port;
            req_model.scheme = scheme.to_string();
        }

        // Scope check
        let in_scope = {
            let mut sm = sitemap.write().await;
            sm.is_in_scope(&req_model.scheme, &req_model.host, req_model.port, &req_model.path)
        };

        // Intercept check
        let final_req = if interceptor.should_intercept_request(&req_model, in_scope).await {
            let _ = event_tx.send(ProxyEvent::RequestIntercepted(req_model.clone()));
            let rx = interceptor.queue_request(req_model.clone()).await;
            match rx.await {
                Ok(InterceptDecision::ForwardModifiedRequest(modified)) => modified,
                Ok(InterceptDecision::ForwardOriginal) => req_model,
                _ => {
                    // Drop request
                    let mut res = hyper::Response::new(Full::new(Bytes::from("Request Dropped by RustProxy")));
                    *res.status_mut() = hyper::StatusCode::BAD_GATEWAY;
                    return Ok(res);
                }
            }
        } else {
            req_model
        };

        // Forward upstream
        let mut flow_record = FlowRecord::new(&flow_id, final_req.clone());
        flow_record.client_addr = Some(client_addr.to_string());
        flow_record.in_scope = in_scope;

        let upstream_res = http_client.send(&final_req).await;

        match upstream_res {
            Ok(response_model) => {
                flow_record.response = Some(response_model.clone());

                // Passive Scanning
                let findings = scanner.scan_flow(&flow_record);
                for finding in &findings {
                    let _ = storage.save_finding(finding);
                    let _ = event_tx.send(ProxyEvent::FindingDetected(finding.clone()));
                }

                // Add to SiteMap
                {
                    let mut sm = sitemap.write().await;
                    sm.add_flow(&flow_record);
                }

                // Save to Storage
                let _ = storage.save_flow(&flow_record);
                let _ = event_tx.send(ProxyEvent::FlowRecorded(flow_record.summary()));

                // Build client response
                let mut builder = hyper::Response::builder()
                    .status(response_model.status);

                for (k, v) in &response_model.headers {
                    if let (Ok(hname), Ok(hval)) = (HeaderName::from_str(k), HeaderValue::from_str(v)) {
                        // Skip hop-by-hop headers
                        if !k.eq_ignore_ascii_case("transfer-encoding") {
                            builder = builder.header(hname, hval);
                        }
                    }
                }

                let client_res = builder
                    .body(Full::new(response_model.body))
                    .unwrap_or_else(|_| hyper::Response::new(Full::new(Bytes::new())));

                Ok(client_res)
            }
            Err(e) => {
                flow_record.error = Some(format!("Upstream error: {:?}", e));
                let _ = storage.save_flow(&flow_record);
                let _ = event_tx.send(ProxyEvent::FlowRecorded(flow_record.summary()));

                let mut err_res = hyper::Response::new(Full::new(Bytes::from(format!(
                    "RustProxy: Upstream Connection Error: {:?}",
                    e
                ))));
                *err_res.status_mut() = hyper::StatusCode::BAD_GATEWAY;
                Ok(err_res)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proxy_server_config() {
        let config = ProxyConfig::default();
        assert_eq!(config.bind_addr.port(), 8080);
    }
}
