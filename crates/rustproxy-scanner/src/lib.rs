use regex::Regex;
use rustproxy_core::{FindingConfidence, FindingSeverity, FlowRecord, SecurityFinding};
use uuid::Uuid;

pub trait PassiveRule: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn check(&self, flow: &FlowRecord) -> Vec<SecurityFinding>;
}

pub struct ScannerEngine {
    rules: Vec<Box<dyn PassiveRule>>,
}

impl ScannerEngine {
    pub fn new() -> Self {
        let mut engine = Self { rules: Vec::new() };
        engine.register_default_rules();
        engine
    }

    pub fn register_rule(&mut self, rule: Box<dyn PassiveRule>) {
        self.rules.push(rule);
    }

    pub fn scan_flow(&self, flow: &FlowRecord) -> Vec<SecurityFinding> {
        let mut findings = Vec::new();
        for rule in &self.rules {
            findings.extend(rule.check(flow));
        }
        findings
    }

    fn register_default_rules(&mut self) {
        self.register_rule(Box::new(MissingCspRule));
        self.register_rule(Box::new(MissingHstsRule));
        self.register_rule(Box::new(MissingXContentTypeOptionsRule));
        self.register_rule(Box::new(MissingXFrameOptionsRule));
        self.register_rule(Box::new(InsecureCookieRule));
        self.register_rule(Box::new(ServerInformationDisclosureRule));
        self.register_rule(Box::new(ErrorDisclosureRule::new()));
    }
}

impl Default for ScannerEngine {
    fn default() -> Self {
        Self::new()
    }
}

// 1. Missing CSP Rule
pub struct MissingCspRule;

impl PassiveRule for MissingCspRule {
    fn id(&self) -> &str {
        "SEC-001"
    }

    fn name(&self) -> &str {
        "Missing Content-Security-Policy (CSP)"
    }

    fn check(&self, flow: &FlowRecord) -> Vec<SecurityFinding> {
        if let Some(res) = &flow.response {
            if res.is_success() {
                let is_html = res
                    .mime_type
                    .as_deref()
                    .map(|m| m.contains("text/html"))
                    .unwrap_or(false);

                if is_html && res.get_header("content-security-policy").is_none() {
                    let mut finding = SecurityFinding::new(
                        Uuid::new_v4().to_string(),
                        &flow.id,
                        "Missing Content-Security-Policy Header",
                        "The response does not specify a Content-Security-Policy (CSP) header. CSP helps prevent cross-site scripting (XSS) and data injection attacks.",
                        FindingSeverity::Medium,
                        FindingConfidence::Certain,
                        &flow.request.uri,
                    );
                    finding.cwe_id = Some(1021);
                    finding.remediation = Some("Configure the web server to send a robust Content-Security-Policy header.".to_string());
                    return vec![finding];
                }
            }
        }
        vec![]
    }
}

// 2. Missing HSTS Rule
pub struct MissingHstsRule;

impl PassiveRule for MissingHstsRule {
    fn id(&self) -> &str {
        "SEC-002"
    }

    fn name(&self) -> &str {
        "Missing Strict-Transport-Security (HSTS)"
    }

    fn check(&self, flow: &FlowRecord) -> Vec<SecurityFinding> {
        if flow.request.scheme.eq_ignore_ascii_case("https") {
            if let Some(res) = &flow.response {
                if res.get_header("strict-transport-security").is_none() {
                    let mut finding = SecurityFinding::new(
                        Uuid::new_v4().to_string(),
                        &flow.id,
                        "Missing Strict-Transport-Security (HSTS) Header",
                        "HTTPS response is missing the Strict-Transport-Security header, leaving users vulnerable to SSL-stripping attacks.",
                        FindingSeverity::Low,
                        FindingConfidence::Certain,
                        &flow.request.uri,
                    );
                    finding.cwe_id = Some(319);
                    finding.remediation = Some("Add 'Strict-Transport-Security: max-age=31536000; includeSubDomains' to all HTTPS responses.".to_string());
                    return vec![finding];
                }
            }
        }
        vec![]
    }
}

// 3. Missing X-Content-Type-Options
pub struct MissingXContentTypeOptionsRule;

impl PassiveRule for MissingXContentTypeOptionsRule {
    fn id(&self) -> &str {
        "SEC-003"
    }

    fn name(&self) -> &str {
        "Missing X-Content-Type-Options"
    }

