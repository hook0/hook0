use chrono::{Duration as ChronoDuration, Utc};
use log::{error, info};
use sqlx::PgPool;
use std::time::Duration;
use tokio::time;

use crate::operational_webhooks_delivery::OperationalWebhookDelivery;

/// Background worker for processing operational webhook deliveries
pub struct OperationalWebhookWorker {
    db: PgPool,
    delivery: OperationalWebhookDelivery,
    interval: Duration,
}

impl OperationalWebhookWorker {
    pub fn new(db: PgPool, interval_seconds: u64) -> Self {
        let delivery = OperationalWebhookDelivery::new(db.clone());
        let interval = Duration::from_secs(interval_seconds);
        
        Self {
            db,
            delivery,
            interval,
        }
    }

    /// Start the background worker
    pub async fn start(self) {
        info!("Starting operational webhook worker with interval {:?}", self.interval);
        
        let mut interval = time::interval(self.interval);
        
        loop {
            interval.tick().await;
            
            if let Err(e) = self.process_batch().await {
                error!("Error processing operational webhooks: {}", e);
            }
        }
    }

    /// Process a batch of operational webhooks
    async fn process_batch(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Process pending deliveries
        self.delivery.process_pending_deliveries().await?;
        
        // Retry failed attempts with exponential backoff
        self.retry_failed_attempts().await?;
        
        // Clean up old operational events
        self.cleanup_old_events().await?;
        
        // Update message statistics
        self.update_statistics().await?;
        
        Ok(())
    }

    /// Retry failed webhook attempts with exponential backoff
    async fn retry_failed_attempts(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Get failed attempts that are ready for retry
        let failed_attempts = sqlx::query!(
            r#"
            WITH attempt_counts AS (
                SELECT 
                    oa.operational_attempt__id,
                    oa.operational_event__id,
                    oa.operational_endpoint__id,
                    oa.created_at,
                    oa.attempted_at,
                    COUNT(prev.operational_attempt__id) as previous_attempts
                FROM webhook.operational_attempt oa
                LEFT JOIN webhook.operational_attempt prev ON 
                    prev.operational_event__id = oa.operational_event__id 
                    AND prev.operational_endpoint__id = oa.operational_endpoint__id
                    AND prev.created_at < oa.created_at
                WHERE oa.status = 'failed'
                GROUP BY oa.operational_attempt__id, oa.operational_event__id, 
                         oa.operational_endpoint__id, oa.created_at, oa.attempted_at
            )
            SELECT 
                operational_attempt__id,
                operational_event__id,
                created_at,
                previous_attempts
            FROM attempt_counts
            WHERE previous_attempts < 5
              AND attempted_at < NOW() - INTERVAL '1 minute' * POWER(2, LEAST(previous_attempts::int, 5))
            LIMIT 50
            "#
        )
        .fetch_all(&self.db)
        .await?;

        for attempt in failed_attempts {
            // Reset status to pending for retry
            sqlx::query!(
                r#"
                UPDATE webhook.operational_attempt
                SET status = 'pending'
                WHERE operational_attempt__id = $1
                "#,
                attempt.operational_attempt__id
            )
            .execute(&self.db)
            .await?;
            
            info!(
                "Retrying operational webhook attempt {} (attempt #{})",
                attempt.operational_attempt__id,
                attempt.previous_attempts.unwrap_or(0) + 1
            );
        }

        Ok(())
    }

    /// Clean up old operational events (older than 30 days)
    async fn cleanup_old_events(&self) -> Result<(), Box<dyn std::error::Error>> {
        let cutoff_date = Utc::now() - ChronoDuration::days(30);
        
        let deleted = sqlx::query!(
            r#"
            DELETE FROM webhook.operational_event
            WHERE occurred_at < $1
            "#,
            cutoff_date
        )
        .execute(&self.db)
        .await?;
        
        if deleted.rows_affected() > 0 {
            info!("Cleaned up {} old operational events", deleted.rows_affected());
        }
        
        Ok(())
    }

