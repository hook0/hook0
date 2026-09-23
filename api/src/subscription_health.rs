use actix_web::rt::time::sleep;
use chrono::{DateTime, Utc};
use clap::crate_name;
use opentelemetry::trace::Tracer;
use opentelemetry::{global, trace::TraceContextExt};
use sqlx::postgres::types::PgInterval;
use sqlx::{AssertSqlSafe, PgPool, query, query_scalar};
use std::time::{Duration, Instant};
use tracing::{debug, error, info, trace};
use uuid::Uuid;

use crate::humanize::humanize_duration;
use crate::opentelemetry::{
    report_subscription_health_probe_duration, report_subscription_health_probe_subscriptions,
    report_subscription_health_transition,
};

const STARTUP_GRACE_PERIOD: Duration = Duration::from_secs(15);

/// Response error names that are caused by Hook0 itself rather than by the subscription's target; they are ignored when classifying subscriptions
const HOOK0_CAUSED_RESPONSE_ERRORS: &[&str] = &["E_INTERNAL", "E_INVALID_HEADER"];

#[derive(Debug, Clone)]
pub struct ProbeParams {
    pub window: Duration,
    pub unhealthy_min_attempts: u32,
    pub max_failure_ratio: f64,
    pub recovering_max_duration: Duration,
    pub recovering_min_duration: Duration,
    pub recovering_min_attempts: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Recovering,
}

