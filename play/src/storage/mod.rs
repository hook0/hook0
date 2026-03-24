use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::Serialize;
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;
use zeroize::Zeroize;

use crate::audit;

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
#[derive(Debug, Clone, Serialize)]
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

/// In-memory storage for webhooks
#[derive(Debug)]
pub struct WebhookStorage {
    /// Webhooks by token -> webhook_id -> webhook
    webhooks: DashMap<String, DashMap<String, StoredWebhook>>,
    /// Webhook order by token (for FIFO eviction)
    webhook_order: DashMap<String, Vec<String>>,
    /// Session info by token
    sessions: DashMap<String, TokenSession>,
    /// Maximum webhooks per token (0 = unlimited)
    max_webhooks_per_token: usize,
    /// Optional encryption handler
    encryption: Option<StorageEncryption>,
}

impl Default for WebhookStorage {
    fn default() -> Self {
        Self {
            webhooks: DashMap::new(),
            webhook_order: DashMap::new(),
            sessions: DashMap::new(),
            max_webhooks_per_token: 1000, // Default limit
            encryption: None,
        }
    }
}

impl WebhookStorage {
    /// Create a new storage instance with default limits
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new storage instance with custom limits
    pub fn with_limits(max_webhooks_per_token: usize) -> Self {
        Self {
            webhooks: DashMap::new(),
            webhook_order: DashMap::new(),
            sessions: DashMap::new(),
            max_webhooks_per_token,
            encryption: None,
        }
    }

    /// Enable encryption with the given key
    pub fn enable_encryption(&mut self, key: &[u8; 32]) {
        self.encryption = Some(StorageEncryption::new(key));
    }

    /// Create a new webhook record
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

    /// Store a webhook with FIFO eviction if limit exceeded
    pub fn store_webhook(&self, mut webhook: StoredWebhook) {
        let token = webhook.token.clone();
        let webhook_id = webhook.id.clone();

        // Encrypt body if encryption is enabled
        if let Some(ref enc) = self.encryption {
            let encrypted_body = enc.encrypt(&webhook.body_raw);
            webhook.body_raw = encrypted_body;
            webhook.encrypted = true;
        }

        // Get or create the token's webhook map
        let token_webhooks = self.webhooks.entry(token.clone()).or_default();

        // Evict oldest webhooks if limit exceeded (FIFO)
        if self.max_webhooks_per_token > 0 {
            let mut order = self.webhook_order.entry(token.clone()).or_default();

            while order.len() >= self.max_webhooks_per_token {
                if let Some(oldest_id) = order.first().cloned() {
                    // Secure deletion: zero out the body before removing
                    if let Some((_, mut removed)) = token_webhooks.remove(&oldest_id) {
                        removed.body_raw.zeroize();
                        removed.body.zeroize();
                    }
                    order.remove(0);
                } else {
                    break;
                }
            }

            order.push(webhook_id.clone());
        }

        token_webhooks.insert(webhook_id, webhook);

        // Update session stats
        let mut session = self.sessions.entry(token).or_default();
        session.total_webhooks += 1;
        session.last_activity = Utc::now();
    }

