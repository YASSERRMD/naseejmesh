//! Observability Module
//!
//! Provides metrics collection, distributed tracing, and structured logging
//! for production monitoring of the NaseejMesh gateway.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Metrics collector for gateway operations
#[derive(Debug)]
pub struct MetricsCollector {
    /// Total requests processed
    requests_total: AtomicU64,
    /// Total requests that succeeded (2xx)
    requests_success: AtomicU64,
    /// Total requests that failed (4xx, 5xx)
    requests_error: AtomicU64,
    /// Total bytes received
    bytes_in: AtomicU64,
    /// Total bytes sent
    bytes_out: AtomicU64,
    /// Request latencies by route (in milliseconds)
    latencies: RwLock<HashMap<String, LatencyStats>>,
    /// Active connections
    active_connections: AtomicU64,
    /// Gateway start time
    start_time: Instant,
}

/// Latency statistics for a route
#[derive(Debug, Clone, Default)]
pub struct LatencyStats {
    pub count: u64,
    pub sum_ms: u64,
    pub min_ms: u64,
    pub max_ms: u64,
    /// P50, P90, P99 percentiles (approximate)
    pub percentiles: [u64; 3],
}

impl LatencyStats {
    pub fn avg_ms(&self) -> u64 {
        if self.count == 0 {
            0
        } else {
            self.sum_ms / self.count
        }
    }
}

/// Request metrics for a single request
#[derive(Debug, Clone)]
pub struct RequestMetrics {
    pub route_id: String,
    pub method: String,
    pub status_code: u16,
    pub latency_ms: u64,
    pub bytes_in: u64,
    pub bytes_out: u64,
}

