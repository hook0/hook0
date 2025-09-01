use base64::Engine;
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use log::{error, info, warn};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::Sha256;
use sqlx::{query, PgPool};
use std::time::Duration;
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone)]
pub struct OperationalWebhookDelivery {
    db: PgPool,
    client: reqwest::Client,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeliveryAttempt {
    pub operational_attempt_id: Uuid,
    pub operational_event_id: Uuid,
    pub operational_endpoint_id: Uuid,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebhookPayload {
    #[serde(rename = "type")]
    pub event_type: String,
    pub timestamp: DateTime<Utc>,
    pub data: Value,
}

impl OperationalWebhookDelivery {
    pub fn new(db: PgPool) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("Hook0/1.0")
            .build()
            .expect("Failed to create HTTP client");

        Self { db, client }
    }

    /// Generate webhook signature similar to Svix's signing mechanism
    pub fn generate_signature(
        secret: &Uuid,
        timestamp: &DateTime<Utc>,
        payload: &[u8],
    ) -> String {
        let timestamp_str = timestamp.timestamp().to_string();
        let signed_content = format!("{}.{}", timestamp_str, String::from_utf8_lossy(payload));
        
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(signed_content.as_bytes());
        
        let result = mac.finalize();
        let signature = base64::engine::general_purpose::STANDARD.encode(result.into_bytes());
        
        // Return signature in format: v1,<signature>
        format!("v1,{}", signature)
    }

    /// Verify webhook signature
    #[allow(dead_code)]
    pub fn verify_signature(
        secret: &Uuid,
        signature_header: &str,
        timestamp_header: &str,
        payload: &[u8],
    ) -> bool {
        // Parse timestamp
        let timestamp = match timestamp_header.parse::<i64>() {
            Ok(ts) => ts,
            Err(_) => return false,
        };

        // Check if timestamp is not too old (5 minutes)
        let current_timestamp = Utc::now().timestamp();
        if (current_timestamp - timestamp).abs() > 300 {
            warn!("Webhook signature timestamp too old");
            return false;
        }

        // Parse signature header (format: v1,<signature1> v1,<signature2>)
        let signatures: Vec<&str> = signature_header.split(' ').collect();
        
        for sig in signatures {
            if !sig.starts_with("v1,") {
                continue;
            }
            
            let signature = &sig[3..];
            let signed_content = format!("{}.{}", timestamp, String::from_utf8_lossy(payload));
            
            let mut mac = match HmacSha256::new_from_slice(secret.as_bytes()) {
                Ok(m) => m,
                Err(_) => return false,
            };
            mac.update(signed_content.as_bytes());
            
            let expected = base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes());
            
            if signature == expected {
                return true;
            }
        }
        
        false
    }

    /// Process pending operational webhook deliveries
    pub async fn process_pending_deliveries(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Fetch pending operational webhook attempts
        let pending_attempts = query!(
            r#"
            SELECT 
                oa.operational_attempt__id,
                oa.operational_event__id,
                oa.operational_endpoint__id,
                oe.url,
                oe.headers,
                oe.secret,
                ev.event_type__name,
                ev.payload,
                ev.occurred_at
            FROM webhook.operational_attempt oa
            JOIN webhook.operational_endpoint oe ON oe.operational_endpoint__id = oa.operational_endpoint__id
            JOIN webhook.operational_event ev ON ev.operational_event__id = oa.operational_event__id
            WHERE oa.status = 'pending'
              AND oe.is_enabled = true
              AND oe.deleted_at IS NULL
            LIMIT 100
            "#
        )
        .fetch_all(&self.db)
        .await?;

        for attempt in pending_attempts {
            let payload = WebhookPayload {
                event_type: attempt.event_type__name.clone(),
                timestamp: attempt.occurred_at,
                data: attempt.payload,
            };

            let payload_bytes = serde_json::to_vec(&payload)?;
            
            // Generate signature
            let signature = Self::generate_signature(
                &attempt.secret,
                &attempt.occurred_at,
                &payload_bytes,
            );

            // Build headers
            let mut headers = HeaderMap::new();
            headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
            headers.insert("webhook-id", HeaderValue::from_str(&attempt.operational_event__id.to_string())?);
            headers.insert("webhook-timestamp", HeaderValue::from_str(&attempt.occurred_at.timestamp().to_string())?);
            headers.insert("webhook-signature", HeaderValue::from_str(&signature)?);

            // Add custom headers from endpoint configuration
            if !attempt.headers.is_null() && let Some(headers_obj) = attempt.headers.as_object() {
                for (key, value) in headers_obj {
                    if let Some(val_str) = value.as_str() && let Ok(header_name) = HeaderName::from_bytes(key.as_bytes()) && let Ok(header_value) = HeaderValue::from_str(val_str) {
                        headers.insert(header_name, header_value);
                    }
                }
            }

            // Send webhook
            let result = self.client
                .post(&attempt.url)
                .headers(headers.clone())
                .body(payload_bytes)
                .send()
                .await;

            match result {
                Ok(response) => {
                    let status_code = response.status().as_u16() as i32;
                    let response_headers = serde_json::to_value(
                        response.headers()
                            .iter()
                            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
                            .collect::<std::collections::HashMap<_, _>>()
                    )?;
                    let response_body = response.text().await.unwrap_or_default();

                    let status = if (200..300).contains(&status_code) {
                        "success"
                    } else {
                        "failed"
                    };

                    // Update attempt status
                    query!(
                        r#"
                        UPDATE webhook.operational_attempt
                        SET status = $1,
                            response_status_code = $2,
                            response_headers = $3,
                            response_body = $4,
                            attempted_at = statement_timestamp()
                        WHERE operational_attempt__id = $5
                        "#,
                        status,
                        status_code,
                        response_headers,
                        response_body,
                        attempt.operational_attempt__id
                    )
                    .execute(&self.db)
                    .await?;

                    info!(
                        "Delivered operational webhook {} to {} with status {}",
                        attempt.operational_event__id, attempt.url, status_code
                    );

                    // Check if we need to trigger failure events
                    if status == "failed" {
                        self.handle_delivery_failure(
                            attempt.operational_endpoint__id,
                            attempt.operational_event__id,
                        ).await?;
                    }
                }
                Err(e) => {
                    error!(
                        "Failed to deliver operational webhook {} to {}: {}",
                        attempt.operational_event__id, attempt.url, e
                    );

                    // Update attempt status
                    query!(
                        r#"
                        UPDATE webhook.operational_attempt
                        SET status = 'failed',
                            error_message = $1,
                            attempted_at = statement_timestamp()
                        WHERE operational_attempt__id = $2
                        "#,
                        e.to_string(),
                        attempt.operational_attempt__id
                    )
                    .execute(&self.db)
                    .await?;

                    self.handle_delivery_failure(
                        attempt.operational_endpoint__id,
                        attempt.operational_event__id,
                    ).await?;
                }
            }
        }

        Ok(())
    }

