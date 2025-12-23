//! MQTT Client Implementation
//!
//! Wrapper around rumqttc for MQTT connectivity with automatic
//! reconnection and message handling.

use bytes::Bytes;
use rumqttc::{AsyncClient, Event, Incoming, MqttOptions, QoS};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn};

use crate::context::{ContextBuilder, NaseejContext, ProtocolType, TraceId};
use crate::telemetry::extract_trace_from_mqtt;

/// MQTT QoS levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MqttQos {
    AtMostOnce = 0,
    AtLeastOnce = 1,
    ExactlyOnce = 2,
}

impl From<MqttQos> for QoS {
    fn from(qos: MqttQos) -> Self {
        match qos {
            MqttQos::AtMostOnce => QoS::AtMostOnce,
            MqttQos::AtLeastOnce => QoS::AtLeastOnce,
            MqttQos::ExactlyOnce => QoS::ExactlyOnce,
        }
    }
}

impl From<QoS> for MqttQos {
    fn from(qos: QoS) -> Self {
        match qos {
            QoS::AtMostOnce => MqttQos::AtMostOnce,
            QoS::AtLeastOnce => MqttQos::AtLeastOnce,
            QoS::ExactlyOnce => MqttQos::ExactlyOnce,
        }
    }
}

/// MQTT client configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MqttClientConfig {
    /// Client ID (must be unique per connection)
    pub client_id: String,

    /// Broker host
    pub host: String,

    /// Broker port
    #[serde(default = "default_mqtt_port")]
    pub port: u16,

    /// Keep alive interval in seconds
    #[serde(default = "default_keep_alive")]
    pub keep_alive_secs: u64,

    /// Enable clean session
    #[serde(default = "default_clean_session")]
    pub clean_session: bool,

    /// Topics to subscribe to
    #[serde(default)]
    pub subscriptions: Vec<SubscriptionConfig>,

    /// Optional username
    pub username: Option<String>,

    /// Optional password
    pub password: Option<String>,
}

fn default_mqtt_port() -> u16 {
    1883
}

fn default_keep_alive() -> u64 {
    60
}

fn default_clean_session() -> bool {
    true
}

/// Topic subscription configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionConfig {
    /// Topic filter (supports wildcards: + and #)
    pub topic: String,

    /// QoS level
    #[serde(default)]
    pub qos: MqttQos,
}

impl Default for MqttQos {
    fn default() -> Self {
        MqttQos::AtLeastOnce
    }
}

/// Received MQTT message
#[derive(Debug, Clone)]
pub struct MqttMessage {
    /// Topic the message was published to
    pub topic: String,

    /// Message payload
    pub payload: Bytes,

    /// QoS level
    pub qos: MqttQos,

    /// Retain flag
    pub retain: bool,
}

impl MqttMessage {
    /// Convert to NaseejContext
    pub fn to_context(&self) -> NaseejContext {
        ContextBuilder::new(ProtocolType::Mqtt, &self.topic)
            .payload(self.payload.clone())
            .metadata("mqtt.qos", format!("{}", self.qos as u8))
            .metadata("mqtt.retain", self.retain.to_string())
            .build()
    }
}

/// MQTT client wrapper
pub struct MqttClient {
    /// Async client for publishing
    client: AsyncClient,

    /// Configuration
    config: MqttClientConfig,

    /// Message channel receiver
    message_rx: mpsc::Receiver<MqttMessage>,

    /// Cancellation token
    cancel: CancellationToken,
}