/// Exported metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub uptime_seconds: u64,
    pub requests_total: u64,
    pub requests_success: u64,
    pub requests_error: u64,
    pub bytes_in: u64,
    pub bytes_out: u64,
    pub active_connections: u64,
    pub routes: HashMap<String, RouteMetrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteMetrics {
    pub count: u64,
    pub avg_latency_ms: u64,
    pub min_latency_ms: u64,
    pub max_latency_ms: u64,
    pub p50_ms: u64,
    pub p90_ms: u64,
    pub p99_ms: u64,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            requests_total: AtomicU64::new(0),
            requests_success: AtomicU64::new(0),
            requests_error: AtomicU64::new(0),
            bytes_in: AtomicU64::new(0),
            bytes_out: AtomicU64::new(0),
            latencies: RwLock::new(HashMap::new()),
            active_connections: AtomicU64::new(0),
            start_time: Instant::now(),
        }
    }

    /// Record metrics for a completed request
    pub fn record_request(&self, metrics: RequestMetrics) {
        self.requests_total.fetch_add(1, Ordering::Relaxed);
        self.bytes_in.fetch_add(metrics.bytes_in, Ordering::Relaxed);
        self.bytes_out.fetch_add(metrics.bytes_out, Ordering::Relaxed);

        if metrics.status_code >= 200 && metrics.status_code < 400 {
            self.requests_success.fetch_add(1, Ordering::Relaxed);
        } else {
            self.requests_error.fetch_add(1, Ordering::Relaxed);
        }

        // Update latency stats
        let mut latencies = self.latencies.write();
        let stats = latencies.entry(metrics.route_id).or_default();
        stats.count += 1;
        stats.sum_ms += metrics.latency_ms;
        if stats.min_ms == 0 || metrics.latency_ms < stats.min_ms {
            stats.min_ms = metrics.latency_ms;
        }
        if metrics.latency_ms > stats.max_ms {
            stats.max_ms = metrics.latency_ms;
        }
    }

    /// Increment active connections
    pub fn connection_opened(&self) {
        self.active_connections.fetch_add(1, Ordering::Relaxed);
    }

    /// Decrement active connections
    pub fn connection_closed(&self) {
        self.active_connections.fetch_sub(1, Ordering::Relaxed);
    }

    /// Get current metrics snapshot
    pub fn snapshot(&self) -> MetricsSnapshot {
        let latencies = self.latencies.read();
        let routes: HashMap<String, RouteMetrics> = latencies
            .iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    RouteMetrics {
                        count: v.count,
                        avg_latency_ms: v.avg_ms(),
                        min_latency_ms: v.min_ms,
                        max_latency_ms: v.max_ms,
                        p50_ms: v.percentiles[0],
                        p90_ms: v.percentiles[1],
                        p99_ms: v.percentiles[2],
                    },
                )
            })
            .collect();

        MetricsSnapshot {
            uptime_seconds: self.start_time.elapsed().as_secs(),
            requests_total: self.requests_total.load(Ordering::Relaxed),
            requests_success: self.requests_success.load(Ordering::Relaxed),
            requests_error: self.requests_error.load(Ordering::Relaxed),
            bytes_in: self.bytes_in.load(Ordering::Relaxed),
            bytes_out: self.bytes_out.load(Ordering::Relaxed),
            active_connections: self.active_connections.load(Ordering::Relaxed),
            routes,
        }
    }

    /// Export metrics in Prometheus format
    pub fn prometheus_export(&self) -> String {
        let snapshot = self.snapshot();
        let mut output = String::new();

        output.push_str("# HELP naseej_uptime_seconds Gateway uptime\n");
        output.push_str("# TYPE naseej_uptime_seconds gauge\n");
        output.push_str(&format!("naseej_uptime_seconds {}\n", snapshot.uptime_seconds));

        output.push_str("# HELP naseej_requests_total Total requests\n");
        output.push_str("# TYPE naseej_requests_total counter\n");
        output.push_str(&format!("naseej_requests_total {}\n", snapshot.requests_total));

        output.push_str("# HELP naseej_requests_success Successful requests\n");
        output.push_str("# TYPE naseej_requests_success counter\n");
        output.push_str(&format!("naseej_requests_success {}\n", snapshot.requests_success));

        output.push_str("# HELP naseej_requests_error Failed requests\n");
        output.push_str("# TYPE naseej_requests_error counter\n");
        output.push_str(&format!("naseej_requests_error {}\n", snapshot.requests_error));

        output.push_str("# HELP naseej_bytes_in Bytes received\n");
        output.push_str("# TYPE naseej_bytes_in counter\n");
        output.push_str(&format!("naseej_bytes_in {}\n", snapshot.bytes_in));

        output.push_str("# HELP naseej_bytes_out Bytes sent\n");
        output.push_str("# TYPE naseej_bytes_out counter\n");
        output.push_str(&format!("naseej_bytes_out {}\n", snapshot.bytes_out));

        output.push_str("# HELP naseej_active_connections Active connections\n");
        output.push_str("# TYPE naseej_active_connections gauge\n");
        output.push_str(&format!("naseej_active_connections {}\n", snapshot.active_connections));

        for (route, metrics) in &snapshot.routes {
            output.push_str(&format!(
                "naseej_route_requests_total{{route=\"{}\"}} {}\n",
                route, metrics.count
            ));
            output.push_str(&format!(
                "naseej_route_latency_avg_ms{{route=\"{}\"}} {}\n",
                route, metrics.avg_latency_ms
            ));
        }

        output
    }

    /// Reset all metrics
    pub fn reset(&self) {
        self.requests_total.store(0, Ordering::Relaxed);
        self.requests_success.store(0, Ordering::Relaxed);
        self.requests_error.store(0, Ordering::Relaxed);
        self.bytes_in.store(0, Ordering::Relaxed);
        self.bytes_out.store(0, Ordering::Relaxed);
        self.latencies.write().clear();
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Distributed tracing context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceContext {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub sampled: bool,
}

impl TraceContext {
    /// Create a new root trace
    pub fn new() -> Self {
        Self {
            trace_id: uuid::Uuid::new_v4().to_string(),
            span_id: uuid::Uuid::new_v4().to_string()[..16].to_string(),
            parent_span_id: None,
            sampled: true,
        }
    }

    /// Create a child span
    pub fn child(&self) -> Self {
        Self {
            trace_id: self.trace_id.clone(),
            span_id: uuid::Uuid::new_v4().to_string()[..16].to_string(),
            parent_span_id: Some(self.span_id.clone()),
            sampled: self.sampled,
        }
    }

    /// Parse from W3C traceparent header
    pub fn from_traceparent(header: &str) -> Option<Self> {
        let parts: Vec<&str> = header.split('-').collect();
        if parts.len() >= 4 {
            Some(Self {
                trace_id: parts[1].to_string(),
                span_id: parts[2].to_string(),
                parent_span_id: None,
                sampled: parts[3] == "01",
            })
        } else {
            None
        }
    }

    /// Format as W3C traceparent header
    pub fn to_traceparent(&self) -> String {
        let flags = if self.sampled { "01" } else { "00" };
        format!("00-{}-{}-{}", self.trace_id, self.span_id, flags)
    }
}

impl Default for TraceContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_recording() {
        let collector = MetricsCollector::new();

        collector.record_request(RequestMetrics {
            route_id: "test_route".to_string(),
            method: "GET".to_string(),
            status_code: 200,
            latency_ms: 50,
            bytes_in: 100,
            bytes_out: 500,
        });

        let snapshot = collector.snapshot();
        assert_eq!(snapshot.requests_total, 1);
        assert_eq!(snapshot.requests_success, 1);
        assert_eq!(snapshot.bytes_in, 100);
    }

    #[test]
    fn test_trace_context() {
        let root = TraceContext::new();
        let child = root.child();

        assert_eq!(child.trace_id, root.trace_id);
        assert_eq!(child.parent_span_id, Some(root.span_id.clone()));
    }

    #[test]
    fn test_traceparent_parsing() {
        let header = "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01";
        let ctx = TraceContext::from_traceparent(header).unwrap();

        assert_eq!(ctx.trace_id, "0af7651916cd43dd8448eb211c80319c");
        assert!(ctx.sampled);
    }

    #[test]
    fn test_prometheus_export() {
        let collector = MetricsCollector::new();
        collector.record_request(RequestMetrics {
            route_id: "api".to_string(),
            method: "GET".to_string(),
            status_code: 200,
            latency_ms: 10,
            bytes_in: 50,
            bytes_out: 200,
        });

        let prom = collector.prometheus_export();
        assert!(prom.contains("naseej_requests_total 1"));
    }
}
