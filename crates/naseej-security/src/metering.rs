//! Usage Metering
//!
//! Async metering sidecar for tracking API usage.
//! Collects metrics without blocking request processing.

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{debug, error, info};

/// Usage event for metering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageEvent {
    /// Timestamp (Unix millis)
    pub timestamp: i64,

    /// Client identifier
    pub client_id: String,

    /// Request path
    pub path: String,

    /// HTTP method
    pub method: String,

    /// Response status code
    pub status_code: u16,

    /// Request size in bytes
    pub request_bytes: u64,

    /// Response size in bytes
    pub response_bytes: u64,

    /// Latency in microseconds
    pub latency_us: u64,

    /// Route ID (if matched)
    pub route_id: Option<String>,

    /// Protocol (http, mqtt, grpc, soap)
    pub protocol: String,
}

impl UsageEvent {
    /// Create a new usage event
    pub fn new(client_id: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            timestamp: chrono::Utc::now().timestamp_millis(),
            client_id: client_id.into(),
            path: path.into(),
            method: "GET".to_string(),
            status_code: 200,
            request_bytes: 0,
            response_bytes: 0,
            latency_us: 0,
            route_id: None,
            protocol: "http".to_string(),
        }
    }

    /// Set method
    pub fn with_method(mut self, method: impl Into<String>) -> Self {
        self.method = method.into();
        self
    }

    /// Set status code
    pub fn with_status(mut self, status: u16) -> Self {
        self.status_code = status;
        self
    }

    /// Set request size
    pub fn with_request_bytes(mut self, bytes: u64) -> Self {
        self.request_bytes = bytes;
        self
    }

    /// Set response size
    pub fn with_response_bytes(mut self, bytes: u64) -> Self {
        self.response_bytes = bytes;
        self
    }

    /// Set latency
    pub fn with_latency_us(mut self, latency_us: u64) -> Self {
        self.latency_us = latency_us;
        self
    }

    /// Set route ID
    pub fn with_route_id(mut self, route_id: impl Into<String>) -> Self {
        self.route_id = Some(route_id.into());
        self
    }

    /// Set protocol
    pub fn with_protocol(mut self, protocol: impl Into<String>) -> Self {
        self.protocol = protocol.into();
        self
    }
}

/// Metering configuration
#[derive(Debug, Clone)]
pub struct MeterConfig {
    /// Channel buffer size
    pub channel_size: usize,

    /// Batch size for flushing
    pub batch_size: usize,

    /// Flush interval
    pub flush_interval: Duration,
}

impl Default for MeterConfig {
    fn default() -> Self {
        Self {
            channel_size: 10_000,
            batch_size: 100,
            flush_interval: Duration::from_secs(1),
        }
    }
}

/// Meter handle for sending events
#[derive(Clone)]
pub struct Meter {
    sender: mpsc::Sender<UsageEvent>,
}

impl Meter {
    /// Record a usage event (non-blocking)
    pub fn record(&self, event: UsageEvent) {
        // Use try_send to avoid blocking
        if let Err(e) = self.sender.try_send(event) {
            debug!(error = %e, "Failed to record usage event (channel full)");
        }
    }

    /// Create a convenience method for quick recording
    pub fn quick_record(
        &self,
        client_id: &str,
        path: &str,
        method: &str,
        status: u16,
        latency_us: u64,
    ) {
        self.record(
            UsageEvent::new(client_id, path)
                .with_method(method)
                .with_status(status)
                .with_latency_us(latency_us),
        );
    }
}

/// Meter collector that processes events
pub struct MeterCollector {
    receiver: mpsc::Receiver<UsageEvent>,
    batch: Vec<UsageEvent>,
    config: MeterConfig,
    total_events: u64,
    total_bytes: u64,
}

impl MeterCollector {
    /// Create a new meter and collector pair
    pub fn new(config: MeterConfig) -> (Meter, Self) {
        let (sender, receiver) = mpsc::channel(config.channel_size);

        let meter = Meter { sender };
        let collector = Self {
            receiver,
            batch: Vec::with_capacity(config.batch_size),
            config,
            total_events: 0,
            total_bytes: 0,
        };

        (meter, collector)
    }

