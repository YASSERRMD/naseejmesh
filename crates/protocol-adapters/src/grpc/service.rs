//! Dynamic gRPC Service
//!
//! Runtime gRPC service that uses reflection to handle any method
//! without compile-time proto generation.

use std::sync::Arc;

use bytes::Bytes;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, info};

use crate::context::{ContextBuilder, NaseejContext, ProtocolType};
use super::transcoder::GrpcTranscoder;

/// Configuration for dynamic gRPC service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrpcServiceConfig {
    /// Service identifier
    pub id: String,

    /// Port to listen on
    pub port: u16,

    /// Host to bind
    #[serde(default = "default_host")]
    pub host: String,

    /// Enable reflection service
    #[serde(default = "default_reflection")]
    pub reflection_enabled: bool,

    /// Descriptor sets to load (base64 encoded)
    #[serde(default)]
    pub descriptor_sets: Vec<String>,
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_reflection() -> bool {
    true
}

/// Dynamic gRPC service handler
pub struct DynamicGrpcService {
    /// Service configuration
    config: GrpcServiceConfig,

    /// Transcoder for JSON ↔ Protobuf
    transcoder: Arc<RwLock<GrpcTranscoder>>,

    /// Loaded service names
    services: Vec<String>,
}

impl DynamicGrpcService {
    /// Create a new dynamic gRPC service
    pub fn new(config: GrpcServiceConfig) -> Self {
        Self {
            config,
            transcoder: Arc::new(RwLock::new(GrpcTranscoder::new())),
            services: Vec::new(),
        }
    }

    /// Load descriptor sets from configuration
    pub async fn load_descriptors(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut transcoder = self.transcoder.write().await;

        for (i, descriptor_b64) in self.config.descriptor_sets.iter().enumerate() {
            // Decode base64
            let data = base64_decode(descriptor_b64)?;

            // Load into transcoder
            transcoder.load_descriptor_set(&data)?;

            debug!(index = i, "Loaded descriptor set");
        }

        // Cache service names
        self.services = transcoder.list_services();

        info!(
            services = ?self.services,
            messages = transcoder.list_message_types().len(),
            "Loaded gRPC descriptors"
        );

        Ok(())
    }

    /// Handle a gRPC request
    ///
    /// This converts the incoming protobuf to JSON, processes it,
    /// and converts the response back to protobuf.
    pub async fn handle_request(
        &self,
        service: &str,
        method: &str,
        request_data: Bytes,
        input_type: &str,
        output_type: &str,
    ) -> Result<Bytes, Box<dyn std::error::Error + Send + Sync>> {
        debug!(
            service = %service,
            method = %method,
            input = %input_type,
            output = %output_type,
            "Handling gRPC request"
        );

        let mut transcoder = self.transcoder.write().await;

        // Transcode request to JSON
        let json_request = transcoder.protobuf_to_json(&request_data, input_type)?;

        debug!(
            json_size = json_request.len(),
            "Transcoded request to JSON"
        );

        // Create NaseejContext for routing
        let _ctx = ContextBuilder::new(ProtocolType::Grpc, format!("/{}/{}", service, method))
            .payload(json_request.clone())
            .content_type("application/json")
            .metadata("grpc.service", service)
            .metadata("grpc.method", method)
            .metadata("grpc.input_type", input_type)
            .metadata("grpc.output_type", output_type)
            .build();

        // For now, return an empty response
        // In full implementation, this would route to upstream and transcode response
        let response_json = r#"{"status": "ok"}"#;
        let response_proto = transcoder.json_to_protobuf(
            response_json.as_bytes(),
            output_type,
        )?;

        Ok(response_proto)
    }

    /// Convert a NaseejContext to gRPC response
    pub async fn context_to_grpc(
        &self,
        ctx: &NaseejContext,
        output_type: &str,
    ) -> Result<Bytes, Box<dyn std::error::Error + Send + Sync>> {
        let mut transcoder = self.transcoder.write().await;

        // Assume context payload is JSON, transcode to protobuf
        let proto_data = transcoder.json_to_protobuf(&ctx.payload, output_type)?;

        Ok(proto_data)
    }

    /// Convert gRPC request to NaseejContext
    pub async fn grpc_to_context(
        &self,
        service: &str,
        method: &str,
        data: Bytes,
        input_type: &str,
    ) -> Result<NaseejContext, Box<dyn std::error::Error + Send + Sync>> {
        let mut transcoder = self.transcoder.write().await;

        // Transcode protobuf to JSON
        let json_data = transcoder.protobuf_to_json(&data, input_type)?;

        let ctx = ContextBuilder::new(ProtocolType::Grpc, format!("/{}/{}", service, method))
            .payload(json_data)
            .content_type("application/json")
            .metadata("grpc.service", service)
            .metadata("grpc.method", method)
            .metadata("grpc.input_type", input_type)
            .build();

        Ok(ctx)
    }

    /// Get list of available services
    pub fn services(&self) -> &[String] {
        &self.services
    }

    /// Get the bind address
    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.config.host, self.config.port)
    }
}

/// Simple base64 decoder
fn base64_decode(input: &str) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    fn decode_char(c: u8) -> Option<u8> {
        ALPHABET.iter().position(|&x| x == c).map(|p| p as u8)
    }

    let input = input.trim().replace('\n', "").replace('\r', "");
    let input = input.trim_end_matches('=');
    let mut output = Vec::new();

    let chars: Vec<u8> = input.as_bytes().to_vec();
    let mut i = 0;
    
    while i + 4 <= chars.len() {
        let mut n: u32 = 0;
        for j in 0..4 {
            if let Some(val) = decode_char(chars[i + j]) {
                n = (n << 6) | (val as u32);
            }
        }
        output.push(((n >> 16) & 0xFF) as u8);
        output.push(((n >> 8) & 0xFF) as u8);
        output.push((n & 0xFF) as u8);
        i += 4;
    }

    // Handle remaining chars
    let remaining = chars.len() - i;
    if remaining >= 2 {
        let mut n: u32 = 0;
        for j in 0..remaining {
            if let Some(val) = decode_char(chars[i + j]) {
                n = (n << 6) | (val as u32);
            }
        }
        n <<= 6 * (4 - remaining);
        output.push(((n >> 16) & 0xFF) as u8);
        if remaining >= 3 {
            output.push(((n >> 8) & 0xFF) as u8);
        }
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_creation() {
        let config = GrpcServiceConfig {
            id: "test".to_string(),
            port: 50051,
            host: "0.0.0.0".to_string(),
            reflection_enabled: true,
            descriptor_sets: vec![],
        };

        let service = DynamicGrpcService::new(config);
        assert_eq!(service.bind_address(), "0.0.0.0:50051");
        assert!(service.services().is_empty());
    }

    #[test]
    fn test_config_defaults() {
        let json = r#"{"id": "test", "port": 50051}"#;
        let config: GrpcServiceConfig = serde_json::from_str(json).unwrap();

        assert_eq!(config.host, "0.0.0.0");
        assert!(config.reflection_enabled);
    }
}
