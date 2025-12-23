//! Rhai Scripting Engine
//!
//! Provides safe, embedded scripting for data transformations.
//! The AI generates Rhai scripts that can transform messages between protocols.

use rhai::{Dynamic, Engine, ImmutableString, Scope, AST};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tracing::{debug, error, info, warn};

/// Rhai engine errors
#[derive(Debug, Error)]
pub enum RhaiError {
    #[error("Script compilation failed: {0}")]
    CompileError(String),

    #[error("Script execution failed: {0}")]
    ExecutionError(String),

    #[error("Type conversion failed: {0}")]
    ConversionError(String),

    #[error("Validation failed: {0}")]
    ValidationError(String),
}

/// Script validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Transformation context passed to scripts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformContext {
    /// Input payload as JSON
    pub payload: JsonValue,

    /// Metadata map
    pub metadata: HashMap<String, String>,

    /// Source protocol
    pub protocol: String,

    /// Destination path
    pub destination: String,
}

/// Rhai scripting engine for data transformations
pub struct RhaiEngine {
    /// The Rhai engine instance
    engine: Engine,

    /// Compiled script cache
    script_cache: HashMap<String, Arc<AST>>,
}

impl RhaiEngine {
    /// Create a new Rhai engine with NaseejMesh functions
    pub fn new() -> Self {
        let mut engine = Engine::new();

        // Set safety limits
        engine.set_max_operations(100_000);
        engine.set_max_expr_depths(64, 64);
        engine.set_max_string_size(1024 * 1024); // 1MB
        engine.set_max_array_size(10_000);
        engine.set_max_map_size(10_000);

        // Register custom functions for transformations
        Self::register_functions(&mut engine);

        Self {
            engine,
            script_cache: HashMap::new(),
        }
    }

    /// Register custom functions available in scripts
    fn register_functions(engine: &mut Engine) {
        // JSON parsing
        engine.register_fn("parse_json", |s: &str| -> Dynamic {
            match serde_json::from_str::<JsonValue>(s) {
                Ok(v) => json_to_dynamic(&v),
                Err(_) => Dynamic::UNIT,
            }
        });

        // JSON stringification
        engine.register_fn("to_json", |d: Dynamic| -> ImmutableString {
            match dynamic_to_json(&d) {
                Ok(v) => serde_json::to_string(&v).unwrap_or_default().into(),
                Err(_) => "{}".into(),
            }
        });

        // String manipulation
        engine.register_fn("trim", |s: &str| -> ImmutableString {
            s.trim().into()
        });

        engine.register_fn("uppercase", |s: &str| -> ImmutableString {
            s.to_uppercase().into()
        });

        engine.register_fn("lowercase", |s: &str| -> ImmutableString {
            s.to_lowercase().into()
        });

        // Date/time functions
        engine.register_fn("now_utc", || -> ImmutableString {
            chrono::Utc::now().to_rfc3339().into()
        });

        engine.register_fn("timestamp", || -> i64 {
            chrono::Utc::now().timestamp()
        });

        // Logging
        engine.register_fn("log_debug", |msg: &str| {
            debug!(rhai = true, "{}", msg);
        });

        engine.register_fn("log_info", |msg: &str| {
            info!(rhai = true, "{}", msg);
        });

        engine.register_fn("log_warn", |msg: &str| {
            warn!(rhai = true, "{}", msg);
        });

        // UUID generation
        engine.register_fn("uuid", || -> ImmutableString {
            uuid::Uuid::new_v4().to_string().into()
        });
    }

    /// Validate a script without executing it
    pub fn validate(&self, script: &str) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Check for potentially dangerous patterns
        if script.contains("std::") {
            warnings.push("Script contains std:: references which may not be available".to_string());
        }

        // Try to compile
        match self.engine.compile(script) {
            Ok(_) => {
                debug!("Script validation passed");
            }
            Err(e) => {
                errors.push(format!("Compilation error: {}", e));
            }
        }

