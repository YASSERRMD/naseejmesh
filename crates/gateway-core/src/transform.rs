//! Rhai Transformation Middleware
//!
//! Executes AI-generated Rhai scripts for data transformation.
//! Scripts are compiled once and executed for each request.

use rhai::{Dynamic, Engine, ImmutableString, Scope, AST};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::sync::Arc;
use thiserror::Error;
use tracing::{debug, info, warn};

/// Transformation errors
#[derive(Debug, Error)]
pub enum TransformError {
    #[error("Script compilation failed: {0}")]
    CompileError(String),

    #[error("Script execution failed: {0}")]
    ExecutionError(String),

    #[error("Timeout exceeded")]
    Timeout,

    #[error("Output conversion failed: {0}")]
    OutputError(String),
}

/// Transformation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformResult {
    /// Transformed output
    pub output: String,

    /// Execution time in microseconds
    pub execution_us: u64,

    /// Any warnings generated
    #[serde(default)]
    pub warnings: Vec<String>,
}

/// Compiled Rhai transformer
pub struct RhaiTransformer {
    /// Shared Rhai engine
    engine: Arc<Engine>,

    /// Compiled AST (reusable)
    ast: AST,

    /// Script source (for debugging)
    source: String,
}

impl RhaiTransformer {
    /// Create a new transformer from script source
    pub fn new(script: &str) -> Result<Self, TransformError> {
        let engine = Self::create_engine();

        let ast = engine
            .compile(script)
            .map_err(|e| TransformError::CompileError(e.to_string()))?;

        info!(
            script_len = script.len(),
            "Compiled Rhai transformation script"
        );

        Ok(Self {
            engine: Arc::new(engine),
            ast,
            source: script.to_string(),
        })
    }

    /// Create a configured Rhai engine
    fn create_engine() -> Engine {
        let mut engine = Engine::new();

        // Safety limits
        engine.set_max_operations(100_000);
        engine.set_max_expr_depths(64, 64);
        engine.set_max_string_size(1024 * 1024); // 1MB
        engine.set_max_array_size(10_000);
        engine.set_max_map_size(10_000);

        // Register helper functions
        Self::register_helpers(&mut engine);

        engine
    }

    /// Register helper functions for transformations
    fn register_helpers(engine: &mut Engine) {
        // JSON parsing
        engine.register_fn("parse_json", |s: &str| -> Dynamic {
            match serde_json::from_str::<JsonValue>(s) {
                Ok(v) => json_to_dynamic(&v),
                Err(_) => Dynamic::UNIT,
            }
        });

        // JSON stringification
        engine.register_fn("to_json", |d: Dynamic| -> ImmutableString {
            dynamic_to_json(&d)
                .map(|v| serde_json::to_string(&v).unwrap_or_default())
                .unwrap_or_default()
                .into()
        });

        // Pretty JSON
        engine.register_fn("to_json_pretty", |d: Dynamic| -> ImmutableString {
            dynamic_to_json(&d)
                .map(|v| serde_json::to_string_pretty(&v).unwrap_or_default())
                .unwrap_or_default()
                .into()
        });

        // XML wrapping (simple)
        engine.register_fn("wrap_xml", |tag: &str, content: &str| -> ImmutableString {
            format!("<{}>{}</{}>", tag, escape_xml(content), tag).into()
        });

        // String utilities
        engine.register_fn("trim", |s: &str| -> ImmutableString { s.trim().into() });
        engine.register_fn("upper", |s: &str| -> ImmutableString { s.to_uppercase().into() });
        engine.register_fn("lower", |s: &str| -> ImmutableString { s.to_lowercase().into() });

        // Number conversions (f64)
        engine.register_fn("celsius_to_fahrenheit", |c: f64| -> f64 { c * 9.0 / 5.0 + 32.0 });
        engine.register_fn("fahrenheit_to_celsius", |f: f64| -> f64 { (f - 32.0) * 5.0 / 9.0 });
        
        // Number conversions (i64 -> f64 for JSON compatibility)
        engine.register_fn("celsius_to_fahrenheit", |c: i64| -> f64 { (c as f64) * 9.0 / 5.0 + 32.0 });
        engine.register_fn("fahrenheit_to_celsius", |f: i64| -> f64 { ((f as f64) - 32.0) * 5.0 / 9.0 });

        // Timestamp
        engine.register_fn("now_iso", || -> ImmutableString {
            chrono::Utc::now().to_rfc3339().into()
        });

        engine.register_fn("timestamp_ms", || -> i64 {
            chrono::Utc::now().timestamp_millis()
        });

        // UUID
        engine.register_fn("uuid", || -> ImmutableString {
            uuid::Uuid::new_v4().to_string().into()
        });

        // Logging
        engine.register_fn("log", |msg: &str| {
            info!(rhai = true, "{}", msg);
        });

        engine.register_fn("debug", |msg: &str| {
            debug!(rhai = true, "{}", msg);
        });

        engine.register_fn("warn", |msg: &str| {
            warn!(rhai = true, "{}", msg);
        });
    }

