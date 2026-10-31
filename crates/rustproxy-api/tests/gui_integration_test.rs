use anyhow::Result;
use rustproxy_api::{start_api_server, ApiService};
use rustproxy_intercept::InterceptManager;
use rustproxy_repeater::RepeaterEngine;
use rustproxy_sitemap::SiteMap;
use rustproxy_storage::StorageEngine;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, RwLock};

#[tokio::test]
async fn test_gui_api_and_static_serving() -> Result<()> {
    let storage = Arc::new(StorageEngine::open_in_memory()?);
    let interceptor = Arc::new(InterceptManager::new());
    let client = Arc::new(rustproxy_http::HttpClient::new_insecure()?);
    let repeater = Arc::new(RwLock::new(RepeaterEngine::new(client)));
    let sitemap = Arc::new(RwLock::new(SiteMap::new()));
    let scanner = Arc::new(rustproxy_scanner::ScannerEngine::new());
    let (event_tx, _) = broadcast::channel(100);

    let api_service = Arc::new(ApiService::new(
        storage.clone(),
        interceptor.clone(),
        repeater.clone(),
        sitemap.clone(),
        scanner.clone(),
        event_tx,
    ));

    // Bind on ephemeral port
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let api_addr: SocketAddr = listener.local_addr()?;
    drop(listener);

    let runner = api_service.clone();
    tokio::spawn(async move {
        let _ = start_api_server(runner, api_addr).await;
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // 1. Test Static Index.html Serving (GUI Frontend)
    let mut stream = TcpStream::connect(api_addr).await?;
    stream
        .write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
        .await?;

    let mut response_buf = Vec::new();
    stream.read_to_end(&mut response_buf).await?;
    let response_str = String::from_utf8_lossy(&response_buf);

    assert!(response_str.contains("HTTP/1.1 200 OK"));
    assert!(response_str.contains("text/html"));
    assert!(response_str.contains("<div id=\"root\""));

    // 2. Test /api/stats
    let mut stream = TcpStream::connect(api_addr).await?;
    stream
        .write_all(b"GET /api/stats HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
        .await?;

    let mut stats_buf = Vec::new();
    stream.read_to_end(&mut stats_buf).await?;
    let stats_str = String::from_utf8_lossy(&stats_buf);
    assert!(stats_str.contains("HTTP/1.1 200 OK"));
    assert!(stats_str.contains("\"total_flows\":0"));

    // 3. Test /api/intercept/toggle
    let mut stream = TcpStream::connect(api_addr).await?;
    let post_body = "{\"enabled\":true}";
    let post_req = format!(
        "POST /api/intercept/toggle HTTP/1.1\r\n\
         Host: localhost\r\n\
         Content-Type: application/json\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\
         \r\n\
         {}",
        post_body.len(),
        post_body
    );
    stream.write_all(post_req.as_bytes()).await?;

    let mut toggle_buf = Vec::new();
    stream.read_to_end(&mut toggle_buf).await?;
    let toggle_str = String::from_utf8_lossy(&toggle_buf);
    assert!(toggle_str.contains("HTTP/1.1 200 OK"));
    assert!(toggle_str.contains("\"enabled\":true"));
    assert!(interceptor.is_enabled());

    // 4. Test /api/decoder/encode
    let mut stream = TcpStream::connect(api_addr).await?;
    let codec_body = "{\"text\":\"admin:pass\",\"format\":\"base64\"}";
    let codec_req = format!(
        "POST /api/decoder/encode HTTP/1.1\r\n\
         Host: localhost\r\n\
         Content-Type: application/json\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\
         \r\n\
         {}",
        codec_body.len(),
        codec_body
    );
    stream.write_all(codec_req.as_bytes()).await?;

    let mut codec_buf = Vec::new();
    stream.read_to_end(&mut codec_buf).await?;
    let codec_str = String::from_utf8_lossy(&codec_buf);
    assert!(codec_str.contains("HTTP/1.1 200 OK"));
    assert!(codec_str.contains("YWRtaW46cGFzcw=="));

    Ok(())
}
