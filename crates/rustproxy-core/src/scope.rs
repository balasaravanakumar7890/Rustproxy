use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScopeAction {
    Include,
    Exclude,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeRule {
    pub id: String,
    pub enabled: bool,
    pub action: ScopeAction,
    pub protocol: Option<String>,
    pub host_pattern: String,
    pub port: Option<u16>,
    pub path_pattern: Option<String>,
    #[serde(skip)]
    host_regex: Option<Regex>,
    #[serde(skip)]
    path_regex: Option<Regex>,
}

impl ScopeRule {
    pub fn new(
        id: impl Into<String>,
        action: ScopeAction,
        host_pattern: impl Into<String>,
    ) -> Result<Self, regex::Error> {
        let hp = host_pattern.into();
        let host_re = Regex::new(&hp)?;

        Ok(Self {
            id: id.into(),
            enabled: true,
            action,
            protocol: None,
            host_pattern: hp,
            port: None,
            path_pattern: None,
            host_regex: Some(host_re),
            path_regex: None,
        })
    }

    pub fn with_path(mut self, path_pattern: impl Into<String>) -> Result<Self, regex::Error> {
        let pp = path_pattern.into();
        let path_re = Regex::new(&pp)?;
        self.path_pattern = Some(pp);
        self.path_regex = Some(path_re);
        Ok(self)
    }

    pub fn matches(&mut self, scheme: &str, host: &str, port: u16, path: &str) -> bool {
        if !self.enabled {
            return false;
        }

        if let Some(proto) = &self.protocol {
            if !proto.eq_ignore_ascii_case(scheme) {
                return false;
            }
        }

        if let Some(p) = self.port {
            if p != port {
                return false;
            }
        }

        if self.host_regex.is_none() {
            self.host_regex = Regex::new(&self.host_pattern).ok();
        }

        if let Some(re) = &self.host_regex {
            if !re.is_match(host) {
                return false;
            }
        }

        if let Some(pattern) = &self.path_pattern {
            if self.path_regex.is_none() {
                self.path_regex = Regex::new(pattern).ok();
            }
            if let Some(re) = &self.path_regex {
                if !re.is_match(path) {
                    return false;
                }
            }
        }

        true
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScopeConfig {
    pub rules: Vec<ScopeRule>,
    pub enabled: bool,
}

impl ScopeConfig {
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            enabled: true,
        }
    }

    pub fn is_in_scope(&mut self, scheme: &str, host: &str, port: u16, path: &str) -> bool {
        if !self.enabled || self.rules.is_empty() {
            return true;
        }

        let mut included = false;

        for rule in &mut self.rules {
            if rule.matches(scheme, host, port, path) {
                match rule.action {
                    ScopeAction::Include => included = true,
                    ScopeAction::Exclude => return false,
                }
            }
        }

        included
    }
}
