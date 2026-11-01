use anyhow::Result;
use regex::Regex;
use rustproxy_core::{RequestModel, ResponseModel};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::{oneshot, RwLock};
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InterceptDecision {
    ForwardOriginal,
    ForwardModifiedRequest(RequestModel),
    ForwardModifiedResponse(ResponseModel),
    Drop,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InterceptTarget {
    Request,
    Response,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterceptRule {
    pub id: String,
    pub enabled: bool,
    pub target: InterceptTarget,
    pub method: Option<String>,
    pub url_pattern: Option<String>,
    pub in_scope_only: bool,
    #[serde(skip)]
    url_regex: Option<Regex>,
}

impl InterceptRule {
    pub fn new_request_rule(id: impl Into<String>, url_pattern: Option<String>) -> Result<Self> {
        let url_regex = if let Some(p) = &url_pattern {
            Some(Regex::new(p)?)
        } else {
            None
        };

        Ok(Self {
            id: id.into(),
            enabled: true,
            target: InterceptTarget::Request,
            method: None,
            url_pattern,
            in_scope_only: false,
            url_regex,
        })
    }

    pub fn matches_request(&mut self, req: &RequestModel, in_scope: bool) -> bool {
        if !self.enabled {
            return false;
        }
        if self.in_scope_only && !in_scope {
            return false;
        }
        if let Some(m) = &self.method {
            if !m.eq_ignore_ascii_case(&req.method) {
                return false;
            }
        }
        if let Some(pattern) = &self.url_pattern {
            if self.url_regex.is_none() {
                self.url_regex = Regex::new(pattern).ok();
            }
            if let Some(re) = &self.url_regex {
                if !re.is_match(&req.uri) {
                    return false;
                }
            }
        }
        true
    }

    pub fn new_response_rule(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            enabled: true,
            target: InterceptTarget::Response,
            method: None,
            url_pattern: None,
            in_scope_only: false,
            url_regex: None,
        }
    }

    pub fn matches_response(&mut self, _res: &ResponseModel, in_scope: bool) -> bool {
        if !self.enabled {
            return false;
        }
        if self.in_scope_only && !in_scope {
            return false;
        }
        true
    }
}

pub struct InterceptManager {
    is_intercept_on: AtomicBool,
    rules: RwLock<Vec<InterceptRule>>,
    pending_requests:
        Arc<RwLock<HashMap<String, (RequestModel, oneshot::Sender<InterceptDecision>)>>>,
    pending_responses:
        Arc<RwLock<HashMap<String, (ResponseModel, oneshot::Sender<InterceptDecision>)>>>,
}

impl InterceptManager {
    pub fn new() -> Self {
        Self {
            is_intercept_on: AtomicBool::new(false),
            rules: RwLock::new(Vec::new()),
            pending_requests: Arc::new(RwLock::new(HashMap::new())),
            pending_responses: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.is_intercept_on.load(Ordering::Relaxed)
    }

    pub fn set_enabled(&self, enabled: bool) {
        self.is_intercept_on.store(enabled, Ordering::Relaxed);
        info!("Intercept set to: {}", enabled);
    }

    pub async fn add_rule(&self, rule: InterceptRule) {
        let mut rules = self.rules.write().await;
        rules.push(rule);
    }

    pub async fn should_intercept_request(&self, req: &RequestModel, in_scope: bool) -> bool {
        if !self.is_enabled() {
            return false;
        }
        let mut rules = self.rules.write().await;
        if rules.is_empty() {
            return true;
        }
        for rule in rules.iter_mut() {
            if let InterceptTarget::Request = rule.target {
                if rule.matches_request(req, in_scope) {
                    return true;
                }
            }
        }
        false
    }

    pub async fn should_intercept_response(&self, res: &ResponseModel, in_scope: bool) -> bool {
        if !self.is_enabled() {
            return false;
        }
        let mut rules = self.rules.write().await;
        for rule in rules.iter_mut() {
            if let InterceptTarget::Response = rule.target {
                if rule.matches_response(res, in_scope) {
                    return true;
                }
            }
        }
        false
    }

    pub async fn queue_request(&self, req: RequestModel) -> oneshot::Receiver<InterceptDecision> {
        let (tx, rx) = oneshot::channel();
        let mut pending = self.pending_requests.write().await;
        pending.insert(req.id.clone(), (req, tx));
        rx
    }

    pub async fn forward_request(&self, id: &str, modified: Option<RequestModel>) -> bool {
        let mut pending = self.pending_requests.write().await;
        if let Some((_, tx)) = pending.remove(id) {
            let decision = match modified {
                Some(req) => InterceptDecision::ForwardModifiedRequest(req),
                None => InterceptDecision::ForwardOriginal,
            };
            let _ = tx.send(decision);
            true
        } else {
            false
        }
    }

    pub async fn drop_request(&self, id: &str) -> bool {
        let mut pending = self.pending_requests.write().await;
        if let Some((_, tx)) = pending.remove(id) {
            let _ = tx.send(InterceptDecision::Drop);
            true
        } else {
            false
        }
    }

    pub async fn queue_response(&self, res: ResponseModel) -> oneshot::Receiver<InterceptDecision> {
        let (tx, rx) = oneshot::channel();
        let mut pending = self.pending_responses.write().await;
        pending.insert(res.id.clone(), (res, tx));
        rx
    }

    pub async fn forward_response(&self, id: &str, modified: Option<ResponseModel>) -> bool {
        let mut pending = self.pending_responses.write().await;
        if let Some((_, tx)) = pending.remove(id) {
            let decision = match modified {
                Some(res) => InterceptDecision::ForwardModifiedResponse(res),
                None => InterceptDecision::ForwardOriginal,
            };
            let _ = tx.send(decision);
            true
        } else {
            false
        }
    }

    pub async fn drop_response(&self, id: &str) -> bool {
        let mut pending = self.pending_responses.write().await;
        if let Some((_, tx)) = pending.remove(id) {
            let _ = tx.send(InterceptDecision::Drop);
            true
        } else {
            false
        }
    }

    pub async fn get_pending_request_list(&self) -> Vec<RequestModel> {
        let pending = self.pending_requests.read().await;
        pending.values().map(|(r, _)| r.clone()).collect()
    }

    pub async fn get_pending_response_list(&self) -> Vec<ResponseModel> {
        let pending = self.pending_responses.read().await;
        pending.values().map(|(r, _)| r.clone()).collect()
    }
}

impl Default for InterceptManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;

    #[tokio::test]
    async fn test_interception_queue_flow() {
        let manager = InterceptManager::new();
        manager.set_enabled(true);

        let req = RequestModel::new(
            "flow-1",
            "GET",
            "http://example.com/test",
            "HTTP/1.1",
            vec![],
            Bytes::new(),
        );
        assert!(manager.should_intercept_request(&req, true).await);

        let rx = manager.queue_request(req.clone()).await;

        let pending = manager.get_pending_request_list().await;
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].id, "flow-1");

        // Operator forwards request
        let forwarded = manager.forward_request("flow-1", None).await;
        assert!(forwarded);

        let decision = rx.await.expect("Channel closed");
        match decision {
            InterceptDecision::ForwardOriginal => {}
            _ => panic!("Expected ForwardOriginal decision"),
        }
    }
}
