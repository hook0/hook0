use chrono::{DateTime, Utc};
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use std::collections::HashMap;
use std::time::Duration;
use tracing::{error, warn};

use crate::audit;

use super::traits::WebhookStorageBackend;
use super::{StorageEncryption, StoredResponse, StoredWebhook, TokenSession};

/// TTL for all Redis keys: 24 hours
const KEY_TTL_SECONDS: u64 = 86400;

/// Maximum webhooks per token in the list
const MAX_WEBHOOKS_PER_TOKEN: isize = 1000;

/// Redis-backed storage for webhooks.
/// Uses Redis Standalone with native TTL for automatic expiration.
pub struct RedisStorage {
    conn: ConnectionManager,
    max_webhooks_per_token: isize,
    encryption: Option<StorageEncryption>,
}

impl std::fmt::Debug for RedisStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedisStorage")
            .field("max_webhooks_per_token", &self.max_webhooks_per_token)
            .finish()
    }
}

impl RedisStorage {
    /// Create a new Redis storage backend.
    /// Returns an error if the connection cannot be established.
    pub async fn new(redis_url: &str) -> Result<Self, redis::RedisError> {
        let client = redis::Client::open(redis_url)?;
        let conn = ConnectionManager::new(client).await?;
        Ok(Self {
            conn,
            max_webhooks_per_token: MAX_WEBHOOKS_PER_TOKEN,
            encryption: None,
        })
    }

    /// Create a new Redis storage backend with a custom max webhooks limit.
    pub async fn with_limits(
        redis_url: &str,
        max_webhooks_per_token: usize,
    ) -> Result<Self, redis::RedisError> {
        let client = redis::Client::open(redis_url)?;
        let conn = ConnectionManager::new(client).await?;
        Ok(Self {
            conn,
            max_webhooks_per_token: max_webhooks_per_token as isize,
            encryption: None,
        })
    }

    /// Session key: session:{token}
    fn session_key(token: &str) -> String {
        format!("session:{}", token)
    }

    /// Webhook list key: webhooks:{token}
    fn webhooks_list_key(token: &str) -> String {
        format!("webhooks:{}", token)
    }

    /// Individual webhook key: webhook:{token}:{id}
    fn webhook_key(token: &str, id: &str) -> String {
        format!("webhook:{}:{}", token, id)
    }

    /// Serialize headers to JSON for Redis storage
    fn serialize_headers(headers: &HashMap<String, String>) -> String {
        serde_json::to_string(headers).unwrap_or_else(|_| "{}".to_string())
    }

    /// Deserialize headers from JSON
    fn deserialize_headers(json: &str) -> HashMap<String, String> {
        serde_json::from_str(json).unwrap_or_default()
    }

    /// Store a webhook as a Redis hash
    async fn store_webhook_hash(
        &self,
        conn: &mut ConnectionManager,
        token: &str,
        webhook: &StoredWebhook,
    ) -> Result<(), redis::RedisError> {
        let key = Self::webhook_key(token, &webhook.id);

        let mut fields: Vec<(&str, String)> = vec![
            ("id", webhook.id.clone()),
            ("token", webhook.token.clone()),
            ("method", webhook.method.clone()),
            ("path", webhook.path.clone()),
            ("headers", Self::serialize_headers(&webhook.headers)),
            ("body", webhook.body.clone()),
            (
                "body_raw",
                base64::Engine::encode(
                    &base64::engine::general_purpose::STANDARD,
                    &webhook.body_raw,
                ),
            ),
            ("body_size", webhook.body_size.to_string()),
            ("received_at", webhook.received_at.to_rfc3339()),
            ("forwarded", webhook.forwarded.to_string()),
            ("encrypted", webhook.encrypted.to_string()),
        ];

        if let Some(ref query) = webhook.query {
            fields.push(("query", query.clone()));
        }
        if let Some(ref ct) = webhook.content_type {
            fields.push(("content_type", ct.clone()));
        }
        if let Some(ref resp) = webhook.response {
            fields.push(("response", serde_json::to_string(resp).unwrap_or_default()));
        }

        redis::pipe()
            .hset_multiple(&key, &fields)
            .expire(&key, KEY_TTL_SECONDS as i64)
            .query_async(conn)
            .await
    }

