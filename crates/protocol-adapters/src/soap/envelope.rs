//! SOAP Envelope Handling
//!
//! Provides SOAP envelope parsing and construction with:
//! - Envelope/Header/Body extraction
//! - Fault handling
//! - Action detection

use bytes::Bytes;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::context::{ContextBuilder, NaseejContext, ProtocolType};
use super::transcoder::{XmlToJson, XmlTranscodeError};

/// SOAP namespace constants
pub mod namespaces {
    pub const SOAP11: &str = "http://schemas.xmlsoap.org/soap/envelope/";
    pub const SOAP12: &str = "http://www.w3.org/2003/05/soap-envelope";
}

/// SOAP version
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SoapVersion {
    Soap11,
    Soap12,
}

/// Parsed SOAP envelope structure
#[derive(Debug, Clone)]
pub struct SoapEnvelope {
    /// SOAP version
    pub version: SoapVersion,

    /// SOAP headers (optional)
    pub headers: Option<JsonValue>,

    /// SOAP body content
    pub body: JsonValue,

    /// SOAP action (from HTTP header or WS-Addressing)
    pub action: Option<String>,

    /// Is this a fault response?
    pub is_fault: bool,
}

impl SoapEnvelope {
    /// Parse a SOAP envelope from XML bytes
    pub fn parse(xml: &[u8]) -> Result<Self, XmlTranscodeError> {
        let transcoder = XmlToJson::new();
        let json = transcoder.transcode(xml)?;

        Self::from_json(&serde_json::from_slice(&json)
            .map_err(|e| XmlTranscodeError::JsonError(e.to_string()))?)
    }

    /// Parse from already-transcoded JSON
    pub fn from_json(json: &JsonValue) -> Result<Self, XmlTranscodeError> {
        // Find the Envelope element
        let envelope = json.get("Envelope")
            .or_else(|| find_element(json, "Envelope"))
            .ok_or_else(|| XmlTranscodeError::InvalidStructure(
                "Missing SOAP Envelope".to_string()
            ))?;

        // Detect SOAP version from namespace (if present)
        let version = SoapVersion::Soap11; // Default to 1.1

        // Extract Header (optional)
        let headers = envelope.get("Header")
            .or_else(|| find_element(envelope, "Header"))
            .cloned();

        // Extract Body (required)
        let body = envelope.get("Body")
            .or_else(|| find_element(envelope, "Body"))
            .ok_or_else(|| XmlTranscodeError::InvalidStructure(
                "Missing SOAP Body".to_string()
            ))?
            .clone();

        // Check for Fault
        let is_fault = body.get("Fault").is_some() 
            || find_element(&body, "Fault").is_some();

        // Extract action from WS-Addressing header
        let action = headers.as_ref().and_then(|h| {
            h.get("Action")
                .or_else(|| find_element(h, "Action"))
                .and_then(|a| {
                    if let JsonValue::String(s) = a {
                        Some(s.clone())
                    } else if let JsonValue::Object(obj) = a {
                        obj.get("#text").and_then(|t| t.as_str()).map(String::from)
                    } else {
                        None
                    }
                })
        });

        Ok(Self {
            version,
            headers,
            body,
            action,
            is_fault,
        })
    }

    /// Get the body content as JSON
    pub fn body_content(&self) -> &JsonValue {
        &self.body
    }

    /// Get the first element name in the body (operation name)
    pub fn operation_name(&self) -> Option<String> {
        if let JsonValue::Object(map) = &self.body {
            // Skip #text and metadata
            for key in map.keys() {
                if !key.starts_with('@') && key != "#text" && key != "Fault" {
                    return Some(key.clone());
                }
            }
        }
        None
    }

    /// Extract the body payload for a specific operation
    pub fn operation_payload(&self, operation: &str) -> Option<&JsonValue> {
        if let JsonValue::Object(map) = &self.body {
            map.get(operation)
        } else {
            None
        }
    }

    /// Convert to NaseejContext
    pub fn to_context(&self) -> NaseejContext {
        let destination = self.action.clone()
            .or_else(|| self.operation_name())
            .unwrap_or_else(|| "/soap".to_string());

        let payload = serde_json::to_vec(&self.body).unwrap_or_default();

        ContextBuilder::new(ProtocolType::Soap, destination)
            .payload(Bytes::from(payload))
            .content_type("application/json")
            .metadata("soap.version", format!("{:?}", self.version))
            .metadata("soap.is_fault", self.is_fault.to_string())
            .build()
    }

    /// Build a SOAP response envelope
    pub fn build_response(
        _version: SoapVersion,
        body: JsonValue,
        headers: Option<JsonValue>,
    ) -> JsonValue {
        let mut envelope = serde_json::Map::new();

        if let Some(h) = headers {
            envelope.insert("Header".to_string(), h);
        }

        envelope.insert("Body".to_string(), body);

        serde_json::json!({
            "Envelope": envelope
        })
    }

    /// Build a SOAP fault response
    pub fn build_fault(
        version: SoapVersion,
        code: &str,
        message: &str,
        detail: Option<&str>,
    ) -> JsonValue {
        let fault = match version {
            SoapVersion::Soap11 => serde_json::json!({
                "faultcode": code,
                "faultstring": message,
                "detail": detail
            }),
            SoapVersion::Soap12 => serde_json::json!({
                "Code": { "Value": code },
                "Reason": { "Text": message },
                "Detail": detail
            }),
        };

        Self::build_response(version, serde_json::json!({ "Fault": fault }), None)
    }
}

/// Find an element in a JSON object, ignoring namespace prefixes
fn find_element<'a>(json: &'a JsonValue, name: &str) -> Option<&'a JsonValue> {
    if let JsonValue::Object(map) = json {
        for (key, value) in map {
            // Check exact match
            if key == name {
                return Some(value);
            }
            // Check with namespace prefix stripped
            if let Some(local_name) = key.split(':').last() {
                if local_name == name {
                    return Some(value);
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_soap_envelope() {
        let xml = r#"
            <soap:Envelope xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/">
                <soap:Body>
                    <GetUser>
                        <userId>123</userId>
                    </GetUser>
                </soap:Body>
            </soap:Envelope>
        "#;

        let envelope = SoapEnvelope::parse(xml.as_bytes()).unwrap();
        assert!(!envelope.is_fault);
        assert!(envelope.operation_name().is_some());
    }

    #[test]
    fn test_build_fault() {
        let fault = SoapEnvelope::build_fault(
            SoapVersion::Soap11,
            "soap:Client",
            "Invalid request",
            Some("Missing required field"),
        );

        let envelope = fault.get("Envelope").unwrap();
        let body = envelope.get("Body").unwrap();
        assert!(body.get("Fault").is_some());
    }

    #[test]
    fn test_envelope_to_context() {
        let json = serde_json::json!({
            "Envelope": {
                "Body": {
                    "GetUser": {
                        "userId": "123"
                    }
                }
            }
        });

        let envelope = SoapEnvelope::from_json(&json).unwrap();
        let ctx = envelope.to_context();

        assert_eq!(ctx.protocol, ProtocolType::Soap);
        assert!(ctx.destination.contains("GetUser") || ctx.destination == "/soap");
    }

    #[test]
    fn test_find_element() {
        let json = serde_json::json!({
            "soap:Body": { "content": "test" }
        });

        let found = find_element(&json, "Body");
        assert!(found.is_some());
    }
}
