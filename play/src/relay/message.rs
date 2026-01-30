use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Messages sent from CLI client to server
#[derive(Debug, Clone, Deserialize)]
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

#[derive(Debug, Clone, Deserialize)]
pub struct ClientStartData {
    /// Token to listen on (e.g., "c_abc123...")
    pub token: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ClientResponseData {
    /// ID of the webhook request being responded to
    pub id: String,
    /// HTTP status code to return
    pub status: u16,
    /// Response headers
    #[serde(default)]
    pub headers: HashMap<String, String>,
    /// Base64-encoded response body
    #[serde(default)]
    pub body: String,
}

/// Messages sent from server to CLI client
#[derive(Debug, Clone, Serialize)]
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

#[derive(Debug, Clone, Serialize)]
pub struct ServerStartedData {
    /// URL where webhooks will be received
    pub webhook_url: String,
    /// URL to view received webhooks
    pub view_url: String,
}

#[derive(Debug, Clone, Serialize)]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ServerErrorData {
    /// Error code
    pub code: String,
    /// Human-readable error message
    pub message: String,
}

fn default_version() -> u16 {
    1
}

impl ServerMessage {
    /// Create a "started" message
    pub fn started(webhook_url: String, view_url: String) -> Self {
        Self::Started {
            version: 1,
            data: ServerStartedData {
                webhook_url,
                view_url,
            },
        }
    }

    /// Create a "request" message for forwarding a webhook
    pub fn request(
        id: String,
        method: String,
        path: String,
        body: Vec<u8>,
        headers: HashMap<String, String>,
        query: Option<String>,
    ) -> Self {
        Self::Request {
            version: 1,
            data: ServerRequestData {
                id,
                method,
                path,
                body: base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &body),
                headers,
                query,
            },
        }
    }

    /// Create an "error" message
    pub fn error(code: &str, message: &str) -> Self {
        Self::Error {
            version: 1,
            data: ServerErrorData {
                code: code.to_owned(),
                message: message.to_owned(),
            },
        }
    }

    /// Create a "pong" message
    pub fn pong() -> Self {
        Self::Pong
    }
}
