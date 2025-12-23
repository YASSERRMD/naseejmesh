//! Streaming XML-to-JSON Transcoder
//!
//! Provides event-based XML parsing and JSON conversion without
//! building a full DOM tree, maintaining low memory usage.

use bytes::Bytes;
use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;
use serde_json::{Map, Value as JsonValue};
use std::io::BufRead;
use thiserror::Error;

/// XML transcoding errors
#[derive(Debug, Error)]
pub enum XmlTranscodeError {
    #[error("XML parse error: {0}")]
    ParseError(String),

    #[error("JSON serialization error: {0}")]
    JsonError(String),

    #[error("Encoding error: {0}")]
    EncodingError(String),

    #[error("Invalid XML structure: {0}")]
    InvalidStructure(String),
}

impl From<quick_xml::Error> for XmlTranscodeError {
    fn from(e: quick_xml::Error) -> Self {
        XmlTranscodeError::ParseError(e.to_string())
    }
}

impl From<serde_json::Error> for XmlTranscodeError {
    fn from(e: serde_json::Error) -> Self {
        XmlTranscodeError::JsonError(e.to_string())
    }
}

/// XML to JSON transcoder using streaming parsing
pub struct XmlToJson {
    /// Strip namespace prefixes from element names
    strip_namespaces: bool,

    /// Include XML attributes in JSON output
    include_attributes: bool,

    /// Attribute prefix in JSON (e.g., "@" for "@attr")
    attribute_prefix: String,

    /// Text content key (e.g., "#text")
    text_key: String,
}

impl Default for XmlToJson {
    fn default() -> Self {
        Self {
            strip_namespaces: true,
            include_attributes: true,
            attribute_prefix: "@".to_string(),
            text_key: "#text".to_string(),
        }
    }
}

impl XmlToJson {
    /// Create a new transcoder with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Configure namespace stripping
    pub fn strip_namespaces(mut self, strip: bool) -> Self {
        self.strip_namespaces = strip;
        self
    }

    /// Configure attribute inclusion
    pub fn include_attributes(mut self, include: bool) -> Self {
        self.include_attributes = include;
        self
    }

    /// Convert XML bytes to JSON
    pub fn transcode(&self, xml: &[u8]) -> Result<Bytes, XmlTranscodeError> {
        let mut reader = Reader::from_reader(xml);
        reader.config_mut().trim_text(true);

        let json = self.parse_element(&mut reader, None)?;
        let bytes = serde_json::to_vec(&json)?;

        Ok(Bytes::from(bytes))
    }

    /// Convert XML string to JSON
    pub fn transcode_str(&self, xml: &str) -> Result<JsonValue, XmlTranscodeError> {
        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(true);

        self.parse_element_str(&mut reader, None)
    }

