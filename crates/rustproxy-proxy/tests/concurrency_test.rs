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

async fn spawn_concurrency_proxy() -> Result<(SocketAddr, Arc<StorageEngine>, Arc<InterceptManager>)> {
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
    Ok((proxy_addr, storage, interceptor))
}

#[tokio::test]
async fn test_high_concurrency_throughput() -> Result<()> {
    // Upstream server responding to concurrent connections
    let mock_listener = TcpListener::bind("127.0.0.1:0").await?;
    let mock_addr = mock_listener.local_addr()?;

    tokio::spawn(async move {
        while let Ok((mut socket, _)) = mock_listener.accept().await {
            tokio::spawn(async move {
                let mut buf = [0u8; 1024];
                let _ = socket.read(&mut buf).await;
                let body = "{\"status\":\"concurrent_ok\"}";
                let res = format!(
                    "HTTP/1.1 200 OK\r\n\
                     Content-Length: {}\r\n\
                     Content-Type: application/json\r\n\
                     Connection: close\r\n\
                     \r\n\
                     {}",
                    body.len(),
                    body
                );
                let _ = socket.write_all(res.as_bytes()).await;
            });
        }
    });

    let (proxy_addr, storage, _) = spawn_concurrency_proxy().await?;

    // Fire 50 concurrent requests simultaneously using tokio tasks
    const CONCURRENT_CLIENTS: usize = 50;
    let mut handles = Vec::new();

    for client_id in 0..CONCURRENT_CLIENTS {
        let p_addr = proxy_addr;
        let m_port = mock_addr.port();
        let handle = tokio::spawn(async move {
            let mut client = TcpStream::connect(p_addr).await?;
            let req = format!(
                "GET http://127.0.0.1:{}/item/{} HTTP/1.1\r\n\
                 Host: 127.0.0.1:{}\r\n\
                 X-Client-Id: client-{}\r\n\
                 Connection: close\r\n\
                 \r\n",
                m_port, client_id, m_port, client_id
            );
            client.write_all(req.as_bytes()).await?;

            let mut res_bytes = Vec::new();
            client.read_to_end(&mut res_bytes).await?;
            let res_str = String::from_utf8_lossy(&res_bytes);
            assert!(res_str.contains("HTTP/1.1 200 OK"), "Failed on client {}", client_id);
            assert!(res_str.contains("concurrent_ok"));
            Ok::<(), anyhow::Error>(())
        });
        handles.push(handle);
    }

    // Await all 50 concurrent requests
    for (i, h) in handles.into_iter().enumerate() {
        let res = h.await?;
        assert!(res.is_ok(), "Task {} encountered error: {:?}", i, res);
    }

    // Allow background write tasks to settle
    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;

    // Verify all 50 flows were recorded cleanly in SQLite without deadlocks
    let flows = storage.list_flows(&rustproxy_storage::FlowFilter::all())?;
    assert_eq!(flows.len(), CONCURRENT_CLIENTS);

    Ok(())
}

#[tokio::test]
async fn test_interception_concurrency_queue() -> Result<()> {
    let mock_listener = TcpListener::bind("127.0.0.1:0").await?;
    let mock_addr = mock_listener.local_addr()?;

    tokio::spawn(async move {
        while let Ok((mut socket, _)) = mock_listener.accept().await {
            tokio::spawn(async move {
                let mut buf = [0u8; 1024];
                let _ = socket.read(&mut buf).await;
                let body = "upstream_ack";
                let res = format!(
                    "HTTP/1.1 200 OK\r\n\
                     Content-Length: {}\r\n\
                     Connection: close\r\n\
                     \r\n\
                     {}",
                    body.len(),
                    body
                );
                let _ = socket.write_all(res.as_bytes()).await;
            });
        }
    });

    let (proxy_addr, _, interceptor) = spawn_concurrency_proxy().await?;

    // Enable interceptor
    interceptor.set_enabled(true);

    const CONCURRENT_INTERCEPTS: usize = 5;
    let mut client_handles = Vec::new();

    for req_i in 0..CONCURRENT_INTERCEPTS {
        let p_addr = proxy_addr;
        let m_port = mock_addr.port();
        let h = tokio::spawn(async move {
            let mut client = TcpStream::connect(p_addr).await?;
            let req = format!(
                "GET http://127.0.0.1:{}/intercept/{} HTTP/1.1\r\n\
                 Host: 127.0.0.1:{}\r\n\
                 Connection: close\r\n\
                 \r\n",
                m_port, req_i, m_port
            );
            client.write_all(req.as_bytes()).await?;

            let mut res_bytes = Vec::new();
            client.read_to_end(&mut res_bytes).await?;
            let res_str = String::from_utf8_lossy(&res_bytes);
            Ok::<String, anyhow::Error>(res_str.to_string())
        });
        client_handles.push(h);
    }

    // Wait for all 5 requests to enter the interception pending queue
    let mut pending = Vec::new();
    for _ in 0..50 {
        pending = interceptor.get_pending_request_list().await;
        if pending.len() == CONCURRENT_INTERCEPTS {
            break;
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
    }
    assert_eq!(pending.len(), CONCURRENT_INTERCEPTS);

    // Operator forwards 4 of them and drops 1
    for (idx, p_req) in pending.iter().enumerate() {
        if idx == 0 {
            interceptor.drop_request(&p_req.id).await;
        } else {
            interceptor.forward_request(&p_req.id, None).await;
        }
    }

    // Verify client responses
    let mut dropped_count = 0;
    let mut forwarded_count = 0;

    for h in client_handles {
        let res_str = h.await??;
        if res_str.contains("502 Bad Gateway") || res_str.contains("Request Dropped") {
            dropped_count += 1;
        } else if res_str.contains("upstream_ack") {
            forwarded_count += 1;
        }
    }

    assert_eq!(dropped_count, 1);
    assert_eq!(forwarded_count, 4);

    Ok(())
}
