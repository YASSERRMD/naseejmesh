//! Universal Message Context
//!
//! NaseejContext is the "common currency" of the platform. All protocol adapters
//! convert their native formats to this unified representation for routing and
//! observability.

use bytes::Bytes;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Unique trace identifier for distributed tracing
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TraceId(pub String);

impl TraceId {
    /// Generate a new random trace ID
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    /// Create from an existing string
    pub fn from_string(s: impl Into<String>) -> Self {
        Self(s.into())
    }
}

impl Default for TraceId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for TraceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Protocol types supported by NaseejMesh
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProtocolType {
    /// HTTP/1.1 and HTTP/2
    Http,
    /// MQTT v3.1.1 and v5
    Mqtt,
    /// gRPC (HTTP/2 + Protobuf)
    Grpc,
    /// SOAP/XML over HTTP
    Soap,
}

impl std::fmt::Display for ProtocolType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProtocolType::Http => write!(f, "http"),
            ProtocolType::Mqtt => write!(f, "mqtt"),
            ProtocolType::Grpc => write!(f, "grpc"),
            ProtocolType::Soap => write!(f, "soap"),
        }
    }
}

/// Universal message context for protocol-agnostic routing
///
/// This struct normalizes data from any protocol (HTTP, MQTT, gRPC, SOAP)
/// into a common structure. It carries:
/// - Trace ID for distributed tracing
/// - Raw payload as zero-copy Bytes
/// - Metadata map (headers, topics, properties)
/// - Source protocol identity
#[derive(Debug, Clone)]
pub struct NaseejContext {
    /// Unique trace ID for OpenTelemetry correlation
    pub trace_id: TraceId,

    /// Span ID within the trace (optional)
    pub span_id: Option<String>,

    /// Parent span ID for context propagation
    pub parent_span_id: Option<String>,

    /// The raw payload (zero-copy Bytes from Phase 1)
    pub payload: Bytes,

    /// Content type of the payload (e.g., "application/json")
    pub content_type: Option<String>,

    /// Metadata normalized to a Map (Headers, Topics, Properties)
    pub metadata: HashMap<String, String>,

    /// The source protocol identity
    pub protocol: ProtocolType,

    /// Timestamp when the message was received
    pub timestamp: DateTime<Utc>,

    /// Source address (IP:port or client ID)
    pub source: Option<String>,

    /// Target destination (path, topic, method)
    pub destination: String,

    /// HTTP method equivalent (GET, POST, PUBLISH, etc.)
    pub method: Option<String>,
}

impl NaseejContext {
    /// Create a new context with minimal required fields
    pub fn new(
        protocol: ProtocolType,
        destination: impl Into<String>,
        payload: Bytes,
    ) -> Self {
        Self {
            trace_id: TraceId::new(),
            span_id: None,
            parent_span_id: None,
            payload,
            content_type: None,
            metadata: HashMap::new(),
            protocol,
            timestamp: Utc::now(),
            source: None,
            destination: destination.into(),
            method: None,
        }
    }

    /// Create context with an existing trace ID (for propagation)
    pub fn with_trace_id(mut self, trace_id: TraceId) -> Self {
        self.trace_id = trace_id;
        self
    }

    /// Set the content type
    pub fn with_content_type(mut self, content_type: impl Into<String>) -> Self {
        self.content_type = Some(content_type.into());
        self
    }

    /// Add a metadata entry
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Set the source address
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// Set the method
    pub fn with_method(mut self, method: impl Into<String>) -> Self {
        self.method = Some(method.into());
        self
    }

    /// Get a metadata value
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }

    /// Check if payload is empty
    pub fn is_empty(&self) -> bool {
        self.payload.is_empty()
    }

    /// Get payload length
    pub fn payload_len(&self) -> usize {
        self.payload.len()
    }

    /// Extract trace context for propagation
    pub fn trace_context(&self) -> HashMap<String, String> {
        let mut ctx = HashMap::new();
        ctx.insert("traceparent".to_string(), format!(
            "00-{}-{}-01",
            self.trace_id,
            self.span_id.as_deref().unwrap_or("0000000000000000")
        ));
        ctx
    }
}

/// Builder for creating NaseejContext from protocol-specific sources
pub struct ContextBuilder {
    context: NaseejContext,
}

impl ContextBuilder {
    pub fn new(protocol: ProtocolType, destination: impl Into<String>) -> Self {
        Self {
            context: NaseejContext::new(protocol, destination, Bytes::new()),
        }
    }

    pub fn payload(mut self, payload: Bytes) -> Self {
        self.context.payload = payload;
        self
    }

    pub fn trace_id(mut self, trace_id: TraceId) -> Self {
        self.context.trace_id = trace_id;
        self
    }

    pub fn content_type(mut self, content_type: impl Into<String>) -> Self {
        self.context.content_type = Some(content_type.into());
        self
    }

    pub fn metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.metadata.insert(key.into(), value.into());
        self
    }

    pub fn source(mut self, source: impl Into<String>) -> Self {
        self.context.source = Some(source.into());
        self
    }

    pub fn method(mut self, method: impl Into<String>) -> Self {
        self.context.method = Some(method.into());
        self
    }

    pub fn build(self) -> NaseejContext {
        self.context
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_id_generation() {
        let id1 = TraceId::new();
        let id2 = TraceId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_context_creation() {
        let ctx = NaseejContext::new(
            ProtocolType::Http,
            "/api/users",
            Bytes::from("test payload"),
        );

        assert_eq!(ctx.protocol, ProtocolType::Http);
        assert_eq!(ctx.destination, "/api/users");
        assert_eq!(ctx.payload_len(), 12);
    }

    #[test]
    fn test_context_builder() {
        let ctx = ContextBuilder::new(ProtocolType::Mqtt, "sensors/temperature")
            .payload(Bytes::from(r#"{"temp": 22.5}"#))
            .content_type("application/json")
            .metadata("qos", "1")
            .source("device-001")
            .build();

        assert_eq!(ctx.protocol, ProtocolType::Mqtt);
        assert_eq!(ctx.destination, "sensors/temperature");
        assert_eq!(ctx.content_type, Some("application/json".to_string()));
        assert_eq!(ctx.get_metadata("qos"), Some(&"1".to_string()));
    }

    #[test]
    fn test_protocol_display() {
        assert_eq!(format!("{}", ProtocolType::Http), "http");
        assert_eq!(format!("{}", ProtocolType::Mqtt), "mqtt");
        assert_eq!(format!("{}", ProtocolType::Grpc), "grpc");
        assert_eq!(format!("{}", ProtocolType::Soap), "soap");
    }
}
