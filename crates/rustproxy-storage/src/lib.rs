use anyhow::Result;
use bytes::Bytes;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OpenFlags};
use rustproxy_core::{
    FindingConfidence, FindingSeverity, FlowRecord, FlowSummary, RequestModel, ResponseModel,
    SecurityFinding, TlsInfo,
};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::{Arc, Mutex};
use tracing::info;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FlowFilter {
    pub search_query: Option<String>,
    pub method: Option<String>,
    pub status_code_min: Option<u16>,
    pub status_code_max: Option<u16>,
    pub in_scope_only: bool,
    pub has_error_only: bool,
    pub limit: usize,
    pub offset: usize,
}

impl FlowFilter {
    pub fn all() -> Self {
        Self {
            limit: 1000,
            ..Default::default()
        }
    }
}

pub struct StorageEngine {
    conn: Arc<Mutex<Connection>>,
}

impl StorageEngine {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let conn = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;

        // Performance optimizations: WAL mode, normal sync
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA foreign_keys = ON;
             PRAGMA temp_store = MEMORY;",
        )?;

        let engine = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        engine.init_tables()?;
        Ok(engine)
    }

    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let engine = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        engine.init_tables()?;
        Ok(engine)
    }

    fn init_tables(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS flows (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                method TEXT NOT NULL,
                scheme TEXT NOT NULL,
                host TEXT NOT NULL,
                port INTEGER NOT NULL,
                path TEXT NOT NULL,
                query TEXT,
                status_code INTEGER,
                mime_type TEXT,
                request_size INTEGER NOT NULL,
                response_size INTEGER,
                duration_ms INTEGER,
                in_scope INTEGER NOT NULL,
                comment TEXT,
                highlight TEXT,
                has_error INTEGER NOT NULL,
                error_msg TEXT
            );

            CREATE TABLE IF NOT EXISTS flow_bodies (
                flow_id TEXT PRIMARY KEY,
                request_headers TEXT NOT NULL,
                request_body BLOB,
                request_version TEXT NOT NULL,
                request_uri TEXT NOT NULL,
                response_status_text TEXT,
                response_version TEXT,
                response_headers TEXT,
                response_body BLOB,
                tls_info TEXT,
                FOREIGN KEY (flow_id) REFERENCES flows(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS findings (
                id TEXT PRIMARY KEY,
                flow_id TEXT NOT NULL,
                title TEXT NOT NULL,
                description TEXT NOT NULL,
                severity TEXT NOT NULL,
                confidence TEXT NOT NULL,
                cwe_id INTEGER,
                remediation TEXT,
                evidence TEXT,
                url TEXT NOT NULL,
                timestamp TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_flows_timestamp ON flows(timestamp DESC);
            CREATE INDEX IF NOT EXISTS idx_flows_host ON flows(host);
            CREATE INDEX IF NOT EXISTS idx_flows_status ON flows(status_code);
            CREATE INDEX IF NOT EXISTS idx_flows_scope ON flows(in_scope);
            CREATE INDEX IF NOT EXISTS idx_findings_flow ON findings(flow_id);",
        )?;
        info!("Initialized SQLite tables and indexes");
        Ok(())
    }

    pub fn save_flow(&self, flow: &FlowRecord) -> Result<()> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;

        let status_code = flow.response.as_ref().map(|r| r.status);
        let mime_type = flow.response.as_ref().and_then(|r| r.mime_type.as_deref());
        let response_size = flow.response.as_ref().map(|r| r.body_length);
        let duration_ms = flow.response.as_ref().map(|r| r.duration_ms);

        tx.execute(
            "INSERT OR REPLACE INTO flows (
                id, session_id, timestamp, method, scheme, host, port, path, query,
                status_code, mime_type, request_size, response_size, duration_ms,
                in_scope, comment, highlight, has_error, error_msg
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19)",
            params![
                flow.id,
                flow.session_id,
                flow.timestamp.to_rfc3339(),
                flow.request.method,
                flow.request.scheme,
                flow.request.host,
                flow.request.port,
                flow.request.path,
                flow.request.query,
                status_code,
                mime_type,
                flow.request.body_length,
                response_size,
                duration_ms,
                if flow.in_scope { 1 } else { 0 },
                flow.comment,
                flow.highlight,
                if flow.error.is_some() { 1 } else { 0 },
                flow.error,
            ],
        )?;

        let req_headers_json = serde_json::to_string(&flow.request.headers)?;
        let res_headers_json = flow
            .response
            .as_ref()
            .map(|r| serde_json::to_string(&r.headers))
            .transpose()?;

        let tls_json = flow
            .tls_info
            .as_ref()
            .map(|t| serde_json::to_string(t))
            .transpose()?;

        let res_status_text = flow.response.as_ref().map(|r| r.status_text.as_str());
        let res_version = flow.response.as_ref().map(|r| r.version.as_str());
        let res_body = flow.response.as_ref().map(|r| r.body.as_ref());

        tx.execute(
            "INSERT OR REPLACE INTO flow_bodies (
                flow_id, request_headers, request_body, request_version, request_uri,
                response_status_text, response_version, response_headers, response_body, tls_info
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                flow.id,
                req_headers_json,
                flow.request.body.as_ref(),
                flow.request.version,
                flow.request.uri,
                res_status_text,
                res_version,
                res_headers_json,
                res_body,
                tls_json,
            ],
        )?;

        tx.commit()?;
        Ok(())
    }

    pub fn get_flow(&self, id: &str) -> Result<Option<FlowRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT f.id, f.session_id, f.timestamp, f.method, f.scheme, f.host, f.port, f.path, f.query,
                    f.status_code, f.mime_type, f.request_size, f.response_size, f.duration_ms,
                    f.in_scope, f.comment, f.highlight, f.has_error, f.error_msg,
                    b.request_headers, b.request_body, b.request_version, b.request_uri,
                    b.response_status_text, b.response_version, b.response_headers, b.response_body, b.tls_info
             FROM flows f
             JOIN flow_bodies b ON f.id = b.flow_id
             WHERE f.id = ?1",
        )?;

        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            let flow_id: String = row.get(0)?;
            let session_id: String = row.get(1)?;
            let ts_str: String = row.get(2)?;
            let timestamp = DateTime::parse_from_rfc3339(&ts_str)
                .unwrap_or_default()
                .with_timezone(&Utc);

            let method: String = row.get(3)?;
            let req_headers_json: String = row.get(19)?;
            let req_headers: Vec<(String, String)> = serde_json::from_str(&req_headers_json)?;
            let req_body_bytes: Vec<u8> = row.get(20)?;
            let req_version: String = row.get(21)?;
            let req_uri: String = row.get(22)?;

            let request = RequestModel::new(
                &flow_id,
                method,
                req_uri,
                req_version,
                req_headers,
                Bytes::from(req_body_bytes),
            );

            let status_code: Option<u16> = row.get(9)?;
            let response = if let Some(code) = status_code {
                let status_text: String = row.get(23)?;
                let res_version: String = row.get(24)?;
                let res_headers_json: Option<String> = row.get(25)?;
                let res_headers: Vec<(String, String)> = res_headers_json
                    .and_then(|j| serde_json::from_str(&j).ok())
                    .unwrap_or_default();
                let res_body_bytes: Vec<u8> = row.get(26)?;
                let duration_ms: u64 = row.get(13)?;

                Some(ResponseModel::new(
                    &flow_id,
                    code,
                    status_text,
                    res_version,
                    res_headers,
                    Bytes::from(res_body_bytes),
                    duration_ms,
                ))
            } else {
                None
            };

            let in_scope_num: i32 = row.get(14)?;
            let comment: Option<String> = row.get(15)?;
            let highlight: Option<String> = row.get(16)?;
            let error_msg: Option<String> = row.get(18)?;

            let tls_info: Option<TlsInfo> = row
                .get::<_, Option<String>>(27)?
                .and_then(|j| serde_json::from_str(&j).ok());

            let mut record = FlowRecord::new(flow_id, request);
            record.session_id = session_id;
            record.timestamp = timestamp;
            record.response = response;
            record.in_scope = in_scope_num != 0;
            record.comment = comment;
            record.highlight = highlight;
            record.error = error_msg;
            record.tls_info = tls_info;

            Ok(Some(record))
        } else {
            Ok(None)
        }
    }

    pub fn list_flows(&self, filter: &FlowFilter) -> Result<Vec<FlowSummary>> {
        let conn = self.conn.lock().unwrap();
        let limit = if filter.limit == 0 { 500 } else { filter.limit };

        let mut query = String::from(
            "SELECT id, timestamp, method, host, port, path, query, status_code, mime_type,
                    request_size, response_size, duration_ms, in_scope, comment, highlight, has_error
             FROM flows WHERE 1=1",
        );

        if filter.in_scope_only {
            query.push_str(" AND in_scope = 1");
        }
        if filter.has_error_only {
            query.push_str(" AND has_error = 1");
        }
        if let Some(method) = &filter.method {
            query.push_str(&format!(" AND method = '{}'", method.replace('\'', "''")));
        }
        if let Some(min) = filter.status_code_min {
            query.push_str(&format!(" AND status_code >= {}", min));
        }
        if let Some(max) = filter.status_code_max {
            query.push_str(&format!(" AND status_code <= {}", max));
        }
        if let Some(search) = &filter.search_query {
            let escaped = search.replace('\'', "''");
            query.push_str(&format!(
                " AND (host LIKE '%{}%' OR path LIKE '%{}%' OR method LIKE '%{}%')",
                escaped, escaped, escaped
            ));
        }

        query.push_str(&format!(
            " ORDER BY timestamp DESC LIMIT {} OFFSET {}",
            limit, filter.offset
        ));

        let mut stmt = conn.prepare(&query)?;
        let rows = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let ts_str: String = row.get(1)?;
            let timestamp = DateTime::parse_from_rfc3339(&ts_str)
                .unwrap_or_default()
                .with_timezone(&Utc);
            let method: String = row.get(2)?;
            let host: String = row.get(3)?;
            let port: u16 = row.get(4)?;
            let path: String = row.get(5)?;
            let query_param: Option<String> = row.get(6)?;
            let status: Option<u16> = row.get(7)?;
            let mime_type: Option<String> = row.get(8)?;
            let request_len: usize = row.get(9)?;
            let response_len: Option<usize> = row.get(10)?;
            let duration_ms: Option<u64> = row.get(11)?;
            let in_scope_num: i32 = row.get(12)?;
            let comment: Option<String> = row.get(13)?;
            let highlight: Option<String> = row.get(14)?;
            let has_error_num: i32 = row.get(15)?;

            Ok(FlowSummary {
                id,
                timestamp,
                method,
                host,
                port,
                path,
                query: query_param,
                status,
                mime_type,
                request_len,
                response_len,
                duration_ms,
                in_scope: in_scope_num != 0,
                comment,
                highlight,
                has_error: has_error_num != 0,
            })
        })?;

        let mut results = Vec::new();
        for r in rows {
            results.push(r?);
        }
        Ok(results)
    }

    pub fn save_finding(&self, finding: &SecurityFinding) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let severity_str = match finding.severity {
            FindingSeverity::Info => "Info",
            FindingSeverity::Low => "Low",
            FindingSeverity::Medium => "Medium",
            FindingSeverity::High => "High",
            FindingSeverity::Critical => "Critical",
        };
        let confidence_str = match finding.confidence {
            FindingConfidence::Tentative => "Tentative",
            FindingConfidence::Firm => "Firm",
            FindingConfidence::Certain => "Certain",
        };

        conn.execute(
            "INSERT OR REPLACE INTO findings (
                id, flow_id, title, description, severity, confidence, cwe_id, remediation, evidence, url, timestamp
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                finding.id,
                finding.flow_id,
                finding.title,
                finding.description,
                severity_str,
                confidence_str,
                finding.cwe_id,
                finding.remediation,
                finding.evidence,
                finding.url,
                finding.timestamp.to_rfc3339(),
            ],
        )?;

        Ok(())
    }

    pub fn list_findings(&self) -> Result<Vec<SecurityFinding>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, flow_id, title, description, severity, confidence, cwe_id, remediation, evidence, url, timestamp
             FROM findings ORDER BY timestamp DESC",
        )?;

        let rows = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let flow_id: String = row.get(1)?;
            let title: String = row.get(2)?;
            let description: String = row.get(3)?;
            let sev_str: String = row.get(4)?;
            let conf_str: String = row.get(5)?;
            let cwe_id: Option<u32> = row.get(6)?;
            let remediation: Option<String> = row.get(7)?;
            let evidence: Option<String> = row.get(8)?;
            let url: String = row.get(9)?;
            let ts_str: String = row.get(10)?;

            let severity = match sev_str.as_str() {
                "Low" => FindingSeverity::Low,
                "Medium" => FindingSeverity::Medium,
                "High" => FindingSeverity::High,
                "Critical" => FindingSeverity::Critical,
                _ => FindingSeverity::Info,
            };

            let confidence = match conf_str.as_str() {
                "Tentative" => FindingConfidence::Tentative,
                "Certain" => FindingConfidence::Certain,
                _ => FindingConfidence::Firm,
            };

            let timestamp = DateTime::parse_from_rfc3339(&ts_str)
                .unwrap_or_default()
                .with_timezone(&Utc);

            let mut f = SecurityFinding::new(id, flow_id, title, description, severity, confidence, url);
            f.cwe_id = cwe_id;
            f.remediation = remediation;
            f.evidence = evidence;
            f.timestamp = timestamp;
            Ok(f)
        })?;

        let mut results = Vec::new();
        for r in rows {
            results.push(r?);
        }
        Ok(results)
    }

    pub fn count_flows(&self) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        let count: usize = conn.query_row("SELECT COUNT(*) FROM flows", [], |r| r.get(0))?;
        Ok(count)
    }

    pub fn clear_all(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch("DELETE FROM flow_bodies; DELETE FROM flows; DELETE FROM findings;")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_engine_flow_lifecycle() -> Result<()> {
        let storage = StorageEngine::open_in_memory()?;

        let headers = vec![("Content-Type".to_string(), "application/json".to_string())];
        let body = Bytes::from("{\"username\": \"admin\"}");
        let req = RequestModel::new("flow-101", "POST", "http://target.local/api/login", "HTTP/1.1", headers, body);
        let mut flow = FlowRecord::new("flow-101", req);

        let res_headers = vec![("Set-Cookie".to_string(), "session=abc".to_string())];
        let res_body = Bytes::from("{\"status\": \"authenticated\"}");
        flow.response = Some(ResponseModel::new("flow-101", 200, "OK", "HTTP/1.1", res_headers, res_body, 12));

        storage.save_flow(&flow)?;

        assert_eq!(storage.count_flows()?, 1);

        let retrieved = storage.get_flow("flow-101")?.expect("Flow not found");
        assert_eq!(retrieved.request.method, "POST");
        assert_eq!(retrieved.request.host, "target.local");
        assert_eq!(retrieved.response.as_ref().unwrap().status, 200);

        let summaries = storage.list_flows(&FlowFilter::all())?;
        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].path, "/api/login");

        Ok(())
    }
}