    /// Parse an XML element from a BufRead reader
    fn parse_element<R: BufRead>(
        &self,
        reader: &mut Reader<R>,
        start_event: Option<BytesStart>,
    ) -> Result<JsonValue, XmlTranscodeError> {
        let mut buf = Vec::new();

        // If we have a start event, use it; otherwise read the first element
        let start = if let Some(s) = start_event {
            s.to_owned()
        } else {
            loop {
                match reader.read_event_into(&mut buf)? {
                    Event::Start(e) => break e.to_owned(),
                    Event::Empty(e) => {
                        return self.parse_empty_element(&e);
                    }
                    Event::Eof => return Ok(JsonValue::Null),
                    _ => {}
                }
                buf.clear();
            }
        };

        let name = self.element_name(&start);
        let mut obj = Map::new();

        // Add attributes
        if self.include_attributes {
            for attr in start.attributes().flatten() {
                let attr_name = format!(
                    "{}{}",
                    self.attribute_prefix,
                    String::from_utf8_lossy(attr.key.as_ref())
                );
                let attr_value = String::from_utf8_lossy(&attr.value).to_string();
                obj.insert(attr_name, JsonValue::String(attr_value));
            }
        }

        // Parse children
        let mut text_content = String::new();
        let mut children: std::collections::HashMap<String, Vec<JsonValue>> = 
            std::collections::HashMap::new();

        loop {
            buf.clear();
            match reader.read_event_into(&mut buf)? {
                Event::Start(e) => {
                    let child_name = self.element_name(&e);
                    let child_value = self.parse_element(reader, Some(e.to_owned()))?;
                    
                    children
                        .entry(child_name)
                        .or_default()
                        .push(child_value);
                }
                Event::Empty(e) => {
                    let child_name = self.element_name(&e);
                    let child_value = self.parse_empty_element(&e)?;
                    
                    children
                        .entry(child_name)
                        .or_default()
                        .push(child_value);
                }
                Event::Text(e) => {
                    let text = e.unescape()?;
                    if !text.trim().is_empty() {
                        text_content.push_str(&text);
                    }
                }
                Event::CData(e) => {
                    text_content.push_str(&String::from_utf8_lossy(&e));
                }
                Event::End(_) => break,
                Event::Eof => {
                    return Err(XmlTranscodeError::InvalidStructure(
                        "Unexpected end of file".to_string()
                    ));
                }
                _ => {}
            }
        }

        // Build result object
        for (child_name, values) in children {
            if values.len() == 1 {
                obj.insert(child_name, values.into_iter().next().unwrap());
            } else {
                obj.insert(child_name, JsonValue::Array(values));
            }
        }

        // Add text content
        if !text_content.is_empty() {
            if obj.is_empty() {
                // Element only has text, return string directly
                return Ok(JsonValue::Object({
                    let mut m = Map::new();
                    m.insert(name, JsonValue::String(text_content));
                    m
                }));
            } else {
                obj.insert(self.text_key.clone(), JsonValue::String(text_content));
            }
        }

        // Wrap in element name
        let mut result = Map::new();
        result.insert(name, JsonValue::Object(obj));

        Ok(JsonValue::Object(result))
    }

    /// Parse an XML element from a str reader
    fn parse_element_str(
        &self,
        reader: &mut Reader<&[u8]>,
        start_event: Option<BytesStart>,
    ) -> Result<JsonValue, XmlTranscodeError> {
        let mut buf = Vec::new();

        // If we have a start event, use it; otherwise read the first element
        let start = if let Some(s) = start_event {
            s.to_owned()
        } else {
            loop {
                match reader.read_event_into(&mut buf)? {
                    Event::Start(e) => break e.to_owned(),
                    Event::Empty(e) => {
                        return self.parse_empty_element(&e);
                    }
                    Event::Eof => return Ok(JsonValue::Null),
                    _ => {}
                }
                buf.clear();
            }
        };

        let name = self.element_name(&start);
        let mut obj = Map::new();

        // Add attributes
        if self.include_attributes {
            for attr in start.attributes().flatten() {
                let attr_name = format!(
                    "{}{}",
                    self.attribute_prefix,
                    String::from_utf8_lossy(attr.key.as_ref())
                );
                let attr_value = String::from_utf8_lossy(&attr.value).to_string();
                obj.insert(attr_name, JsonValue::String(attr_value));
            }
        }

        // Parse children
        let mut text_content = String::new();
        let mut children: std::collections::HashMap<String, Vec<JsonValue>> = 
            std::collections::HashMap::new();

        loop {
            buf.clear();
            match reader.read_event_into(&mut buf)? {
                Event::Start(e) => {
                    let child_name = self.element_name(&e);
                    let child_value = self.parse_element_str(reader, Some(e.to_owned()))?;
                    
                    children
                        .entry(child_name)
                        .or_default()
                        .push(child_value);
                }
                Event::Empty(e) => {
                    let child_name = self.element_name(&e);
                    let child_value = self.parse_empty_element(&e)?;
                    
                    children
                        .entry(child_name)
                        .or_default()
                        .push(child_value);
                }
                Event::Text(e) => {
                    let text = e.unescape()?;
                    if !text.trim().is_empty() {
                        text_content.push_str(&text);
                    }
                }
                Event::CData(e) => {
                    text_content.push_str(&String::from_utf8_lossy(&e));
                }
                Event::End(_) => break,
                Event::Eof => {
                    return Err(XmlTranscodeError::InvalidStructure(
                        "Unexpected end of file".to_string()
                    ));
                }
                _ => {}
            }
        }

        // Build result object
        for (child_name, values) in children {
            if values.len() == 1 {
                obj.insert(child_name, values.into_iter().next().unwrap());
            } else {
                obj.insert(child_name, JsonValue::Array(values));
            }
        }

        // Add text content
        if !text_content.is_empty() {
            if obj.is_empty() {
                // Element only has text, return string directly
                return Ok(JsonValue::Object({
                    let mut m = Map::new();
                    m.insert(name, JsonValue::String(text_content));
                    m
                }));
            } else {
                obj.insert(self.text_key.clone(), JsonValue::String(text_content));
            }
        }

        // Wrap in element name
        let mut result = Map::new();
        result.insert(name, JsonValue::Object(obj));

        Ok(JsonValue::Object(result))
    }

