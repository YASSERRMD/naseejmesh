//! SOAP/XML Protocol Adapter
//!
//! Provides SOAP handling with streaming XML-to-JSON transcoding:
//! - Event-based XML parsing (no DOM)
//! - SOAP envelope extraction
//! - XML ↔ JSON conversion
//! - Low memory footprint

pub mod transcoder;
pub mod envelope;

pub use transcoder::{XmlToJson, JsonToXml, XmlTranscodeError};
pub use envelope::SoapEnvelope;
