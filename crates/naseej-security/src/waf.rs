//! Web Application Firewall (WAF)
//!
//! High-performance pattern matching for security threats.
//! Uses compiled regex patterns for SQL injection, XSS, and other attacks.

use regex::RegexSet;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, warn};

/// WAF errors
#[derive(Debug, Error)]
pub enum WafError {
    #[error("Pattern compilation failed: {0}")]
    PatternError(String),

    #[error("Request blocked: {reason}")]
    Blocked { reason: String, rule_id: String },
}

/// WAF check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WafResult {
    pub allowed: bool,
    pub triggered_rule: Option<String>,
    pub category: Option<String>,
    pub scan_time_us: u64,
}

/// WAF configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WafConfig {
    pub enabled: bool,
    pub mode: WafMode,
    pub max_body_size: usize,
    #[serde(default)]
    pub custom_patterns: Vec<CustomPattern>,
}

impl Default for WafConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            mode: WafMode::Block,
            max_body_size: 1024 * 1024,
            custom_patterns: vec![],
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum WafMode {
    Block,
    DetectOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomPattern {
    pub id: String,
    pub pattern: String,
    pub category: String,
    pub severity: Severity,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

/// WAF Engine with compiled patterns
pub struct WafEngine {
    config: WafConfig,
    sql_injection: RegexSet,
    xss: RegexSet,
    path_traversal: RegexSet,
    command_injection: RegexSet,
    custom: Option<RegexSet>,
}

impl WafEngine {
    /// Create a new WAF engine with OWASP-inspired rules
    pub fn new(config: WafConfig) -> Result<Self, WafError> {
        let sql_patterns = vec![
            r"(?i)(\b(select|insert|update|delete|drop|union)\b.*\b(from|into|where|set)\b)",
            r"(?i)(--|#|/\*|\*/)",
            r"(?i)(\b(or|and)\b\s+\d+\s*=\s*\d+)",
            r"(?i)(union\s+(all\s+)?select)",
        ];

        let xss_patterns = vec![
            r"(?i)(<script)",
            r"(?i)(javascript\s*:)",
            r"(?i)(on(load|error|click)\s*=)",
            r"(?i)(eval\s*\()",
        ];

        let path_patterns = vec![
            r"(\.\.\/|\.\.\\)",
            r"(?i)(/etc/passwd)",
            r"(?i)(%2e%2e%2f)",
        ];

        let cmd_patterns = vec![
            r"(\||;|\$\(|`)",
            r"(?i)(\b(cat|ls|whoami|id)\b)",
            r"(?i)(/bin/(sh|bash))",
        ];

        let sql_injection = RegexSet::new(&sql_patterns)
            .map_err(|e| WafError::PatternError(e.to_string()))?;
        let xss = RegexSet::new(&xss_patterns)
            .map_err(|e| WafError::PatternError(e.to_string()))?;
        let path_traversal = RegexSet::new(&path_patterns)
            .map_err(|e| WafError::PatternError(e.to_string()))?;
        let command_injection = RegexSet::new(&cmd_patterns)
            .map_err(|e| WafError::PatternError(e.to_string()))?;

        let custom = if !config.custom_patterns.is_empty() {
            let patterns: Vec<&str> = config
                .custom_patterns
                .iter()
                .map(|p| p.pattern.as_str())
                .collect();
            Some(RegexSet::new(&patterns).map_err(|e| WafError::PatternError(e.to_string()))?)
        } else {
            None
        };

        Ok(Self {
            config,
            sql_injection,
            xss,
            path_traversal,
            command_injection,
            custom,
        })
    }

    /// Scan a request payload
    pub fn scan(&self, payload: &str) -> WafResult {
        if !self.config.enabled {
            return WafResult {
                allowed: true,
                triggered_rule: None,
                category: None,
                scan_time_us: 0,
            };
        }

        let start = std::time::Instant::now();
        let content = if payload.len() > self.config.max_body_size {
            &payload[..self.config.max_body_size]
        } else {
            payload
        };

        if let Some((rule_id, category)) = self.check_patterns(content) {
            let scan_time_us = start.elapsed().as_micros() as u64;
            warn!(rule = %rule_id, category = %category, "WAF threat detected");
            return WafResult {
                allowed: self.config.mode == WafMode::DetectOnly,
                triggered_rule: Some(rule_id),
                category: Some(category),
                scan_time_us,
            };
        }

        let scan_time_us = start.elapsed().as_micros() as u64;
        debug!(scan_time_us = scan_time_us, "WAF scan complete");

        WafResult {
            allowed: true,
            triggered_rule: None,
            category: None,
            scan_time_us,
        }
    }

    fn check_patterns(&self, content: &str) -> Option<(String, String)> {
        if self.sql_injection.is_match(content) {
            return Some(("SQL-1".to_string(), "SQL Injection".to_string()));
        }
        if self.xss.is_match(content) {
            return Some(("XSS-1".to_string(), "Cross-Site Scripting".to_string()));
        }
        if self.path_traversal.is_match(content) {
            return Some(("PATH-1".to_string(), "Path Traversal".to_string()));
        }
        if self.command_injection.is_match(content) {
            return Some(("CMD-1".to_string(), "Command Injection".to_string()));
        }
        if let Some(ref custom) = self.custom {
            if custom.is_match(content) {
                if let Some(p) = self.config.custom_patterns.first() {
                    return Some((p.id.clone(), p.category.clone()));
                }
            }
        }
        None
    }

    pub fn scan_path(&self, path: &str) -> WafResult {
        self.scan(path)
    }

    pub fn scan_query(&self, query: &str) -> WafResult {
        self.scan(query)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sql_injection_detection() {
        let engine = WafEngine::new(WafConfig::default()).unwrap();
        let result = engine.scan("SELECT id FROM tbl WHERE x=1 OR 1=1");
        assert!(!result.allowed);
        assert!(result.category.as_ref().unwrap().contains("SQL"));
    }

    #[test]
    fn test_xss_detection() {
        let engine = WafEngine::new(WafConfig::default()).unwrap();
        let result = engine.scan("<script>alert(1)</script>");
        assert!(!result.allowed);
        assert!(result.category.is_some());
    }

    #[test]
    fn test_path_traversal() {
        let engine = WafEngine::new(WafConfig::default()).unwrap();
        let result = engine.scan("../../../etc/passwd");
        assert!(!result.allowed);
    }

    #[test]
    fn test_normal_request() {
        let engine = WafEngine::new(WafConfig::default()).unwrap();
        let result = engine.scan("Hello, this is a normal message");
        assert!(result.allowed);
    }

    #[test]
    fn test_disabled_waf() {
        let config = WafConfig { enabled: false, ..Default::default() };
        let engine = WafEngine::new(config).unwrap();
        let result = engine.scan("DROP TABLE users");
        assert!(result.allowed);
    }
}