    /// Update message delivery statistics
    async fn update_statistics(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Calculate statistics for the last hour
        let period_start = Utc::now() - ChronoDuration::hours(1);
        let period_end = Utc::now();
        
        // Aggregate statistics per application and subscription
        let stats = sqlx::query!(
            r#"
            SELECT 
                s.application__id,
                s.subscription__id,
                COUNT(ra.request_attempt__id) as total_messages,
                COUNT(CASE WHEN ra.succeeded_at IS NOT NULL THEN 1 END) as successful_messages,
                COUNT(CASE WHEN ra.failed_at IS NOT NULL THEN 1 END) as failed_messages,
                COUNT(CASE WHEN ra.succeeded_at IS NULL AND ra.failed_at IS NULL THEN 1 END) as pending_messages,
                AVG(EXTRACT(EPOCH FROM (COALESCE(ra.succeeded_at, ra.failed_at) - ra.created_at)) * 1000)::integer as avg_delivery_time_ms
            FROM webhook.subscription s
            LEFT JOIN webhook.request_attempt ra ON ra.subscription__id = s.subscription__id
            WHERE ra.created_at >= $1 AND ra.created_at < $2
            GROUP BY s.application__id, s.subscription__id
            "#,
            period_start,
            period_end
        )
        .fetch_all(&self.db)
        .await?;

        for stat in &stats {
            // Insert or update statistics
            sqlx::query!(
                r#"
                INSERT INTO webhook.message_stats (
                    application__id,
                    subscription__id,
                    period_start,
                    period_end,
                    total_messages,
                    successful_messages,
                    failed_messages,
                    pending_messages,
                    avg_delivery_time_ms
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                ON CONFLICT (application__id, subscription__id, period_start, period_end)
                DO UPDATE SET
                    total_messages = EXCLUDED.total_messages,
                    successful_messages = EXCLUDED.successful_messages,
                    failed_messages = EXCLUDED.failed_messages,
                    pending_messages = EXCLUDED.pending_messages,
                    avg_delivery_time_ms = EXCLUDED.avg_delivery_time_ms
                "#,
                stat.application__id,
                stat.subscription__id,
                period_start,
                period_end,
                stat.total_messages.unwrap_or(0) as i32,
                stat.successful_messages.unwrap_or(0) as i32,
                stat.failed_messages.unwrap_or(0) as i32,
                stat.pending_messages.unwrap_or(0) as i32,
                stat.avg_delivery_time_ms
            )
            .execute(&self.db)
            .await?;
        }
        
        info!("Updated message statistics for {} subscriptions", stats.len());
        
        Ok(())
    }

    /// Handle message attempt exhaustion (all retries failed)
    pub async fn check_exhausted_attempts(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Find attempts that have exhausted all retries
        let exhausted = sqlx::query!(
            r#"
            SELECT 
                oa.operational_event__id,
                oa.operational_endpoint__id,
                oe.application__id,
                COUNT(oa.operational_attempt__id) as attempt_count
            FROM webhook.operational_attempt oa
            JOIN webhook.operational_endpoint oe ON oe.operational_endpoint__id = oa.operational_endpoint__id
            WHERE oa.status = 'failed'
            GROUP BY oa.operational_event__id, oa.operational_endpoint__id, oe.application__id
            HAVING COUNT(oa.operational_attempt__id) >= 5
            "#
        )
        .fetch_all(&self.db)
        .await?;

        for row in exhausted {
            // Trigger operational event for exhausted attempts
            sqlx::query!(
                r#"
                SELECT webhook.trigger_operational_event($1, 'message.attempt.exhausted', $2)
                "#,
                row.application__id,
                serde_json::json!({
                    "message_id": row.operational_event__id,
                    "endpoint_id": row.operational_endpoint__id,
                    "attempts": row.attempt_count
                })
            )
            .fetch_one(&self.db)
            .await?;
            
            info!(
                "Message attempts exhausted for event {} to endpoint {} after {} attempts",
                row.operational_event__id, row.operational_endpoint__id, row.attempt_count.unwrap_or(0)
            );
        }
        
        Ok(())
    }
}

/// Start the operational webhook worker as a background task
pub fn spawn_operational_webhook_worker(db: PgPool) {
    tokio::spawn(async move {
        let worker = OperationalWebhookWorker::new(db, 10); // Process every 10 seconds
        worker.start().await;
    });
}