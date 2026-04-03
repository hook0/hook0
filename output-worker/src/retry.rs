//! Retry delay computation: strategy-aware with default fallback.
//!
//! When a webhook delivery fails, this module decides how long to wait before
//! the next attempt — or whether to give up entirely. It loads the subscription's
//! retry schedule from the DB (custom, linear, or increasing strategy) and falls
//! back to a hardcoded escalating table for legacy subscriptions that predate
//! the retry-schedule feature.

use std::time::Duration;

use sqlx::PgConnection;
use tracing::{error, warn};

use hook0_protobuf::RequestAttempt;

use crate::work::{Response, ResponseError};

/// Maximum delay cap for any single retry (7 days).
/// Prevents overflow from increasing strategy with high base_delay * wait_factor^n.
const MAX_RETRY_DELAY_SECS: Duration = Duration::from_secs(7 * 24 * 3600);

/// Row from a LEFT JOIN between subscription and retry_schedule.
/// All fields are `Option` because the join produces NULLs when no schedule is assigned.
#[derive(Debug, sqlx::FromRow)]
pub(crate) struct SubscriptionRetrySchedule {
    strategy: Option<String>,
    max_retries: Option<i32>,
    custom_intervals: Option<Vec<i32>>,
    linear_delay: Option<i32>,
    increasing_base_delay: Option<i32>,
    increasing_wait_factor: Option<f64>,
}

/// Decide the retry delay for a failed attempt, or give up (return `None`).
/// Loads the subscription's retry schedule from the DB, then delegates to
/// `compute_scheduled_retry_delay`. Returns `Err` only on DB failures.
pub(crate) async fn compute_next_retry(
    conn: &mut PgConnection,
    attempt: &RequestAttempt,
    response: &Response,
    max_retries: u8,
) -> Result<Option<Duration>, sqlx::Error> {
    match response.response_error {
        // Signing secret is broken — retrying won't help, so bail immediately
        Some(ResponseError::InvalidHeader) => {
            let msg = response
                .body
                .as_ref()
                .and_then(|bytes| str::from_utf8(bytes).ok())
                .unwrap_or("???");
            error!(request_attempt_id = %attempt.request_attempt_id, "Could not construct signature ({msg}); giving up");
            Ok(None)
        }
        _ => {
            // InvalidTarget means the target URL, method, or custom headers are malformed —
            // we log a warning but still proceed with the normal retry path (the issue may be transient or fixable by the user)
            if let Some(ResponseError::InvalidTarget) = response.response_error {
                let msg = response
                    .body
                    .as_ref()
                    .and_then(|bytes| str::from_utf8(bytes).ok())
                    .unwrap_or("???");
                warn!(request_attempt_id = %attempt.request_attempt_id, "Invalid target ({msg}); continuing as normal");
            }

            let sub = sqlx::query_as::<_, SubscriptionRetrySchedule>(
                r#"
                    SELECT
                        rs.strategy,
                        rs.max_retries,
                        rs.custom_intervals,
                        rs.linear_delay,
                        rs.increasing_base_delay,
                        rs.increasing_wait_factor
                    FROM webhook.subscription AS s
                    INNER JOIN event.application AS a ON a.application__id = s.application__id
                    LEFT JOIN webhook.retry_schedule AS rs ON rs.retry_schedule__id = s.retry_schedule__id
                    WHERE s.subscription__id = $1
                        AND s.deleted_at IS NULL
                        AND s.is_enabled
                        AND a.deleted_at IS NULL
                "#,
            )
            .bind(attempt.subscription_id)
            .fetch_optional(conn)
            .await?;

            match sub {
                Some(info) => Ok(compute_scheduled_retry_delay(&info, attempt.retry_count, max_retries)),
                None => Ok(None),
            }
        }
    }
}

