use anyhow::Result;
use rustproxy_intercept::InterceptManager;
use rustproxy_proxy::{ProxyConfig, ProxyEvent, ProxyServer};
use rustproxy_repeater::RepeaterEngine;
use rustproxy_sitemap::SiteMap;
use rustproxy_storage::StorageEngine;
use rustproxy_tls::CertificateAuthority;
use std::fs;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Install default crypto provider for Rustls 0.23
    let _ = rustls::crypto::ring::default_provider().install_default();

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    let _ = tracing::subscriber::set_global_default(subscriber);

    println!(r#"
  ____           _   ____                    
 |  _ \ _   _ ___| |_|  _ \ _ __ _____  _   _ 
 | |_) | | | / __| __| |_) | '__/ _ \ \/ / | | |
 |  _ <| |_| \__ \ |_|  __/| | | (_) >  <| |_| |
 |_| \_\\__,_|___/\__|_|   |_|  \___/_/\_\\__, |
                                          |___/ 
  RustProxy v0.1.0 - Defensive Traffic Analysis Platform
  Built with Rust, Tokio, Rustls, and Hyper
"#);

    let home_dir = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let ca_dir = PathBuf::from(home_dir).join(".rustproxy");
    fs::create_dir_all(&ca_dir)?;

    let ca_cert_path = ca_dir.join("rustproxy_ca.crt");
    let ca_key_path = ca_dir.join("rustproxy_ca.key");

    let ca = Arc::new(CertificateAuthority::generate()?);
    fs::write(&ca_cert_path, &ca.ca_cert_pem)?;
    fs::write(&ca_key_path, &ca.ca_key_pem)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&ca_dir, fs::Permissions::from_mode(0o700));
        let _ = fs::set_permissions(&ca_key_path, fs::Permissions::from_mode(0o600));
    }

    println!("  [+] Generated Root CA Certificate: {}", ca_cert_path.display());
    println!("  [!] Install this certificate in your browser/system to inspect HTTPS traffic without warnings.");
    println!();

    let db_path = "rustproxy.db";
    let storage = Arc::new(StorageEngine::open(db_path)?);
    println!("  [+] SQLite Flow Database: {}", db_path);

    let interceptor = Arc::new(InterceptManager::new());
    let sitemap = Arc::new(RwLock::new(SiteMap::new()));
    let scanner = Arc::new(rustproxy_scanner::ScannerEngine::new());

    let client = Arc::new(rustproxy_http::HttpClient::new_insecure()?);
    let repeater = Arc::new(RwLock::new(RepeaterEngine::new(client)));

    let bind_addr: SocketAddr = "127.0.0.1:8080".parse()?;
    let config = ProxyConfig {
        bind_addr,
        upstream_insecure: true,
    };

    let proxy = ProxyServer::new(
        config,
        ca,
        storage.clone(),
        interceptor.clone(),
        sitemap.clone(),
        scanner.clone(),
    )?;

    let api_service = Arc::new(rustproxy_api::ApiService::new(
        storage.clone(),
        interceptor.clone(),
        repeater.clone(),
        sitemap.clone(),
        scanner.clone(),
        proxy.event_sender(),
    ));

    let api_addr: SocketAddr = "127.0.0.1:8081".parse()?;
    let api_runner = api_service.clone();
    tokio::spawn(async move {
        if let Err(e) = rustproxy_api::start_api_server(api_runner, api_addr).await {
            tracing::error!("API server error: {:?}", e);
        }
    });

    let mut event_rx = proxy.subscribe_events();

    // Spawn event logger task
    tokio::spawn(async move {
        while let Ok(event) = event_rx.recv().await {
            match event {
                ProxyEvent::FlowRecorded(flow) => {
                    let status = flow.status.map(|s| s.to_string()).unwrap_or_else(|| "ERR".to_string());
                    let duration = flow.duration_ms.map(|d| format!("{}ms", d)).unwrap_or_else(|| "-".to_string());
                    println!(
                        "  [FLOW] {:<6} {:<4} {:<30} {:<30} [{}] ({} bytes)",
                        flow.method,
                        status,
                        flow.host,
                        flow.path,
                        duration,
                        flow.response_len.unwrap_or(flow.request_len),
                    );
                }
                ProxyEvent::FindingDetected(finding) => {
                    println!(
                        "  [SCAN] [{:?}] {} (CWE-{:?}) -> {}",
                        finding.severity,
                        finding.title,
                        finding.cwe_id.unwrap_or(0),
                        finding.url,
                    );
                }
                ProxyEvent::RequestIntercepted(req) => {
                    println!(
                        "  [PAUSE] Intercepted Request {} {} -> Awaiting operator decision",
                        req.method, req.uri
                    );
                }
                ProxyEvent::ResponseIntercepted(res) => {
                    println!(
                        "  [PAUSE] Intercepted Response status {} ({} bytes) -> Awaiting operator decision",
                        res.status, res.body.len()
                    );
                }
            }
        }
    });

    println!("  [*] Proxy listening on http://{}", bind_addr);
    println!("  [*] Desktop/Web API listening on http://{}", api_addr);
    println!("  [*] Configure client proxy settings to http://{}", bind_addr);
    println!("  [*] Press Ctrl+C to stop.");
    println!();

    proxy.run().await?;

    Ok(())
}