    fn check(&self, flow: &FlowRecord) -> Vec<SecurityFinding> {
        if let Some(res) = &flow.response {
            if let Some(val) = res.get_header("x-content-type-options") {
                if val.eq_ignore_ascii_case("nosniff") {
                    return vec![];
                }
            }
            let mut finding = SecurityFinding::new(
                Uuid::new_v4().to_string(),
                &flow.id,
                "Missing X-Content-Type-Options Header",
                "The X-Content-Type-Options: nosniff header instructs the browser not to override the declared response MIME type.",
                FindingSeverity::Low,
                FindingConfidence::Certain,
                &flow.request.uri,
            );
            finding.cwe_id = Some(16);
            finding.remediation = Some("Ensure all responses include 'X-Content-Type-Options: nosniff'.".to_string());
            return vec![finding];
        }
        vec![]
    }
}

// 4. Missing X-Frame-Options
pub struct MissingXFrameOptionsRule;

impl PassiveRule for MissingXFrameOptionsRule {
    fn id(&self) -> &str {
        "SEC-004"
    }

    fn name(&self) -> &str {
        "Missing Anti-Clickjacking Frame Protections"
    }

    fn check(&self, flow: &FlowRecord) -> Vec<SecurityFinding> {
        if let Some(res) = &flow.response {
            let is_html = res.mime_type.as_deref().map(|m| m.contains("text/html")).unwrap_or(false);
            if is_html {
                let has_xfo = res.get_header("x-frame-options").is_some();
                let has_csp_fa = res.get_header("content-security-policy")
                    .map(|csp| csp.contains("frame-ancestors"))
                    .unwrap_or(false);

                if !has_xfo && !has_csp_fa {
                    let mut finding = SecurityFinding::new(
                        Uuid::new_v4().to_string(),
                        &flow.id,
                        "Missing Clickjacking Protection",
                        "Neither X-Frame-Options nor CSP frame-ancestors is present on this HTML page, potentially allowing clickjacking in iframes.",
                        FindingSeverity::Medium,
                        FindingConfidence::Firm,
                        &flow.request.uri,
                    );
                    finding.cwe_id = Some(1021);
                    finding.remediation = Some("Set 'X-Frame-Options: DENY' or use CSP 'frame-ancestors 'none';'.".to_string());
                    return vec![finding];
                }
            }
        }
        vec![]
    }
}

// 5. Insecure Cookie Flags
pub struct InsecureCookieRule;

impl PassiveRule for InsecureCookieRule {
    fn id(&self) -> &str {
        "SEC-005"
    }

    fn name(&self) -> &str {
        "Insecure Cookie Flags (Missing Secure / HttpOnly / SameSite)"
    }

    fn check(&self, flow: &FlowRecord) -> Vec<SecurityFinding> {
        let mut findings = Vec::new();
        if let Some(res) = &flow.response {
            for (name, val) in &res.headers {
                if name.eq_ignore_ascii_case("set-cookie") {
                    let lower = val.to_lowercase();
                    let cookie_name = val.split('=').next().unwrap_or("unknown").trim();

                    if !lower.contains("secure") && flow.request.scheme.eq_ignore_ascii_case("https") {
                        let mut f = SecurityFinding::new(
                            Uuid::new_v4().to_string(),
                            &flow.id,
                            format!("Cookie '{}' Missing 'Secure' Flag", cookie_name),
                            format!("The cookie '{}' was set over HTTPS without the Secure flag.", cookie_name),
                            FindingSeverity::Medium,
                            FindingConfidence::Certain,
                            &flow.request.uri,
                        );
                        f.cwe_id = Some(614);
                        f.evidence = Some(val.clone());
                        findings.push(f);
                    }

                    if !lower.contains("httponly") {
                        let mut f = SecurityFinding::new(
                            Uuid::new_v4().to_string(),
                            &flow.id,
                            format!("Cookie '{}' Missing 'HttpOnly' Flag", cookie_name),
                            format!("The cookie '{}' is accessible via client-side JavaScript.", cookie_name),
                            FindingSeverity::Low,
                            FindingConfidence::Certain,
                            &flow.request.uri,
                        );
                        f.cwe_id = Some(1004);
                        f.evidence = Some(val.clone());
                        findings.push(f);
                    }

                    if !lower.contains("samesite") {
                        let mut f = SecurityFinding::new(
                            Uuid::new_v4().to_string(),
                            &flow.id,
                            format!("Cookie '{}' Missing 'SameSite' Attribute", cookie_name),
                            format!("The cookie '{}' does not specify SameSite=Lax or SameSite=Strict.", cookie_name),
                            FindingSeverity::Low,
                            FindingConfidence::Certain,
                            &flow.request.uri,
                        );
                        f.cwe_id = Some(1275);
                        f.evidence = Some(val.clone());
                        findings.push(f);
                    }
                }
            }
        }
        findings
    }
}

// 6. Server Banner Information Disclosure
pub struct ServerInformationDisclosureRule;

impl PassiveRule for ServerInformationDisclosureRule {
    fn id(&self) -> &str {
        "SEC-006"
    }

