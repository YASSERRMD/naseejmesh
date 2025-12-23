//! Mock backend server for testing

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

/// Mock backend state
#[derive(Default)]
pub struct MockState {
    pub requests: RwLock<Vec<RecordedRequest>>,
    pub responses: RwLock<HashMap<String, MockResponse>>,
}

/// Recorded request for verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordedRequest {
    pub path: String,
    pub method: String,
    pub body: Option<String>,
    pub headers: HashMap<String, String>,
    pub timestamp: i64,
}

/// Mock response configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockResponse {
    pub status: u16,
    pub body: String,
    pub delay_ms: Option<u64>,
}

impl Default for MockResponse {
    fn default() -> Self {
        Self {
            status: 200,
            body: r#"{"status": "ok"}"#.to_string(),
            delay_ms: None,
        }
    }
}

/// Mock backend server
pub struct MockBackend {
    state: Arc<MockState>,
    port: u16,
    shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

impl MockBackend {
    /// Create a new mock backend
    pub fn new(port: u16) -> Self {
        Self {
            state: Arc::new(MockState::default()),
            port,
            shutdown_tx: None,
        }
    }

    /// Add a mock response for a path
    pub async fn mock_response(&self, path: impl Into<String>, response: MockResponse) {
        self.state
            .responses
            .write()
            .await
            .insert(path.into(), response);
    }

    /// Get recorded requests
    pub async fn get_requests(&self) -> Vec<RecordedRequest> {
        self.state.requests.read().await.clone()
    }

    /// Clear recorded requests
    pub async fn clear_requests(&self) {
        self.state.requests.write().await.clear();
    }

    /// Start the mock server
    pub async fn start(&mut self) -> Result<(), std::io::Error> {
        let state = self.state.clone();
        let port = self.port;

        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
        self.shutdown_tx = Some(shutdown_tx);

        let app = Router::new()
            .route("/", get(root_handler))
            .route("/health", get(health_handler))
            .route("/api/*path", get(api_handler).post(api_handler))
            .route("/echo", post(echo_handler))
            .route("/delay/:ms", get(delay_handler))
            .route("/status/:code", get(status_handler))
            .with_state(state);

        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
        info!(port = port, "Mock backend started");

        tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    let _ = shutdown_rx.await;
                })
                .await
                .ok();
        });

        // Give server time to start
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        Ok(())
    }

    /// Stop the mock server
    pub fn stop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
    }

    /// Get the base URL
    pub fn url(&self) -> String {
        format!("http://localhost:{}", self.port)
    }
}

async fn root_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({"service": "mock-backend", "status": "running"}))
}

async fn health_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({"healthy": true}))
}

async fn api_handler(
    State(state): State<Arc<MockState>>,
    axum::extract::Path(path): axum::extract::Path<String>,
    body: Option<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    // Record the request
    let request = RecordedRequest {
        path: format!("/api/{}", path),
        method: "GET".to_string(),
        body,
        headers: HashMap::new(),
        timestamp: chrono::Utc::now().timestamp_millis(),
    };
    state.requests.write().await.push(request);

    // Check for mock response
    if let Some(mock) = state.responses.read().await.get(&format!("/api/{}", path)) {
        if let Some(delay) = mock.delay_ms {
            tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
        }
        let status = StatusCode::from_u16(mock.status).unwrap_or(StatusCode::OK);
        let body: serde_json::Value = serde_json::from_str(&mock.body).unwrap_or_default();
        return (status, Json(body));
    }

    (StatusCode::OK, Json(serde_json::json!({"path": path, "received": true})))
}

async fn echo_handler(body: String) -> String {
    body
}

async fn delay_handler(
    axum::extract::Path(ms): axum::extract::Path<u64>,
) -> Json<serde_json::Value> {
    tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
    Json(serde_json::json!({"delayed_ms": ms}))
}

async fn status_handler(
    axum::extract::Path(code): axum::extract::Path<u16>,
) -> (StatusCode, Json<serde_json::Value>) {
    let status = StatusCode::from_u16(code).unwrap_or(StatusCode::OK);
    (status, Json(serde_json::json!({"status_code": code})))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_backend_creation() {
        let backend = MockBackend::new(9999);
        assert_eq!(backend.port, 9999);
    }

    #[tokio::test]
    async fn test_mock_response_config() {
        let backend = MockBackend::new(9998);
        backend.mock_response("/api/test", MockResponse {
            status: 201,
            body: r#"{"created": true}"#.to_string(),
            delay_ms: None,
        }).await;

        let responses = backend.state.responses.read().await;
        assert!(responses.contains_key("/api/test"));
    }
}