/// Computes the retry delay based on the subscription's assigned retry schedule.
/// Falls back to the default hardcoded backoff when no schedule is assigned.
fn compute_scheduled_retry_delay(
    info: &SubscriptionRetrySchedule,
    retry_count: i16,
    global_max_retries: u8,
) -> Option<Duration> {
    // Negative retry_count means a corrupt attempt record — bail out rather than panic on cast
    if retry_count < 0 {
        return None;
    }

    match info.strategy.as_deref() {
        Some("increasing") => {
            let Some(max) = info.max_retries else {
                tracing::warn!("Retry schedule has strategy 'increasing' but max_retries is NULL — skipping retry");
                return None;
            };
            if retry_count >= max as i16 {
                return None;
            }
            let Some(base) = info.increasing_base_delay else {
                tracing::warn!("Retry schedule has strategy 'increasing' but increasing_base_delay is NULL — skipping retry");
                return None;
            };
            let Some(factor) = info.increasing_wait_factor else {
                tracing::warn!("Retry schedule has strategy 'increasing' but increasing_wait_factor is NULL — skipping retry");
                return None;
            };
            let secs = (base as f64) * factor.powi(i32::from(retry_count));
            let delay = Duration::try_from_secs_f64(secs).unwrap_or(MAX_RETRY_DELAY_SECS);
            Some(delay.min(MAX_RETRY_DELAY_SECS))
        }
        Some("linear") => {
            let Some(max) = info.max_retries else {
                tracing::warn!("Retry schedule has strategy 'linear' but max_retries is NULL — skipping retry");
                return None;
            };
            if retry_count >= max as i16 {
                return None;
            }
            let Some(delay_secs) = info.linear_delay else {
                tracing::warn!("Retry schedule has strategy 'linear' but linear_delay is NULL — skipping retry");
                return None;
            };
            Some(Duration::from_secs(delay_secs as u64))
        }
        Some("custom") => {
            let intervals = info.custom_intervals.as_deref().unwrap_or(&[]);
            intervals
                .get(usize::try_from(retry_count).ok()?)
                .map(|&d| Duration::from_secs(d as u64))
        }
        _ => {
            // Legacy path: subscriptions created before retry schedules have no schedule.
            // Fall back to the original hardcoded backoff so existing behavior is preserved.
            compute_default_retry_delay(global_max_retries, retry_count)
        }
    }
}

/// Hardcoded escalating backoff table: 3s, 10s, 3m, 30m, 1h, 3h, 5h, then 10h for every
/// subsequent retry. These values were chosen to give transient failures a quick second
/// chance while spacing out retries for persistent outages.
fn compute_default_retry_delay(max_retries: u8, retry_count: i16) -> Option<Duration> {
    if retry_count < max_retries.into() {
        match retry_count {
            0 => Some(Duration::from_secs(3)),
            1 => Some(Duration::from_secs(10)),
            2 => Some(Duration::from_secs(3 * 60)),
            3 => Some(Duration::from_secs(30 * 60)),
            4 => Some(Duration::from_hours(1)),
            5 => Some(Duration::from_hours(3)),
            6 => Some(Duration::from_hours(5)),
            _ => Some(Duration::from_hours(10)),
        }
    } else {
        None
    }
}