    /// Read a webhook from a Redis hash
    async fn read_webhook_hash(
        &self,
        conn: &mut ConnectionManager,
        token: &str,
        webhook_id: &str,
    ) -> Result<Option<StoredWebhook>, redis::RedisError> {
        let key = Self::webhook_key(token, webhook_id);

        let fields: HashMap<String, String> = conn.hgetall(&key).await?;

        if fields.is_empty() {
            return Ok(None);
        }

        let id = fields.get("id").cloned().unwrap_or_default();
        let stored_token = fields.get("token").cloned().unwrap_or_default();
        let method = fields.get("method").cloned().unwrap_or_default();
        let path = fields.get("path").cloned().unwrap_or_default();
        let query = fields.get("query").cloned();
        let headers = fields
            .get("headers")
            .map(|h| Self::deserialize_headers(h))
            .unwrap_or_default();
        let body = fields.get("body").cloned().unwrap_or_default();
        let body_raw_b64 = fields.get("body_raw").cloned().unwrap_or_default();
        let body_raw =
            base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &body_raw_b64)
                .unwrap_or_default();
        let body_size = fields
            .get("body_size")
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0);
        let content_type = fields.get("content_type").cloned();
        let received_at = fields
            .get("received_at")
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(Utc::now);
        let forwarded = fields
            .get("forwarded")
            .map(|s| s == "true")
            .unwrap_or(false);
        let encrypted = fields
            .get("encrypted")
            .map(|s| s == "true")
            .unwrap_or(false);
        let response = fields
            .get("response")
            .and_then(|s| serde_json::from_str::<StoredResponse>(s).ok());

        let mut webhook = StoredWebhook {
            id,
            token: stored_token,
            method,
            path,
            query,
            headers,
            body_raw,
            body,
            body_size,
            content_type,
            received_at,
            forwarded,
            response,
            encrypted,
        };

        // Decrypt if needed
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

        Ok(Some(webhook))
    }

    /// Read session from Redis hash
    async fn read_session(
        &self,
        conn: &mut ConnectionManager,
        token: &str,
    ) -> Result<Option<TokenSession>, redis::RedisError> {
        let key = Self::session_key(token);
        let fields: HashMap<String, String> = conn.hgetall(&key).await?;

        if fields.is_empty() {
            return Ok(None);
        }

        let connected = fields
            .get("connected")
            .map(|s| s == "true")
            .unwrap_or(false);
        let connected_from = fields.get("connected_from").cloned();
        let created_at = fields
            .get("created_at")
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(Utc::now);
        let total_webhooks = fields
            .get("total_webhooks")
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);
        let forwarded_webhooks = fields
            .get("forwarded_webhooks")
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);
        let last_activity = fields
            .get("last_activity")
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(Utc::now);
        let connected_at = fields
            .get("connected_at")
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        Ok(Some(TokenSession {
            connected,
            connected_from,
            created_at,
            total_webhooks,
            forwarded_webhooks,
            last_activity,
            connected_at,
        }))
    }

    /// Write session to Redis hash with TTL
    async fn write_session(
        &self,
        conn: &mut ConnectionManager,
        token: &str,
        session: &TokenSession,
    ) -> Result<(), redis::RedisError> {
        let key = Self::session_key(token);

        let mut fields: Vec<(&str, String)> = vec![
            ("connected", session.connected.to_string()),
            ("created_at", session.created_at.to_rfc3339()),
            ("total_webhooks", session.total_webhooks.to_string()),
            ("forwarded_webhooks", session.forwarded_webhooks.to_string()),
            ("last_activity", session.last_activity.to_rfc3339()),
        ];

        if let Some(ref from) = session.connected_from {
            fields.push(("connected_from", from.clone()));
        }
        if let Some(at) = session.connected_at {
            fields.push(("connected_at", at.to_rfc3339()));
        }

        redis::pipe()
            .hset_multiple(&key, &fields)
            .expire(&key, KEY_TTL_SECONDS as i64)
            .query_async(conn)
            .await
    }
}

#[async_trait::async_trait]
impl WebhookStorageBackend for RedisStorage {
    async fn store_webhook(&self, mut webhook: StoredWebhook) {
        let mut conn = self.conn.clone();
        let token = webhook.token.clone();
        let webhook_id = webhook.id.clone();

        // Encrypt body if encryption is enabled
        if let Some(ref enc) = self.encryption {
            let encrypted_body = enc.encrypt(&webhook.body_raw);
            webhook.body_raw = encrypted_body;
            webhook.encrypted = true;
        }

        // Store the webhook hash
        if let Err(e) = self.store_webhook_hash(&mut conn, &token, &webhook).await {
            error!("Redis: failed to store webhook {}: {}", webhook_id, e);
            return;
        }

        // Push to the webhooks list (newest first = LPUSH)
        let list_key = Self::webhooks_list_key(&token);
        let result: Result<(), redis::RedisError> = redis::pipe()
            .lpush(&list_key, &webhook_id)
            .ltrim(&list_key, 0, self.max_webhooks_per_token - 1)
            .expire(&list_key, KEY_TTL_SECONDS as i64)
            .query_async(&mut conn)
            .await;

        if let Err(e) = result {
            error!("Redis: failed to update webhook list for {}: {}", token, e);
        }

        // Update session stats
        let session_key = Self::session_key(&token);
        let result: Result<(), redis::RedisError> = redis::pipe()
            .hincr(&session_key, "total_webhooks", 1i64)
            .hset(&session_key, "last_activity", Utc::now().to_rfc3339())
            .expire(&session_key, KEY_TTL_SECONDS as i64)
            .query_async(&mut conn)
            .await;

        if let Err(e) = result {
            error!("Redis: failed to update session stats for {}: {}", token, e);
        }
    }

