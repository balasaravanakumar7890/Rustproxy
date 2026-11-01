pub mod server;

use anyhow::Result;
use rustproxy_comparer::{Comparer, DiffResult};
use rustproxy_core::{FlowRecord, FlowSummary, RequestModel, ResponseModel, SecurityFinding};
use rustproxy_decoder::Decoder;
use rustproxy_intercept::InterceptManager;
use rustproxy_proxy::ProxyEvent;
use rustproxy_repeater::{RepeaterEngine, RepeaterTab};
use rustproxy_sitemap::{SiteMap, SiteMapNode};
use rustproxy_storage::{FlowFilter, StorageEngine};
use serde::{Deserialize, Serialize};
pub use server::start_api_server;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyStats {
    pub total_flows: usize,
    pub total_findings: usize,
    pub intercept_enabled: bool,
    pub pending_intercepts: usize,
}

pub struct ApiService {
    storage: Arc<StorageEngine>,
    interceptor: Arc<InterceptManager>,
    repeater: Arc<RwLock<RepeaterEngine>>,
    sitemap: Arc<RwLock<SiteMap>>,
    scanner: Arc<rustproxy_scanner::ScannerEngine>,
    event_tx: broadcast::Sender<ProxyEvent>,
}

impl ApiService {
    pub fn new(
        storage: Arc<StorageEngine>,
        interceptor: Arc<InterceptManager>,
        repeater: Arc<RwLock<RepeaterEngine>>,
        sitemap: Arc<RwLock<SiteMap>>,
        scanner: Arc<rustproxy_scanner::ScannerEngine>,
        event_tx: broadcast::Sender<ProxyEvent>,
    ) -> Self {
        Self {
            storage,
            interceptor,
            repeater,
            sitemap,
            scanner,
            event_tx,
        }
    }

    // Stats
    pub async fn get_stats(&self) -> Result<ProxyStats> {
        let total_flows = self.storage.count_flows()?;
        let total_findings = self.storage.list_findings()?.len();
        let intercept_enabled = self.interceptor.is_enabled();
        let pending = self.interceptor.get_pending_request_list().await.len();

        Ok(ProxyStats {
            total_flows,
            total_findings,
            intercept_enabled,
            pending_intercepts: pending,
        })
    }

    // Intercept
    pub fn is_intercept_enabled(&self) -> bool {
        self.interceptor.is_enabled()
    }

    pub fn set_intercept_enabled(&self, enabled: bool) {
        self.interceptor.set_enabled(enabled);
    }

    pub async fn get_pending_intercepts(&self) -> Vec<RequestModel> {
        self.interceptor.get_pending_request_list().await
    }

    pub async fn get_pending_response_intercepts(&self) -> Vec<ResponseModel> {
        self.interceptor.get_pending_response_list().await
    }

    pub async fn forward_intercepted(&self, id: &str, modified: Option<RequestModel>) -> bool {
        self.interceptor.forward_request(id, modified).await
    }

    pub async fn drop_intercepted(&self, id: &str) -> bool {
        self.interceptor.drop_request(id).await
    }

    pub async fn forward_intercepted_response(
        &self,
        id: &str,
        modified: Option<ResponseModel>,
    ) -> bool {
        self.interceptor.forward_response(id, modified).await
    }

    pub async fn drop_intercepted_response(&self, id: &str) -> bool {
        self.interceptor.drop_response(id).await
    }

    // History
    pub fn list_flows(&self, filter: &FlowFilter) -> Result<Vec<FlowSummary>> {
        self.storage.list_flows(filter)
    }

    pub fn get_flow(&self, id: &str) -> Result<Option<FlowRecord>> {
        self.storage.get_flow(id)
    }

    pub fn clear_history(&self) -> Result<()> {
        self.storage.clear_all()
    }

    // Repeater
    pub async fn list_repeater_tabs(&self) -> Vec<RepeaterTab> {
        let rep = self.repeater.read().await;
        rep.list_tabs()
    }

    pub async fn create_repeater_tab(
        &self,
        name: Option<String>,
        req: Option<RequestModel>,
    ) -> String {
        let mut rep = self.repeater.write().await;
        rep.create_tab(name, req)
    }