        ValidationResult {
            valid: errors.is_empty(),
            errors,
            warnings,
        }
    }

    /// Compile and cache a script
    pub fn compile(&mut self, script_id: &str, script: &str) -> Result<(), RhaiError> {
        let ast = self.engine
            .compile(script)
            .map_err(|e| RhaiError::CompileError(e.to_string()))?;

        self.script_cache.insert(script_id.to_string(), Arc::new(ast));
        debug!(script_id = %script_id, "Script compiled and cached");

        Ok(())
    }

    /// Execute a script with the given context
    pub fn execute(
        &self,
        script: &str,
        ctx: &TransformContext,
    ) -> Result<TransformContext, RhaiError> {
        // Compile the script
        let ast = self.engine
            .compile(script)
            .map_err(|e| RhaiError::CompileError(e.to_string()))?;

        self.execute_ast(&ast, ctx)
    }

    /// Execute a cached script
    pub fn execute_cached(
        &self,
        script_id: &str,
        ctx: &TransformContext,
    ) -> Result<TransformContext, RhaiError> {
        let ast = self.script_cache
            .get(script_id)
            .ok_or_else(|| RhaiError::ExecutionError(format!("Script not found: {}", script_id)))?;

        self.execute_ast(ast, ctx)
    }

    /// Execute an AST with context
    fn execute_ast(
        &self,
        ast: &AST,
        ctx: &TransformContext,
    ) -> Result<TransformContext, RhaiError> {
        let mut scope = Scope::new();

        // Inject context variables
        scope.push("payload", json_to_dynamic(&ctx.payload));
        scope.push("metadata", ctx.metadata.clone());
        scope.push("protocol", ctx.protocol.clone());
        scope.push("destination", ctx.destination.clone());

        // Execute
        self.engine
            .run_ast_with_scope(&mut scope, ast)
            .map_err(|e| RhaiError::ExecutionError(e.to_string()))?;

        // Extract modified context
        let payload = scope
            .get_value::<Dynamic>("payload")
            .map(|d| dynamic_to_json(&d).unwrap_or(JsonValue::Null))
            .unwrap_or(ctx.payload.clone());

        let metadata = scope
            .get_value::<HashMap<String, String>>("metadata")
            .unwrap_or_else(|| ctx.metadata.clone());

        let destination = scope
            .get_value::<String>("destination")
            .unwrap_or_else(|| ctx.destination.clone());

        Ok(TransformContext {
            payload,
            metadata,
            protocol: ctx.protocol.clone(),
            destination,
        })
    }

    /// Get the number of cached scripts
    pub fn cached_count(&self) -> usize {
        self.script_cache.len()
    }
}

impl Default for RhaiEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert JSON value to Rhai Dynamic
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

/// Convert Rhai Dynamic to JSON value
fn dynamic_to_json(value: &Dynamic) -> Result<JsonValue, RhaiError> {
    if value.is_unit() {
        return Ok(JsonValue::Null);
    }

    if let Some(b) = value.clone().try_cast::<bool>() {
        return Ok(JsonValue::Bool(b));
    }

    if let Some(i) = value.clone().try_cast::<i64>() {
        return Ok(JsonValue::Number(i.into()));
    }

    if let Some(f) = value.clone().try_cast::<f64>() {
        return Ok(serde_json::Number::from_f64(f)
            .map(JsonValue::Number)
            .unwrap_or(JsonValue::Null));
    }

    if let Some(s) = value.clone().try_cast::<ImmutableString>() {
        return Ok(JsonValue::String(s.to_string()));
    }

    if let Some(arr) = value.clone().try_cast::<Vec<Dynamic>>() {
        let json_arr: Result<Vec<JsonValue>, _> = arr.iter().map(dynamic_to_json).collect();
        return Ok(JsonValue::Array(json_arr?));
    }

    if let Some(map) = value.clone().try_cast::<rhai::Map>() {
        let json_obj: Result<serde_json::Map<String, JsonValue>, _> = map
            .iter()
            .map(|(k, v)| dynamic_to_json(v).map(|jv| (k.to_string(), jv)))
            .collect();
        return Ok(JsonValue::Object(json_obj?));
    }

    // Fallback to string representation
    Ok(JsonValue::String(format!("{:?}", value)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_valid_script() {
        let engine = RhaiEngine::new();
        let result = engine.validate("let x = 1 + 2;");
        assert!(result.valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validate_invalid_script() {
        let engine = RhaiEngine::new();
        let result = engine.validate("let x = ;");
        assert!(!result.valid);
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_execute_simple_transform() {
        let engine = RhaiEngine::new();
        let ctx = TransformContext {
            payload: serde_json::json!({"temp": 20}),
            metadata: HashMap::new(),
            protocol: "mqtt".to_string(),
            destination: "/sensors".to_string(),
        };

        let script = r#"
            payload["temp_f"] = payload["temp"] * 9 / 5 + 32;
        "#;

        let result = engine.execute(script, &ctx).unwrap();
        assert!(result.payload.get("temp_f").is_some());
    }

    #[test]
    fn test_built_in_functions() {
        let engine = RhaiEngine::new();
        let ctx = TransformContext {
            payload: serde_json::json!({}),
            metadata: HashMap::new(),
            protocol: "http".to_string(),
            destination: "/api".to_string(),
        };

        let script = r#"
            payload["timestamp"] = timestamp();
            payload["id"] = uuid();
        "#;

        let result = engine.execute(script, &ctx).unwrap();
        assert!(result.payload.get("timestamp").is_some());
        assert!(result.payload.get("id").is_some());
    }

    #[test]
    fn test_json_functions() {
        let engine = RhaiEngine::new();
        let result = engine.validate(r#"
            let data = parse_json("{\"a\": 1}");
            let json = to_json(data);
        "#);
        assert!(result.valid);
    }
}
