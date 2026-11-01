use anyhow::Result;
use rustproxy_intercept::InterceptManager;
use rustproxy_proxy::{ProxyConfig, ProxyServer};
use rustproxy_sitemap::SiteMap;
use rustproxy_storage::{FlowFilter, StorageEngine};
use rustproxy_tls::CertificateAuthority;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;

#[tokio::test]
async fn test_end_to_end_proxy_pipeline() -> Result<()> {
    // 1. Start a local mock upstream server
    let mock_listener = TcpListener::bind("127.0.0.1:0").await?;
    let mock_addr = mock_listener.local_addr()?;

    tokio::spawn(async move {
        while let Ok((mut socket, _)) = mock_listener.accept().await {
            tokio::spawn(async move {
                let mut buf = [0u8; 1024];
                let _ = socket.read(&mut buf).await;
                let body = "{\"status\":\"ok\"}";
                let response = format!(
                    "HTTP/1.1 200 OK\r\n\
                     Content-Type: application/json\r\n\
                     Server: VulnerableServer/1.2.3\r\n\
                     Content-Length: {}\r\n\
                     Connection: close\r\n\
                     \r\n\
                     {}",
                    body.len(),
                    body
                );
                let _ = socket.write_all(response.as_bytes()).await;
            });
        }
    });

    // 2. Start the Proxy Server on an ephemeral port
    let proxy_port_listener = TcpListener::bind("127.0.0.1:0").await?;
    let proxy_addr: SocketAddr = proxy_port_listener.local_addr()?;
    drop(proxy_port_listener); // Release port for proxy server to bind

    let ca = Arc::new(CertificateAuthority::generate()?);
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
        ca,
        storage.clone(),
        interceptor.clone(),
        sitemap.clone(),
        scanner.clone(),
    )?);

    let proxy_runner = proxy.clone();
    tokio::spawn(async move {
        let _ = proxy_runner.run().await;
    });

    // Wait 50ms for proxy server to bind and start listening
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // 3. Connect client TCP socket to Proxy and issue an HTTP request
    let mut client_stream = TcpStream::connect(proxy_addr).await?;

    let http_request = format!(
        "GET http://127.0.0.1:{}/api/test HTTP/1.1\r\n\
         Host: 127.0.0.1:{}\r\n\
         User-Agent: IntegrationTest/1.0\r\n\
         Connection: close\r\n\
         \r\n",
        mock_addr.port(),
        mock_addr.port(),
    );

    client_stream.write_all(http_request.as_bytes()).await?;

    let mut response_bytes = Vec::new();
    client_stream.read_to_end(&mut response_bytes).await?;
    let response_str = String::from_utf8_lossy(&response_bytes);
    println!("DEBUG RESPONSE RECEIVED:\n{}", response_str);

    // 4. Assert client received response from mock server through proxy
    assert!(response_str.contains("HTTP/1.1 200 OK"));
    assert!(response_str.contains("{\"status\":\"ok\"}"));

    // Wait a brief moment for async storage and scanner logging
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // 5. Assert Flow was recorded into SQLite StorageEngine
    assert_eq!(storage.count_flows()?, 1);
    let flows = storage.list_flows(&FlowFilter::all())?;
    assert_eq!(flows.len(), 1);
    assert_eq!(flows[0].method, "GET");
    assert_eq!(flows[0].path, "/api/test");
    assert_eq!(flows[0].status, Some(200));

    // 6. Assert SiteMap registered the endpoint
    let hosts = sitemap.read().await.get_hosts();
    assert_eq!(hosts.len(), 1);
    let api_seg = hosts[0]
        .children
        .get("api")
        .expect("Missing /api segment in sitemap");
    let test_seg = api_seg
        .children
        .get("test")
        .expect("Missing /test segment in sitemap");
    assert!(test_seg.is_endpoint);

    // 7. Assert Passive Scanner detected findings on the response
    let findings = storage.list_findings()?;
    assert!(!findings.is_empty());
    let titles: Vec<&str> = findings.iter().map(|f| f.title.as_str()).collect();
    assert!(titles.contains(&"Server Header Version Disclosure"));

    Ok(())
}