    pub async fn update_repeater_tab(&self, id: &str, req: RequestModel) -> bool {
        let mut rep = self.repeater.write().await;
        rep.update_tab_request(id, req)
    }

    pub async fn duplicate_repeater_tab(&self, id: &str) -> Option<String> {
        let mut rep = self.repeater.write().await;
        rep.duplicate_tab(id)
    }

    pub async fn close_repeater_tab(&self, id: &str) {
        let mut rep = self.repeater.write().await;
        rep.close_tab(id);
    }

    pub async fn send_repeater_tab(&self, id: &str) -> Result<ResponseModel> {
        let mut rep = self.repeater.write().await;
        rep.send_tab(id).await
    }

    // Scanner
    pub fn list_findings(&self) -> Result<Vec<SecurityFinding>> {
        self.storage.list_findings()
    }

    pub fn scanner(&self) -> &rustproxy_scanner::ScannerEngine {
        &self.scanner
    }

    // SiteMap
    pub async fn get_sitemap(&self) -> Vec<SiteMapNode> {
        let sm = self.sitemap.read().await;
        sm.get_hosts()
    }

    // Decoder
    pub fn decode(&self, text: &str, format: &str) -> Result<String> {
        match format.to_lowercase().as_str() {
            "url" => Decoder::url_decode(text),
            "base64" => {
                let bytes = Decoder::base64_decode(text)?;
                Ok(String::from_utf8_lossy(&bytes).to_string())
            }
            "hex" => {
                let bytes = Decoder::hex_decode(text)?;
                Ok(String::from_utf8_lossy(&bytes).to_string())
            }
            "html" => Ok(Decoder::html_unescape(text)),
            _ => Err(anyhow::anyhow!("Unsupported format: {}", format)),
        }
    }

    pub fn encode(&self, text: &str, format: &str) -> Result<String> {
        match format.to_lowercase().as_str() {
            "url" => Ok(Decoder::url_encode(text)),
            "base64" => Ok(Decoder::base64_encode(text.as_bytes())),
            "hex" => Ok(Decoder::hex_encode(text.as_bytes())),
            "html" => Ok(Decoder::html_escape(text)),
            _ => Err(anyhow::anyhow!("Unsupported format: {}", format)),
        }
    }

    // Comparer
    pub fn diff(&self, old: &str, new: &str) -> DiffResult {
        Comparer::diff_text(old, new)
    }

    pub fn subscribe_events(&self) -> broadcast::Receiver<ProxyEvent> {
        self.event_tx.subscribe()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustproxy_http::HttpClient;

    #[tokio::test]
    async fn test_api_service_lifecycle() -> Result<()> {
        let storage = Arc::new(StorageEngine::open_in_memory()?);
        let interceptor = Arc::new(InterceptManager::new());
        let client = Arc::new(HttpClient::new_insecure()?);
        let repeater = Arc::new(RwLock::new(RepeaterEngine::new(client)));
        let sitemap = Arc::new(RwLock::new(SiteMap::new()));
        let scanner = Arc::new(rustproxy_scanner::ScannerEngine::new());
        let (event_tx, _) = broadcast::channel(100);

        let api = ApiService::new(storage, interceptor, repeater, sitemap, scanner, event_tx);

        let stats = api.get_stats().await?;
        assert_eq!(stats.total_flows, 0);
        assert!(!stats.intercept_enabled);

        api.set_intercept_enabled(true);
        assert!(api.is_intercept_enabled());

        let _tab_id = api
            .create_repeater_tab(Some("My Test Tab".to_string()), None)
            .await;
        let tabs = api.list_repeater_tabs().await;
        assert_eq!(tabs.len(), 1);
        assert_eq!(tabs[0].name, "My Test Tab");

        let encoded = api.encode("test <script>", "html")?;
        assert!(encoded.contains("&lt;script&gt;"));

        let diff = api.diff("line1\nline2", "line1\nline3");
        assert_eq!(diff.additions, 1);
        assert_eq!(diff.deletions, 1);

        Ok(())
    }
}
