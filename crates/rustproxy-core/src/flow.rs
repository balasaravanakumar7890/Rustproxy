use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::http_model::{RequestModel, ResponseModel, TlsInfo};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowRecord {
    pub id: String,
    pub session_id: String,
    pub timestamp: DateTime<Utc>,
    pub request: RequestModel,
    pub response: Option<ResponseModel>,
    pub tls_info: Option<TlsInfo>,
    pub client_addr: Option<String>,
    pub in_scope: bool,
    pub comment: Option<String>,
    pub highlight: Option<String>,
    pub is_intercepted: bool,
    pub error: Option<String>,
}

impl FlowRecord {
    pub fn new(id: impl Into<String>, request: RequestModel) -> Self {
        Self {
            id: id.into(),
            session_id: "default".to_string(),
            timestamp: Utc::now(),
            request,
            response: None,
            tls_info: None,
            client_addr: None,
            in_scope: true,
            comment: None,
            highlight: None,
            is_intercepted: false,
            error: None,
        }
    }

    pub fn summary(&self) -> FlowSummary {
        FlowSummary {
            id: self.id.clone(),
            timestamp: self.timestamp,
            method: self.request.method.clone(),
            host: self.request.host.clone(),
            port: self.request.port,
            path: self.request.path.clone(),
            query: self.request.query.clone(),
            status: self.response.as_ref().map(|r| r.status),
            mime_type: self.response.as_ref().and_then(|r| r.mime_type.clone()),
            request_len: self.request.body_length,
            response_len: self.response.as_ref().map(|r| r.body_length),
            duration_ms: self.response.as_ref().map(|r| r.duration_ms),
            in_scope: self.in_scope,
            comment: self.comment.clone(),
            highlight: self.highlight.clone(),
            has_error: self.error.is_some(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowSummary {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub method: String,
    pub host: String,
    pub port: u16,
    pub path: String,
    pub query: Option<String>,
    pub status: Option<u16>,
    pub mime_type: Option<String>,
    pub request_len: usize,
    pub response_len: Option<usize>,
    pub duration_ms: Option<u64>,
    pub in_scope: bool,
    pub comment: Option<String>,
    pub highlight: Option<String>,
    pub has_error: bool,
}
