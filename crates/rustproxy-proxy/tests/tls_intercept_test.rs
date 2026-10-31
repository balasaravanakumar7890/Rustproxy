use anyhow::Result;
use rustproxy_intercept::InterceptManager;
use rustproxy_proxy::{ProxyConfig, ProxyServer};
use rustproxy_sitemap::SiteMap;
use rustproxy_storage::{FlowFilter, StorageEngine};
use rustproxy_tls::{create_mitm_server_config, CertificateAuthority};
use rustls::pki_types::ServerName;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;
use tokio_rustls::{TlsAcceptor, TlsConnector};

#[tokio::test]
async fn test_tls_mitm_interception_tunnel() -> Result<()> {
    let _ = rustls::crypto::ring::default_provider().install_default();

    // 1. Generate Root CA for proxy
    let ca = Arc::new(CertificateAuthority::generate()?);

    // 2. Start mock upstream HTTPS server
    let mock_upstream_ca = Arc::new(CertificateAuthority::generate()?);
    let upstream_server_config = create_mitm_server_config(mock_upstream_ca)?;
    let upstream_acceptor = TlsAcceptor::from(upstream_server_config);

    let upstream_listener = TcpListener::bind("127.0.0.1:0").await?;
    let upstream_addr = upstream_listener.local_addr()?;

    tokio::spawn(async move {
        while let Ok((socket, _)) = upstream_listener.accept().await {
            let acceptor = upstream_acceptor.clone();
            tokio::spawn(async move {
                if let Ok(mut tls_stream) = acceptor.accept(socket).await {
                    let mut buf = [0u8; 1024];
                    let _ = tls_stream.read(&mut buf).await;
                    let body = "{\"secure_data\": true}";
                    let response = format!(
                        "HTTP/1.1 200 OK\r\n\
                         Content-Type: application/json\r\n\
                         Content-Length: {}\r\n\
                         Connection: close\r\n\
                         \r\n\
                         {}",
                        body.len(),
                        body
                    );
                    let _ = tls_stream.write_all(response.as_bytes()).await;
                }
            });
        }
    });

    // 3. Start Proxy Server
    let proxy_port_listener = TcpListener::bind("127.0.0.1:0").await?;
    let proxy_addr: SocketAddr = proxy_port_listener.local_addr()?;
    drop(proxy_port_listener);

    let storage = Arc::new(StorageEngine::open_in_memory()?);
    let interceptor = Arc::new(InterceptManager::new());
    let sitemap = Arc::new(RwLock::new(SiteMap::new()));
    let scanner = Arc::new(rustproxy_scanner::ScannerEngine::new());

    let config = ProxyConfig {
        bind_addr: proxy_addr,
        upstream_insecure: true,
    };

    let proxy = Arc::new(ProxyServer::new(
        config,
        ca.clone(),
        storage.clone(),
        interceptor.clone(),
        sitemap.clone(),
        scanner.clone(),
    )?);

    let proxy_runner = proxy.clone();
    tokio::spawn(async move {
        let _ = proxy_runner.run().await;
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // 4. Client connects to Proxy via TCP and initiates CONNECT tunnel
    let mut proxy_conn = TcpStream::connect(proxy_addr).await?;

    let connect_req = format!(
        "CONNECT 127.0.0.1:{} HTTP/1.1\r\n\
         Host: 127.0.0.1:{}\r\n\
         \r\n",
        upstream_addr.port(),
        upstream_addr.port(),
    );
    proxy_conn.write_all(connect_req.as_bytes()).await?;

    let mut connect_res_buf = [0u8; 256];
    let n = proxy_conn.read(&mut connect_res_buf).await?;
    let connect_res = String::from_utf8_lossy(&connect_res_buf[..n]);
    assert!(connect_res.contains("200 Connection Established"));

    // 5. Build TLS Client trusting Proxy's Root CA
    let mut root_store = rustls::RootCertStore::empty();
    let ca_cert_der = rustls::pki_types::CertificateDer::from(ca.ca_cert_der.clone());
    root_store.add(ca_cert_der)?;

    let client_config = rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    let connector = TlsConnector::from(Arc::new(client_config));
    let server_name = ServerName::try_from("127.0.0.1".to_string())?;

    // Perform TLS handshake through the tunnel
    let mut tls_client_stream = connector.connect(server_name, proxy_conn).await?;

    // 6. Send decrypted HTTP GET inside TLS tunnel
    let https_get = format!(
        "GET /secure-data HTTP/1.1\r\n\
         Host: 127.0.0.1:{}\r\n\
         Connection: close\r\n\
         \r\n",
        upstream_addr.port(),
    );
    tls_client_stream.write_all(https_get.as_bytes()).await?;

    let mut secure_res_buf = Vec::new();
    tls_client_stream.read_to_end(&mut secure_res_buf).await?;
    let secure_res = String::from_utf8_lossy(&secure_res_buf);

    assert!(secure_res.contains("HTTP/1.1 200 OK"));
    assert!(secure_res.contains("{\"secure_data\": true}"));

    // 7. Verify flow is logged in SQLite
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    let flows = storage.list_flows(&FlowFilter::all())?;
    assert_eq!(flows.len(), 1);
    assert_eq!(flows[0].path, "/secure-data");
    assert_eq!(flows[0].status, Some(200));

    Ok(())
}
