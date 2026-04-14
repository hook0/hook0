use std::time::Duration;

use super::{StoredResponse, StoredWebhook, TokenSession};

/// Trait defining the webhook storage interface.
/// Implemented by both InMemoryStorage and RedisStorage.
#[async_trait::async_trait]
pub trait WebhookStorageBackend: Send + Sync {
    /// Store a webhook with FIFO eviction if limit exceeded
    async fn store_webhook(&self, webhook: StoredWebhook);

    /// Get all webhooks for a token (decrypting if needed)
    async fn get_webhooks(&self, token: &str) -> Vec<StoredWebhook>;

    /// Get a specific webhook (decrypting if needed)
    async fn get_webhook(&self, token: &str, webhook_id: &str) -> Option<StoredWebhook>;

    /// Mark a webhook as forwarded
    async fn mark_forwarded(&self, token: &str, webhook_id: &str);

    /// Update a webhook with a response
    async fn update_webhook_response(
        &self,
        token: &str,
        webhook_id: &str,
        response: StoredResponse,
    );

    /// Get or create session for a token
    async fn get_or_create_session(&self, token: &str) -> TokenSession;

    /// Set connected state for a token
    async fn set_connected(&self, token: &str, from: Option<String>);

    /// Set disconnected state for a token
    async fn set_disconnected(&self, token: &str);

    /// Check if a token is connected
    async fn is_connected(&self, token: &str) -> bool;

    /// Delete a specific webhook with secure memory zeroing
    async fn delete_webhook(&self, token: &str, webhook_id: &str) -> bool;

    /// Delete all webhooks for a token with secure memory zeroing
    async fn delete_all_webhooks(&self, token: &str) -> usize;

    /// Remove expired webhooks (older than TTL)
    /// Returns the number of webhooks removed
    async fn cleanup_expired(&self, ttl: Duration) -> usize;

    /// Find sessions that have exceeded their session timeout
    async fn find_timed_out_sessions(&self, session_timeout: Duration) -> Vec<String>;

    /// Find sessions that have been idle too long
    async fn find_idle_sessions(&self, idle_timeout: Duration) -> Vec<String>;

    /// Update last activity timestamp for a token
    async fn touch_session(&self, token: &str);

    /// Enable encryption with the given key
    fn enable_encryption(&mut self, key: &[u8; 32]);
}
