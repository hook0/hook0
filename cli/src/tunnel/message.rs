//! WebSocket protocol messages for the tunnel client.
//! These match the messages defined in the hooks server.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Messages sent from CLI client to server
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    /// Initial handshake to start listening
    Start {
        #[serde(default = "default_version")]
        version: u16,
        data: ClientStartData,
    },
    /// Response to a forwarded webhook request
    Response {
        #[serde(default = "default_version")]
        version: u16,
        data: ClientResponseData,
    },
    /// Keepalive ping
    Ping,
}

#[derive(Debug, Clone, Serialize)]
pub struct ClientStartData {
    /// Token to listen on (e.g., "c_abc123...")
    pub token: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ClientResponseData {
    /// ID of the webhook request being responded to
    pub id: String,
    /// HTTP status code to return
    pub status: u16,
    /// Response headers
    pub headers: HashMap<String, String>,
    /// Base64-encoded response body
    pub body: String,
}

/// Messages sent from server to CLI client
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    /// Confirmation that listening has started
    Started {
        #[serde(default = "default_version")]
        version: u16,
        data: ServerStartedData,
    },
    /// Incoming webhook request to forward
    Request {
        #[serde(default = "default_version")]
        version: u16,
        data: ServerRequestData,
    },
    /// Error message
    Error {
        #[serde(default = "default_version")]
        version: u16,
        data: ServerErrorData,
    },
    /// Keepalive pong
    Pong,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerStartedData {
    /// URL where webhooks will be received
    pub webhook_url: String,
    /// URL to view received webhooks
    pub view_url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerRequestData {
    /// Unique ID for this request (for response correlation)
    pub id: String,
    /// HTTP method
    pub method: String,
    /// Request path (after the token)
    pub path: String,
    /// Base64-encoded request body
    pub body: String,
    /// Request headers
    pub headers: HashMap<String, String>,
    /// Query string (if any)
    pub query: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerErrorData {
    /// Error code
    pub code: String,
    /// Human-readable error message
    pub message: String,
}

fn default_version() -> u16 {
    1
}

impl ClientMessage {
    /// Create a "start" message
    pub fn start(token: String) -> Self {
        Self::Start {
            version: 1,
            data: ClientStartData { token },
        }
    }

    /// Create a "response" message
    pub fn response(
        id: String,
        status: u16,
        headers: HashMap<String, String>,
        body: Vec<u8>,
    ) -> Self {
        Self::Response {
            version: 1,
            data: ClientResponseData {
                id,
                status,
                headers,
                body: base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &body),
            },
        }
    }

    /// Create a "ping" message
    pub fn ping() -> Self {
        Self::Ping
    }
}
