//! HTTP test client for integration tests

use reqwest::{Client, Response, StatusCode};
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TestClientError {
    #[error("Request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),

    #[error("Unexpected status: expected {expected}, got {actual}")]
    UnexpectedStatus { expected: u16, actual: u16 },

    #[error("Response parse failed: {0}")]
    ParseFailed(String),
}

/// HTTP test client with convenience methods
pub struct TestClient {
    client: Client,
    base_url: String,
}

impl TestClient {
    /// Create a new test client
    pub fn new(base_url: impl Into<String>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url: base_url.into(),
        }
    }

    /// Create client pointing to localhost
    pub fn localhost(port: u16) -> Self {
        Self::new(format!("http://localhost:{}", port))
    }

    /// GET request
    pub async fn get(&self, path: &str) -> Result<Response, TestClientError> {
        let url = format!("{}{}", self.base_url, path);
        Ok(self.client.get(&url).send().await?)
    }

    /// POST request with JSON body
    pub async fn post<T: Serialize>(&self, path: &str, body: &T) -> Result<Response, TestClientError> {
        let url = format!("{}{}", self.base_url, path);
        Ok(self.client.post(&url).json(body).send().await?)
    }

    /// POST request with raw body
    pub async fn post_raw(&self, path: &str, body: &str, content_type: &str) -> Result<Response, TestClientError> {
        let url = format!("{}{}", self.base_url, path);
        Ok(self.client
            .post(&url)
            .header("content-type", content_type)
            .body(body.to_string())
            .send()
            .await?)
    }

    /// PUT request with JSON body
    pub async fn put<T: Serialize>(&self, path: &str, body: &T) -> Result<Response, TestClientError> {
        let url = format!("{}{}", self.base_url, path);
        Ok(self.client.put(&url).json(body).send().await?)
    }

    /// DELETE request
    pub async fn delete(&self, path: &str) -> Result<Response, TestClientError> {
        let url = format!("{}{}", self.base_url, path);
        Ok(self.client.delete(&url).send().await?)
    }

    /// GET and parse JSON response
    pub async fn get_json<T: DeserializeOwned>(&self, path: &str) -> Result<T, TestClientError> {
        let resp = self.get(path).await?;
        resp.json().await.map_err(TestClientError::from)
    }

    /// POST and parse JSON response
    pub async fn post_json<R: Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &R,
    ) -> Result<T, TestClientError> {
        let resp = self.post(path, body).await?;
        resp.json().await.map_err(TestClientError::from)
    }

    /// Health check
    pub async fn health_check(&self) -> Result<bool, TestClientError> {
        match self.get("/_gateway/health").await {
            Ok(resp) => Ok(resp.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    /// Wait for service to be ready
    pub async fn wait_for_ready(&self, timeout: Duration) -> Result<(), TestClientError> {
        let start = std::time::Instant::now();
        let check_interval = Duration::from_millis(100);

        while start.elapsed() < timeout {
            if self.health_check().await? {
                return Ok(());
            }
            tokio::time::sleep(check_interval).await;
        }

        Err(TestClientError::ParseFailed("Service not ready within timeout".to_string()))
    }

    /// Expect specific status code
    pub async fn expect_status(&self, path: &str, expected: StatusCode) -> Result<Response, TestClientError> {
        let resp = self.get(path).await?;
        if resp.status() != expected {
            return Err(TestClientError::UnexpectedStatus {
                expected: expected.as_u16(),
                actual: resp.status().as_u16(),
            });
        }
        Ok(resp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = TestClient::localhost(8080);
        assert!(client.base_url.contains("8080"));
    }
}
