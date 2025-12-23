//! # Protocol Adapters
//!
//! Multi-protocol adapters for NaseejMesh - the Polyglot Protocol Fabric.
//!
//! This crate provides:
//! - `NaseejContext`: Universal message context for all protocols
//! - `Supervisor`: Dynamic listener management with graceful shutdown
//! - Protocol adapters: MQTT, gRPC, SOAP

pub mod context;
pub mod supervisor;
pub mod telemetry;
pub mod mqtt;
pub mod grpc;
pub mod soap;

pub use context::{NaseejContext, ProtocolType};
pub use supervisor::{ListenerManager, ListenerConfig, ServiceConfig};
pub use telemetry::TelemetryConfig;