    fn name(&self) -> &str {
        "Server Version Banner Information Disclosure"
    }

    fn check(&self, flow: &FlowRecord) -> Vec<SecurityFinding> {
        let mut findings = Vec::new();
        if let Some(res) = &flow.response {
            if let Some(server) = res.get_header("server") {
                if server.contains('/') || server.chars().any(|c| c.is_ascii_digit()) {
                    let mut f = SecurityFinding::new(
                        Uuid::new_v4().to_string(),
                        &flow.id,
                        "Server Header Version Disclosure",
                        format!("The Server header discloses software version information: '{}'", server),
                        FindingSeverity::Info,
                        FindingConfidence::Certain,
                        &flow.request.uri,
                    );
                    f.cwe_id = Some(200);
                    f.evidence = Some(server.to_string());
                    findings.push(f);
                }
            }
            if let Some(xpb) = res.get_header("x-powered-by") {
                let mut f = SecurityFinding::new(
                    Uuid::new_v4().to_string(),
                    &flow.id,
                    "X-Powered-By Header Information Disclosure",
                    format!("The X-Powered-By header exposes backend technology details: '{}'", xpb),
                    FindingSeverity::Info,
                    FindingConfidence::Certain,
                    &flow.request.uri,
                );
                f.cwe_id = Some(200);
                f.evidence = Some(xpb.to_string());
                findings.push(f);
            }
        }
        findings
    }
}

// 7. Stack Trace / Internal Error Disclosure
pub struct ErrorDisclosureRule {
    stack_trace_regex: Regex,
}

impl ErrorDisclosureRule {
    pub fn new() -> Self {
        let pattern = r"(?i)(Traceback \(most recent call last\)|Exception in thread|NullPointerException|at [a-zA-Z0-9_\.]+\([a-zA-Z0-9_\.]+\.java:[0-9]+\)|fatal error:.*on line [0-9]+|Microsoft OLE DB Provider for SQL Server)";
        Self {
            stack_trace_regex: Regex::new(pattern).unwrap(),
        }
    }
}

impl Default for ErrorDisclosureRule {
    fn default() -> Self {
        Self::new()
    }
}

impl PassiveRule for ErrorDisclosureRule {
    fn id(&self) -> &str {
        "SEC-007"
    }

    fn name(&self) -> &str {
        "Application Stack Trace / Error Disclosure"
    }

    fn check(&self, flow: &FlowRecord) -> Vec<SecurityFinding> {
        if let Some(res) = &flow.response {
            if res.status >= 500 {
                let body_str = res.body_as_string_lossy();
                if let Some(mat) = self.stack_trace_regex.find(&body_str) {
                    let mut f = SecurityFinding::new(
                        Uuid::new_v4().to_string(),
                        &flow.id,
                        "Internal Stack Trace Disclosed in Response",
                        "The server returned an unhandled internal error containing a stack trace or debug information.",
                        FindingSeverity::High,
                        FindingConfidence::Firm,
                        &flow.request.uri,
                    );
                    f.cwe_id = Some(209);
                    f.evidence = Some(mat.as_str().to_string());
                    return vec![f];
                }
            }
        }
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use rustproxy_core::{RequestModel, ResponseModel};

    #[test]
    fn test_passive_scanner_detects_missing_headers_and_cookie_issues() {
        let engine = ScannerEngine::new();

        let req = RequestModel::new("f-1", "GET", "https://example.com/app", "HTTP/1.1", vec![], Bytes::new());
        let mut flow = FlowRecord::new("f-1", req);

        let res_headers = vec![
            ("Content-Type".to_string(), "text/html".to_string()),
            ("Server".to_string(), "Apache/2.4.51 (Ubuntu)".to_string()),
            ("Set-Cookie".to_string(), "sessionid=xyz; Path=/".to_string()),
        ];
        flow.response = Some(ResponseModel::new("f-1", 200, "OK", "HTTP/1.1", res_headers, Bytes::from("<html><body>Welcome</body></html>"), 20));

        let findings = engine.scan_flow(&flow);
        assert!(!findings.is_empty());

        let titles: Vec<&str> = findings.iter().map(|f| f.title.as_str()).collect();
        assert!(titles.contains(&"Missing Content-Security-Policy Header"));
        assert!(titles.contains(&"Missing Strict-Transport-Security (HSTS) Header"));
        assert!(titles.contains(&"Missing X-Content-Type-Options Header"));
        assert!(titles.contains(&"Missing Anti-Clickjacking Frame Protections") || titles.contains(&"Missing Clickjacking Protection"));
        assert!(titles.iter().any(|t| t.contains("Secure") || t.contains("HttpOnly")));
        assert!(titles.contains(&"Server Header Version Disclosure"));
    }
}