    /// Execute the transformation
    pub fn execute(&self, input: &str) -> Result<TransformResult, TransformError> {
        let start = std::time::Instant::now();

        let mut scope = Scope::new();
        scope.push("input", input.to_string());
        scope.push("output", String::new());

        // Execute script
        self.engine
            .run_ast_with_scope(&mut scope, &self.ast)
            .map_err(|e| TransformError::ExecutionError(e.to_string()))?;

        // Get output
        let output = scope
            .get_value::<String>("output")
            .unwrap_or_default();

        let execution_us = start.elapsed().as_micros() as u64;

        debug!(
            input_len = input.len(),
            output_len = output.len(),
            execution_us = execution_us,
            "Transformation complete"
        );

        Ok(TransformResult {
            output,
            execution_us,
            warnings: vec![],
        })
    }

    /// Get the script source
    pub fn source(&self) -> &str {
        &self.source
    }
}

/// Validate a script without executing it
pub fn validate_script(script: &str) -> Result<(), TransformError> {
    let engine = RhaiTransformer::create_engine();
    engine
        .compile(script)
        .map_err(|e| TransformError::CompileError(e.to_string()))?;
    Ok(())
}

/// Simulate a transformation (dry run)
pub fn simulate(script: &str, input: &str) -> Result<TransformResult, TransformError> {
    let transformer = RhaiTransformer::new(script)?;
    transformer.execute(input)
}

/// Convert JSON to Rhai Dynamic
fn json_to_dynamic(value: &JsonValue) -> Dynamic {
    match value {
        JsonValue::Null => Dynamic::UNIT,
        JsonValue::Bool(b) => Dynamic::from(*b),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                Dynamic::from(i)
            } else if let Some(f) = n.as_f64() {
                Dynamic::from(f)
            } else {
                Dynamic::UNIT
            }
        }
        JsonValue::String(s) => Dynamic::from(s.clone()),
        JsonValue::Array(arr) => {
            let vec: Vec<Dynamic> = arr.iter().map(json_to_dynamic).collect();
            Dynamic::from(vec)
        }
        JsonValue::Object(obj) => {
            let map: rhai::Map = obj
                .iter()
                .map(|(k, v)| (k.clone().into(), json_to_dynamic(v)))
                .collect();
            Dynamic::from(map)
        }
    }
}

/// Convert Rhai Dynamic to JSON
fn dynamic_to_json(value: &Dynamic) -> Option<JsonValue> {
    if value.is_unit() {
        return Some(JsonValue::Null);
    }

    if let Some(b) = value.clone().try_cast::<bool>() {
        return Some(JsonValue::Bool(b));
    }

    if let Some(i) = value.clone().try_cast::<i64>() {
        return Some(JsonValue::Number(i.into()));
    }

    if let Some(f) = value.clone().try_cast::<f64>() {
        return serde_json::Number::from_f64(f).map(JsonValue::Number);
    }

    if let Some(s) = value.clone().try_cast::<ImmutableString>() {
        return Some(JsonValue::String(s.to_string()));
    }

    if let Some(arr) = value.clone().try_cast::<Vec<Dynamic>>() {
        let json_arr: Vec<JsonValue> = arr.iter().filter_map(dynamic_to_json).collect();
        return Some(JsonValue::Array(json_arr));
    }

    if let Some(map) = value.clone().try_cast::<rhai::Map>() {
        let json_obj: serde_json::Map<String, JsonValue> = map
            .iter()
            .filter_map(|(k, v)| dynamic_to_json(v).map(|jv| (k.to_string(), jv)))
            .collect();
        return Some(JsonValue::Object(json_obj));
    }

    None
}

/// Escape XML special characters
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_transform() {
        let script = r#"
            let data = parse_json(input);
            data["processed"] = true;
            output = to_json(data);
        "#;

        let transformer = RhaiTransformer::new(script).unwrap();
        let result = transformer.execute(r#"{"value": 42}"#).unwrap();

        assert!(result.output.contains("processed"));
        assert!(result.output.contains("true"));
    }

    #[test]
    fn test_temperature_conversion() {
        let script = r#"
            let data = parse_json(input);
            let temp_c = data["temp"];
            data["temp_f"] = celsius_to_fahrenheit(temp_c);
            output = to_json(data);
        "#;

        let transformer = RhaiTransformer::new(script).unwrap();
        let result = transformer.execute(r#"{"temp": 20}"#).unwrap();

        assert!(result.output.contains("68")); // 20°C = 68°F
    }

    #[test]
    fn test_xml_wrapping() {
        let script = r#"
            output = wrap_xml("temperature", input);
        "#;

        let transformer = RhaiTransformer::new(script).unwrap();
        let result = transformer.execute("25").unwrap();

        assert_eq!(result.output, "<temperature>25</temperature>");
    }

    #[test]
    fn test_validate_valid_script() {
        let result = validate_script("let x = 1 + 2;");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_invalid_script() {
        let result = validate_script("let x = ;");
        assert!(result.is_err());
    }

    #[test]
    fn test_simulate() {
        let script = r#"output = upper(input);"#;
        let result = simulate(script, "hello").unwrap();
        assert_eq!(result.output, "HELLO");
    }

    #[test]
    fn test_helper_functions() {
        // Test uuid
        let script = r#"output = uuid();"#;
        let result = simulate(script, "").unwrap();
        assert_eq!(result.output.len(), 36); // UUID format

        // Test timestamp
        let script = r#"output = "" + timestamp_ms();"#;
        let result = simulate(script, "").unwrap();
        assert!(result.output.parse::<i64>().is_ok());
    }
}