    async fn get_webhooks(&self, token: &str) -> Vec<StoredWebhook> {
        audit::log_audit(
            audit::AuditEvent::WebhookViewed,
            Some(token),
            None,
            "list_all",
        );

        let mut conn = self.conn.clone();
        let list_key = Self::webhooks_list_key(token);

        // Get all webhook IDs from the list
        let ids: Vec<String> = match conn.lrange(&list_key, 0, -1).await {
            Ok(ids) => ids,
            Err(e) => {
                error!("Redis: failed to get webhook list for {}: {}", token, e);
                return Vec::new();
            }
        };

        let mut webhooks = Vec::with_capacity(ids.len());
        for id in &ids {
            match self.read_webhook_hash(&mut conn, token, id).await {
                Ok(Some(wh)) => webhooks.push(wh),
                Ok(None) => {
                    // Webhook expired or deleted but still in list; ignore
                }
                Err(e) => {
                    warn!("Redis: failed to read webhook {}:{}: {}", token, id, e);
                }
            }
        }

        webhooks
    }

    async fn get_webhook(&self, token: &str, webhook_id: &str) -> Option<StoredWebhook> {
        audit::log_audit(
            audit::AuditEvent::WebhookViewed,
            Some(token),
            None,
            webhook_id,
        );

        let mut conn = self.conn.clone();
        match self.read_webhook_hash(&mut conn, token, webhook_id).await {
            Ok(wh) => wh,
            Err(e) => {
                error!(
                    "Redis: failed to get webhook {}:{}: {}",
                    token, webhook_id, e
                );
                None
            }
        }
    }

    async fn mark_forwarded(&self, token: &str, webhook_id: &str) {
        let mut conn = self.conn.clone();
        let key = Self::webhook_key(token, webhook_id);

        let result: Result<(), redis::RedisError> = conn.hset(&key, "forwarded", "true").await;
        if let Err(e) = result {
            error!(
                "Redis: failed to mark forwarded {}:{}: {}",
                token, webhook_id, e
            );
        }

        // Update session stats
        let session_key = Self::session_key(token);
        let result: Result<(), redis::RedisError> = redis::pipe()
            .hincr(&session_key, "forwarded_webhooks", 1i64)
            .hset(&session_key, "last_activity", Utc::now().to_rfc3339())
            .query_async(&mut conn)
            .await;

        if let Err(e) = result {
            error!(
                "Redis: failed to update forwarded stats for {}: {}",
                token, e
            );
        }
    }

    async fn update_webhook_response(
        &self,
        token: &str,
        webhook_id: &str,
        response: StoredResponse,
    ) {
        let mut conn = self.conn.clone();
        let key = Self::webhook_key(token, webhook_id);

        let response_json = serde_json::to_string(&response).unwrap_or_default();
        let result: Result<(), redis::RedisError> =
            conn.hset(&key, "response", &response_json).await;
        if let Err(e) = result {
            error!(
                "Redis: failed to store response for {}:{}: {}",
                token, webhook_id, e
            );
        }
    }

    async fn get_or_create_session(&self, token: &str) -> TokenSession {
        let mut conn = self.conn.clone();

        match self.read_session(&mut conn, token).await {
            Ok(Some(session)) => session,
            Ok(None) => {
                // Create default session
                let session = TokenSession::default();
                if let Err(e) = self.write_session(&mut conn, token, &session).await {
                    error!("Redis: failed to create session for {}: {}", token, e);
                }
                session
            }
            Err(e) => {
                error!("Redis: failed to read session for {}: {}", token, e);
                TokenSession::default()
            }
        }
    }