    /// Run the collector loop
    pub async fn run<F, Fut>(mut self, flush_fn: F)
    where
        F: Fn(Vec<UsageEvent>) -> Fut,
        Fut: std::future::Future<Output = Result<(), String>>,
    {
        info!("Starting metering collector");

        let mut interval = tokio::time::interval(self.config.flush_interval);

        loop {
            tokio::select! {
                Some(event) = self.receiver.recv() => {
                    self.total_bytes += event.request_bytes + event.response_bytes;
                    self.batch.push(event);
                    self.total_events += 1;

                    if self.batch.len() >= self.config.batch_size {
                        let batch = std::mem::take(&mut self.batch);
                        if let Err(e) = flush_fn(batch).await {
                            error!(error = %e, "Failed to flush usage batch");
                        }
                    }
                }

                _ = interval.tick() => {
                    if !self.batch.is_empty() {
                        let batch = std::mem::take(&mut self.batch);
                        debug!(batch_size = batch.len(), "Flushing usage batch");
                        if let Err(e) = flush_fn(batch).await {
                            error!(error = %e, "Failed to flush usage batch");
                        }
                    }
                }

                else => break,
            }
        }

        // Final flush
        if !self.batch.is_empty() {
            let batch = std::mem::take(&mut self.batch);
            let _ = flush_fn(batch).await;
        }

        info!(
            total_events = self.total_events,
            total_bytes = self.total_bytes,
            "Metering collector stopped"
        );
    }

    /// Get statistics
    pub fn stats(&self) -> (u64, u64) {
        (self.total_events, self.total_bytes)
    }
}

/// Utility to aggregate usage by client
#[derive(Debug, Default, Serialize)]
pub struct ClientUsageSummary {
    pub client_id: String,
    pub total_requests: u64,
    pub total_bytes_in: u64,
    pub total_bytes_out: u64,
    pub avg_latency_us: u64,
    pub error_count: u64,
}

impl ClientUsageSummary {
    /// Aggregate events into summary
    pub fn from_events(client_id: &str, events: &[UsageEvent]) -> Self {
        let total_requests = events.len() as u64;
        let total_bytes_in: u64 = events.iter().map(|e| e.request_bytes).sum();
        let total_bytes_out: u64 = events.iter().map(|e| e.response_bytes).sum();
        let total_latency: u64 = events.iter().map(|e| e.latency_us).sum();
        let error_count = events.iter().filter(|e| e.status_code >= 400).count() as u64;

        Self {
            client_id: client_id.to_string(),
            total_requests,
            total_bytes_in,
            total_bytes_out,
            avg_latency_us: if total_requests > 0 {
                total_latency / total_requests
            } else {
                0
            },
            error_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_meter_record() {
        let config = MeterConfig {
            channel_size: 100,
            batch_size: 10,
            flush_interval: Duration::from_millis(100),
        };

        let (meter, mut collector) = MeterCollector::new(config);

        // Record some events
        for i in 0..5 {
            meter.record(
                UsageEvent::new("client1", format!("/api/test/{}", i))
                    .with_status(200)
                    .with_latency_us(1000),
            );
        }

        // Give time for events to be received
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Close sender to stop collector
        drop(meter);
    }

    #[test]
    fn test_usage_event_builder() {
        let event = UsageEvent::new("client1", "/api/users")
            .with_method("POST")
            .with_status(201)
            .with_request_bytes(1024)
            .with_response_bytes(256)
            .with_latency_us(5000)
            .with_route_id("user-service")
            .with_protocol("http");

        assert_eq!(event.client_id, "client1");
        assert_eq!(event.method, "POST");
        assert_eq!(event.status_code, 201);
        assert_eq!(event.request_bytes, 1024);
        assert_eq!(event.latency_us, 5000);
        assert_eq!(event.route_id, Some("user-service".to_string()));
    }

    #[test]
    fn test_client_usage_summary() {
        let events = vec![
            UsageEvent::new("client1", "/api/a")
                .with_request_bytes(100)
                .with_response_bytes(200)
                .with_latency_us(1000)
                .with_status(200),
            UsageEvent::new("client1", "/api/b")
                .with_request_bytes(150)
                .with_response_bytes(300)
                .with_latency_us(2000)
                .with_status(500),
        ];

        let summary = ClientUsageSummary::from_events("client1", &events);

        assert_eq!(summary.total_requests, 2);
        assert_eq!(summary.total_bytes_in, 250);
        assert_eq!(summary.total_bytes_out, 500);
        assert_eq!(summary.avg_latency_us, 1500);
        assert_eq!(summary.error_count, 1);
    }
}