    /// Parse an empty element (self-closing tag)
    fn parse_empty_element(&self, event: &BytesStart) -> Result<JsonValue, XmlTranscodeError> {
        let name = self.element_name(event);
        let mut obj = Map::new();

        // Add attributes
        if self.include_attributes {
            for attr in event.attributes().flatten() {
                let attr_name = format!(
                    "{}{}",
                    self.attribute_prefix,
                    String::from_utf8_lossy(attr.key.as_ref())
                );
                let attr_value = String::from_utf8_lossy(&attr.value).to_string();
                obj.insert(attr_name, JsonValue::String(attr_value));
            }
        }

        let mut result = Map::new();
        if obj.is_empty() {
            result.insert(name, JsonValue::Null);
        } else {
            result.insert(name, JsonValue::Object(obj));
        }

        Ok(JsonValue::Object(result))
    }

    /// Extract element name, optionally stripping namespace prefix
    fn element_name(&self, event: &BytesStart) -> String {
        let full_name = String::from_utf8_lossy(event.name().as_ref()).to_string();
        
        if self.strip_namespaces {
            full_name
                .split(':')
                .last()
                .unwrap_or(&full_name)
                .to_string()
        } else {
            full_name
        }
    }
}

/// JSON to XML transcoder
pub struct JsonToXml {
    /// Root element name
    root_name: String,

    /// Add XML declaration
    add_declaration: bool,

    /// Indent output
    pretty_print: bool,
}

impl Default for JsonToXml {
    fn default() -> Self {
        Self {
            root_name: "root".to_string(),
            add_declaration: true,
            pretty_print: false,
        }
    }
}

impl JsonToXml {
    /// Create a new transcoder
    pub fn new(root_name: impl Into<String>) -> Self {
        Self {
            root_name: root_name.into(),
            ..Default::default()
        }
    }

    /// Configure XML declaration
    pub fn with_declaration(mut self, add: bool) -> Self {
        self.add_declaration = add;
        self
    }

    /// Configure pretty printing
    pub fn pretty_print(mut self, pretty: bool) -> Self {
        self.pretty_print = pretty;
        self
    }

