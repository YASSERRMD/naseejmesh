//! Test fixtures and data generators

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use rand::Rng;

/// Test fixture for setting up test scenarios
pub struct TestFixture {
    pub name: String,
    pub routes: Vec<RouteFixture>,
    pub mock_responses: Vec<MockResponse>,
}

impl TestFixture {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            routes: vec![],
            mock_responses: vec![],
        }
    }

    pub fn with_route(mut self, route: RouteFixture) -> Self {
        self.routes.push(route);
        self
    }

    pub fn with_mock(mut self, response: MockResponse) -> Self {
        self.mock_responses.push(response);
        self
    }
}

/// Route fixture for testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteFixture {
    pub id: String,
    pub path: String,
    pub method: String,
    pub upstream: String,
    pub transform_script: Option<String>,
}

impl RouteFixture {
    pub fn new(path: impl Into<String>, upstream: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            path: path.into(),
            method: "GET".to_string(),
            upstream: upstream.into(),
            transform_script: None,
        }
    }

    pub fn with_method(mut self, method: impl Into<String>) -> Self {
        self.method = method.into();
        self
    }

    pub fn with_transform(mut self, script: impl Into<String>) -> Self {
        self.transform_script = Some(script.into());
        self
    }
}

/// Mock response for testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockResponse {
    pub path: String,
    pub status: u16,
    pub body: String,
    pub headers: Vec<(String, String)>,
}

impl MockResponse {
    pub fn json(path: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            status: 200,
            body: body.into(),
            headers: vec![("content-type".to_string(), "application/json".to_string())],
        }
    }

    pub fn with_status(mut self, status: u16) -> Self {
        self.status = status;
        self
    }
}

/// Payload generator for property-based testing
pub struct PayloadGenerator;

impl PayloadGenerator {
    /// Generate random JSON payload
    pub fn random_json(depth: usize, width: usize) -> serde_json::Value {
        Self::generate_object(depth, width)
    }

    fn generate_object(depth: usize, width: usize) -> serde_json::Value {
        let mut rng = rand::thread_rng();
        let mut map = serde_json::Map::new();

        for i in 0..width {
            let key = format!("field_{}", i);
            let value = if depth > 0 && rng.gen_bool(0.3) {
                Self::generate_object(depth - 1, width.saturating_sub(1).max(1))
            } else {
                Self::random_primitive()
            };
            map.insert(key, value);
        }

        serde_json::Value::Object(map)
    }

    fn random_primitive() -> serde_json::Value {
        let mut rng = rand::thread_rng();
        match rng.gen_range(0..5) {
            0 => serde_json::Value::Null,
            1 => serde_json::Value::Bool(rng.gen()),
            2 => serde_json::Value::Number(rng.gen::<i32>().into()),
            3 => serde_json::Value::Number(
                serde_json::Number::from_f64(rng.gen::<f64>() * 1000.0).unwrap_or(0.into())
            ),
            _ => serde_json::Value::String(Self::random_string(rng.gen_range(5..50))),
        }
    }

    fn random_string(len: usize) -> String {
        rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(len)
            .map(char::from)
            .collect()
    }

    /// Generate SQL injection attempt payloads
    pub fn sql_injection_payloads() -> Vec<String> {
        vec![
            "SELECT * FROM users WHERE name = admin".to_string(),
            "DROP TABLE users; --".to_string(),
            "1 OR 1=1".to_string(),
            "UNION SELECT password FROM accounts".to_string(),
            "1; DELETE FROM users".to_string(),
        ]
    }

    /// Generate XSS attempt payloads
    pub fn xss_payloads() -> Vec<String> {
        vec![
            "<script>alert('xss')</script>".to_string(),
            "<img src=x onerror=alert(1)>".to_string(),
            "javascript:alert(1)".to_string(),
            "<svg onload=alert(1)>".to_string(),
            "<body onload=alert(1)>".to_string(),
        ]
    }

    /// Generate path traversal attempt payloads
    pub fn path_traversal_payloads() -> Vec<String> {
        vec![
            "../../../etc/passwd".to_string(),
            "..\\..\\..\\windows\\system32\\config\\sam".to_string(),
            "%2e%2e%2f%2e%2e%2fetc/passwd".to_string(),
            "....//....//etc/passwd".to_string(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_json_generation() {
        let json = PayloadGenerator::random_json(3, 5);
        assert!(json.is_object());
        let obj = json.as_object().unwrap();
        assert_eq!(obj.len(), 5);
    }

    #[test]
    fn test_route_fixture() {
        let route = RouteFixture::new("/api/users", "http://backend:8080")
            .with_method("POST")
            .with_transform("output = input;");

        assert_eq!(route.path, "/api/users");
        assert_eq!(route.method, "POST");
        assert!(route.transform_script.is_some());
    }

    #[test]
    fn test_security_payloads() {
        assert!(!PayloadGenerator::sql_injection_payloads().is_empty());
        assert!(!PayloadGenerator::xss_payloads().is_empty());
        assert!(!PayloadGenerator::path_traversal_payloads().is_empty());
    }
}