    /// Get all webhooks for a token (decrypting if needed)
    pub fn get_webhooks(&self, token: &str) -> Vec<StoredWebhook> {
        audit::log_audit(
            audit::AuditEvent::WebhookViewed,
            Some(token),
            None,
            "list_all",
        );

        self.webhooks
            .get(token)
            .map(|m| {
                m.iter()
                    .map(|r| self.maybe_decrypt_webhook(r.value().clone()))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get a specific webhook (decrypting if needed)
    pub fn get_webhook(&self, token: &str, webhook_id: &str) -> Option<StoredWebhook> {
        audit::log_audit(
            audit::AuditEvent::WebhookViewed,
            Some(token),
            None,
            webhook_id,
        );

        self.webhooks.get(token).and_then(|m| {
            m.get(webhook_id)
                .map(|r| self.maybe_decrypt_webhook(r.clone()))
        })
    }

    /// Decrypt a webhook body if it was encrypted
    fn maybe_decrypt_webhook(&self, mut webhook: StoredWebhook) -> StoredWebhook {
        if webhook.encrypted {
            if let Some(ref enc) = self.encryption {
                if let Some(decrypted) = enc.decrypt(&webhook.body_raw) {
                    webhook.body = base64::Engine::encode(
                        &base64::engine::general_purpose::STANDARD,
                        &decrypted,
                    );
                    webhook.body_raw = decrypted;
                    webhook.encrypted = false;
                }
            }
        }
        webhook
    }

    /// Mark a webhook as forwarded
    pub fn mark_forwarded(&self, token: &str, webhook_id: &str) {
        if let Some(token_webhooks) = self.webhooks.get(token) {
            if let Some(mut webhook) = token_webhooks.get_mut(webhook_id) {
                webhook.forwarded = true;
            }
        }

        // Update session stats
        if let Some(mut session) = self.sessions.get_mut(token) {
            session.forwarded_webhooks += 1;
            session.last_activity = Utc::now();
        }
    }

    /// Update a webhook with a response
    pub fn update_webhook_response(&self, token: &str, webhook_id: &str, response: StoredResponse) {
        if let Some(token_webhooks) = self.webhooks.get(token) {
            if let Some(mut webhook) = token_webhooks.get_mut(webhook_id) {
                webhook.response = Some(response);
            }
        }
    }

    /// Get or create session for a token
    pub fn get_or_create_session(&self, token: &str) -> TokenSession {
        self.sessions.entry(token.to_owned()).or_default().clone()
    }

    /// Set connected state for a token
    pub fn set_connected(&self, token: &str, from: Option<String>) {
        let mut session = self.sessions.entry(token.to_owned()).or_default();
        session.connected = true;
        session.connected_from = from;
        session.connected_at = Some(Utc::now());
        session.last_activity = Utc::now();

        audit::log_audit(
            audit::AuditEvent::SessionCreated,
            Some(token),
            session.connected_from.as_deref(),
            "connected",
        );
    }

    /// Set disconnected state for a token
    pub fn set_disconnected(&self, token: &str) {
        if let Some(mut session) = self.sessions.get_mut(token) {
            session.connected = false;
            session.connected_from = None;
            session.connected_at = None;

            audit::log_audit(
                audit::AuditEvent::SessionDisconnected,
                Some(token),
                None,
                "disconnected",
            );
        }
    }

    /// Check if a token is connected
    pub fn is_connected(&self, token: &str) -> bool {
        self.sessions
            .get(token)
            .map(|s| s.connected)
            .unwrap_or(false)
    }

    /// Delete a specific webhook with secure memory zeroing
    pub fn delete_webhook(&self, token: &str, webhook_id: &str) -> bool {
        let removed = self.webhooks.get(token).and_then(|m| m.remove(webhook_id));

        if let Some((_, mut webhook)) = removed {
            // Secure deletion: zero out sensitive data
            webhook.body_raw.zeroize();
            webhook.body.zeroize();
            if let Some(ref mut resp) = webhook.response {
                resp.body.zeroize();
            }

            // Remove from order tracking
            if let Some(mut order) = self.webhook_order.get_mut(token) {
                order.retain(|id| id != webhook_id);
            }

            audit::log_audit(
                audit::AuditEvent::WebhookDeleted,
                Some(token),
                None,
                webhook_id,
            );

            true
        } else {
            false
        }
    }

    /// Delete all webhooks for a token with secure memory zeroing
    pub fn delete_all_webhooks(&self, token: &str) -> usize {
        let count = self.webhooks.get(token).map(|m| m.len()).unwrap_or(0);

        if let Some((_, token_webhooks)) = self.webhooks.remove(token) {
            for (_, mut webhook) in token_webhooks {
                webhook.body_raw.zeroize();
                webhook.body.zeroize();
                if let Some(ref mut resp) = webhook.response {
                    resp.body.zeroize();
                }
            }
        }

        self.webhook_order.remove(token);

        audit::log_audit(
            audit::AuditEvent::WebhookDeleted,
            Some(token),
            None,
            &format!("all ({})", count),
        );

        count
    }

    /// Remove expired webhooks (older than TTL)
    /// Returns the number of webhooks removed
    pub fn cleanup_expired(&self, ttl: Duration) -> usize {
        let cutoff =
            Utc::now() - chrono::Duration::from_std(ttl).unwrap_or(chrono::Duration::hours(24));
        let mut removed_count = 0;

        let tokens: Vec<String> = self.webhooks.iter().map(|r| r.key().clone()).collect();

        for token in tokens {
            if let Some(token_webhooks) = self.webhooks.get(&token) {
                let expired_ids: Vec<String> = token_webhooks
                    .iter()
                    .filter(|r| r.value().received_at < cutoff)
                    .map(|r| r.key().clone())
                    .collect();

                for id in &expired_ids {
                    if let Some((_, mut webhook)) = token_webhooks.remove(id) {
                        webhook.body_raw.zeroize();
                        webhook.body.zeroize();
                        if let Some(ref mut resp) = webhook.response {
                            resp.body.zeroize();
                        }
                        removed_count += 1;
                    }
                }

                // Update order tracking
                if !expired_ids.is_empty() {
                    if let Some(mut order) = self.webhook_order.get_mut(&token) {
                        order.retain(|id| !expired_ids.contains(id));
                    }
                }
            }

            // Clean up empty token entries
            if self
                .webhooks
                .get(&token)
                .map(|m| m.is_empty())
                .unwrap_or(true)
            {
                self.webhooks.remove(&token);
                self.webhook_order.remove(&token);
            }
        }

        if removed_count > 0 {
            audit::log_audit(
                audit::AuditEvent::WebhookExpired,
                None,
                None,
                &format!("{} webhooks expired", removed_count),
            );
        }

        removed_count
    }

    /// Find sessions that have exceeded their session timeout
    /// Returns tokens of sessions that should be disconnected
    pub fn find_timed_out_sessions(&self, session_timeout: Duration) -> Vec<String> {
        let now = Utc::now();
        let timeout =
            chrono::Duration::from_std(session_timeout).unwrap_or(chrono::Duration::hours(24));

        self.sessions
            .iter()
            .filter(|r| {
                let session = r.value();
                session.connected
                    && session
                        .connected_at
                        .map(|t| now - t > timeout)
                        .unwrap_or(false)
            })
            .map(|r| r.key().clone())
            .collect()
    }

    /// Find sessions that have been idle too long
    /// Returns tokens of sessions that should be disconnected
    pub fn find_idle_sessions(&self, idle_timeout: Duration) -> Vec<String> {
        let now = Utc::now();
        let timeout =
            chrono::Duration::from_std(idle_timeout).unwrap_or(chrono::Duration::hours(1));

        self.sessions
            .iter()
            .filter(|r| {
                let session = r.value();
                session.connected && now - session.last_activity > timeout
            })
            .map(|r| r.key().clone())
            .collect()
    }

    /// Update last activity timestamp for a token
    pub fn touch_session(&self, token: &str) {
        if let Some(mut session) = self.sessions.get_mut(token) {
            session.last_activity = Utc::now();
        }
    }
}
