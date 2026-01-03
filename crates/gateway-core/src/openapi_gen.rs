//! OpenAPI Documentation Generator
//!
//! Generates OpenAPI v3.1 documentation from current route configuration.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use crate::config::{Route, RouterMap};
use std::collections::HashMap;

/// OpenAPI Document Structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenApiDoc {
    pub openapi: String,
    pub info: OpenApiInfo,
    pub paths: HashMap<String, PathItem>,
    pub components: Option<Components>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenApiInfo {
    pub title: String,
    pub version: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PathItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub get: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub put: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patch: Option<Operation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    pub summary: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "operationId")]
    pub operation_id: Option<String>,
    pub responses: HashMap<String, Response>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<HashMap<String, MediaType>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaType {
    pub schema: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Components {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schemas: Option<HashMap<String, Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_schemes: Option<HashMap<String, Value>>,
}

/// Generator for OpenAPI docs
pub struct OpenApiGenerator;

impl OpenApiGenerator {
    pub fn generate(routes: &RouterMap) -> OpenApiDoc {
        let mut paths: HashMap<String, PathItem> = HashMap::new();

        // Group routes by path
        // Note: RouterMap is optimized for matching, not listing.
        // We need access to the underlying list of routes from somewhere else ideally.
        // But assuming we can iterate or have the list:
        
        /* 
           This implementation assumes we pass a list of routes, not RouterMap.
           Refactoring required: passing Vec<Route> instead.
        */
        
        OpenApiDoc {
            openapi: "3.1.0".to_string(),
            info: OpenApiInfo {
                title: "NaseejMesh Gateway API".to_string(),
                version: "1.0.0".to_string(),
                description: Some("Auto-generated API documentation for deployed routes".to_string()),
            },
            paths,
            components: None,
        }
    }

    pub fn generate_from_routes(routes: &[Route]) -> OpenApiDoc {
        let mut paths: HashMap<String, PathItem> = HashMap::new();

        for route in routes {
            let path_key = route.path.clone(); // In reality need to normalize path params
            let item = paths.entry(path_key).or_default();

            for method in &route.methods {
                let op = Operation {
                    summary: Some(format!("{} {}", method, route.path)),
                    description: Some(format!("Upstream: {}", route.upstream)),
                    operation_id: Some(format!("{}_{}", method.to_lowercase(), route.id)),
                    responses: {
                        let mut r = HashMap::new();
                        r.insert("200".to_string(), Response {
                            description: "Successful response".to_string(),
                            content: None,
                        });
                        r
                    },
                };

                match method.as_str() {
                    "GET" => item.get = Some(op),
                    "POST" => item.post = Some(op),
                    "PUT" => item.put = Some(op),
                    "DELETE" => item.delete = Some(op),
                    "PATCH" => item.patch = Some(op),
                    _ => {}
                }
            }
        }

        OpenApiDoc {
            openapi: "3.1.0".to_string(),
            info: OpenApiInfo {
                title: "NaseejMesh Gateway API".to_string(),
                version: "1.0.0".to_string(),
                description: Some("Auto-generated API documentation for deployed routes".to_string()),
            },
            paths,
            components: Some(Components {
                schemas: None,
                security_schemes: Some({
                    let mut s = HashMap::new();
                    s.insert("BearerAuth".to_string(), json!({
                        "type": "http",
                        "scheme": "bearer",
                        "bearerFormat": "JWT"
                    }));
                    s
                }),
            }),
        }
    }
}
