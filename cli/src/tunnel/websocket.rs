use std::collections::HashMap;

use chrono::{DateTime, Utc};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum StreamError {
    #[error("WebSocket connection error: {0}")]
    ConnectionError(#[from] tokio_tungstenite::tungstenite::Error),

    #[error("URL parse error: {0}")]
    UrlError(#[from] url::ParseError),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Connection closed")]
    ConnectionClosed,

    #[error("Send error: {0}")]
    SendError(String),
}

/// Messages sent/received over the WebSocket stream
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamMessage {
    /// Server sends connection info
    Connected {
        webhook_url: String,
        session_id: String,
    },
    /// Server sends an incoming webhook
    WebhookReceived {
        request_id: String,
        event_id: Uuid,
        event_type: String,
        payload: String,
        headers: HashMap<String, String>,
        received_at: DateTime<Utc>,
    },
    /// Client sends the response after forwarding
    WebhookResponse {
        request_id: String,
        status_code: u16,
        headers: HashMap<String, String>,
        body: Option<String>,
        elapsed_ms: i64,
    },
    /// Ping for keepalive
    Ping,
    /// Pong response
    Pong,
    /// Error message
    Error { message: String },
}

/// Events emitted by the stream client
#[derive(Debug, Clone)]
pub enum StreamEvent {
    /// Connected to the stream server
    Connected {
        webhook_url: String,
        session_id: String,
    },
    /// Received a webhook to forward
    WebhookReceived {
        request_id: String,
        event_id: Uuid,
        event_type: String,
        payload: String,
        headers: HashMap<String, String>,
        received_at: DateTime<Utc>,
    },
    /// Connection error
    Error(String),
    /// Connection closed
    Disconnected,
}

/// Client for the webhook streaming service
pub struct StreamClient {
    stream_url: String,
    tx: Option<mpsc::Sender<StreamMessage>>,
}

impl StreamClient {
    /// Create a new stream client
    pub fn new(stream_url: String) -> Self {
        Self {
            stream_url,
            tx: None,
        }
    }

    /// Connect to the stream server and return a channel for receiving events
    pub async fn connect(
        &mut self,
    ) -> Result<mpsc::Receiver<StreamEvent>, StreamError> {
        // Use the string URL directly - tungstenite can parse it
        let (ws_stream, _) = connect_async(&self.stream_url).await?;
        let (mut write, mut read) = ws_stream.split();

        // Channel for sending responses back to the server
        let (tx, mut rx) = mpsc::channel::<StreamMessage>(32);
        self.tx = Some(tx);

        // Channel for emitting events to the caller
        let (event_tx, event_rx) = mpsc::channel::<StreamEvent>(32);

        // Spawn task to handle incoming messages
        let event_tx_clone = event_tx.clone();
        tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        match serde_json::from_str::<StreamMessage>(&text) {
                            Ok(stream_msg) => {
                                let event = match stream_msg {
                                    StreamMessage::Connected { webhook_url, session_id } => {
                                        StreamEvent::Connected { webhook_url, session_id }
                                    }
                                    StreamMessage::WebhookReceived {
                                        request_id,
                                        event_id,
                                        event_type,
                                        payload,
                                        headers,
                                        received_at,
                                    } => StreamEvent::WebhookReceived {
                                        request_id,
                                        event_id,
                                        event_type,
                                        payload,
                                        headers,
                                        received_at,
                                    },
                                    StreamMessage::Ping => {
                                        // Respond with pong (handled in write task)
                                        continue;
                                    }
                                    StreamMessage::Pong => continue,
                                    StreamMessage::Error { message } => {
                                        StreamEvent::Error(message)
                                    }
                                    StreamMessage::WebhookResponse { .. } => continue,
                                };

                                if event_tx_clone.send(event).await.is_err() {
                                    break;
                                }
                            }
                            Err(e) => {
                                tracing::warn!("Failed to parse stream message: {}", e);
                            }
                        }
                    }
                    Ok(Message::Ping(_)) => {
                        // Will be handled by tungstenite automatically with pong
                    }
                    Ok(Message::Close(_)) => {
                        let _ = event_tx_clone.send(StreamEvent::Disconnected).await;
                        break;
                    }
                    Err(e) => {
                        let _ = event_tx_clone.send(StreamEvent::Error(e.to_string())).await;
                        break;
                    }
                    _ => {}
                }
            }
            let _ = event_tx_clone.send(StreamEvent::Disconnected).await;
        });

        // Spawn task to handle outgoing messages
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                let json = match serde_json::to_string(&msg) {
                    Ok(j) => j,
                    Err(e) => {
                        tracing::error!("Failed to serialize message: {}", e);
                        continue;
                    }
                };

                if write.send(Message::Text(json)).await.is_err() {
                    break;
                }
            }
        });

        Ok(event_rx)
    }

    /// Send a webhook response back to the server
    pub async fn send_response(
        &self,
        request_id: String,
        status_code: u16,
        headers: HashMap<String, String>,
        body: Option<String>,
        elapsed_ms: i64,
    ) -> Result<(), StreamError> {
        let tx = self
            .tx
            .as_ref()
            .ok_or_else(|| StreamError::SendError("Not connected".to_string()))?;

        let msg = StreamMessage::WebhookResponse {
            request_id,
            status_code,
            headers,
            body,
            elapsed_ms,
        };

        tx.send(msg)
            .await
            .map_err(|e| StreamError::SendError(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_message_serialization() {
        let msg = StreamMessage::Connected {
            webhook_url: "https://example.com/webhook".to_string(),
            session_id: "abc123".to_string(),
        };

        let json = serde_json::to_string(&msg).expect("serialization should work");
        assert!(json.contains("connected"));
        assert!(json.contains("webhook_url"));

        let parsed: StreamMessage = serde_json::from_str(&json).expect("deserialization should work");
        if let StreamMessage::Connected { webhook_url, session_id } = parsed {
            assert_eq!(webhook_url, "https://example.com/webhook");
            assert_eq!(session_id, "abc123");
        } else {
            panic!("Expected Connected message");
        }
    }

    #[test]
    fn test_webhook_received_serialization() {
        let msg = StreamMessage::WebhookReceived {
            request_id: "req-123".to_string(),
            event_id: Uuid::new_v4(),
            event_type: "user.account.created".to_string(),
            payload: "eyJ1c2VyX2lkIjogMTIzfQ==".to_string(),
            headers: HashMap::from([("Content-Type".to_string(), "application/json".to_string())]),
            received_at: Utc::now(),
        };

        let json = serde_json::to_string(&msg).expect("serialization should work");
        assert!(json.contains("webhook_received"));
        assert!(json.contains("event_type"));
    }

    #[test]
    fn test_webhook_response_serialization() {
        let msg = StreamMessage::WebhookResponse {
            request_id: "req-123".to_string(),
            status_code: 200,
            headers: HashMap::new(),
            body: Some("OK".to_string()),
            elapsed_ms: 45,
        };

        let json = serde_json::to_string(&msg).expect("serialization should work");
        assert!(json.contains("webhook_response"));
        assert!(json.contains("status_code"));
    }
}