    async fn set_connected(&self, token: &str, from: Option<String>) {
        let mut conn = self.conn.clone();
        let key = Self::session_key(token);
        let now = Utc::now();

        let mut fields: Vec<(&str, String)> = vec![
            ("connected", "true".to_string()),
            ("connected_at", now.to_rfc3339()),
            ("last_activity", now.to_rfc3339()),
        ];

        if let Some(ref from_addr) = from {
            fields.push(("connected_from", from_addr.clone()));
        }

        // Ensure created_at exists (set if not present via HSETNX in a separate call)
        let _: Result<bool, redis::RedisError> =
            conn.hset_nx(&key, "created_at", now.to_rfc3339()).await;

        let result: Result<(), redis::RedisError> = redis::pipe()
            .hset_multiple(&key, &fields)
            .expire(&key, KEY_TTL_SECONDS as i64)
            .query_async(&mut conn)
            .await;

        if let Err(e) = result {
            error!("Redis: failed to set connected for {}: {}", token, e);
        }

        audit::log_audit(
            audit::AuditEvent::SessionCreated,
            Some(token),
            from.as_deref(),
            "connected",
        );
    }

    async fn set_disconnected(&self, token: &str) {
        let mut conn = self.conn.clone();
        let key = Self::session_key(token);

        let result: Result<(), redis::RedisError> = redis::pipe()
            .hset(&key, "connected", "false")
            .hdel(&key, "connected_from")
            .hdel(&key, "connected_at")
            .query_async(&mut conn)
            .await;

        if let Err(e) = result {
            error!("Redis: failed to set disconnected for {}: {}", token, e);
        }

        audit::log_audit(
            audit::AuditEvent::SessionDisconnected,
            Some(token),
            None,
            "disconnected",
        );
    }

    async fn is_connected(&self, token: &str) -> bool {
        let mut conn = self.conn.clone();
        let key = Self::session_key(token);

        let result: Result<Option<String>, redis::RedisError> = conn.hget(&key, "connected").await;

        match result {
            Ok(Some(val)) => val == "true",
            Ok(None) => false,
            Err(e) => {
                error!("Redis: failed to check connected for {}: {}", token, e);
                false
            }
        }
    }

    async fn delete_webhook(&self, token: &str, webhook_id: &str) -> bool {
        let mut conn = self.conn.clone();
        let key = Self::webhook_key(token, webhook_id);

        // Check existence first
        let exists: Result<bool, redis::RedisError> = conn.exists(&key).await;
        let existed = exists.unwrap_or(false);

        if !existed {
            return false;
        }

        // Delete the webhook hash
        let result: Result<(), redis::RedisError> = conn.del(&key).await;
        if let Err(e) = result {
            error!(
                "Redis: failed to delete webhook {}:{}: {}",
                token, webhook_id, e
            );
            return false;
        }

        // Remove from the webhooks list
        let list_key = Self::webhooks_list_key(token);
        let result: Result<(), redis::RedisError> = conn.lrem(&list_key, 1, webhook_id).await;
        if let Err(e) = result {
            warn!(
                "Redis: failed to remove from list {}:{}: {}",
                token, webhook_id, e
            );
        }

        audit::log_audit(
            audit::AuditEvent::WebhookDeleted,
            Some(token),
            None,
            webhook_id,
        );

        true
    }

    async fn delete_all_webhooks(&self, token: &str) -> usize {
        let mut conn = self.conn.clone();
        let list_key = Self::webhooks_list_key(token);

        // Get all webhook IDs
        let ids: Vec<String> = match conn.lrange(&list_key, 0, -1).await {
            Ok(ids) => ids,
            Err(e) => {
                error!(
                    "Redis: failed to list webhooks for deletion {}: {}",
                    token, e
                );
                return 0;
            }
        };

        let count = ids.len();

        if count > 0 {
            // Delete all webhook hashes
            let keys: Vec<String> = ids.iter().map(|id| Self::webhook_key(token, id)).collect();

            let result: Result<(), redis::RedisError> = conn.del(&keys).await;
            if let Err(e) = result {
                error!(
                    "Redis: failed to delete webhook hashes for {}: {}",
                    token, e
                );
            }

            // Delete the list itself
            let result: Result<(), redis::RedisError> = conn.del(&list_key).await;
            if let Err(e) = result {
                error!("Redis: failed to delete webhook list for {}: {}", token, e);
            }
        }

        audit::log_audit(
            audit::AuditEvent::WebhookDeleted,
            Some(token),
            None,
            &format!("all ({})", count),
        );

        count
    }

    async fn cleanup_expired(&self, _ttl: Duration) -> usize {
        // Redis handles expiration natively via TTL on each key.
        // No manual cleanup needed. Return 0 as no manual removal was done.
        0
    }