    /// Handle delivery failures and potentially disable endpoints
    async fn handle_delivery_failure(
        &self,
        endpoint_id: Uuid,
        _event_id: Uuid,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Update consecutive failure count
        let result = query!(
            r#"
            UPDATE webhook.subscription
            SET consecutive_failures = consecutive_failures + 1,
                last_failure_at = statement_timestamp()
            WHERE subscription__id = $1
            RETURNING consecutive_failures, application__id
            "#,
            endpoint_id
        )
        .fetch_optional(&self.db)
        .await?;

        if let Some(row) = result {
            // Auto-disable after 10 consecutive failures
            if row.consecutive_failures >= 10 {
                query!(
                    r#"
                    UPDATE webhook.subscription
                    SET is_enabled = false,
                        auto_disabled_at = statement_timestamp()
                    WHERE subscription__id = $1
                    "#,
                    endpoint_id
                )
                .execute(&self.db)
                .await?;

                // Trigger operational event for endpoint disabled
                query!(
                    r#"
                    SELECT webhook.trigger_operational_event($1, 'endpoint.disabled', $2)
                    "#,
                    row.application__id,
                    serde_json::json!({
                        "endpoint_id": endpoint_id,
                        "reason": "Too many consecutive failures",
                        "failure_count": row.consecutive_failures
                    })
                )
                .fetch_one(&self.db)
                .await?;

                warn!("Auto-disabled endpoint {} after {} consecutive failures", endpoint_id, row.consecutive_failures);
            }
        }

        Ok(())
    }

    /// Reset consecutive failures on successful delivery
    #[allow(dead_code)]
    pub async fn reset_failure_count(&self, endpoint_id: Uuid) -> Result<(), Box<dyn std::error::Error>> {
        query!(
            r#"
            UPDATE webhook.subscription
            SET consecutive_failures = 0
            WHERE subscription__id = $1
            "#,
            endpoint_id
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Utc, TimeZone};
    use uuid::Uuid;

    #[test]
    fn test_webhook_signature_generation() {
        let secret = Uuid::new_v4();
        let timestamp = Utc.timestamp_opt(1234567890, 0).unwrap();
        let payload = b"test payload content";

        // Generate signature
        let signature = OperationalWebhookDelivery::generate_signature(&secret, &timestamp, payload);

        // The signature should start with "v1,"
        assert!(signature.starts_with("v1,"));
        
        // The signature should be deterministic
        let signature2 = OperationalWebhookDelivery::generate_signature(&secret, &timestamp, payload);
        assert_eq!(signature, signature2);
    }

    #[test]
    fn test_webhook_signature_with_different_payloads() {
        let secret = Uuid::new_v4();
        let timestamp = Utc.timestamp_opt(1234567890, 0).unwrap();
        
        let payload1 = b"payload one";
        let payload2 = b"payload two";

        // Generate signatures for different payloads
        let signature1 = OperationalWebhookDelivery::generate_signature(&secret, &timestamp, payload1);
        let signature2 = OperationalWebhookDelivery::generate_signature(&secret, &timestamp, payload2);

        // Signatures should be different for different payloads
        assert_ne!(signature1, signature2);
    }

    #[test]
    fn test_webhook_signature_with_different_timestamps() {
        let secret = Uuid::new_v4();
        let timestamp1 = Utc.timestamp_opt(1234567890, 0).unwrap();
        let timestamp2 = Utc.timestamp_opt(1234567891, 0).unwrap();
        let payload = b"test payload";

        // Generate signatures for different timestamps
        let signature1 = OperationalWebhookDelivery::generate_signature(&secret, &timestamp1, payload);
        let signature2 = OperationalWebhookDelivery::generate_signature(&secret, &timestamp2, payload);

        // Signatures should be different for different timestamps
        assert_ne!(signature1, signature2);
    }

    #[test]
    fn test_webhook_signature_with_different_secrets() {
        let secret1 = Uuid::new_v4();
        let secret2 = Uuid::new_v4();
        let timestamp = Utc.timestamp_opt(1234567890, 0).unwrap();
        let payload = b"test payload";

        // Generate signatures for different secrets
        let signature1 = OperationalWebhookDelivery::generate_signature(&secret1, &timestamp, payload);
        let signature2 = OperationalWebhookDelivery::generate_signature(&secret2, &timestamp, payload);

        // Signatures should be different for different secrets
        assert_ne!(signature1, signature2);
    }
}