    /// Convert JSON to XML
    pub fn transcode(&self, json: &JsonValue) -> Result<Bytes, XmlTranscodeError> {
        let mut output = Vec::new();

        if self.add_declaration {
            output.extend_from_slice(b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>");
            if self.pretty_print {
                output.push(b'\n');
            }
        }

        self.write_value(&mut output, &self.root_name, json)?;

        Ok(Bytes::from(output))
    }

    /// Write a JSON value as XML
    fn write_value(
        &self,
        output: &mut Vec<u8>,
        name: &str,
        value: &JsonValue,
    ) -> Result<(), XmlTranscodeError> {
        match value {
            JsonValue::Null => {
                output.extend_from_slice(format!("<{}/>", name).as_bytes());
            }
            JsonValue::Bool(b) => {
                output.extend_from_slice(format!("<{}>{}</{}>", name, b, name).as_bytes());
            }
            JsonValue::Number(n) => {
                output.extend_from_slice(format!("<{}>{}</{}>", name, n, name).as_bytes());
            }
            JsonValue::String(s) => {
                let escaped = escape_xml(s);
                output.extend_from_slice(format!("<{}>{}</{}>", name, escaped, name).as_bytes());
            }
            JsonValue::Array(arr) => {
                for item in arr {
                    self.write_value(output, name, item)?;
                }
            }
            JsonValue::Object(obj) => {
                output.extend_from_slice(format!("<{}", name).as_bytes());

                // Write attributes first
                for (key, val) in obj {
                    if key.starts_with('@') {
                        let attr_name = &key[1..];
                        if let JsonValue::String(attr_val) = val {
                            output.extend_from_slice(
                                format!(" {}=\"{}\"", attr_name, escape_xml_attr(attr_val)).as_bytes()
                            );
                        }
                    }
                }

                output.push(b'>');

                // Write children
                for (key, val) in obj {
                    if !key.starts_with('@') && key != "#text" {
                        self.write_value(output, key, val)?;
                    }
                }

                // Write text content
                if let Some(text) = obj.get("#text") {
                    if let JsonValue::String(s) = text {
                        output.extend_from_slice(escape_xml(s).as_bytes());
                    }
                }

                output.extend_from_slice(format!("</{}>", name).as_bytes());
            }
        }

        Ok(())
    }
}

/// Escape special XML characters in text
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
}

/// Escape special XML characters in attributes
fn escape_xml_attr(s: &str) -> String {
    escape_xml(s)
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_xml_to_json() {
        let xml = r#"<root><name>test</name><value>42</value></root>"#;
        let transcoder = XmlToJson::new();
        let result = transcoder.transcode_str(xml).unwrap();

        assert!(result.is_object());
        let root = result.get("root").unwrap();
        assert!(root.get("name").is_some());
        assert!(root.get("value").is_some());
    }

    #[test]
    fn test_xml_with_attributes() {
        let xml = r#"<item id="123" type="product">content</item>"#;
        let transcoder = XmlToJson::new();
        let result = transcoder.transcode_str(xml).unwrap();

        let item = result.get("item").unwrap();
        assert_eq!(item.get("@id").unwrap(), "123");
        assert_eq!(item.get("@type").unwrap(), "product");
    }

    #[test]
    fn test_xml_namespace_stripping() {
        let xml = r#"<soap:Envelope><soap:Body>test</soap:Body></soap:Envelope>"#;
        let transcoder = XmlToJson::new().strip_namespaces(true);
        let result = transcoder.transcode_str(xml).unwrap();

        // Should have "Envelope" not "soap:Envelope"
        assert!(result.get("Envelope").is_some());
    }

    #[test]
    fn test_json_to_xml() {
        let json = serde_json::json!({
            "name": "test",
            "value": 42
        });

        let transcoder = JsonToXml::new("data").with_declaration(false);
        let result = transcoder.transcode(&json).unwrap();
        let xml = String::from_utf8(result.to_vec()).unwrap();

        assert!(xml.contains("<data>"));
        assert!(xml.contains("<name>test</name>"));
        assert!(xml.contains("<value>42</value>"));
    }

    #[test]
    fn test_escape_xml() {
        assert_eq!(escape_xml("<test>"), "&lt;test&gt;");
        assert_eq!(escape_xml("a & b"), "a &amp; b");
    }
}