    async fn find_timed_out_sessions(&self, _session_timeout: Duration) -> Vec<String> {
        // With Redis, session timeouts are handled by the WebSocket layer.
        // The in-memory session scan approach doesn't apply because we can't
        // efficiently SCAN all session keys. The WebSocket handler already
        // disconnects timed-out sessions. Return empty.
        Vec::new()
    }

    async fn find_idle_sessions(&self, _idle_timeout: Duration) -> Vec<String> {
        // Same rationale as find_timed_out_sessions: idle detection is handled
        // by the WebSocket handler, and Redis TTL handles key expiration.
        Vec::new()
    }

    async fn touch_session(&self, token: &str) {
        let mut conn = self.conn.clone();
        let key = Self::session_key(token);

        let result: Result<(), redis::RedisError> = redis::pipe()
            .hset(&key, "last_activity", Utc::now().to_rfc3339())
            .expire(&key, KEY_TTL_SECONDS as i64)
            .query_async(&mut conn)
            .await;

        if let Err(e) = result {
            error!("Redis: failed to touch session {}: {}", token, e);
        }
    }

    fn enable_encryption(&mut self, key: &[u8; 32]) {
        self.encryption = Some(StorageEncryption::new(key));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_key_format() {
        let key = RedisStorage::session_key("c_abc123");
        assert_eq!(key, "session:c_abc123");
    }

    #[test]
    fn test_session_key_with_special_chars() {
        let key = RedisStorage::session_key("c_ABCdef0123456789");
        assert_eq!(key, "session:c_ABCdef0123456789");
    }

    #[test]
    fn test_webhooks_list_key_format() {
        let key = RedisStorage::webhooks_list_key("c_mytoken");
        assert_eq!(key, "webhooks:c_mytoken");
    }

    #[test]
    fn test_webhook_key_format() {
        let key = RedisStorage::webhook_key("c_mytoken", "wh-12345");
        assert_eq!(key, "webhook:c_mytoken:wh-12345");
    }

    #[test]
    fn test_webhook_key_uniqueness() {
        let key1 = RedisStorage::webhook_key("token_a", "id1");
        let key2 = RedisStorage::webhook_key("token_a", "id2");
        let key3 = RedisStorage::webhook_key("token_b", "id1");
        assert_ne!(key1, key2);
        assert_ne!(key1, key3);
        assert_ne!(key2, key3);
    }

    #[test]
    fn test_serialize_headers_roundtrip() {
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());
        headers.insert("x-custom".to_string(), "value".to_string());

        let serialized = RedisStorage::serialize_headers(&headers);
        let deserialized = RedisStorage::deserialize_headers(&serialized);

        assert_eq!(headers, deserialized);
    }

    #[test]
    fn test_serialize_headers_empty() {
        let headers = HashMap::new();
        let serialized = RedisStorage::serialize_headers(&headers);
        let deserialized = RedisStorage::deserialize_headers(&serialized);
        assert_eq!(headers, deserialized);
    }

    #[test]
    fn test_deserialize_headers_invalid_json() {
        let result = RedisStorage::deserialize_headers("not json");
        assert!(result.is_empty());
    }

    #[test]
    fn test_deserialize_headers_empty_object() {
        let result = RedisStorage::deserialize_headers("{}");
        assert!(result.is_empty());
    }

    #[test]
    fn test_serialize_headers_special_characters() {
        let mut headers = HashMap::new();
        headers.insert(
            "key".to_string(),
            "value with \"quotes\" and \\slashes".to_string(),
        );
        headers.insert(
            "unicode".to_string(),
            "valeur avec des accents: e\u{0301}".to_string(),
        );

        let serialized = RedisStorage::serialize_headers(&headers);
        let deserialized = RedisStorage::deserialize_headers(&serialized);

        assert_eq!(headers, deserialized);
    }

    #[test]
    fn test_key_ttl_constant() {
        assert_eq!(KEY_TTL_SECONDS, 86400, "TTL should be 24 hours in seconds");
    }

    #[test]
    fn test_max_webhooks_per_token_constant() {
        assert_eq!(MAX_WEBHOOKS_PER_TOKEN, 1000);
    }

    #[test]
    fn test_keys_are_namespaced_and_distinct() {
        let token = "c_test";
        let session = RedisStorage::session_key(token);
        let list = RedisStorage::webhooks_list_key(token);
        let webhook = RedisStorage::webhook_key(token, "id1");

        // All keys for the same token must be distinct
        assert_ne!(session, list);
        assert_ne!(session, webhook);
        assert_ne!(list, webhook);

        // All keys use the token as part of the key
        assert!(session.contains(token));
        assert!(list.contains(token));
        assert!(webhook.contains(token));
    }
}
