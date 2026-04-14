mod memory;
mod redis_backend;
mod traits;

pub use memory::InMemoryStorage;
pub use redis_backend::RedisStorage;
pub use traits::WebhookStorageBackend;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
/// Stored webhook request
#[derive(Debug, Clone, Serialize)]
pub struct StoredWebhook {
    /// Unique identifier for this webhook
    pub id: String,
    /// Token this webhook was sent to
    pub token: String,
    /// HTTP method
    pub method: String,
    /// Request path (after the token)
    pub path: String,
    /// Query string (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    /// Request headers
    pub headers: HashMap<String, String>,
    /// Request body (raw bytes)
    #[serde(skip)]
    pub body_raw: Vec<u8>,
    /// Request body as base64
    pub body: String,
    /// Body size in bytes
    pub body_size: usize,
    /// Content type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    /// When the webhook was received
    pub received_at: DateTime<Utc>,
    /// Whether the webhook was forwarded to a connected client
    pub forwarded: bool,
    /// Response from the CLI client (if forwarded)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response: Option<StoredResponse>,
    /// Whether the body is encrypted
    #[serde(skip)]
    pub encrypted: bool,
}

/// Response from the CLI client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredResponse {
    /// HTTP status code
    pub status: u16,
    /// Response headers
    pub headers: HashMap<String, String>,
    /// Response body as base64
    pub body: String,
    /// Body size in bytes
    pub body_size: usize,
    /// When the response was received
    pub received_at: DateTime<Utc>,
    /// Response time in milliseconds
    pub response_time_ms: u64,
}

/// Session information for a token
#[derive(Debug, Clone, Serialize)]
pub struct TokenSession {
    /// Whether a CLI client is currently connected
    pub connected: bool,
    /// Host of the connected client (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connected_from: Option<String>,
    /// When the session was created
    pub created_at: DateTime<Utc>,
    /// Total webhooks received
    pub total_webhooks: u64,
    /// Webhooks forwarded to CLI
    pub forwarded_webhooks: u64,
    /// When the last activity occurred (webhook received or forwarded)
    pub last_activity: DateTime<Utc>,
    /// When the connection was established (if connected)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connected_at: Option<DateTime<Utc>>,
}

impl Default for TokenSession {
    fn default() -> Self {
        Self {
            connected: false,
            connected_from: None,
            created_at: Utc::now(),
            total_webhooks: 0,
            forwarded_webhooks: 0,
            last_activity: Utc::now(),
            connected_at: None,
        }
    }
}

/// Encryption handler for webhook storage
pub struct StorageEncryption {
    key: [u8; 32],
}

impl StorageEncryption {
    /// Create a new encryption handler with a 32-byte key
    pub fn new(key: &[u8; 32]) -> Self {
        Self { key: *key }
    }

    /// Generate a random encryption key
    pub fn generate_key() -> [u8; 32] {
        use aes_gcm::aead::rand_core::RngCore;
        use aes_gcm::aead::OsRng;
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);
        key
    }

    /// Encrypt data using AES-256-GCM
    pub fn encrypt(&self, plaintext: &[u8]) -> Vec<u8> {
        use aes_gcm::aead::Aead;
        use aes_gcm::aead::OsRng;
        use aes_gcm::{AeadCore, Aes256Gcm, KeyInit};

        let cipher = Aes256Gcm::new_from_slice(&self.key).expect("valid 32-byte key");
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

        let ciphertext = cipher
            .encrypt(&nonce, plaintext)
            .expect("AES-256-GCM encryption failed");

        // Prepend nonce to ciphertext
        let mut result = nonce.to_vec();
        result.extend_from_slice(&ciphertext);
        result
    }

    /// Decrypt data using AES-256-GCM
    pub fn decrypt(&self, data: &[u8]) -> Option<Vec<u8>> {
        use aes_gcm::aead::Aead;
        use aes_gcm::{Aes256Gcm, KeyInit};

        if data.len() < 12 {
            return None;
        }

        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce: [u8; 12] = nonce_bytes.try_into().ok()?;
        let cipher = Aes256Gcm::new_from_slice(&self.key).expect("valid 32-byte key");

        cipher.decrypt(&nonce.into(), ciphertext).ok()
    }
}

impl std::fmt::Debug for StorageEncryption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StorageEncryption")
            .field("key", &"[REDACTED]")
            .finish()
    }
}

/// Create a new webhook record (shared helper, not backend-specific)
pub fn create_webhook(
    token: &str,
    method: &str,
    path: &str,
    query: Option<&str>,
    headers: HashMap<String, String>,
    body: Vec<u8>,
) -> StoredWebhook {
    let content_type = headers.get("content-type").cloned();
    let body_size = body.len();
    let body_b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &body);

    StoredWebhook {
        id: Uuid::new_v4().to_string(),
        token: token.to_owned(),
        method: method.to_owned(),
        path: path.to_owned(),
        query: query.map(|s| s.to_owned()),
        headers,
        body_raw: body,
        body: body_b64,
        body_size,
        content_type,
        received_at: Utc::now(),
        forwarded: false,
        response: None,
        encrypted: false,
    }
}

