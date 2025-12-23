//! gRPC Dynamic Transcoder
//!
//! Provides JSON ↔ Protobuf transcoding using prost-reflect,
//! enabling dynamic gRPC handling without compile-time proto generation.

use bytes::Bytes;
use prost::Message;
use prost_reflect::{DescriptorPool, DynamicMessage, MessageDescriptor, ReflectMessage, Value};
use serde_json::{Map, Value as JsonValue};
use std::collections::HashMap;
use thiserror::Error;
use tracing::{debug, warn};

/// Transcoding errors
#[derive(Debug, Error)]
pub enum TranscodeError {
    #[error("Descriptor not found: {name}")]
    DescriptorNotFound { name: String },

    #[error("Failed to parse protobuf: {0}")]
    ProtobufParse(String),

    #[error("Failed to serialize protobuf: {0}")]
    ProtobufSerialize(String),

    #[error("Failed to parse JSON: {0}")]
    JsonParse(String),

    #[error("Failed to serialize JSON: {0}")]
    JsonSerialize(String),

    #[error("Field type mismatch: expected {expected}, got {actual}")]
    TypeMismatch { expected: String, actual: String },

    #[error("Invalid descriptor set: {0}")]
    InvalidDescriptor(String),
}

/// gRPC Transcoder for JSON ↔ Protobuf conversion
pub struct GrpcTranscoder {
    /// Descriptor pool containing all loaded proto definitions
    pool: DescriptorPool,

    /// Cache of message descriptors by full name
    message_cache: HashMap<String, MessageDescriptor>,
}

impl GrpcTranscoder {
    /// Create a new transcoder with an empty descriptor pool
    pub fn new() -> Self {
        Self {
            pool: DescriptorPool::new(),
            message_cache: HashMap::new(),
        }
    }

    /// Create from a pre-built descriptor pool
    pub fn from_pool(pool: DescriptorPool) -> Self {
        Self {
            pool,
            message_cache: HashMap::new(),
        }
    }

    /// Load a descriptor set (compiled .proto files)
    pub fn load_descriptor_set(&mut self, data: &[u8]) -> Result<(), TranscodeError> {
        // Decode the file descriptor set
        let fds = prost_types::FileDescriptorSet::decode(data)
            .map_err(|e| TranscodeError::InvalidDescriptor(e.to_string()))?;

        // Add to pool
        self.pool = DescriptorPool::from_file_descriptor_set(fds)
            .map_err(|e| TranscodeError::InvalidDescriptor(e.to_string()))?;

        // Clear cache
        self.message_cache.clear();

        debug!("Loaded descriptor set with {} types", self.pool.all_messages().count());

        Ok(())
    }

    /// Get a message descriptor by its full name (e.g., "package.MessageName")
    pub fn get_message_descriptor(&mut self, full_name: &str) -> Result<MessageDescriptor, TranscodeError> {
        if let Some(desc) = self.message_cache.get(full_name) {
            return Ok(desc.clone());
        }

        let desc = self.pool
            .get_message_by_name(full_name)
            .ok_or_else(|| TranscodeError::DescriptorNotFound {
                name: full_name.to_string(),
            })?;

        self.message_cache.insert(full_name.to_string(), desc.clone());
        Ok(desc)
    }

    /// Transcode JSON to Protobuf binary
    pub fn json_to_protobuf(
        &mut self,
        json: &[u8],
        message_type: &str,
    ) -> Result<Bytes, TranscodeError> {
        let desc = self.get_message_descriptor(message_type)?;

        // Parse JSON
        let json_value: JsonValue = serde_json::from_slice(json)
            .map_err(|e| TranscodeError::JsonParse(e.to_string()))?;

        // Create dynamic message
        let mut msg = DynamicMessage::new(desc);

        // Populate fields from JSON
        if let JsonValue::Object(obj) = json_value {
            self.populate_message(&mut msg, &obj)?;
        }

        // Serialize to protobuf
        let buf = msg.encode_to_vec();

        Ok(Bytes::from(buf))
    }

    /// Transcode Protobuf binary to JSON
    pub fn protobuf_to_json(
        &mut self,
        data: &[u8],
        message_type: &str,
    ) -> Result<Bytes, TranscodeError> {
        let desc = self.get_message_descriptor(message_type)?;

        // Parse protobuf
        let msg = DynamicMessage::decode(desc, data)
            .map_err(|e| TranscodeError::ProtobufParse(e.to_string()))?;

        // Convert to JSON
        let json = self.message_to_json(&msg)?;

        // Serialize
        let json_bytes = serde_json::to_vec(&json)
            .map_err(|e| TranscodeError::JsonSerialize(e.to_string()))?;

        Ok(Bytes::from(json_bytes))
    }

    /// Populate a DynamicMessage from JSON object
    fn populate_message(
        &self,
        msg: &mut DynamicMessage,
        obj: &Map<String, JsonValue>,
    ) -> Result<(), TranscodeError> {
        let desc = msg.descriptor();

        for (key, value) in obj {
            if let Some(field) = desc.get_field_by_name(key) {
                let prost_value = self.json_to_prost_value(value, &field)?;
                msg.set_field(&field, prost_value);
            } else {
                warn!(field = %key, "Unknown field in JSON, skipping");
            }
        }

        Ok(())
    }

