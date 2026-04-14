use chrono::Utc;
use dashmap::DashMap;
use std::time::Duration;
use zeroize::Zeroize;

use crate::audit;

use super::traits::WebhookStorageBackend;
use super::{StorageEncryption, StoredResponse, StoredWebhook, TokenSession};

/// In-memory storage for webhooks using DashMap
#[derive(Debug)]
pub struct InMemoryStorage {
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

impl Default for InMemoryStorage {
    fn default() -> Self {
        Self {
            webhooks: DashMap::new(),
            webhook_order: DashMap::new(),
            sessions: DashMap::new(),
            max_webhooks_per_token: 1000,
            encryption: None,
        }
    }
}

impl InMemoryStorage {
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
}

#[async_trait::async_trait]
impl WebhookStorageBackend for InMemoryStorage {
    async fn store_webhook(&self, mut webhook: StoredWebhook) {
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

    async fn get_webhooks(&self, token: &str) -> Vec<StoredWebhook> {
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

    async fn get_webhook(&self, token: &str, webhook_id: &str) -> Option<StoredWebhook> {
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

    async fn mark_forwarded(&self, token: &str, webhook_id: &str) {
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

    async fn update_webhook_response(
        &self,
        token: &str,
        webhook_id: &str,
        response: StoredResponse,
    ) {
        if let Some(token_webhooks) = self.webhooks.get(token) {
            if let Some(mut webhook) = token_webhooks.get_mut(webhook_id) {
                webhook.response = Some(response);
            }
        }
    }

    async fn get_or_create_session(&self, token: &str) -> TokenSession {
        self.sessions.entry(token.to_owned()).or_default().clone()
    }

    async fn set_connected(&self, token: &str, from: Option<String>) {
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

    async fn set_disconnected(&self, token: &str) {
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

    async fn is_connected(&self, token: &str) -> bool {
        self.sessions
            .get(token)
            .map(|s| s.connected)
            .unwrap_or(false)
    }

    async fn delete_webhook(&self, token: &str, webhook_id: &str) -> bool {
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

    async fn delete_all_webhooks(&self, token: &str) -> usize {
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

    async fn cleanup_expired(&self, ttl: Duration) -> usize {
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

    async fn find_timed_out_sessions(&self, session_timeout: Duration) -> Vec<String> {
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

    async fn find_idle_sessions(&self, idle_timeout: Duration) -> Vec<String> {
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

    async fn touch_session(&self, token: &str) {
        if let Some(mut session) = self.sessions.get_mut(token) {
            session.last_activity = Utc::now();
        }
    }

    fn enable_encryption(&mut self, key: &[u8; 32]) {
        self.encryption = Some(StorageEncryption::new(key));
    }
}
