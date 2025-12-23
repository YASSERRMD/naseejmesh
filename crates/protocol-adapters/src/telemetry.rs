//! OpenTelemetry Integration
//!
//! Provides distributed tracing across all protocol adapters with:
//! - Context propagation (HTTP, MQTT, gRPC)
//! - Span lifecycle management
//! - OTLP exporter configuration

use opentelemetry::KeyValue;
use opentelemetry_sdk::trace::TracerProvider;
use opentelemetry_sdk::Resource;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;

use crate::context::{NaseejContext, ProtocolType, TraceId};

/// Telemetry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    /// Enable OpenTelemetry tracing
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Service name for traces
    #[serde(default = "default_service_name")]
    pub service_name: String,

    /// OTLP endpoint (e.g., http://localhost:4317)
    pub otlp_endpoint: Option<String>,

    /// Sampling ratio (0.0 - 1.0)
    #[serde(default = "default_sampling_ratio")]
    pub sampling_ratio: f64,

    /// Additional resource attributes
    #[serde(default)]
    pub attributes: HashMap<String, String>,
}

fn default_enabled() -> bool {
    false
}

fn default_service_name() -> String {
    "naseejmesh-gateway".to_string()
}

fn default_sampling_ratio() -> f64 {
    1.0
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            service_name: default_service_name(),
            otlp_endpoint: None,
            sampling_ratio: default_sampling_ratio(),
            attributes: HashMap::new(),
        }
    }
}

/// Span types for the gateway
#[derive(Debug, Clone, Copy)]
pub enum SpanKind {
    /// Ingress span - message received from client
    Ingress,
    /// Router span - routing decision
    Router,
    /// Transform span - payload transformation
    Transform,
    /// Egress span - sending to upstream
    Egress,
}

impl SpanKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            SpanKind::Ingress => "ingress",
            SpanKind::Router => "router",
            SpanKind::Transform => "transform",
            SpanKind::Egress => "egress",
        }
    }
}

/// Extract trace context from HTTP headers (W3C Trace Context format)
pub fn extract_trace_from_http(headers: &HashMap<String, String>) -> Option<TraceId> {
    headers.get("traceparent").and_then(|value| {
        // Format: 00-{trace_id}-{span_id}-{flags}
        let parts: Vec<&str> = value.split('-').collect();
        if parts.len() >= 2 {
            Some(TraceId::from_string(parts[1]))
        } else {
            None
        }
    })
}

/// Extract trace context from MQTT user properties (MQTT v5)
pub fn extract_trace_from_mqtt(properties: &HashMap<String, String>) -> Option<TraceId> {
    // MQTT can use user properties or payload metadata
    properties.get("trace_id")
        .or_else(|| properties.get("traceparent"))
        .map(|v| {
            if v.contains('-') {
                // W3C format
                let parts: Vec<&str> = v.split('-').collect();
                if parts.len() >= 2 {
                    TraceId::from_string(parts[1])
                } else {
                    TraceId::from_string(v.as_str())
                }
            } else {
                TraceId::from_string(v.as_str())
            }
        })
}

/// Extract trace context from gRPC metadata
pub fn extract_trace_from_grpc(metadata: &HashMap<String, String>) -> Option<TraceId> {
    // gRPC uses standard HTTP headers
    extract_trace_from_http(metadata)
}

/// Generate W3C traceparent header value
pub fn generate_traceparent(ctx: &NaseejContext) -> String {
    format!(
        "00-{}-{}-01",
        ctx.trace_id,
        ctx.span_id.as_deref().unwrap_or("0000000000000000")
    )
}

/// Create a tracing span for the context
#[macro_export]
macro_rules! create_span {
    ($kind:expr, $ctx:expr) => {
        tracing::info_span!(
            "naseej",
            otel.name = $kind.as_str(),
            trace_id = %$ctx.trace_id,
            protocol = %$ctx.protocol,
            destination = %$ctx.destination,
        )
    };
}

/// Span attributes for observability
pub struct SpanAttributes {
    pub protocol: ProtocolType,
    pub destination: String,
    pub method: Option<String>,
    pub content_type: Option<String>,
    pub payload_size: usize,
}

impl SpanAttributes {
    pub fn from_context(ctx: &NaseejContext) -> Self {
        Self {
            protocol: ctx.protocol,
            destination: ctx.destination.clone(),
            method: ctx.method.clone(),
            content_type: ctx.content_type.clone(),
            payload_size: ctx.payload_len(),
        }
    }

    pub fn to_key_values(&self) -> Vec<KeyValue> {
        let mut kvs = vec![
            KeyValue::new("protocol", self.protocol.to_string()),
            KeyValue::new("destination", self.destination.clone()),
            KeyValue::new("payload_size", self.payload_size as i64),
        ];

        if let Some(method) = &self.method {
            kvs.push(KeyValue::new("method", method.clone()));
        }

        if let Some(ct) = &self.content_type {
            kvs.push(KeyValue::new("content_type", ct.clone()));
        }

        kvs
    }
}

/// Initialize OpenTelemetry (placeholder - full implementation requires OTLP setup)
pub fn init_telemetry(config: &TelemetryConfig) -> Option<TracerProvider> {
    if !config.enabled {
        info!("OpenTelemetry tracing disabled");
        return None;
    }

    info!(
        service = %config.service_name,
        "Initializing OpenTelemetry"
    );

    // Build resource with service info
    let resource = Resource::new(vec![
        KeyValue::new("service.name", config.service_name.clone()),
        KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
    ]);

    // Create a simple provider
    let provider = TracerProvider::builder()
        .with_config(
            opentelemetry_sdk::trace::Config::default()
                .with_resource(resource)
        )
        .build();

    Some(provider)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_trace_from_http() {
        let mut headers = HashMap::new();
        headers.insert(
            "traceparent".to_string(),
            "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01".to_string()
        );

        let trace_id = extract_trace_from_http(&headers);
        assert!(trace_id.is_some());
        assert_eq!(trace_id.unwrap().0, "0af7651916cd43dd8448eb211c80319c");
    }

    #[test]
    fn test_extract_trace_from_mqtt() {
        let mut props = HashMap::new();
        props.insert("trace_id".to_string(), "abc123".to_string());

        let trace_id = extract_trace_from_mqtt(&props);
        assert!(trace_id.is_some());
        assert_eq!(trace_id.unwrap().0, "abc123");
    }

    #[test]
    fn test_span_attributes() {
        use bytes::Bytes;
        
        let ctx = NaseejContext::new(
            ProtocolType::Http,
            "/api/test",
            Bytes::from("test"),
        ).with_method("POST")
         .with_content_type("application/json");

        let attrs = SpanAttributes::from_context(&ctx);
        assert_eq!(attrs.protocol, ProtocolType::Http);
        assert_eq!(attrs.destination, "/api/test");
        assert_eq!(attrs.method, Some("POST".to_string()));
        assert_eq!(attrs.payload_size, 4);
    }

    #[test]
    fn test_telemetry_config_default() {
        let config = TelemetryConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.service_name, "naseejmesh-gateway");
        assert_eq!(config.sampling_ratio, 1.0);
    }
}
