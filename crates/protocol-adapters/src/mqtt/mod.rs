//! MQTT Protocol Adapter
//!
//! Provides MQTT client functionality for NaseejMesh, enabling:
//! - Connection to MQTT brokers
//! - Topic subscription and publishing
//! - Message conversion to NaseejContext
//! - QoS handling

pub mod client;
pub mod bridge;

pub use client::MqttClient;
pub use bridge::MqttBridge;
