use anyhow::Result;
use rustproxy_intercept::InterceptManager;
use rustproxy_proxy::{ProxyConfig, ProxyServer};
use rustproxy_sitemap::SiteMap;
use rustproxy_storage::StorageEngine;
use rustproxy_tls::CertificateAuthority;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;

async fn spawn_test_proxy() -> Result<(SocketAddr, Arc<StorageEngine>)> {
    let port_listener = TcpListener::bind("127.0.0.1:0").await?;
    let proxy_addr: SocketAddr = port_listener.local_addr()?;
    drop(port_listener);

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

    tokio::spawn(async move {
        let _ = proxy.run().await;
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    Ok((proxy_addr, storage))
}

#[tokio::test]
async fn test_protocol_keep_alive() -> Result<()> {
    // Upstream server responding to requests based on URI path
    let mock_listener = TcpListener::bind("127.0.0.1:0").await?;
    let mock_addr = mock_listener.local_addr()?;

    tokio::spawn(async move {
        while let Ok((mut socket, _)) = mock_listener.accept().await {
            tokio::spawn(async move {
                let mut buf = [0u8; 4096];
                while let Ok(n) = socket.read(&mut buf).await {
                    if n == 0 {
                        break;
                    }
                    let req_str = String::from_utf8_lossy(&buf[..n]);
                    let path = req_str
                        .lines()
                        .next()
                        .and_then(|l| l.split_whitespace().nth(1))
                        .unwrap_or("/")
                        .to_string();

                    let body = format!("response-for-{}", path);
                    let res = format!(
                        "HTTP/1.1 200 OK\r\n\
                         Content-Length: {}\r\n\
                         Connection: keep-alive\r\n\
                         \r\n\
                         {}",
                        body.len(),
                        body
                    );
                    if socket.write_all(res.as_bytes()).await.is_err() {
                        break;
                    }
                }
            });
        }
    });

    let (proxy_addr, _) = spawn_test_proxy().await?;

    // Client connects once and sends 3 requests sequentially over keep-alive
    let mut client = TcpStream::connect(proxy_addr).await?;

    for i in 1..=3 {
        let conn_hdr = if i == 3 { "close" } else { "keep-alive" };
        let req = format!(
            "GET http://127.0.0.1:{}/seq/{} HTTP/1.1\r\n\
             Host: 127.0.0.1:{}\r\n\
             Connection: {}\r\n\
             \r\n",
            mock_addr.port(),
            i,
            mock_addr.port(),
            conn_hdr
        );
        client.write_all(req.as_bytes()).await?;

        let mut buf = [0u8; 1024];
        let n = client.read(&mut buf).await?;
        let res_str = String::from_utf8_lossy(&buf[..n]);
        assert!(res_str.contains("HTTP/1.1 200 OK"));
        assert!(res_str.contains(&format!("response-for-/seq/{}", i)));
    }

    Ok(())
}

#[tokio::test]
async fn test_protocol_chunked_transfer_encoding() -> Result<()> {
    // Upstream sends chunked response
    let mock_listener = TcpListener::bind("127.0.0.1:0").await?;
    let mock_addr = mock_listener.local_addr()?;

    tokio::spawn(async move {
        while let Ok((mut socket, _)) = mock_listener.accept().await {
            tokio::spawn(async move {
                let mut buf = [0u8; 1024];
                let _ = socket.read(&mut buf).await;
                let chunked_res = "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\n5\r\nhello\r\n6\r\n world\r\n0\r\n\r\n";
                let _ = socket.write_all(chunked_res.as_bytes()).await;
            });
        }
    });

    let (proxy_addr, storage) = spawn_test_proxy().await?;

    let mut client = TcpStream::connect(proxy_addr).await?;
    let req = format!(
        "GET http://127.0.0.1:{}/chunked HTTP/1.1\r\n\
         Host: 127.0.0.1:{}\r\n\
         Connection: close\r\n\
         \r\n",
        mock_addr.port(),
        mock_addr.port()
    );
    client.write_all(req.as_bytes()).await?;

    let mut res_bytes = Vec::new();
    client.read_to_end(&mut res_bytes).await?;
    let res_str = String::from_utf8_lossy(&res_bytes);

    assert!(res_str.contains("HTTP/1.1 200 OK"));
    assert!(res_str.contains("hello world"));

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    let flows = storage.list_flows(&rustproxy_storage::FlowFilter::all())?;
    assert_eq!(flows.len(), 1);
    assert_eq!(flows[0].response_len, Some(11));

    Ok(())
}

#[tokio::test]
async fn test_protocol_large_payload_5mb() -> Result<()> {
    // 5MB payload
    let payload_size = 5 * 1024 * 1024;
    let mock_listener = TcpListener::bind("127.0.0.1:0").await?;
    let mock_addr = mock_listener.local_addr()?;

    tokio::spawn(async move {
        while let Ok((mut socket, _)) = mock_listener.accept().await {
            tokio::spawn(async move {
                let mut header_buf = Vec::new();
                let mut byte = [0u8; 1];
                while let Ok(1) = socket.read(&mut byte).await {
                    header_buf.push(byte[0]);
                    if header_buf.ends_with(b"\r\n\r\n") {
                        break;
                    }
                }

                // Send 5MB response
                let header = format!(
                    "HTTP/1.1 200 OK\r\n\
                     Content-Length: {}\r\n\
                     Content-Type: application/octet-stream\r\n\
                     Connection: close\r\n\
                     \r\n",
                    payload_size
                );
                let _ = socket.write_all(header.as_bytes()).await;

                let chunk = vec![0x42u8; 64 * 1024]; // 64KB blocks of 'B'
                let mut sent = 0;
                while sent < payload_size {
                    let to_send = std::cmp::min(chunk.len(), payload_size - sent);
                    if socket.write_all(&chunk[..to_send]).await.is_err() {
                        break;
                    }
                    sent += to_send;
                }
            });
        }
    });

    let (proxy_addr, _) = spawn_test_proxy().await?;

    let mut client = TcpStream::connect(proxy_addr).await?;
    let req = format!(
        "GET http://127.0.0.1:{}/large HTTP/1.1\r\n\
         Host: 127.0.0.1:{}\r\n\
         Connection: close\r\n\
         \r\n",
        mock_addr.port(),
        mock_addr.port()
    );
    client.write_all(req.as_bytes()).await?;

    let mut res_bytes = Vec::new();
    client.read_to_end(&mut res_bytes).await?;

    assert!(res_bytes.starts_with(b"HTTP/1.1 200 OK"));
    // Header + 5MB body
    assert!(res_bytes.len() >= payload_size);

    Ok(())
}

#[tokio::test]
async fn test_protocol_binary_payload_transparency() -> Result<()> {
    // Generate all byte values from 0x00 through 0xFF
    let binary_body: Vec<u8> = (0u8..=255u8).cycle().take(1024).collect();
    let expected_body = binary_body.clone();

    let mock_listener = TcpListener::bind("127.0.0.1:0").await?;
    let mock_addr = mock_listener.local_addr()?;

    tokio::spawn(async move {
        while let Ok((mut socket, _)) = mock_listener.accept().await {
            let body = binary_body.clone();
            tokio::spawn(async move {
                let mut buf = [0u8; 1024];
                let _ = socket.read(&mut buf).await;
                let header = format!(
                    "HTTP/1.1 200 OK\r\n\
                     Content-Length: {}\r\n\
                     Content-Type: application/octet-stream\r\n\
                     Connection: close\r\n\
                     \r\n",
                    body.len()
                );
                let _ = socket.write_all(header.as_bytes()).await;
                let _ = socket.write_all(&body).await;
            });
        }
    });

    let (proxy_addr, storage) = spawn_test_proxy().await?;

    let mut client = TcpStream::connect(proxy_addr).await?;
    let req = format!(
        "GET http://127.0.0.1:{}/binary HTTP/1.1\r\n\
         Host: 127.0.0.1:{}\r\n\
         Connection: close\r\n\
         \r\n",
        mock_addr.port(),
        mock_addr.port()
    );
    client.write_all(req.as_bytes()).await?;

    let mut res_bytes = Vec::new();
    client.read_to_end(&mut res_bytes).await?;

    // Find \r\n\r\n boundary separating headers from body
    let body_offset = res_bytes
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .expect("Headers delimiter not found")
        + 4;

    let received_body = &res_bytes[body_offset..];
    assert_eq!(received_body, &expected_body[..]);

    // Check storage fidelity
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    let flows = storage.list_flows(&rustproxy_storage::FlowFilter::all())?;
    assert_eq!(flows.len(), 1);
    let detail = storage.get_flow(&flows[0].id)?.expect("Flow should exist");
    let stored_response_body = detail.response.expect("Response should exist").body;
    assert_eq!(stored_response_body.as_ref(), &expected_body[..]);

    Ok(())
}

#[tokio::test]
async fn test_protocol_cookies_and_redirects() -> Result<()> {
    let mock_listener = TcpListener::bind("127.0.0.1:0").await?;
    let mock_addr = mock_listener.local_addr()?;

    tokio::spawn(async move {
        while let Ok((mut socket, _)) = mock_listener.accept().await {
            tokio::spawn(async move {
                let mut buf = [0u8; 2048];
                let _ = socket.read(&mut buf).await;
                let res = "HTTP/1.1 302 Found\r\n\
                           Location: /login\r\n\
                           Set-Cookie: session_id=secret123; HttpOnly; Secure; SameSite=Strict\r\n\
                           Content-Length: 0\r\n\
                           Connection: close\r\n\
                           \r\n";
                let _ = socket.write_all(res.as_bytes()).await;
            });
        }
    });

    let (proxy_addr, _) = spawn_test_proxy().await?;

    let mut client = TcpStream::connect(proxy_addr).await?;
    let req = format!(
        "GET http://127.0.0.1:{}/auth HTTP/1.1\r\n\
         Host: 127.0.0.1:{}\r\n\
         Cookie: tracking_token=xyz987\r\n\
         Connection: close\r\n\
         \r\n",
        mock_addr.port(),
        mock_addr.port()
    );
    client.write_all(req.as_bytes()).await?;

    let mut res_bytes = Vec::new();
    client.read_to_end(&mut res_bytes).await?;
    let res_str = String::from_utf8_lossy(&res_bytes);
    let res_str_lower = res_str.to_lowercase();

    assert!(res_str.contains("HTTP/1.1 302 Found"));
    assert!(res_str_lower.contains("location: /login"));
    assert!(res_str_lower.contains("set-cookie: session_id=secret123"));

    Ok(())
}
