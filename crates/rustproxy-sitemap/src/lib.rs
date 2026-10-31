use rustproxy_core::{FlowRecord, ScopeConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteMapNode {
    pub id: String,
    pub name: String,
    pub path: String,
    pub is_endpoint: bool,
    pub flow_ids: Vec<String>,
    pub methods: Vec<String>,
    pub status_codes: Vec<u16>,
    pub children: HashMap<String, SiteMapNode>,
}

impl SiteMapNode {
    pub fn new(id: impl Into<String>, name: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            path: path.into(),
            is_endpoint: false,
            flow_ids: Vec::new(),
            methods: Vec::new(),
            status_codes: Vec::new(),
            children: HashMap::new(),
        }
    }
}

pub struct SiteMap {
    hosts: HashMap<String, SiteMapNode>,
    scope: ScopeConfig,
}

impl SiteMap {
    pub fn new() -> Self {
        Self {
            hosts: HashMap::new(),
            scope: ScopeConfig::new(),
        }
    }

    pub fn set_scope(&mut self, scope: ScopeConfig) {
        self.scope = scope;
    }

    pub fn scope_mut(&mut self) -> &mut ScopeConfig {
        &mut self.scope
    }

    pub fn is_in_scope(&mut self, scheme: &str, host: &str, port: u16, path: &str) -> bool {
        self.scope.is_in_scope(scheme, host, port, path)
    }

    pub fn add_flow(&mut self, flow: &FlowRecord) {
        let host_key = format!("{}://{}:{}", flow.request.scheme, flow.request.host, flow.request.port);
        let host_node = self.hosts.entry(host_key.clone()).or_insert_with(|| {
            SiteMapNode::new(host_key.clone(), host_key.clone(), "/")
        });

        // Split path into segments
        let segments: Vec<&str> = flow
            .request
            .path
            .split('/')
            .filter(|s| !s.is_empty())
            .collect();

        let mut current = host_node;
        let mut accumulated_path = String::new();

        for (i, seg) in segments.iter().enumerate() {
            accumulated_path.push('/');
            accumulated_path.push_str(seg);

            let is_last = i == segments.len() - 1;
            let next_node = current
                .children
                .entry(seg.to_string())
                .or_insert_with(|| SiteMapNode::new(accumulated_path.clone(), *seg, accumulated_path.clone()));

            if is_last {
                next_node.is_endpoint = true;
                if !next_node.flow_ids.contains(&flow.id) {
                    next_node.flow_ids.push(flow.id.clone());
                }
                if !next_node.methods.contains(&flow.request.method) {
                    next_node.methods.push(flow.request.method.clone());
                }
                if let Some(res) = &flow.response {
                    if !next_node.status_codes.contains(&res.status) {
                        next_node.status_codes.push(res.status);
                    }
                }
            }

            current = next_node;
        }
    }

    pub fn get_hosts(&self) -> Vec<SiteMapNode> {
        self.hosts.values().cloned().collect()
    }
}

impl Default for SiteMap {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use rustproxy_core::{RequestModel, ResponseModel};

    #[test]
    fn test_sitemap_hierarchy_aggregation() {
        let mut sitemap = SiteMap::new();

        let req1 = RequestModel::new("f-1", "GET", "https://api.test.local/v1/users", "HTTP/1.1", vec![], Bytes::new());
        let mut flow1 = FlowRecord::new("f-1", req1);
        flow1.response = Some(ResponseModel::new("f-1", 200, "OK", "HTTP/1.1", vec![], Bytes::new(), 10));

        let req2 = RequestModel::new("f-2", "POST", "https://api.test.local/v1/users", "HTTP/1.1", vec![], Bytes::new());
        let mut flow2 = FlowRecord::new("f-2", req2);
        flow2.response = Some(ResponseModel::new("f-2", 201, "Created", "HTTP/1.1", vec![], Bytes::new(), 15));

        sitemap.add_flow(&flow1);
        sitemap.add_flow(&flow2);

        let hosts = sitemap.get_hosts();
        assert_eq!(hosts.len(), 1);
        let host = &hosts[0];
        assert_eq!(host.name, "https://api.test.local:443");

        let v1 = host.children.get("v1").expect("Missing v1 segment");
        let users = v1.children.get("users").expect("Missing users segment");
        assert!(users.is_endpoint);
        assert_eq!(users.flow_ids.len(), 2);
        assert_eq!(users.methods, vec!["GET", "POST"]);
        assert_eq!(users.status_codes, vec![200, 201]);
    }
}