impl HealthStatus {
    /// Parse a status stored in `webhook.subscription_health` (no row means healthy)
    fn from_db(status: Option<&str>) -> Self {
        match status {
            Some("degraded") => Self::Degraded,
            Some("recovering") => Self::Recovering,
            _ => Self::Healthy,
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            Self::Healthy => "healthy",
            Self::Degraded => "degraded",
            Self::Recovering => "recovering",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Verdict {
    Healthy,
    Unhealthy,
}

/// Classify a subscription from the request attempts completed in the probe window (failures caused by Hook0 are not counted at all); `None` means there is not enough data to decide
fn classify(total: i64, failures: i64, params: &ProbeParams) -> Option<Verdict> {
    let failure_ratio = failures as f64 / total as f64;
    if total >= i64::from(params.unhealthy_min_attempts) && failure_ratio > params.max_failure_ratio
    {
        Some(Verdict::Unhealthy)
    } else if total >= 1 && failure_ratio <= params.max_failure_ratio {
        Some(Verdict::Healthy)
    } else {
        None
    }
}

/// Compute the new health status of a subscription, if it must change
fn next_status(
    current: HealthStatus,
    since: Option<DateTime<Utc>>,
    verdict: Verdict,
    completed_since_recovering: i64,
    now: DateTime<Utc>,
    params: &ProbeParams,
) -> Option<HealthStatus> {
    match (current, verdict) {
        (HealthStatus::Healthy, Verdict::Healthy) => None,
        (HealthStatus::Healthy, Verdict::Unhealthy) => Some(HealthStatus::Degraded),
        (HealthStatus::Degraded, Verdict::Healthy) => Some(HealthStatus::Recovering),
        (HealthStatus::Degraded, Verdict::Unhealthy) => None,
        (HealthStatus::Recovering, Verdict::Healthy) => {
            let recovering_for = since
                .and_then(|since| (now - since).to_std().ok())
                .unwrap_or_default();
            if recovering_for >= params.recovering_max_duration
                || (recovering_for >= params.recovering_min_duration
                    && completed_since_recovering >= i64::from(params.recovering_min_attempts))
            {
                Some(HealthStatus::Healthy)
            } else {
                None
            }
        }
        (HealthStatus::Recovering, Verdict::Unhealthy) => Some(HealthStatus::Degraded),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transition {
    pub subscription_id: Uuid,
    pub application_id: Uuid,
    pub from: HealthStatus,
    pub to: HealthStatus,
}

#[derive(Debug, Clone, Default)]
pub struct ProbeReport {
    pub transitions: Vec<Transition>,
    pub healthy: u64,
    pub unhealthy: u64,
    pub unknown: u64,
}

/// Write the subscription health settings that output workers read from `infrastructure.state`
pub async fn share_enforcement_settings(
    db: &PgPool,
    enforced: bool,
    enforced_for: &[Uuid],
) -> Result<(), sqlx::Error> {
    let mut tx = db.begin().await?;
    query!(
        "
            INSERT INTO infrastructure.state (key, value_boolean)
            VALUES ('subscription_health_enforced', $1)
            ON CONFLICT (key) DO UPDATE
                SET value_boolean = EXCLUDED.value_boolean
        ",
        enforced,
    )
    .execute(&mut *tx)
    .await?;
    query!(
        "
            INSERT INTO infrastructure.state (key, value_uuids)
            VALUES ('subscription_health_enforced_for', $1)
            ON CONFLICT (key) DO UPDATE
                SET value_uuids = EXCLUDED.value_uuids
        ",
        enforced_for,
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await
}

pub async fn periodically_probe_subscription_health(
    db: &PgPool,
    period: Duration,
    params: ProbeParams,
) {
    let timeout = period / 2;
    sleep(STARTUP_GRACE_PERIOD).await;

    loop {
        let start = Instant::now();
        match probe_subscription_health(db, &params, timeout).await {
            Ok(Some(report)) => {
                report_subscription_health_probe_duration(start.elapsed());
                report_subscription_health_probe_subscriptions("healthy", report.healthy);
                report_subscription_health_probe_subscriptions("unhealthy", report.unhealthy);
                report_subscription_health_probe_subscriptions("unknown", report.unknown);
                for t in &report.transitions {
                    report_subscription_health_transition(t.from.as_str(), t.to.as_str());
                    info!(
                        subscription_id = %t.subscription_id,
                        application_id = %t.application_id,
                        "Subscription health changed from {} to {}",
                        t.from.as_str(),
                        t.to.as_str(),
                    );
                }
                debug!(
                    "Subscription health probe classified {} subscriptions as healthy, {} as unhealthy and {} as unknown, and made {} transitions in {}",
                    report.healthy,
                    report.unhealthy,
                    report.unknown,
                    report.transitions.len(),
                    humanize_duration(start.elapsed())
                );
            }
            Ok(None) => {
                debug!("Subscription health probe skipped because another instance is running one");
            }
            Err(e) => {
                error!("Could not probe subscription health: {e}");
            }
        }

        sleep(period).await;
    }
}

/// Run one subscription health probe; returns `None` if another API instance is running one
pub async fn probe_subscription_health(
    db: &PgPool,
    params: &ProbeParams,
    timeout: Duration,
) -> Result<Option<ProbeReport>, sqlx::Error> {
    global::tracer(crate_name!())
        .in_span("probe_subscription_health", |cx| async move {
            trace!("Probing subscription health...");

            let window = to_interval(params.window)?;
            let recovering_min_duration = to_interval(params.recovering_min_duration)?;
            let recovering_max_duration = to_interval(params.recovering_max_duration)?;

            let mut tx = db.begin().await?;
            query(AssertSqlSafe(format!(
                "SET LOCAL statement_timeout = '{}s'",
                timeout.as_secs()
            )))
            .execute(&mut *tx)
            .await?;

            let acquired = query_scalar!(
                r#"SELECT pg_try_advisory_xact_lock(hashtext('subscription_health_probe')) AS "acquired!""#
            )
            .fetch_one(&mut *tx)
            .await?;

            let report = if acquired {
                // Completed request attempts are found through their response, whose primary key is a UUIDv7 and can therefore be range-scanned by creation time
                // (bounding the range on both sides keeps out the few responses that were created with a random UUIDv4 before the switch to UUIDv7).
                // The range goes back further than the window when a recovering subscription needs its completed attempts counted since it entered this state.
                let stats = query!(
                    r#"
                        WITH latest AS (
                            SELECT DISTINCT ON (subscription__id) subscription__id, status, created_at
                            FROM webhook.subscription_health
                            ORDER BY subscription__id, created_at DESC, subscription_health__id DESC
                        ), recovering AS (
                            SELECT subscription__id, created_at AS since
                            FROM latest
                            WHERE status = 'recovering'
                                AND created_at <= statement_timestamp() - $2::interval
                                AND created_at > statement_timestamp() - $3::interval
                        ), completed AS (
                            SELECT ra.subscription__id, r.response_error__name, COALESCE(ra.succeeded_at, ra.failed_at) AS completed_at
                            FROM webhook.response AS r
                            INNER JOIN webhook.request_attempt AS ra ON ra.response__id = r.response__id
                            WHERE r.response__id BETWEEN
                                    (SELECT uuidv7(LEAST(-$1::interval, (SELECT MIN(since) FROM recovering) - statement_timestamp())))
                                AND (SELECT uuidv7())
                                AND (r.response_error__name IS NULL OR r.response_error__name <> ALL($4::text[]))
                        )
                        SELECT
                            c.subscription__id AS "subscription_id!",
                            s.application__id AS application_id,
                            l.status AS "status?",
                            l.created_at AS "since?",
                            statement_timestamp() AS "now!",
                            COUNT(*) FILTER (WHERE c.completed_at >= statement_timestamp() - $1::interval) AS "total!",
                            COUNT(*) FILTER (WHERE c.completed_at >= statement_timestamp() - $1::interval AND c.response_error__name IS NOT NULL) AS "failures!",
                            COUNT(*) FILTER (WHERE c.completed_at >= rec.since) AS "completed_since_recovering!"
                        FROM completed AS c
                        INNER JOIN webhook.subscription AS s ON s.subscription__id = c.subscription__id AND s.is_enabled AND s.deleted_at IS NULL
                        INNER JOIN event.application AS a ON a.application__id = s.application__id AND a.deleted_at IS NULL
                        LEFT JOIN latest AS l ON l.subscription__id = c.subscription__id
                        LEFT JOIN recovering AS rec ON rec.subscription__id = c.subscription__id
                        GROUP BY c.subscription__id, s.application__id, l.status, l.created_at
                    "#,
                    window,
                    recovering_min_duration,
                    recovering_max_duration,
                    HOOK0_CAUSED_RESPONSE_ERRORS as &[&str],
                )
                .fetch_all(&mut *tx)
                .await?;
                cx.span().add_event("subscription_health.stats_fetched", Vec::new());

                let mut report = ProbeReport::default();
                for row in stats {
                    let current = HealthStatus::from_db(row.status.as_deref());
                    match classify(row.total, row.failures, params) {
                        Some(verdict) => {
                            match verdict {
                                Verdict::Healthy => report.healthy += 1,
                                Verdict::Unhealthy => report.unhealthy += 1,
                            }
                            if let Some(to) = next_status(
                                current,
                                row.since,
                                verdict,
                                row.completed_since_recovering,
                                row.now,
                                params,
                            ) {
                                report.transitions.push(Transition {
                                    subscription_id: row.subscription_id,
                                    application_id: row.application_id,
                                    from: current,
                                    to,
                                });
                            }
                        }
                        None => report.unknown += 1,
                    }
                }

                let (subscription_ids, statuses): (Vec<Uuid>, Vec<&str>) = report
                    .transitions
                    .iter()
                    .map(|t| (t.subscription_id, t.to.as_str()))
                    .unzip();
                query!(
                    "
                        INSERT INTO webhook.subscription_health (subscription__id, status)
                        SELECT * FROM UNNEST($1::uuid[], $2::text[])
                    ",
                    &subscription_ids,
                    &statuses as &[&str],
                )
                .execute(&mut *tx)
                .await?;

                query!(
                    "
                        INSERT INTO infrastructure.state (key, value_timestamptz)
                        VALUES ('last_probe_subscription_health', statement_timestamp())
                        ON CONFLICT (key) DO UPDATE
                            SET value_timestamptz = EXCLUDED.value_timestamptz
                            WHERE state.value_timestamptz IS NULL
                                OR state.value_timestamptz < EXCLUDED.value_timestamptz
                    "
                )
                .execute(&mut *tx)
                .await?;

                Some(report)
            } else {
                None
            };

            tx.commit().await?;
            Ok(report)
        })
        .await
}

fn to_interval(duration: Duration) -> Result<PgInterval, sqlx::Error> {
    PgInterval::try_from(duration).map_err(sqlx::Error::Encode)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeDelta;

    fn params() -> ProbeParams {
        ProbeParams {
            window: Duration::from_secs(15 * 60),
            unhealthy_min_attempts: 10,
            max_failure_ratio: 0.3,
            recovering_max_duration: Duration::from_secs(60 * 60),
            recovering_min_duration: Duration::from_secs(10 * 60),
            recovering_min_attempts: 10,
        }
    }

    #[test]
    fn classify_follows_thresholds() {
        let p = params();
        assert_eq!(classify(0, 0, &p), None);
        assert_eq!(classify(1, 0, &p), Some(Verdict::Healthy));
        assert_eq!(classify(10, 3, &p), Some(Verdict::Healthy));
        assert_eq!(classify(10, 4, &p), Some(Verdict::Unhealthy));
        assert_eq!(classify(9, 9, &p), None);
        assert_eq!(classify(100, 31, &p), Some(Verdict::Unhealthy));
    }

    #[test]
    fn next_status_follows_transition_table() {
        use HealthStatus::*;
        let p = params();
        let now = Utc::now();
        let since = Some(now - TimeDelta::minutes(1));
        let good = Verdict::Healthy;
        let bad = Verdict::Unhealthy;

        assert_eq!(next_status(Healthy, None, good, 0, now, &p), None);
        assert_eq!(next_status(Healthy, None, bad, 0, now, &p), Some(Degraded));
        assert_eq!(
            next_status(Degraded, since, good, 0, now, &p),
            Some(Recovering)
        );
        assert_eq!(next_status(Degraded, since, bad, 0, now, &p), None);
        assert_eq!(
            next_status(Recovering, since, bad, 0, now, &p),
            Some(Degraded)
        );
    }

    #[test]
    fn recovering_exits_by_duration_or_by_attempts() {
        use HealthStatus::*;
        let p = params();
        let now = Utc::now();
        let entered = |minutes| Some(now - TimeDelta::minutes(minutes));
        let good = Verdict::Healthy;

        // Too early, whatever the traffic
        assert_eq!(
            next_status(Recovering, entered(9), good, 1000, now, &p),
            None
        );
        // Early exit needs enough completed attempts
        assert_eq!(next_status(Recovering, entered(10), good, 9, now, &p), None);
        assert_eq!(
            next_status(Recovering, entered(10), good, 10, now, &p),
            Some(Healthy)
        );
        // Past the maximum duration, traffic does not matter
        assert_eq!(next_status(Recovering, entered(59), good, 0, now, &p), None);
        assert_eq!(
            next_status(Recovering, entered(60), good, 0, now, &p),
            Some(Healthy)
        );
    }

    mod db {
        use super::*;
        use crate::google_ads::test_support::{seed_event, seed_org, seed_user};

        /// Seed an application with an event type, and return `(application_id, event_id)`
        async fn seed_application(pool: &PgPool) -> (Uuid, Uuid) {
            let user = seed_user(pool).await;
            let org = seed_org(pool, user).await;
            seed_event(pool, org).await
        }

        /// Seed an enabled subscription of the application to its `test.resource.created` event type
        async fn seed_subscription(pool: &PgPool, application_id: Uuid) -> Uuid {
            let subscription_id = Uuid::new_v4();
            sqlx::query(
                r#"
                    INSERT INTO webhook.subscription
                        (subscription__id, application__id, is_enabled, secret, metadata, labels, target__id, created_at, updated_at)
                    VALUES ($1, $2, true, public.gen_random_uuid(), '{}'::jsonb, '{"env":"test"}'::jsonb, public.gen_random_uuid(), statement_timestamp(), statement_timestamp())
                "#,
            )
            .bind(subscription_id)
            .bind(application_id)
            .execute(pool)
            .await
            .expect("seed subscription");
            sqlx::query(
                "INSERT INTO webhook.subscription__event_type (application__id, subscription__id, event_type__name) VALUES ($1, $2, 'test.resource.created')",
            )
            .bind(application_id)
            .bind(subscription_id)
            .execute(pool)
            .await
            .expect("seed subscription event type");
            subscription_id
        }

        /// Seed `count` request attempts completed `minutes_ago`, successful if `error` is `None`
        async fn seed_completed_attempts(
            pool: &PgPool,
            application_id: Uuid,
            event_id: Uuid,
            subscription_id: Uuid,
            count: u32,
            minutes_ago: i32,
            error: Option<&str>,
        ) {
            if let Some(e) = error {
                sqlx::query("INSERT INTO webhook.response_error (response_error__name) VALUES ($1) ON CONFLICT DO NOTHING")
                    .bind(e)
                    .execute(pool)
                    .await
                    .expect("seed response error");
            }
            for _ in 0..count {
                // The response ID must be consistent with the completion time, as the probe relies on both
                sqlx::query(
                    r#"
                        WITH r AS (
                            INSERT INTO webhook.response (response__id, response_error__name)
                            VALUES (uuidv7(-make_interval(mins => $4)), $5::text)
                            RETURNING response__id
                        )
                        INSERT INTO webhook.request_attempt (event__id, subscription__id, application__id, picked_at, succeeded_at, failed_at, response__id)
                        SELECT $1, $2, $3, c.at, CASE WHEN $5::text IS NULL THEN c.at END, CASE WHEN $5::text IS NOT NULL THEN c.at END, r.response__id
                        FROM r, (SELECT statement_timestamp() - make_interval(mins => $4) AS at) AS c
                    "#,
                )
                .bind(event_id)
                .bind(subscription_id)
                .bind(application_id)
                .bind(minutes_ago)
                .bind(error)
                .execute(pool)
                .await
                .expect("seed completed request attempt");
            }
        }

        async fn set_health(pool: &PgPool, subscription_id: Uuid, status: &str, minutes_ago: i32) {
            sqlx::query(
                "INSERT INTO webhook.subscription_health (subscription__id, status, created_at) VALUES ($1, $2, statement_timestamp() - make_interval(mins => $3))",
            )
            .bind(subscription_id)
            .bind(status)
            .bind(minutes_ago)
            .execute(pool)
            .await
            .expect("seed subscription health");
        }

        async fn effective_health(
            pool: &PgPool,
            application_id: Uuid,
            subscription_id: Uuid,
        ) -> String {
            sqlx::query_scalar("SELECT webhook.effective_subscription_health($1, $2)")
                .bind(subscription_id)
                .bind(application_id)
                .fetch_one(pool)
                .await
                .expect("compute effective subscription health")
        }

        async fn probe(pool: &PgPool) -> ProbeReport {
            probe_subscription_health(pool, &params(), Duration::from_secs(60))
                .await
                .expect("probe subscription health")
                .expect("probe was not skipped")
        }

        fn transition_of(report: &ProbeReport, subscription_id: Uuid) -> Option<HealthStatus> {
            report
                .transitions
                .iter()
                .find(|t| t.subscription_id == subscription_id)
                .map(|t| t.to)
        }

        #[sqlx::test]
        async fn effective_health_only_applies_when_enforced(pool: PgPool) {
            let (application_id, _) = seed_application(&pool).await;
            let subscription_id = seed_subscription(&pool, application_id).await;
            let other_subscription_id = seed_subscription(&pool, application_id).await;
            set_health(&pool, subscription_id, "degraded", 1).await;

            // No settings were shared yet
            assert_eq!(
                effective_health(&pool, application_id, subscription_id).await,
                "healthy"
            );

            share_enforcement_settings(&pool, true, &[]).await.unwrap();
            assert_eq!(
                effective_health(&pool, application_id, subscription_id).await,
                "degraded"
            );
            assert_eq!(
                effective_health(&pool, application_id, other_subscription_id).await,
                "healthy",
                "a subscription without any health row is healthy"
            );

            share_enforcement_settings(&pool, true, &[application_id])
                .await
                .unwrap();
            assert_eq!(
                effective_health(&pool, application_id, subscription_id).await,
                "degraded"
            );

            share_enforcement_settings(&pool, true, &[Uuid::new_v4()])
                .await
                .unwrap();
            assert_eq!(
                effective_health(&pool, application_id, subscription_id).await,
                "healthy"
            );

            share_enforcement_settings(&pool, false, &[]).await.unwrap();
            assert_eq!(
                effective_health(&pool, application_id, subscription_id).await,
                "healthy"
            );
        }

        #[sqlx::test]
        async fn dispatch_pauses_request_attempts_of_unhealthy_subscriptions(pool: PgPool) {
            let (application_id, _) = seed_application(&pool).await;
            let degraded_subscription_id = seed_subscription(&pool, application_id).await;
            let healthy_subscription_id = seed_subscription(&pool, application_id).await;
            set_health(&pool, degraded_subscription_id, "degraded", 1).await;
            share_enforcement_settings(&pool, true, &[]).await.unwrap();

            let event_id = ingest(&pool, application_id).await;
            let paused = paused_of(&pool, event_id).await;
            assert_eq!(paused.get(&degraded_subscription_id), Some(&true));
            assert_eq!(paused.get(&healthy_subscription_id), Some(&false));

            // Replaying an event dispatches it again, with the same rule
            sqlx::query("UPDATE webhook.request_attempt SET failed_at = statement_timestamp() WHERE event__id = $1")
                .bind(event_id)
                .execute(&pool)
                .await
                .expect("complete request attempts");
            sqlx::query("UPDATE event.event SET dispatched_at = NULL WHERE event__id = $1")
                .bind(event_id)
                .execute(&pool)
                .await
                .expect("replay event");
            let paused = paused_of(&pool, event_id).await;
            assert_eq!(paused.get(&degraded_subscription_id), Some(&true));
            assert_eq!(paused.get(&healthy_subscription_id), Some(&false));

            // In shadow mode, health states have no effect
            share_enforcement_settings(&pool, false, &[]).await.unwrap();
            let event_id = ingest(&pool, application_id).await;
            let paused = paused_of(&pool, event_id).await;
            assert_eq!(paused.get(&degraded_subscription_id), Some(&false));
        }

        async fn ingest(pool: &PgPool, application_id: Uuid) -> Uuid {
            sqlx::query_scalar(
                r#"
                    INSERT INTO event.event (application__id, event_type__name, payload_content_type, ip, occurred_at, labels)
                    VALUES ($1, 'test.resource.created', 'application/json', '127.0.0.1'::inet, statement_timestamp(), '{"env":"test"}'::jsonb)
                    RETURNING event__id
                "#,
            )
            .bind(application_id)
            .fetch_one(pool)
            .await
            .expect("ingest event")
        }

        /// Whether each pending request attempt of the event is paused, by subscription
        async fn paused_of(pool: &PgPool, event_id: Uuid) -> std::collections::HashMap<Uuid, bool> {
            sqlx::query_as::<_, (Uuid, bool)>(
                "SELECT subscription__id, paused FROM webhook.request_attempt WHERE event__id = $1 AND succeeded_at IS NULL AND failed_at IS NULL",
            )
            .bind(event_id)
            .fetch_all(pool)
            .await
            .expect("fetch request attempts")
            .into_iter()
            .collect()
        }

        #[sqlx::test]
        async fn probe_follows_the_transition_table(pool: PgPool) {
            let (application_id, event_id) = seed_application(&pool).await;

            // healthy + unhealthy ⇒ degraded
            let failing = seed_subscription(&pool, application_id).await;
            seed_completed_attempts(
                &pool,
                application_id,
                event_id,
                failing,
                7,
                2,
                Some("E_HTTP"),
            )
            .await;
            seed_completed_attempts(&pool, application_id, event_id, failing, 3, 2, None).await;

            // healthy + healthy ⇒ nothing (3 failures out of 10 is not above the ratio)
            let fine = seed_subscription(&pool, application_id).await;
            seed_completed_attempts(&pool, application_id, event_id, fine, 3, 2, Some("E_HTTP"))
                .await;
            seed_completed_attempts(&pool, application_id, event_id, fine, 7, 2, None).await;

            // Failures caused by Hook0 and old failures are ignored: unknown ⇒ nothing
            let hook0_failures = seed_subscription(&pool, application_id).await;
            seed_completed_attempts(
                &pool,
                application_id,
                event_id,
                hook0_failures,
                10,
                2,
                Some("E_INTERNAL"),
            )
            .await;
            seed_completed_attempts(
                &pool,
                application_id,
                event_id,
                hook0_failures,
                10,
                2,
                Some("E_INVALID_HEADER"),
            )
            .await;
            seed_completed_attempts(
                &pool,
                application_id,
                event_id,
                hook0_failures,
                10,
                20,
                Some("E_HTTP"),
            )
            .await;

            // degraded + healthy ⇒ recovering
            let degraded_recovering = seed_subscription(&pool, application_id).await;
            set_health(&pool, degraded_recovering, "degraded", 30).await;
            seed_completed_attempts(
                &pool,
                application_id,
                event_id,
                degraded_recovering,
                1,
                2,
                None,
            )
            .await;

            // degraded + unhealthy ⇒ nothing
            let degraded_failing = seed_subscription(&pool, application_id).await;
            set_health(&pool, degraded_failing, "degraded", 30).await;
            seed_completed_attempts(
                &pool,
                application_id,
                event_id,
                degraded_failing,
                10,
                2,
                Some("E_TIMEOUT"),
            )
            .await;

            // recovering + unhealthy ⇒ degraded
            let recovering_failing = seed_subscription(&pool, application_id).await;
            set_health(&pool, recovering_failing, "recovering", 30).await;
            seed_completed_attempts(
                &pool,
                application_id,
                event_id,
                recovering_failing,
                10,
                2,
                Some("E_CONNECTION"),
            )
            .await;

            // recovering + healthy, early exit: 10 attempts completed since entering recovering 40 min ago, most of them before the window
            let recovering_busy = seed_subscription(&pool, application_id).await;
            set_health(&pool, recovering_busy, "recovering", 40).await;
            seed_completed_attempts(
                &pool,
                application_id,
                event_id,
                recovering_busy,
                9,
                30,
                None,
            )
            .await;
            seed_completed_attempts(&pool, application_id, event_id, recovering_busy, 1, 2, None)
                .await;

            // recovering + healthy, not enough attempts yet (the ones before entering recovering do not count)
            let recovering_quiet = seed_subscription(&pool, application_id).await;
            set_health(&pool, recovering_quiet, "recovering", 40).await;
            seed_completed_attempts(
                &pool,
                application_id,
                event_id,
                recovering_quiet,
                9,
                45,
                None,
            )
            .await;
            seed_completed_attempts(
                &pool,
                application_id,
                event_id,
                recovering_quiet,
                9,
                2,
                None,
            )
            .await;

            // recovering + healthy, too early whatever the traffic
            let recovering_recent = seed_subscription(&pool, application_id).await;
            set_health(&pool, recovering_recent, "recovering", 5).await;
            seed_completed_attempts(
                &pool,
                application_id,
                event_id,
                recovering_recent,
                20,
                2,
                None,
            )
            .await;

            // recovering + healthy, maximum duration reached
            let recovering_long = seed_subscription(&pool, application_id).await;
            set_health(&pool, recovering_long, "recovering", 61).await;
            seed_completed_attempts(&pool, application_id, event_id, recovering_long, 1, 2, None)
                .await;

            // Cancelled request attempts (no response) are not completed attempts
            let cancelled = seed_subscription(&pool, application_id).await;
            sqlx::query(
                "INSERT INTO webhook.request_attempt (event__id, subscription__id, application__id, failed_at) SELECT $1, $2, $3, statement_timestamp() FROM generate_series(1, 10)",
            )
            .bind(event_id)
            .bind(cancelled)
            .bind(application_id)
            .execute(&pool)
            .await
            .expect("seed cancelled request attempts");

            let report = probe(&pool).await;
            assert_eq!(
                transition_of(&report, failing),
                Some(HealthStatus::Degraded)
            );
            assert_eq!(transition_of(&report, fine), None);
            assert_eq!(transition_of(&report, hook0_failures), None);
            assert_eq!(
                transition_of(&report, degraded_recovering),
                Some(HealthStatus::Recovering)
            );
            assert_eq!(transition_of(&report, degraded_failing), None);
            assert_eq!(
                transition_of(&report, recovering_failing),
                Some(HealthStatus::Degraded)
            );
            assert_eq!(
                transition_of(&report, recovering_busy),
                Some(HealthStatus::Healthy)
            );
            assert_eq!(transition_of(&report, recovering_quiet), None);
            assert_eq!(transition_of(&report, recovering_recent), None);
            assert_eq!(
                transition_of(&report, recovering_long),
                Some(HealthStatus::Healthy)
            );
            assert_eq!(transition_of(&report, cancelled), None);
            assert_eq!(report.transitions.len(), 5);
            assert_eq!(
                report.unknown, 1,
                "only the subscription with Hook0-caused failures is unknown"
            );

            // Transitions are stored, and a second probe does not repeat them
            share_enforcement_settings(&pool, true, &[]).await.unwrap();
            assert_eq!(
                effective_health(&pool, application_id, failing).await,
                "degraded"
            );
            assert_eq!(
                effective_health(&pool, application_id, recovering_busy).await,
                "healthy"
            );
            let report = probe(&pool).await;
            assert_eq!(transition_of(&report, failing), None);

            let last_probe: Option<DateTime<Utc>> = sqlx::query_scalar(
                "SELECT value_timestamptz FROM infrastructure.state WHERE key = 'last_probe_subscription_health'",
            )
            .fetch_optional(&pool)
            .await
            .expect("fetch last probe")
            .flatten();
            assert!(last_probe.is_some());
        }

        #[sqlx::test]
        async fn probe_is_skipped_while_another_instance_runs_one(pool: PgPool) {
            let mut other_instance = pool.acquire().await.unwrap();
            sqlx::query("SELECT pg_advisory_lock(hashtext('subscription_health_probe'))")
                .execute(&mut *other_instance)
                .await
                .unwrap();

            let report = probe_subscription_health(&pool, &params(), Duration::from_secs(60))
                .await
                .unwrap();
            assert!(report.is_none());

            sqlx::query("SELECT pg_advisory_unlock(hashtext('subscription_health_probe'))")
                .execute(&mut *other_instance)
                .await
                .unwrap();
            let report = probe_subscription_health(&pool, &params(), Duration::from_secs(60))
                .await
                .unwrap();
            assert!(report.is_some());
        }
    }
}
