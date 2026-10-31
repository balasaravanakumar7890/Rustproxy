use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum FindingSeverity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FindingConfidence {
    Tentative,
    Firm,
    Certain,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityFinding {
    pub id: String,
    pub flow_id: String,
    pub title: String,
    pub description: String,
    pub severity: FindingSeverity,
    pub confidence: FindingConfidence,
    pub cwe_id: Option<u32>,
    pub remediation: Option<String>,
    pub evidence: Option<String>,
    pub url: String,
    pub timestamp: DateTime<Utc>,
}

impl SecurityFinding {
    pub fn new(
        id: impl Into<String>,
        flow_id: impl Into<String>,
        title: impl Into<String>,
        description: impl Into<String>,
        severity: FindingSeverity,
        confidence: FindingConfidence,
        url: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            flow_id: flow_id.into(),
            title: title.into(),
            description: description.into(),
            severity,
            confidence,
            cwe_id: None,
            remediation: None,
            evidence: None,
            url: url.into(),
            timestamp: Utc::now(),
        }
    }
}