/// Storage backend enum that dispatches to the appropriate implementation.
/// Allows switching between in-memory and Redis at runtime based on REDIS_URL.
pub enum StorageBackend {
    InMemory(InMemoryStorage),
    Redis(RedisStorage),
}

impl std::fmt::Debug for StorageBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InMemory(s) => write!(f, "StorageBackend::InMemory({:?})", s),
            Self::Redis(s) => write!(f, "StorageBackend::Redis({:?})", s),
        }
    }
}

#[async_trait::async_trait]
impl WebhookStorageBackend for StorageBackend {
    async fn store_webhook(&self, webhook: StoredWebhook) {
        match self {
            Self::InMemory(s) => s.store_webhook(webhook).await,
            Self::Redis(s) => s.store_webhook(webhook).await,
        }
    }

    async fn get_webhooks(&self, token: &str) -> Vec<StoredWebhook> {
        match self {
            Self::InMemory(s) => s.get_webhooks(token).await,
            Self::Redis(s) => s.get_webhooks(token).await,
        }
    }

    async fn get_webhook(&self, token: &str, webhook_id: &str) -> Option<StoredWebhook> {
        match self {
            Self::InMemory(s) => s.get_webhook(token, webhook_id).await,
            Self::Redis(s) => s.get_webhook(token, webhook_id).await,
        }
    }

    async fn mark_forwarded(&self, token: &str, webhook_id: &str) {
        match self {
            Self::InMemory(s) => s.mark_forwarded(token, webhook_id).await,
            Self::Redis(s) => s.mark_forwarded(token, webhook_id).await,
        }
    }

    async fn update_webhook_response(
        &self,
        token: &str,
        webhook_id: &str,
        response: StoredResponse,
    ) {
        match self {
            Self::InMemory(s) => s.update_webhook_response(token, webhook_id, response).await,
            Self::Redis(s) => s.update_webhook_response(token, webhook_id, response).await,
        }
    }

    async fn get_or_create_session(&self, token: &str) -> TokenSession {
        match self {
            Self::InMemory(s) => s.get_or_create_session(token).await,
            Self::Redis(s) => s.get_or_create_session(token).await,
        }
    }

    async fn set_connected(&self, token: &str, from: Option<String>) {
        match self {
            Self::InMemory(s) => s.set_connected(token, from).await,
            Self::Redis(s) => s.set_connected(token, from).await,
        }
    }

    async fn set_disconnected(&self, token: &str) {
        match self {
            Self::InMemory(s) => s.set_disconnected(token).await,
            Self::Redis(s) => s.set_disconnected(token).await,
        }
    }

    async fn is_connected(&self, token: &str) -> bool {
        match self {
            Self::InMemory(s) => s.is_connected(token).await,
            Self::Redis(s) => s.is_connected(token).await,
        }
    }

    async fn delete_webhook(&self, token: &str, webhook_id: &str) -> bool {
        match self {
            Self::InMemory(s) => s.delete_webhook(token, webhook_id).await,
            Self::Redis(s) => s.delete_webhook(token, webhook_id).await,
        }
    }

    async fn delete_all_webhooks(&self, token: &str) -> usize {
        match self {
            Self::InMemory(s) => s.delete_all_webhooks(token).await,
            Self::Redis(s) => s.delete_all_webhooks(token).await,
        }
    }

    async fn cleanup_expired(&self, ttl: std::time::Duration) -> usize {
        match self {
            Self::InMemory(s) => s.cleanup_expired(ttl).await,
            Self::Redis(s) => s.cleanup_expired(ttl).await,
        }
    }

    async fn find_timed_out_sessions(&self, session_timeout: std::time::Duration) -> Vec<String> {
        match self {
            Self::InMemory(s) => s.find_timed_out_sessions(session_timeout).await,
            Self::Redis(s) => s.find_timed_out_sessions(session_timeout).await,
        }
    }

    async fn find_idle_sessions(&self, idle_timeout: std::time::Duration) -> Vec<String> {
        match self {
            Self::InMemory(s) => s.find_idle_sessions(idle_timeout).await,
            Self::Redis(s) => s.find_idle_sessions(idle_timeout).await,
        }
    }

    async fn touch_session(&self, token: &str) {
        match self {
            Self::InMemory(s) => s.touch_session(token).await,
            Self::Redis(s) => s.touch_session(token).await,
        }
    }

    fn enable_encryption(&mut self, key: &[u8; 32]) {
        match self {
            Self::InMemory(s) => s.enable_encryption(key),
            Self::Redis(s) => s.enable_encryption(key),
        }
    }
}
