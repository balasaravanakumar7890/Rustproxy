use bytes::Bytes;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TlsInfo {
    pub version: String,
    pub cipher_suite: Option<String>,
    pub sni: Option<String>,
    pub alpn: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestModel {
    pub id: String,
    pub method: String,
    pub uri: String,
    pub version: String,
    pub headers: Vec<(String, String)>,
    #[serde(
        default,
        serialize_with = "serialize_bytes_string_lossy",
        deserialize_with = "deserialize_bytes_from_string"
    )]
    pub body: Bytes,
    pub host: String,
    pub port: u16,
    pub scheme: String,
    pub path: String,
    pub query: Option<String>,
    pub body_length: usize,
}

impl RequestModel {
    pub fn new(
        id: impl Into<String>,
        method: impl Into<String>,
        uri: impl Into<String>,
        version: impl Into<String>,
        headers: Vec<(String, String)>,
        body: Bytes,
    ) -> Self {
        let uri_str = uri.into();
        let parsed_url = url::Url::parse(&uri_str).ok();

        let scheme = parsed_url
            .as_ref()
            .map(|u| u.scheme().to_string())
            .unwrap_or_else(|| "http".to_string());

        let host = parsed_url
            .as_ref()
            .and_then(|u| u.host_str())
            .map(|h| h.to_string())
            .or_else(|| {
                headers.iter().find(|(k, _)| k.eq_ignore_ascii_case("host")).map(|(_, v)| {
                    v.split(':').next().unwrap_or(v).to_string()
                })
            })
            .unwrap_or_else(|| "localhost".to_string());

        let port = parsed_url
            .as_ref()
            .and_then(|u| u.port())
            .unwrap_or_else(|| {
                if scheme == "https" {
                    443
                } else {
                    headers.iter().find(|(k, _)| k.eq_ignore_ascii_case("host")).and_then(|(_, v)| {
                        v.split(':').nth(1).and_then(|p| p.parse::<u16>().ok())
                    }).unwrap_or(80)
                }
            });

        let path = parsed_url
            .as_ref()
            .map(|u| u.path().to_string())
            .unwrap_or_else(|| {
                if uri_str.starts_with('/') {
                    uri_str.split('?').next().unwrap_or("/").to_string()
                } else {
                    "/".to_string()
                }
            });

        let query = parsed_url
            .as_ref()
            .and_then(|u| u.query().map(|q| q.to_string()))
            .or_else(|| {
                uri_str.split_once('?').map(|(_, q)| q.to_string())
            });

        let body_length = body.len();

        Self {
            id: id.into(),
            method: method.into(),
            uri: uri_str,
            version: version.into(),
            headers,
            body,
            host,
            port,
            scheme,
            path,
            query,
            body_length,
        }
    }

    pub fn get_header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }

    pub fn set_header(&mut self, name: &str, value: &str) {
        if let Some(pos) = self.headers.iter().position(|(k, _)| k.eq_ignore_ascii_case(name)) {
            self.headers[pos] = (name.to_string(), value.to_string());
        } else {
            self.headers.push((name.to_string(), value.to_string()));
        }
    }

    pub fn remove_header(&mut self, name: &str) {
        self.headers.retain(|(k, _)| !k.eq_ignore_ascii_case(name));
    }

    pub fn content_type(&self) -> Option<&str> {
        self.get_header("content-type")
    }

    pub fn body_as_string_lossy(&self) -> String {
        String::from_utf8_lossy(&self.body).to_string()
    }

    pub fn to_raw_wire(&self) -> Vec<u8> {
        let mut wire = Vec::new();
        let target_path = if let Some(q) = &self.query {
            format!("{}?{}", self.path, q)
        } else {
            self.path.clone()
        };
        wire.extend_from_slice(format!("{} {} {}\r\n", self.method, target_path, self.version).as_bytes());
        for (k, v) in &self.headers {
            wire.extend_from_slice(format!("{}: {}\r\n", k, v).as_bytes());
        }
        wire.extend_from_slice(b"\r\n");
        wire.extend_from_slice(&self.body);
        wire
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseModel {
    pub id: String,
    pub status: u16,
    pub status_text: String,
    pub version: String,
    pub headers: Vec<(String, String)>,
    #[serde(
        default,
        serialize_with = "serialize_bytes_string_lossy",
        deserialize_with = "deserialize_bytes_from_string"
    )]
    pub body: Bytes,
    pub body_length: usize,
    pub mime_type: Option<String>,
    pub duration_ms: u64,
}

impl ResponseModel {
    pub fn new(
        id: impl Into<String>,
        status: u16,
        status_text: impl Into<String>,
        version: impl Into<String>,
        headers: Vec<(String, String)>,
        body: Bytes,
        duration_ms: u64,
    ) -> Self {
        let mime_type = headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case("content-type"))
            .map(|(_, v)| v.split(';').next().unwrap_or(v).trim().to_string());

        let body_length = body.len();

        Self {
            id: id.into(),
            status,
            status_text: status_text.into(),
            version: version.into(),
            headers,
            body,
            body_length,
            mime_type,
            duration_ms,
        }
    }

    pub fn get_header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }

    pub fn set_header(&mut self, name: &str, value: &str) {
        if let Some(pos) = self.headers.iter().position(|(k, _)| k.eq_ignore_ascii_case(name)) {
            self.headers[pos] = (name.to_string(), value.to_string());
        } else {
            self.headers.push((name.to_string(), value.to_string()));
        }
    }

    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }

    pub fn is_redirect(&self) -> bool {
        (300..400).contains(&self.status)
    }

    pub fn is_error(&self) -> bool {
        (400..600).contains(&self.status)
    }

    pub fn body_as_string_lossy(&self) -> String {
        String::from_utf8_lossy(&self.body).to_string()
    }

    pub fn to_raw_wire(&self) -> Vec<u8> {
        let mut wire = Vec::new();
        wire.extend_from_slice(format!("{} {} {}\r\n", self.version, self.status, self.status_text).as_bytes());
        for (k, v) in &self.headers {
            wire.extend_from_slice(format!("{}: {}\r\n", k, v).as_bytes());
        }
        wire.extend_from_slice(b"\r\n");
        wire.extend_from_slice(&self.body);
        wire
    }
}

pub fn serialize_bytes_string_lossy<S>(bytes: &Bytes, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&String::from_utf8_lossy(bytes))
}

pub fn deserialize_bytes_from_string<'de, D>(deserializer: D) -> Result<Bytes, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt: Option<String> = serde::Deserialize::deserialize(deserializer)?;
    Ok(opt.map(|s| Bytes::from(s.into_bytes())).unwrap_or_else(Bytes::new))
}