    /// Convert JSON value to prost-reflect Value
    fn json_to_prost_value(
        &self,
        json: &JsonValue,
        _field: &prost_reflect::FieldDescriptor,
    ) -> Result<Value, TranscodeError> {
        match json {
            JsonValue::Null => {
                // For null, return a default empty message or skip
                Err(TranscodeError::TypeMismatch {
                    expected: "value".to_string(),
                    actual: "null".to_string(),
                })
            }
            JsonValue::Bool(b) => Ok(Value::Bool(*b)),
            JsonValue::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Ok(Value::I64(i))
                } else if let Some(f) = n.as_f64() {
                    Ok(Value::F64(f))
                } else {
                    Err(TranscodeError::TypeMismatch {
                        expected: "number".to_string(),
                        actual: format!("{:?}", n),
                    })
                }
            }
            JsonValue::String(s) => Ok(Value::String(s.clone())),
            JsonValue::Array(arr) => {
                let values: Result<Vec<_>, _> = arr
                    .iter()
                    .map(|v| self.json_to_prost_value(v, _field))
                    .collect();
                Ok(Value::List(values?))
            }
            JsonValue::Object(_) => {
                // Nested messages would need recursive handling
                // For now, serialize as JSON string
                Ok(Value::String(json.to_string()))
            }
        }
    }

    /// Convert DynamicMessage to JSON value
    fn message_to_json(&self, msg: &DynamicMessage) -> Result<JsonValue, TranscodeError> {
        let mut obj = Map::new();

        for field in msg.descriptor().fields() {
            if msg.has_field(&field) {
                let value = msg.get_field(&field);
                let json_value = self.prost_value_to_json(&value)?;
                obj.insert(field.name().to_string(), json_value);
            }
        }

        Ok(JsonValue::Object(obj))
    }

    /// Convert prost-reflect Value to JSON value
    fn prost_value_to_json(&self, value: &Value) -> Result<JsonValue, TranscodeError> {
        match value {
            Value::Bool(b) => Ok(JsonValue::Bool(*b)),
            Value::I32(i) => Ok(JsonValue::Number((*i).into())),
            Value::I64(i) => Ok(JsonValue::Number((*i).into())),
            Value::U32(u) => Ok(JsonValue::Number((*u).into())),
            Value::U64(u) => Ok(JsonValue::Number((*u).into())),
            Value::F32(f) => Ok(serde_json::Number::from_f64(*f as f64)
                .map(JsonValue::Number)
                .unwrap_or(JsonValue::Null)),
            Value::F64(f) => Ok(serde_json::Number::from_f64(*f)
                .map(JsonValue::Number)
                .unwrap_or(JsonValue::Null)),
            Value::String(s) => Ok(JsonValue::String(s.clone())),
            Value::Bytes(b) => Ok(JsonValue::String(base64_encode(b))),
            Value::EnumNumber(n) => Ok(JsonValue::Number((*n).into())),
            Value::Message(m) => self.message_to_json(m),
            Value::List(arr) => {
                let values: Result<Vec<_>, _> = arr
                    .iter()
                    .map(|v| self.prost_value_to_json(v))
                    .collect();
                Ok(JsonValue::Array(values?))
            }
            Value::Map(map) => {
                let mut obj = Map::new();
                for (k, v) in map {
                    let key = match k {
                        prost_reflect::MapKey::Bool(b) => b.to_string(),
                        prost_reflect::MapKey::I32(i) => i.to_string(),
                        prost_reflect::MapKey::I64(i) => i.to_string(),
                        prost_reflect::MapKey::U32(u) => u.to_string(),
                        prost_reflect::MapKey::U64(u) => u.to_string(),
                        prost_reflect::MapKey::String(s) => s.clone(),
                    };
                    obj.insert(key, self.prost_value_to_json(v)?);
                }
                Ok(JsonValue::Object(obj))
            }
        }
    }

    /// List all available message types
    pub fn list_message_types(&self) -> Vec<String> {
        self.pool
            .all_messages()
            .map(|m| m.full_name().to_string())
            .collect()
    }

    /// List all available services
    pub fn list_services(&self) -> Vec<String> {
        self.pool
            .services()
            .map(|s| s.full_name().to_string())
            .collect()
    }
}

impl Default for GrpcTranscoder {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple base64 encoding
fn base64_encode(bytes: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    
    for chunk in bytes.chunks(3) {
        let mut n: u32 = 0;
        for (i, &byte) in chunk.iter().enumerate() {
            n |= (byte as u32) << (16 - i * 8);
        }
        
        let chars = match chunk.len() {
            3 => 4,
            2 => 3,
            1 => 2,
            _ => 0,
        };
        
        for i in 0..chars {
            let idx = ((n >> (18 - i * 6)) & 0x3F) as usize;
            result.push(ALPHABET[idx] as char);
        }
        
        for _ in chars..4 {
            result.push('=');
        }
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transcoder_creation() {
        let transcoder = GrpcTranscoder::new();
        assert!(transcoder.list_message_types().is_empty());
        assert!(transcoder.list_services().is_empty());
    }

    #[test]
    fn test_descriptor_not_found() {
        let mut transcoder = GrpcTranscoder::new();
        let result = transcoder.get_message_descriptor("nonexistent.Message");
        assert!(result.is_err());
    }

    #[test]
    fn test_base64_encode() {
        assert_eq!(base64_encode(b"hello"), "aGVsbG8=");
        assert_eq!(base64_encode(b"a"), "YQ==");
    }
}