/// Walk the default backoff table, counting how many retries fit within `max_retry_window`.
/// Returns `(effective_retries, cumulative_duration)` — used at startup to log the actual
/// retry budget after clamping to the configured time window.
pub(crate) fn evaluate_retry_policy(max_retries: u8, max_retry_window: Duration) -> (u8, Duration) {
    let mut cumulative = Duration::ZERO;
    let mut effective_retries = 0;

    for i in 0..max_retries {
        match compute_default_retry_delay(max_retries, i.into()) {
            Some(d) => {
                if cumulative + d > max_retry_window {
                    break;
                }
                cumulative += d;
                effective_retries = i + 1;
            }
            None => break,
        }
    }

    (effective_retries, cumulative)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluate_retry_policy_zero_retries() {
        let (retries, cumulative) = evaluate_retry_policy(0, Duration::from_hours(1));
        assert_eq!(retries, 0);
        assert_eq!(cumulative, Duration::ZERO);
    }

    #[test]
    fn test_evaluate_retry_policy_zero_window() {
        let (retries, cumulative) = evaluate_retry_policy(30, Duration::ZERO);
        assert_eq!(retries, 0);
        assert_eq!(cumulative, Duration::ZERO);
    }

    #[test]
    fn test_compute_default_retry_delay_exceeds_max() {
        assert_eq!(compute_default_retry_delay(5, 5), None);
        assert_eq!(compute_default_retry_delay(5, 6), None);
        assert_eq!(compute_default_retry_delay(0, 0), None);
    }

    #[test]
    fn test_evaluate_retry_policy_unlimited_window() {
        let window = Duration::from_hours(365 * 24);
        let (retries, cumulative) = evaluate_retry_policy(30, window);
        assert_eq!(retries, 30);
        assert!(cumulative < window / 10); // Duration is not just the window but the actual cumulative duration
    }

    #[test]
    fn test_evaluate_retry_policy_tight_window() {
        let window = Duration::from_secs(15);
        let (retries, cumulative) = evaluate_retry_policy(30, window);
        assert_eq!(retries, 2);
        assert!(cumulative < window);
    }

    #[test]
    fn scheduled_increasing_delays() {
        let info = SubscriptionRetrySchedule {
            strategy: Some("increasing".to_string()),
            max_retries: Some(5),
            custom_intervals: None,
            linear_delay: None,
            increasing_base_delay: Some(3),
            increasing_wait_factor: Some(3.0),
        };
        assert_eq!(compute_scheduled_retry_delay(&info, 0, 25), Some(Duration::from_secs(3)));
        assert_eq!(compute_scheduled_retry_delay(&info, 1, 25), Some(Duration::from_secs(9)));
        assert_eq!(compute_scheduled_retry_delay(&info, 2, 25), Some(Duration::from_secs(27)));
        assert_eq!(compute_scheduled_retry_delay(&info, 5, 25), None);
    }

    #[test]
    fn scheduled_linear_delays() {
        let info = SubscriptionRetrySchedule {
            strategy: Some("linear".to_string()),
            max_retries: Some(3),
            custom_intervals: None,
            linear_delay: Some(120),
            increasing_base_delay: None,
            increasing_wait_factor: None,
        };
        assert_eq!(compute_scheduled_retry_delay(&info, 0, 25), Some(Duration::from_secs(120)));
        assert_eq!(compute_scheduled_retry_delay(&info, 2, 25), Some(Duration::from_secs(120)));
        assert_eq!(compute_scheduled_retry_delay(&info, 3, 25), None);
    }

    #[test]
    fn scheduled_custom_delays() {
        let info = SubscriptionRetrySchedule {
            strategy: Some("custom".to_string()),
            max_retries: Some(3),
            custom_intervals: Some(vec![10, 60, 300]),
            linear_delay: None,
            increasing_base_delay: None,
            increasing_wait_factor: None,
        };
        assert_eq!(compute_scheduled_retry_delay(&info, 0, 25), Some(Duration::from_secs(10)));
        assert_eq!(compute_scheduled_retry_delay(&info, 1, 25), Some(Duration::from_secs(60)));
        assert_eq!(compute_scheduled_retry_delay(&info, 2, 25), Some(Duration::from_secs(300)));
        assert_eq!(compute_scheduled_retry_delay(&info, 3, 25), None);
    }

    #[test]
    fn no_schedule_falls_back_to_default() {
        let info = SubscriptionRetrySchedule {
            strategy: None,
            max_retries: None,
            custom_intervals: None,
            linear_delay: None,
            increasing_base_delay: None,
            increasing_wait_factor: None,
        };
        assert_eq!(compute_scheduled_retry_delay(&info, 0, 25), Some(Duration::from_secs(3)));
        assert_eq!(compute_scheduled_retry_delay(&info, 1, 25), Some(Duration::from_secs(10)));
        assert_eq!(compute_scheduled_retry_delay(&info, 25, 25), None);
    }

    #[test]
    fn increasing_worst_case_caps_at_max_delay() {
        // Worst-case DB-allowed params: base=3600, factor=10, max_retries=25
        // retry 24 = 3600 * 10^24 which overflows Duration — must cap, not panic
        let info = SubscriptionRetrySchedule {
            strategy: Some("increasing".to_string()),
            max_retries: Some(25),
            custom_intervals: None,
            linear_delay: None,
            increasing_base_delay: Some(3600),
            increasing_wait_factor: Some(10.0),
        };
        let result = compute_scheduled_retry_delay(&info, 24, 25);
        assert_eq!(result, Some(MAX_RETRY_DELAY_SECS));
    }

    #[test]
    fn null_max_retries_increasing_returns_none() {
        let info = SubscriptionRetrySchedule {
            strategy: Some("increasing".to_string()),
            max_retries: None,
            custom_intervals: None,
            linear_delay: None,
            increasing_base_delay: Some(3),
            increasing_wait_factor: Some(3.0),
        };
        assert_eq!(compute_scheduled_retry_delay(&info, 0, 25), None);
    }

    #[test]
    fn null_max_retries_linear_returns_none() {
        let info = SubscriptionRetrySchedule {
            strategy: Some("linear".to_string()),
            max_retries: None,
            custom_intervals: None,
            linear_delay: Some(60),
            increasing_base_delay: None,
            increasing_wait_factor: None,
        };
        assert_eq!(compute_scheduled_retry_delay(&info, 0, 25), None);
    }

    #[test]
    fn negative_retry_count_returns_none() {
        let info = SubscriptionRetrySchedule {
            strategy: Some("custom".to_string()),
            max_retries: Some(3),
            custom_intervals: Some(vec![10, 60, 300]),
            linear_delay: None,
            increasing_base_delay: None,
            increasing_wait_factor: None,
        };
        assert_eq!(compute_scheduled_retry_delay(&info, -1, 25), None);
    }

    #[test]
    fn null_increasing_base_delay_returns_none() {
        let info = SubscriptionRetrySchedule {
            strategy: Some("increasing".to_string()),
            max_retries: Some(5),
            custom_intervals: None,
            linear_delay: None,
            increasing_base_delay: None,
            increasing_wait_factor: Some(3.0),
        };
        assert_eq!(compute_scheduled_retry_delay(&info, 0, 25), None);
    }

    #[test]
    fn null_increasing_wait_factor_returns_none() {
        let info = SubscriptionRetrySchedule {
            strategy: Some("increasing".to_string()),
            max_retries: Some(5),
            custom_intervals: None,
            linear_delay: None,
            increasing_base_delay: Some(3),
            increasing_wait_factor: None,
        };
        assert_eq!(compute_scheduled_retry_delay(&info, 0, 25), None);
    }

    #[test]
    fn null_linear_delay_returns_none() {
        let info = SubscriptionRetrySchedule {
            strategy: Some("linear".to_string()),
            max_retries: Some(3),
            custom_intervals: None,
            linear_delay: None,
            increasing_base_delay: None,
            increasing_wait_factor: None,
        };
        assert_eq!(compute_scheduled_retry_delay(&info, 0, 25), None);
    }

    // -- Integration tests (require DATABASE_URL) -----------------------------

    async fn setup_test_pool() -> Option<sqlx::PgPool> {
        let url = std::env::var("DATABASE_URL").ok()?;
        sqlx::PgPool::connect(&url).await.ok()
    }

    /// Inserts minimum FK-chain: org -> app -> subscription (with retry_schedule) + target_http.
    /// Returns (org_id, sub_id).
    async fn insert_retry_fixtures(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> (uuid::Uuid, uuid::Uuid) {
        let org_id = uuid::Uuid::now_v7();
        let app_id = uuid::Uuid::now_v7();
        let sub_id = uuid::Uuid::now_v7();
        let rs_id = uuid::Uuid::now_v7();
        let target_id = sub_id;

        // Organization
        sqlx::query("INSERT INTO iam.organization (organization__id, name, created_by) VALUES ($1, $2, $3)")
            .bind(org_id)
            .bind("test-org-retry")
            .bind(uuid::Uuid::nil())
            .execute(&mut **tx)
            .await
            .unwrap();

        // Application
        sqlx::query("INSERT INTO event.application (application__id, organization__id, name) VALUES ($1, $2, $3)")
            .bind(app_id)
            .bind(org_id)
            .bind("test-app-retry")
            .execute(&mut **tx)
            .await
            .unwrap();

        // Retry schedule (linear, 120s, max 5)
        sqlx::query(
            r#"INSERT INTO webhook.retry_schedule
               (retry_schedule__id, organization__id, name, strategy, max_retries, linear_delay)
               VALUES ($1, $2, $3, 'linear', 5, 120)"#,
        )
        .bind(rs_id)
        .bind(org_id)
        .bind("test-linear-schedule")
        .execute(&mut **tx)
        .await
        .unwrap();

        // Subscription with retry schedule
        sqlx::query(
            r#"INSERT INTO webhook.subscription
               (subscription__id, application__id, target__id, is_enabled, labels, retry_schedule__id)
               VALUES ($1, $2, $3, true, '{"env":"test"}'::jsonb, $4)"#,
        )
        .bind(sub_id)
        .bind(app_id)
        .bind(target_id)
        .bind(rs_id)
        .execute(&mut **tx)
        .await
        .unwrap();

        // Target HTTP
        sqlx::query("INSERT INTO webhook.target_http (target__id, method, url) VALUES ($1, $2, $3)")
            .bind(target_id)
            .bind("POST")
            .bind("https://example.com/webhook")
            .execute(&mut **tx)
            .await
            .unwrap();

        (org_id, sub_id)
    }

    #[tokio::test]
    #[ignore]
    async fn test_custom_retry_schedule_applied() {
        let pool = match setup_test_pool().await {
            Some(p) => p,
            None => return,
        };

        let mut tx = pool.begin().await.unwrap();
        let (org_id, sub_id) = insert_retry_fixtures(&mut tx).await;
        tx.commit().await.unwrap();

        // Build a fake attempt for this subscription
        let attempt = hook0_protobuf::RequestAttempt {
            application_id: uuid::Uuid::nil(),
            request_attempt_id: uuid::Uuid::now_v7(),
            event_id: uuid::Uuid::now_v7(),
            event_received_at: chrono::Utc::now(),
            subscription_id: sub_id,
            created_at: chrono::Utc::now(),
            retry_count: 0,
            http_method: "POST".to_string(),
            http_url: "https://example.com/webhook".to_string(),
            http_headers: serde_json::json!({}),
            event_type_name: "svc.res.created".to_string(),
            payload: vec![],
            payload_content_type: "application/json".to_string(),
            secret: uuid::Uuid::now_v7(),
        };

        // Simulate an HTTP error response (triggers retry)
        let response = Response {
            response_error: Some(ResponseError::Http),
            http_code: Some(500),
            headers: None,
            body: None,
            elapsed_time: Duration::from_millis(100),
        };

        // Call compute_next_retry with a real DB connection
        let mut conn = pool.acquire().await.unwrap();
        let result = compute_next_retry(&mut conn, &attempt, &response, 25)
            .await
            .unwrap();

        // Should return linear delay of 120s (from our retry schedule)
        assert_eq!(
            result,
            Some(Duration::from_secs(120)),
            "should apply linear retry schedule (120s)"
        );

        // Test retry_count = 4 (last retry before max_retries=5)
        let attempt_last = hook0_protobuf::RequestAttempt {
            retry_count: 4,
            ..attempt.clone()
        };
        let result_last = compute_next_retry(&mut conn, &attempt_last, &response, 25)
            .await
            .unwrap();
        assert_eq!(
            result_last,
            Some(Duration::from_secs(120)),
            "retry 4 should still return 120s"
        );

        // Test retry_count = 5 (at max_retries, should give up)
        let attempt_over = hook0_protobuf::RequestAttempt {
            retry_count: 5,
            ..attempt.clone()
        };
        let result_over = compute_next_retry(&mut conn, &attempt_over, &response, 25)
            .await
            .unwrap();
        assert_eq!(
            result_over, None,
            "retry 5 should return None (max_retries=5 exhausted)"
        );

        // Cleanup
        sqlx::query("DELETE FROM iam.organization WHERE organization__id = $1")
            .bind(org_id)
            .execute(&pool)
            .await
            .unwrap();
    }
}
