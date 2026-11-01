pub mod error;
pub mod finding;
pub mod flow;
pub mod http_model;
pub mod scope;

pub use error::CoreError;
pub use finding::{FindingConfidence, FindingSeverity, SecurityFinding};
pub use flow::{FlowRecord, FlowSummary};
pub use http_model::{RequestModel, ResponseModel, TlsInfo};
pub use scope::{ScopeAction, ScopeConfig, ScopeRule};

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;

    #[test]
    fn test_request_model_creation() {
        let headers = vec![
            ("Host".to_string(), "api.example.com".to_string()),
            ("User-Agent".to_string(), "RustProxy/1.0".to_string()),
        ];
        let body = Bytes::from("{\"test\": true}");
        let req = RequestModel::new(
            "flow-1",
            "POST",
            "https://api.example.com/v1/auth?client=web",
            "HTTP/1.1",
            headers,
            body,
        );

        assert_eq!(req.method, "POST");
        assert_eq!(req.host, "api.example.com");
        assert_eq!(req.port, 443);
        assert_eq!(req.scheme, "https");
        assert_eq!(req.path, "/v1/auth");
        assert_eq!(req.query.as_deref(), Some("client=web"));
        assert_eq!(req.get_header("host"), Some("api.example.com"));
        assert_eq!(req.get_header("user-agent"), Some("RustProxy/1.0"));
        assert_eq!(req.body_length, 14);
    }

    #[test]
    fn test_response_model() {
        let headers = vec![
            (
                "Content-Type".to_string(),
                "application/json; charset=utf-8".to_string(),
            ),
            ("Server".to_string(), "nginx/1.24".to_string()),
        ];
        let body = Bytes::from("{\"status\": \"ok\"}");
        let res = ResponseModel::new("flow-1", 200, "OK", "HTTP/1.1", headers, body, 45);

        assert!(res.is_success());
        assert!(!res.is_error());
        assert_eq!(res.mime_type.as_deref(), Some("application/json"));
        assert_eq!(res.duration_ms, 45);
    }

    #[test]
    fn test_scope_rules() {
        let mut config = ScopeConfig::new();
        let rule = ScopeRule::new("r1", ScopeAction::Include, r".*\.target\.local$").unwrap();
        config.rules.push(rule);

        assert!(config.is_in_scope("https", "app.target.local", 443, "/dashboard"));
        assert!(!config.is_in_scope("https", "other.domain.com", 443, "/"));
    }
}
