//! Dynamic Listener Management (Supervisor Pattern)
//!
//! The ListenerManager spawns and manages protocol listeners based on
//! configuration from SurrealDB. It supports:
//! - Dynamic spawning of new listeners
//! - Graceful shutdown with CancellationToken
//! - Configuration diffing for hot updates

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use arc_swap::ArcSwap;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::{info, warn, error, debug};

use crate::context::ProtocolType;

/// Configuration for a single listener
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ListenerConfig {
    /// Unique identifier for this listener
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// Protocol type (http, mqtt, grpc, soap)
    pub protocol: ProtocolType,

    /// Port to bind
    pub port: u16,

    /// Host to bind (default: 0.0.0.0)
    #[serde(default = "default_host")]
    pub host: String,

    /// Whether this listener is enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Protocol-specific configuration (JSON)
    #[serde(default)]
    pub config: serde_json::Value,

    /// Graceful shutdown timeout in seconds
    #[serde(default = "default_drain_timeout")]
    pub drain_timeout_secs: u64,
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_enabled() -> bool {
    true
}

fn default_drain_timeout() -> u64 {
    30
}

impl ListenerConfig {
    /// Create a new listener config
    pub fn new(id: impl Into<String>, protocol: ProtocolType, port: u16) -> Self {
        Self {
            id: id.into(),
            name: String::new(),
            protocol,
            port,
            host: default_host(),
            enabled: true,
            config: serde_json::Value::Null,
            drain_timeout_secs: default_drain_timeout(),
        }
    }

    /// Socket address string
    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

/// Complete service configuration (all listeners)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServiceConfig {
    /// All configured listeners
    pub listeners: Vec<ListenerConfig>,
}

impl ServiceConfig {
    /// Get a listener by ID
    pub fn get_listener(&self, id: &str) -> Option<&ListenerConfig> {
        self.listeners.iter().find(|l| l.id == id)
    }

    /// Get all enabled listeners
    pub fn enabled_listeners(&self) -> impl Iterator<Item = &ListenerConfig> {
        self.listeners.iter().filter(|l| l.enabled)
    }
}

/// Handle to a running listener
struct ListenerHandle {
    /// Cancellation token for graceful shutdown
    cancel_token: CancellationToken,

    /// Join handle for the listener task
    join_handle: JoinHandle<()>,

    /// The configuration this listener was started with
    config: ListenerConfig,
}

/// Event types for listener lifecycle
#[derive(Debug, Clone)]
pub enum ListenerEvent {
    /// A new listener was started
    Started { id: String, protocol: ProtocolType, port: u16 },

    /// A listener was stopped
    Stopped { id: String },

    /// A listener was restarted with new config
    Restarted { id: String },

    /// An error occurred
    Error { id: String, message: String },
}

/// Manager for dynamic protocol listeners
///
/// The ListenerManager watches for configuration changes and spawns/kills
/// listener tasks accordingly. It uses CancellationToken for graceful shutdown.
pub struct ListenerManager {
    /// Active listeners keyed by their ID
    listeners: HashMap<String, ListenerHandle>,

    /// Shared configuration (updated by watcher)
    config: Arc<ArcSwap<ServiceConfig>>,

    /// Event broadcaster for listener lifecycle events
    event_tx: broadcast::Sender<ListenerEvent>,

    /// Master cancellation token (for shutdown)
    master_cancel: CancellationToken,
}

impl ListenerManager {
    /// Create a new ListenerManager
    pub fn new(config: Arc<ArcSwap<ServiceConfig>>) -> Self {
        let (event_tx, _) = broadcast::channel(100);
        Self {
            listeners: HashMap::new(),
            config,
            event_tx,
            master_cancel: CancellationToken::new(),
        }
    }

    /// Subscribe to listener lifecycle events
    pub fn subscribe(&self) -> broadcast::Receiver<ListenerEvent> {
        self.event_tx.subscribe()
    }

    /// Get the current configuration
    pub fn current_config(&self) -> Arc<ServiceConfig> {
        self.config.load_full()
    }

    /// Reconcile running listeners with the current configuration
    ///
    /// This is the core "diff and apply" logic:
    /// 1. Stop listeners that are no longer in config or disabled
    /// 2. Start new listeners that are in config but not running
    /// 3. Restart listeners whose config has changed
    pub async fn reconcile(&mut self) {
        let config = self.config.load();
        
        // Build set of expected listener IDs
        let expected: HashMap<String, &ListenerConfig> = config
            .enabled_listeners()
            .map(|l| (l.id.clone(), l))
            .collect();

        // Find listeners to stop (in running but not in expected, or disabled)
        let to_stop: Vec<String> = self
            .listeners
            .keys()
            .filter(|id| !expected.contains_key(*id))
            .cloned()
            .collect();

        // Stop removed listeners
        for id in to_stop {
            self.stop_listener(&id).await;
        }

        // Find listeners to start or restart
        for (id, listener_config) in expected {
            if let Some(handle) = self.listeners.get(&id) {
                // Check if config changed
                if handle.config != *listener_config {
                    info!(id = %id, "Listener config changed, restarting");
                    self.stop_listener(&id).await;
                    self.start_listener(listener_config.clone()).await;
                    let _ = self.event_tx.send(ListenerEvent::Restarted { id });
                }
            } else {
                // New listener
                self.start_listener(listener_config.clone()).await;
            }
        }
    }

    /// Start a single listener
    async fn start_listener(&mut self, config: ListenerConfig) {
        let id = config.id.clone();
        let protocol = config.protocol;
        let port = config.port;

        info!(
            id = %id,
            protocol = %protocol,
            port = port,
            "Starting listener"
        );

        let cancel_token = self.master_cancel.child_token();
        let listener_config = config.clone();
        let cancel_clone = cancel_token.clone();

        // Spawn the listener task
        let join_handle = tokio::spawn(async move {
            Self::run_listener(listener_config, cancel_clone).await;
        });

        let handle = ListenerHandle {
            cancel_token,
            join_handle,
            config,
        };

        self.listeners.insert(id.clone(), handle);

        let _ = self.event_tx.send(ListenerEvent::Started { id, protocol, port });
    }

    /// Stop a single listener gracefully
    async fn stop_listener(&mut self, id: &str) {
        if let Some(handle) = self.listeners.remove(id) {
            info!(id = %id, "Stopping listener");

            // Signal cancellation
            handle.cancel_token.cancel();

            // Wait for graceful shutdown with timeout
            let timeout = Duration::from_secs(handle.config.drain_timeout_secs);
            match tokio::time::timeout(timeout, handle.join_handle).await {
                Ok(Ok(())) => {
                    debug!(id = %id, "Listener stopped gracefully");
                }
                Ok(Err(e)) => {
                    warn!(id = %id, error = %e, "Listener task panicked");
                }
                Err(_) => {
                    warn!(id = %id, "Listener shutdown timed out, forcing");
                }
            }

            let _ = self.event_tx.send(ListenerEvent::Stopped { id: id.to_string() });
        }
    }

    /// Run the listener (protocol-specific logic)
    async fn run_listener(config: ListenerConfig, cancel: CancellationToken) {
        debug!(
            id = %config.id,
            protocol = %config.protocol,
            address = %config.bind_address(),
            "Listener task starting"
        );

        // This is where protocol-specific listeners will be implemented
        // For now, we just wait for cancellation
        match config.protocol {
            ProtocolType::Http => {
                Self::run_http_listener(config, cancel).await;
            }
            ProtocolType::Mqtt => {
                Self::run_mqtt_listener(config, cancel).await;
            }
            ProtocolType::Grpc => {
                Self::run_grpc_listener(config, cancel).await;
            }
            ProtocolType::Soap => {
                // SOAP runs on HTTP, delegate to HTTP listener
                Self::run_http_listener(config, cancel).await;
            }
        }
    }

    /// HTTP listener placeholder (implemented in Phase 1, will be wired here)
    async fn run_http_listener(config: ListenerConfig, cancel: CancellationToken) {
        info!(id = %config.id, "HTTP listener ready on {}", config.bind_address());
        
        // Wait for cancellation
        cancel.cancelled().await;
        
        info!(id = %config.id, "HTTP listener shutting down");
    }

    /// MQTT listener placeholder (implemented below)
    async fn run_mqtt_listener(config: ListenerConfig, cancel: CancellationToken) {
        info!(id = %config.id, "MQTT listener ready on {}", config.bind_address());
        
        // Wait for cancellation
        cancel.cancelled().await;
        
        info!(id = %config.id, "MQTT listener shutting down");
    }

    /// gRPC listener placeholder (implemented below)
    async fn run_grpc_listener(config: ListenerConfig, cancel: CancellationToken) {
        info!(id = %config.id, "gRPC listener ready on {}", config.bind_address());
        
        // Wait for cancellation
        cancel.cancelled().await;
        
        info!(id = %config.id, "gRPC listener shutting down");
    }

    /// Shutdown all listeners
    pub async fn shutdown(&mut self) {
        info!("Shutting down all listeners");
        
        // Cancel master token (cancels all child tokens)
        self.master_cancel.cancel();

        // Wait for all listeners to stop
        let ids: Vec<String> = self.listeners.keys().cloned().collect();
        for id in ids {
            self.stop_listener(&id).await;
        }
    }

    /// Get the number of running listeners
    pub fn listener_count(&self) -> usize {
        self.listeners.len()
    }

    /// Check if a specific listener is running
    pub fn is_running(&self, id: &str) -> bool {
        self.listeners.contains_key(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_listener_config() {
        let config = ListenerConfig::new("http-main", ProtocolType::Http, 8080);
        assert_eq!(config.bind_address(), "0.0.0.0:8080");
        assert!(config.enabled);
    }

    #[test]
    fn test_service_config() {
        let mut service = ServiceConfig::default();
        service.listeners.push(ListenerConfig::new("http-1", ProtocolType::Http, 8080));
        service.listeners.push({
            let mut l = ListenerConfig::new("http-2", ProtocolType::Http, 8081);
            l.enabled = false;
            l
        });

        assert_eq!(service.enabled_listeners().count(), 1);
        assert!(service.get_listener("http-1").is_some());
        assert!(service.get_listener("http-2").is_some());
        assert!(service.get_listener("http-3").is_none());
    }

    #[tokio::test]
    async fn test_listener_manager_creation() {
        let config = Arc::new(ArcSwap::from_pointee(ServiceConfig::default()));
        let manager = ListenerManager::new(config);
        
        assert_eq!(manager.listener_count(), 0);
    }

    #[tokio::test]
    async fn test_listener_lifecycle() {
        let mut service = ServiceConfig::default();
        service.listeners.push(ListenerConfig::new("test-1", ProtocolType::Http, 9999));
        
        let config = Arc::new(ArcSwap::from_pointee(service));
        let mut manager = ListenerManager::new(config.clone());

        // Subscribe to events
        let mut events = manager.subscribe();

        // Reconcile should start the listener
        manager.reconcile().await;
        assert_eq!(manager.listener_count(), 1);
        assert!(manager.is_running("test-1"));

        // Check event was sent
        let event = tokio::time::timeout(
            Duration::from_millis(100),
            events.recv()
        ).await;
        assert!(event.is_ok());

        // Shutdown
        manager.shutdown().await;
        assert_eq!(manager.listener_count(), 0);
    }
}
