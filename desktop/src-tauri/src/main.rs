// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use rustproxy_api::ApiService;
use rustproxy_intercept::InterceptManager;
use rustproxy_proxy::{ProxyConfig, ProxyServer};
use rustproxy_repeater::RepeaterEngine;
use rustproxy_sitemap::SiteMap;
use rustproxy_storage::StorageEngine;
use rustproxy_tls::CertificateAuthority;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;

fn main() {
    let _ = rustls::crypto::ring::default_provider().install_default();

    // Spawn async backend services
    std::thread::spawn(|| {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
        rt.block_on(async {
            let ca = Arc::new(CertificateAuthority::generate().expect("Failed to generate CA"));
            let storage = Arc::new(StorageEngine::open("rustproxy.db").expect("Failed to open DB"));
            let interceptor = Arc::new(InterceptManager::new());
            let sitemap = Arc::new(RwLock::new(SiteMap::new()));
            let scanner = Arc::new(rustproxy_scanner::ScannerEngine::new());
            let client = Arc::new(rustproxy_http::HttpClient::new_insecure().expect("Failed client"));
            let repeater = Arc::new(RwLock::new(RepeaterEngine::new(client)));

            let bind_addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
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
            ).expect("Failed to initialize proxy server");

            let api_service = Arc::new(ApiService::new(
                storage.clone(),
                interceptor.clone(),
                repeater.clone(),
                sitemap.clone(),
                scanner.clone(),
                proxy.event_sender(),
            ));

            let api_addr: SocketAddr = "127.0.0.1:8081".parse().unwrap();
            tokio::spawn(async move {
                let _ = rustproxy_api::start_api_server(api_service, api_addr).await;
            });

            let _ = proxy.run().await;
        });
    });

    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
