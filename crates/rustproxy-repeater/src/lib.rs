use anyhow::{anyhow, Result};
use rustproxy_core::{RequestModel, ResponseModel};
use rustproxy_http::HttpClient;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepeaterTab {
    pub id: String,
    pub name: String,
    pub request: RequestModel,
    pub response: Option<ResponseModel>,
    pub history: Vec<(RequestModel, Option<ResponseModel>)>,
    pub follow_redirects: bool,
    pub is_in_flight: bool,
}

impl RepeaterTab {
    pub fn new(id: impl Into<String>, name: impl Into<String>, request: RequestModel) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            request,
            response: None,
            history: Vec::new(),
            follow_redirects: true,
            is_in_flight: false,
        }
    }
}

pub struct RepeaterEngine {
    tabs: HashMap<String, RepeaterTab>,
    tab_order: Vec<String>,
    client: Arc<HttpClient>,
}

impl RepeaterEngine {
    pub fn new(client: Arc<HttpClient>) -> Self {
        Self {
            tabs: HashMap::new(),
            tab_order: Vec::new(),
            client,
        }
    }

    pub fn create_tab(&mut self, name: Option<String>, req: Option<RequestModel>) -> String {
        let id = Uuid::new_v4().to_string();
        let tab_num = self.tab_order.len() + 1;
        let tab_name = name.unwrap_or_else(|| format!("Tab {}", tab_num));

        let request = req.unwrap_or_else(|| {
            RequestModel::new(
                &id,
                "GET",
                "https://localhost/",
                "HTTP/1.1",
                vec![("Host".to_string(), "localhost".to_string())],
                bytes::Bytes::new(),
            )
        });

        let tab = RepeaterTab::new(&id, tab_name, request);
        self.tabs.insert(id.clone(), tab);
        self.tab_order.push(id.clone());
        id
    }

    pub fn duplicate_tab(&mut self, tab_id: &str) -> Option<String> {
        if let Some(tab) = self.tabs.get(tab_id) {
            let new_id = Uuid::new_v4().to_string();
            let new_name = format!("{} (Copy)", tab.name);
            let mut new_req = tab.request.clone();
            new_req.id = new_id.clone();

            let new_tab = RepeaterTab::new(&new_id, new_name, new_req);
            self.tabs.insert(new_id.clone(), new_tab);
            self.tab_order.push(new_id.clone());
            Some(new_id)
        } else {
            None
        }
    }

    pub fn close_tab(&mut self, tab_id: &str) {
        self.tabs.remove(tab_id);
        self.tab_order.retain(|id| id != tab_id);
    }

    pub fn get_tab(&self, tab_id: &str) -> Option<&RepeaterTab> {
        self.tabs.get(tab_id)
    }

    pub fn get_tab_mut(&mut self, tab_id: &str) -> Option<&mut RepeaterTab> {
        self.tabs.get_mut(tab_id)
    }

    pub fn update_tab_request(&mut self, tab_id: &str, req: RequestModel) -> bool {
        if let Some(tab) = self.tabs.get_mut(tab_id) {
            tab.request = req;
            true
        } else {
            false
        }
    }

    pub fn list_tabs(&self) -> Vec<RepeaterTab> {
        self.tab_order
            .iter()
            .filter_map(|id| self.tabs.get(id).cloned())
            .collect()
    }

    pub async fn send_tab(&mut self, tab_id: &str) -> Result<ResponseModel> {
        let (req_to_send, follow_redirects) = {
            let tab = self
                .tabs
                .get_mut(tab_id)
                .ok_or_else(|| anyhow!("Tab not found: {}", tab_id))?;
            tab.is_in_flight = true;
            (tab.request.clone(), tab.follow_redirects)
        };

        let mut current_req = req_to_send;
        let mut redirect_count = 0;
        const MAX_REDIRECTS: usize = 5;

        loop {
            let res = self.client.send(&current_req).await;

            match res {
                Ok(response) => {
                    if follow_redirects && response.is_redirect() && redirect_count < MAX_REDIRECTS {
                        if let Some(loc) = response.get_header("location") {
                            let next_uri = if loc.starts_with("http://") || loc.starts_with("https://") {
                                loc.to_string()
                            } else {
                                format!("{}://{}:{}{}", current_req.scheme, current_req.host, current_req.port, loc)
                            };

                            let next_req = RequestModel::new(
                                &current_req.id,
                                "GET",
                                next_uri,
                                &current_req.version,
                                current_req.headers.clone(),
                                bytes::Bytes::new(),
                            );
                            current_req = next_req;
                            redirect_count += 1;
                            continue;
                        }
                    }

                    if let Some(tab) = self.tabs.get_mut(tab_id) {
                        tab.is_in_flight = false;
                        tab.history.push((tab.request.clone(), Some(response.clone())));
                        tab.response = Some(response.clone());
                    }

                    return Ok(response);
                }
                Err(e) => {
                    if let Some(tab) = self.tabs.get_mut(tab_id) {
                        tab.is_in_flight = false;
                    }
                    return Err(e);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repeater_tabs_lifecycle() {
        let client = Arc::new(HttpClient::new_insecure().unwrap());
        let mut repeater = RepeaterEngine::new(client);

        let tab1_id = repeater.create_tab(Some("Login Endpoint".to_string()), None);
        assert_eq!(repeater.list_tabs().len(), 1);

        let dup_id = repeater.duplicate_tab(&tab1_id).expect("Duplicate failed");
        assert_eq!(repeater.list_tabs().len(), 2);

        repeater.close_tab(&tab1_id);
        assert_eq!(repeater.list_tabs().len(), 1);
        assert_eq!(repeater.list_tabs()[0].id, dup_id);
    }
}