impl MqttClient {
    /// Create a new MQTT client and start the event loop
    pub async fn connect(
        config: MqttClientConfig,
        cancel: CancellationToken,
    ) -> Result<Self, rumqttc::ClientError> {
        info!(
            client_id = %config.client_id,
            host = %config.host,
            port = config.port,
            "Connecting to MQTT broker"
        );

        // Build MQTT options
        let mut options = MqttOptions::new(
            &config.client_id,
            &config.host,
            config.port,
        );
        options.set_keep_alive(Duration::from_secs(config.keep_alive_secs));
        options.set_clean_session(config.clean_session);

        if let (Some(user), Some(pass)) = (&config.username, &config.password) {
            options.set_credentials(user, pass);
        }

        // Create client and event loop
        let (client, mut eventloop) = AsyncClient::new(options, 100);

        // Channel for messages
        let (message_tx, message_rx) = mpsc::channel(1000);

        // Subscribe to configured topics
        for sub in &config.subscriptions {
            client.subscribe(&sub.topic, sub.qos.into()).await?;
            debug!(topic = %sub.topic, "Subscribed to MQTT topic");
        }

        // Spawn event loop handler
        let cancel_clone = cancel.clone();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = cancel_clone.cancelled() => {
                        info!("MQTT event loop cancelled");
                        break;
                    }
                    event = eventloop.poll() => {
                        match event {
                            Ok(Event::Incoming(Incoming::Publish(publish))) => {
                                let msg = MqttMessage {
                                    topic: publish.topic.clone(),
                                    payload: Bytes::from(publish.payload.to_vec()),
                                    qos: publish.qos.into(),
                                    retain: publish.retain,
                                };
                                
                                debug!(
                                    topic = %msg.topic,
                                    size = msg.payload.len(),
                                    "Received MQTT message"
                                );

                                if message_tx.send(msg).await.is_err() {
                                    warn!("Message channel closed");
                                    break;
                                }
                            }
                            Ok(Event::Incoming(Incoming::ConnAck(_))) => {
                                info!("MQTT connected");
                            }
                            Ok(Event::Incoming(Incoming::Disconnect)) => {
                                warn!("MQTT disconnected");
                            }
                            Ok(_) => {}
                            Err(e) => {
                                error!(error = %e, "MQTT connection error");
                                tokio::time::sleep(Duration::from_secs(5)).await;
                            }
                        }
                    }
                }
            }
        });

        Ok(Self {
            client,
            config,
            message_rx,
            cancel,
        })
    }

    /// Publish a message
    pub async fn publish(
        &self,
        topic: &str,
        payload: Bytes,
        qos: MqttQos,
        retain: bool,
    ) -> Result<(), rumqttc::ClientError> {
        self.client.publish(topic, qos.into(), retain, payload.to_vec()).await
    }

    /// Publish from a NaseejContext
    pub async fn publish_context(
        &self,
        ctx: &NaseejContext,
        qos: MqttQos,
    ) -> Result<(), rumqttc::ClientError> {
        self.client.publish(
            &ctx.destination,
            qos.into(),
            false,
            ctx.payload.to_vec(),
        ).await
    }

    /// Receive the next message
    pub async fn recv(&mut self) -> Option<MqttMessage> {
        self.message_rx.recv().await
    }

    /// Subscribe to a new topic
    pub async fn subscribe(
        &self,
        topic: &str,
        qos: MqttQos,
    ) -> Result<(), rumqttc::ClientError> {
        self.client.subscribe(topic, qos.into()).await
    }

    /// Disconnect from the broker
    pub async fn disconnect(&self) -> Result<(), rumqttc::ClientError> {
        self.client.disconnect().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qos_conversion() {
        assert_eq!(QoS::from(MqttQos::AtMostOnce), QoS::AtMostOnce);
        assert_eq!(QoS::from(MqttQos::AtLeastOnce), QoS::AtLeastOnce);
        assert_eq!(QoS::from(MqttQos::ExactlyOnce), QoS::ExactlyOnce);
    }

    #[test]
    fn test_mqtt_message_to_context() {
        let msg = MqttMessage {
            topic: "sensors/temperature".to_string(),
            payload: Bytes::from(r#"{"temp": 22.5}"#),
            qos: MqttQos::AtLeastOnce,
            retain: false,
        };

        let ctx = msg.to_context();
        assert_eq!(ctx.protocol, ProtocolType::Mqtt);
        assert_eq!(ctx.destination, "sensors/temperature");
        assert_eq!(ctx.get_metadata("mqtt.qos"), Some(&"1".to_string()));
    }

    #[test]
    fn test_client_config_defaults() {
        let config: MqttClientConfig = serde_json::from_str(r#"{
            "client_id": "test",
            "host": "localhost"
        }"#).unwrap();

        assert_eq!(config.port, 1883);
        assert_eq!(config.keep_alive_secs, 60);
        assert!(config.clean_session);
    }
}
