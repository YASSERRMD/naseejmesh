//! MQTT Bridge
//!
//! Bridges MQTT messages to the internal routing system, enabling:
//! - Topic-based routing to HTTP/gRPC upstreams
//! - Message transformation
//! - Bidirectional bridging

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn};

use crate::context::{NaseejContext, ProtocolType};
use super::client::{MqttClient, MqttClientConfig, MqttMessage, MqttQos};

/// MQTT bridge configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MqttBridgeConfig {
    /// MQTT client configuration
    pub client: MqttClientConfig,

    /// Topic routing rules
    #[serde(default)]
    pub routes: Vec<TopicRoute>,

    /// Enable message transformation
    #[serde(default)]
    pub transform_enabled: bool,
}

/// Topic routing rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicRoute {
    /// Topic pattern to match (supports MQTT wildcards)
    pub topic_pattern: String,

    /// Target protocol
    pub target_protocol: ProtocolType,

    /// Target destination (URL, topic, etc.)
    pub target_destination: String,

    /// Transform type (none, json, xml)
    #[serde(default)]
    pub transform: TransformType,
}

/// Transform types for message payload
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransformType {
    #[default]
    None,
    Json,
    Xml,
}

/// MQTT Bridge for routing messages
pub struct MqttBridge {
    /// MQTT client
    client: Option<MqttClient>,

    /// Configuration
    config: MqttBridgeConfig,

    /// Outbound message channel sender
    outbound_tx: mpsc::Sender<NaseejContext>,

    /// Inbound message channel receiver
    inbound_rx: mpsc::Receiver<NaseejContext>,

    /// Cancellation token
    cancel: CancellationToken,
}

impl MqttBridge {
    /// Create a new MQTT bridge
    pub fn new(
        config: MqttBridgeConfig,
        cancel: CancellationToken,
    ) -> (Self, mpsc::Sender<NaseejContext>, mpsc::Receiver<NaseejContext>) {
        let (outbound_tx, outbound_rx) = mpsc::channel(1000);
        let (inbound_tx, inbound_rx) = mpsc::channel(1000);

        let bridge = Self {
            client: None,
            config,
            outbound_tx: inbound_tx,
            inbound_rx: outbound_rx,
            cancel,
        };

        (bridge, outbound_tx, inbound_rx)
    }

    /// Start the bridge
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting MQTT bridge");

        // Connect to MQTT broker
        let client = MqttClient::connect(
            self.config.client.clone(),
            self.cancel.clone(),
        ).await?;

        self.client = Some(client);

        Ok(())
    }

    /// Run the bridge event loop
    pub async fn run(&mut self) {
        // Take ownership of client for the event loop
        let mut client = match self.client.take() {
            Some(c) => c,
            None => {
                error!("Bridge not started");
                return;
            }
        };

        loop {
            tokio::select! {
                _ = self.cancel.cancelled() => {
                    info!("MQTT bridge cancelled");
                    break;
                }

                // Handle incoming MQTT messages
                msg = client.recv() => {
                    if let Some(msg) = msg {
                        self.handle_mqtt_message(&msg).await;
                    }
                }

                // Handle outbound messages (from other protocols to MQTT)
                ctx = self.inbound_rx.recv() => {
                    if let Some(ctx) = ctx {
                        Self::handle_outbound_static(&client, ctx).await;
                    }
                }
            }
        }
    }

    /// Handle incoming MQTT message
    async fn handle_mqtt_message(&self, msg: &MqttMessage) {
        debug!(topic = %msg.topic, size = msg.payload.len(), "Processing MQTT message");

        // Convert to context
        let ctx = msg.to_context();

        // Find matching route
        if let Some(route) = self.find_route(&msg.topic) {
            // Apply transformation if needed
            let ctx = self.transform_context(ctx, route);

            // Send to outbound channel
            if self.outbound_tx.send(ctx).await.is_err() {
                warn!("Outbound channel closed");
            }
        } else {
            debug!(topic = %msg.topic, "No route found for topic");
        }
    }

    /// Handle outbound message (to MQTT) - static method to avoid borrow issues
    async fn handle_outbound_static(client: &MqttClient, ctx: NaseejContext) {
        let qos = ctx.get_metadata("mqtt.qos")
            .and_then(|q| q.parse::<u8>().ok())
            .map(|q| match q {
                0 => MqttQos::AtMostOnce,
                2 => MqttQos::ExactlyOnce,
                _ => MqttQos::AtLeastOnce,
            })
            .unwrap_or(MqttQos::AtLeastOnce);

        if let Err(e) = client.publish_context(&ctx, qos).await {
            error!(error = %e, "Failed to publish to MQTT");
        }
    }

    /// Find a matching route for a topic
    fn find_route(&self, topic: &str) -> Option<&TopicRoute> {
        self.config.routes.iter().find(|r| {
            topic_matches(&r.topic_pattern, topic)
        })
    }

    /// Apply transformation to context
    fn transform_context(&self, ctx: NaseejContext, route: &TopicRoute) -> NaseejContext {
        // For now, just update the destination
        // Full transformation will be added later
        NaseejContext {
            destination: route.target_destination.clone(),
            ..ctx
        }
    }
}

/// Check if an MQTT topic matches a pattern
/// Supports + (single level) and # (multi level) wildcards
fn topic_matches(pattern: &str, topic: &str) -> bool {
    let pattern_parts: Vec<&str> = pattern.split('/').collect();
    let topic_parts: Vec<&str> = topic.split('/').collect();

    let mut p_idx = 0;
    let mut t_idx = 0;

    while p_idx < pattern_parts.len() && t_idx < topic_parts.len() {
        match pattern_parts[p_idx] {
            "#" => return true, // Matches everything after
            "+" => {
                // Matches exactly one level
                p_idx += 1;
                t_idx += 1;
            }
            part => {
                if part != topic_parts[t_idx] {
                    return false;
                }
                p_idx += 1;
                t_idx += 1;
            }
        }
    }

    // Both must be exhausted for exact match
    p_idx == pattern_parts.len() && t_idx == topic_parts.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topic_matches_exact() {
        assert!(topic_matches("sensors/temp", "sensors/temp"));
        assert!(!topic_matches("sensors/temp", "sensors/humidity"));
    }

    #[test]
    fn test_topic_matches_single_wildcard() {
        assert!(topic_matches("sensors/+/temp", "sensors/room1/temp"));
        assert!(topic_matches("sensors/+/temp", "sensors/room2/temp"));
        assert!(!topic_matches("sensors/+/temp", "sensors/room1/humidity"));
    }

    #[test]
    fn test_topic_matches_multi_wildcard() {
        assert!(topic_matches("sensors/#", "sensors/room1/temp"));
        assert!(topic_matches("sensors/#", "sensors/room1/room2/temp"));
        assert!(topic_matches("#", "anything/at/all"));
    }

    #[test]
    fn test_topic_route_deserialization() {
        let json = r#"{
            "topic_pattern": "sensors/+/temperature",
            "target_protocol": "http",
            "target_destination": "http://localhost:8080/api/sensors"
        }"#;

        let route: TopicRoute = serde_json::from_str(json).unwrap();
        assert_eq!(route.topic_pattern, "sensors/+/temperature");
        assert_eq!(route.target_protocol, ProtocolType::Http);
    }
}
