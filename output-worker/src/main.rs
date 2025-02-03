mod monitoring;
mod work;

use chrono::{DateTime, Utc};
use clap::{crate_name, crate_version, Parser};
use log::{debug, error, info, trace, warn};
use reqwest::header::{HeaderMap, HeaderName};
use reqwest::Url;
use sqlx::postgres::types::PgInterval;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{query, query_as, PgConnection, PgPool};
use std::cmp::min;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::str::FromStr;
use std::time::Duration;
use strum::VariantNames;
use tokio::sync::mpsc::{channel, Sender};
use tokio::task::JoinSet;
use tokio::time::sleep;
use uuid::Uuid;

use work::*;

#[derive(Debug, Clone, Parser)]
#[clap(author, about, version)]
struct Config {
    /// Optional Sentry DSN for error reporting
    #[clap(long, env)]
    sentry_dsn: Option<String>,

    /// Database URL (with credentials)
    #[clap(long, env, hide_env_values = true)]
    database_url: String,

    /// Worker name (as defined in the infrastructure.worker table)
    #[clap(long, env)]
    worker_name: String,

    /// Worker version (if empty, will use version from Cargo.toml)
    #[clap(long, env)]
    worker_version: Option<String>,

    /// Number of request attempts to handle concurrently
    #[clap(long, env, default_value = "1", value_parser=clap::value_parser!(u8).range(1..=100))]
    concurrent: u8,

    /// Maximum number of fast retries (before doing slow retries)
    #[clap(long, env, default_value = "30")]
    max_fast_retries: u32,

    /// Maximum number of slow retries (before giving up)
    #[clap(long, env, default_value = "30")]
    max_slow_retries: u32,

    /// Heartbeat URL that should be called regularly
    #[clap(long, env)]
    monitoring_heartbeat_url: Option<Url>,

    /// Minimal duration (in second) to wait between sending two heartbeats
    #[clap(long, env, default_value = "60")]
    monitoring_heartbeat_min_period_in_s: u64,

    /// If set to false (default), webhooks that target IPs that are not globally reachable (like "127.0.0.1" for example) will fail
    #[clap(long, env, default_value = "false")]
    disable_target_ip_check: bool,

    /// Timeout for establishing a connection to the target (if exceeded, request attempt will fail)
    #[clap(long, env, value_parser = humantime::parse_duration, default_value = "5s")]
    connect_timeout: Duration,

    /// Timeout for obtaining a HTTP response from the target, including connect phase (if exceeded, request attempt will fail)
    #[clap(long, env, value_parser = humantime::parse_duration, default_value = "15s")]
    timeout: Duration,

    /// Name of the header containing webhook's signature
    #[clap(long, env, default_value = "X-Hook0-Signature")]
    signature_header_name: HeaderName,
}

#[derive(Debug, Clone, Copy)]
enum WorkerType {
    Public { worker_id: Option<Uuid> },
    Private { worker_id: Uuid },
}

impl std::fmt::Display for WorkerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Public {
                worker_id: Some(worker_id),
            } => write!(f, "public (ID={worker_id})"),
            Self::Public { worker_id: None } => write!(f, "public (anonymous)"),
            Self::Private { worker_id } => write!(f, "private (ID={worker_id})"),
        }
    }
}

#[derive(Debug, Clone)]
#[allow(non_snake_case)]
pub struct RequestAttempt {
    pub request_attempt__id: Uuid,
    pub event__id: Uuid,
    pub subscription__id: Uuid,
    pub created_at: DateTime<Utc>,
    pub retry_count: i16,
    pub http_method: String,
    pub http_url: String,
    pub http_headers: serde_json::Value,
    pub event_type__name: String,
    pub payload: Vec<u8>,
    pub payload_content_type: String,
    pub secret: Uuid,
}

impl RequestAttempt {
    /// Parse headers of HTTP target from JSON and prepare them to be fed to reqwest
    fn headers(&self) -> anyhow::Result<HeaderMap> {
        let hashmap = serde_json::from_value::<HashMap<String, String>>(self.http_headers.clone())?;
        let headermap = HeaderMap::try_from(&hashmap)?;
        Ok(headermap)
    }
}

/// Minimum duration to wait when there are no unprocessed items to pick
const MIN_POLLING_SLEEP: Duration = Duration::from_secs(1);

/// Maximum duration to wait when there are no unprocessed items to pick
const MAX_POLLING_SLEEP: Duration = Duration::from_secs(10);

/// How long to wait before first fast retry
const MINIMUM_FAST_RETRY_DELAY: Duration = Duration::from_secs(5);

/// How long to wait between fast retries at maximum
const MAXIMUM_FAST_RETRY_DELAY: Duration = Duration::from_secs(5 * 60);

/// How long to wait between slow retries
const SLOW_RETRY_DELAY: Duration = Duration::from_secs(60 * 60);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::parse();

    let worker_name = config.worker_name.to_owned();
    let worker_version = config
        .worker_version
        .to_owned()
        .unwrap_or_else(|| crate_version!().to_owned());

    // Initialize app logger as well as Sentry integration
    // Return value *must* be kept in a variable or else it will be dropped and Sentry integration won't work
    let _sentry = hook0_sentry_integration::init(crate_name!(), &config.sentry_dsn, &None);

    info!(
        "Starting {} {worker_version} [{worker_name}]",
        crate_name!(),
    );
    debug!(
        "Webhook connect timeout is set to {:?}",
        config.connect_timeout
    );
    debug!("Webhook total timeout is set to {:?}", config.timeout);

    debug!("Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(config.concurrent.into())
        .connect_with(
            PgConnectOptions::from_str(&config.database_url)?
                .application_name(&format!("{}-{worker_version}-{worker_name}", crate_name!(),)),
        )
        .await?;
    info!("Connected to database");

    let worker_type = get_worker_type(&worker_name, &pool).await?;

    if config.disable_target_ip_check {
        warn!("Webhook's target IP check is disabled: this allows the worker to send HTTP requests that target local IP addresses (for example: loopback, LAN, ...); THIS MAY BE A SECURITY ISSUE IN PRODUCTION")
    }

    info!("Upserting response error names");
    let mut tx = pool.begin().await?;
    for error_name in ResponseError::VARIANTS {
        query!(
            "
                INSERT INTO webhook.response_error (response_error__name)
                VALUES ($1)
                ON CONFLICT (response_error__name)
                DO NOTHING
            ",
            error_name,
        )
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    info!("Done upserting response error names");

    let mut tasks = JoinSet::new();
    let expected_tasks_len = usize::from(config.concurrent)
        + if config.monitoring_heartbeat_url.is_some() {
            1
        } else {
            0
        };

    let monitoring_heartbeat_url = config.monitoring_heartbeat_url.to_owned();
    let heartbeat_tx = if let Some(url) = monitoring_heartbeat_url {
        let heartbeat_min_period = Duration::from_secs(config.monitoring_heartbeat_min_period_in_s);
        let (tx, rx) = channel(10);
        let wn = worker_name.to_owned();
        let wv = worker_version.to_owned();
        tasks.spawn(async move {
            let mut rx = rx;
            loop {
                let t = monitoring::heartbeat_sender(heartbeat_min_period, &url, &mut rx, &wn, &wv)
                    .await;
                if let Err(ref e) = t {
                    error!("Monitoring task crashed: {e}");
                }
                sleep(Duration::from_secs(1)).await;
                info!("Restarting monitoring task...");
            }
        });
        Some(tx)
    } else {
        None
    };

    for unit_id in 0..config.concurrent {
        let p = pool.clone();
        let wn = worker_name.to_owned();
        let wv = worker_version.to_owned();
        let tx = heartbeat_tx.to_owned();
        let cfg = config.to_owned();
        tasks.spawn(async move {
            // Start units progressively
            sleep(Duration::from_millis(u64::from(unit_id) * 100)).await;

            loop {
                let t = look_for_work(&cfg, unit_id, &p, &wn, &wv, &worker_type, tx.clone()).await;
                if let Err(ref e) = t {
                    error!("Unit {unit_id} crashed: {e}");
                }
                sleep(Duration::from_secs(1)).await;
                info!("Restarting unit {unit_id}...");
            }
        });
    }

    while tasks.len() >= expected_tasks_len && (tasks.join_next().await).is_some() {}

    Err(anyhow::anyhow!("Fatal error"))
}

#[allow(clippy::too_many_arguments)]
async fn look_for_work(
    config: &Config,
    unit_id: u8,
    pool: &PgPool,
    worker_name: &str,
    worker_version: &str,
    worker_type: &WorkerType,
    heartbeat_tx: Option<Sender<u8>>,
) -> anyhow::Result<()> {
    info!("[unit={unit_id}] Begin looking for work");
    loop {
        trace!("[unit={unit_id}] Fetching next unprocessed request attempt...");
        let mut tx = pool.begin().await?;

        let next_attempt = match worker_type {
            WorkerType::Public { worker_id } => {
                // Only consider request attempts where associated subscription have no dedicated worker specified
                query_as!(
                    RequestAttempt,
                    "
                        SELECT ra.request_attempt__id, ra.event__id, ra.subscription__id, ra.created_at, ra.retry_count, t_http.method AS http_method, t_http.url AS http_url, t_http.headers AS http_headers, e.event_type__name, e.payload AS payload, e.payload_content_type AS payload_content_type, s.secret
                        FROM webhook.request_attempt AS ra
                        INNER JOIN webhook.subscription AS s ON s.subscription__id = ra.subscription__id
                        LEFT JOIN webhook.subscription__worker AS sw ON sw.subscription__id = s.subscription__id
                        INNER JOIN event.application AS a ON a.application__id = s.application__id AND a.deleted_at IS NULL
                        INNER JOIN iam.organization AS o ON o.organization__id = a.organization__id
                        LEFT JOIN iam.organization__worker AS ow ON ow.organization__id = o.organization__id AND ow.default = true
                        INNER JOIN webhook.target_http AS t_http ON t_http.target__id = s.target__id
                        INNER JOIN event.event AS e ON e.event__id = ra.event__id
                        WHERE ra.succeeded_at IS NULL AND ra.failed_at IS NULL AND (ra.delay_until IS NULL OR ra.delay_until <= statement_timestamp()) AND (COALESCE(sw.worker__id, ow.worker__id) IS NULL OR COALESCE(sw.worker__id, ow.worker__id) = $1)
                        ORDER BY ra.created_at ASC
                        LIMIT 1
                        FOR UPDATE OF ra
                        SKIP LOCKED
                    ",
                    worker_id.to_owned(),
                )
                .fetch_optional(&mut *tx)
                .await?
            }
            WorkerType::Private { worker_id } => {
                // Only consider request attempts where associated subscription have at least the currect worker specified as dedicated worker
                query_as!(
                    RequestAttempt,
                    "
                        SELECT ra.request_attempt__id, ra.event__id, ra.subscription__id, ra.created_at, ra.retry_count, t_http.method AS http_method, t_http.url AS http_url, t_http.headers AS http_headers, e.event_type__name, e.payload AS payload, e.payload_content_type AS payload_content_type, s.secret
                        FROM webhook.request_attempt AS ra
                        INNER JOIN webhook.subscription AS s ON s.subscription__id = ra.subscription__id
                        LEFT JOIN webhook.subscription__worker AS sw ON sw.subscription__id = s.subscription__id
                        INNER JOIN event.application AS a ON a.application__id = s.application__id AND a.deleted_at IS NULL
                        INNER JOIN iam.organization AS o ON o.organization__id = a.organization__id
                        LEFT JOIN iam.organization__worker AS ow ON ow.organization__id = o.organization__id AND ow.default = true
                        INNER JOIN webhook.target_http AS t_http ON t_http.target__id = s.target__id
                        INNER JOIN event.event AS e ON e.event__id = ra.event__id
                        WHERE ra.succeeded_at IS NULL AND ra.failed_at IS NULL AND (ra.delay_until IS NULL OR ra.delay_until <= statement_timestamp()) AND (COALESCE(sw.worker__id, ow.worker__id) = $1)
                        ORDER BY ra.created_at ASC
                        LIMIT 1
                        FOR UPDATE OF ra
                        SKIP LOCKED
                    ",
                    &worker_id,
                )
                .fetch_optional(&mut *tx)
                .await?
            }
        };

        if let Some(attempt) = next_attempt {
            // Set picked_at
            debug!(
                "[unit={unit_id}] Picking request attempt {}",
                &attempt.request_attempt__id
            );
            query!(
                "
                    UPDATE webhook.request_attempt
                    SET picked_at = statement_timestamp(), worker_name = $1, worker_version = $2
                    WHERE request_attempt__id = $3
                ",
                &worker_name,
                &worker_version,
                attempt.request_attempt__id
            )
            .execute(&mut *tx)
            .await?;
            info!(
                "[unit={unit_id}] Picked request attempt {}",
                &attempt.request_attempt__id
            );

            // Work
            let response = work(config, &attempt).await;
            debug!(
                "[unit={unit_id}] Got a response for request attempt {} in {} ms",
                &attempt.request_attempt__id,
                &response.elapsed_time_ms()
            );
            trace!("[unit={unit_id}] {:?}", &response);

            // Store response
            debug!(
                "[unit={unit_id}] Storing response for request attempt {}",
                &attempt.request_attempt__id
            );
            let response_id = query!(
                "
                    INSERT INTO webhook.response (response_error__name, http_code, headers, body, elapsed_time_ms)
                    VALUES ($1, $2, $3, $4, $5)
                    RETURNING response__id
                ",
                response.response_error__name(),
                response.http_code(),
                response.headers(),
                response.body,
                response.elapsed_time_ms(),
            )
            .fetch_one(&mut *tx)
            .await?
            .response__id;

            // Associate response and request attempt
            debug!(
                "[unit={unit_id}] Associating response {} with request attempt {}",
                &response_id, &attempt.request_attempt__id
            );
            #[allow(clippy::suspicious_else_formatting)] // Clippy false positive
            query!(
                "UPDATE webhook.request_attempt SET response__id = $1 WHERE request_attempt__id = $2",
                response_id, attempt.request_attempt__id
            )
            .execute(&mut *tx)
            .await?;

            if response.is_success() {
                // Mark attempt as completed
                debug!(
                    "[unit={unit_id}] Completing request attempt {}",
                    &attempt.request_attempt__id
                );
                query!(
                    "UPDATE webhook.request_attempt SET succeeded_at = statement_timestamp() WHERE request_attempt__id = $1",
                    attempt.request_attempt__id
                )
                .execute(&mut *tx)
                .await?;

                info!(
                    "[unit={unit_id}] Request attempt {} was completed sucessfully",
                    &attempt.request_attempt__id
                );
            } else {
                // Mark attempt as failed
                debug!(
                    "[unit={unit_id}] Failing request attempt {}",
                    &attempt.request_attempt__id
                );
                query!(
                    "UPDATE webhook.request_attempt SET failed_at = statement_timestamp() WHERE request_attempt__id = $1",
                    attempt.request_attempt__id
                )
                .execute(&mut *tx)
                .await?;

                // Creating a retry request or giving up
                if let Some(retry_in) = compute_next_retry(
                    &mut tx,
                    &attempt.subscription__id,
                    config.max_fast_retries,
                    config.max_slow_retries,
                    attempt.retry_count,
                )
                .await?
                {
                    let next_retry_count = attempt.retry_count + 1;
                    let retry_id = query!(
                        "
                            INSERT INTO webhook.request_attempt (event__id, subscription__id, delay_until, retry_count)
                            VALUES ($1, $2, statement_timestamp() + $3, $4)
                            RETURNING request_attempt__id
                        ",
                        attempt.event__id,
                        attempt.subscription__id,
                        PgInterval::try_from(retry_in).unwrap(),
                        next_retry_count,
                    )
                    .fetch_one(&mut *tx)
                    .await?
                    .request_attempt__id;

                    info!(
                        "[unit={unit_id}] Request attempt {} failed; retry #{} created as {} to be picked in {}s",
                        &attempt.request_attempt__id,
                        &next_retry_count,
                        &retry_id,
                        &retry_in.as_secs()
                    );
                } else {
                    info!(
                        "[unit={unit_id}] Request attempt {} failed after {} attempts; giving up",
                        &attempt.request_attempt__id, &attempt.retry_count,
                    );
                }
            }

            // Commit transaction
            tx.commit().await?;
        } else {
            trace!("[unit={unit_id}] No unprocessed attempt found");

            // Commit transaction
            tx.commit().await?;

            wait_because_no_work(unit_id).await;
        }

        // Send monitoring heartbeat if necessary
        if let Some(ref tx) = heartbeat_tx {
            tx.send(unit_id).await?;
        }
    }
}

async fn wait_because_no_work(unit_id: u8) {
    // In order to reduce load on the database when there is no work to do, but simultaneously keep a low latency when some work becomes available,
    // we wait a variable duration between checks:
    // - for unit 0, we wait for a short duration, so that new work gets picked up fast
    // - for units 1 and 2, we wait for a medium duration
    // - for units > 3, we wait for a long duration, to avoid unnecessary stress on the database
    // Note: units do not wait after finishing a task (they keep going as fast as possible), they wait only if there is no more work to do
    let sleep_duration = match unit_id {
        0 => MIN_POLLING_SLEEP,
        1 | 2 => (MIN_POLLING_SLEEP + MAX_POLLING_SLEEP) / 2,
        _ => MAX_POLLING_SLEEP,
    };
    sleep(sleep_duration).await;
}

async fn get_worker_type(worker_name: &str, conn: &PgPool) -> Result<WorkerType, sqlx::Error> {
    #[allow(non_snake_case)]
    struct Worker {
        worker__id: Uuid,
        public: bool,
    }
    let worker = query_as!(
        Worker,
        "
            SELECT worker__id, public
            FROM infrastructure.worker
            WHERE name = $1
        ",
        worker_name
    )
    .fetch_optional(conn)
    .await?;
    if let Some(w) = worker {
        let worker_type = if w.public {
            WorkerType::Public {
                worker_id: Some(w.worker__id),
            }
        } else {
            WorkerType::Private {
                worker_id: w.worker__id,
            }
        };
        info!("Worker is running as '{worker_name}' which is {worker_type}",);
        Ok(worker_type)
    } else {
        warn!("Worker name '{worker_name}' was not found in database; worker is running as a public worker");
        Ok(WorkerType::Public { worker_id: None })
    }
}

async fn compute_next_retry(
    conn: &mut PgConnection,
    subscription_id: &Uuid,
    max_fast_retries: u32,
    max_slow_retries: u32,
    retry_count: i16,
) -> Result<Option<Duration>, sqlx::Error> {
    let sub = query!(
        "
            SELECT true AS whatever
            FROM webhook.subscription
            WHERE subscription__id = $1 AND deleted_at IS NULL AND is_enabled
        ",
        subscription_id
    )
    .fetch_optional(conn)
    .await?;

    if sub.is_some() {
        Ok(compute_next_retry_duration(
            max_fast_retries,
            max_slow_retries,
            retry_count,
        ))
    } else {
        // If the subscription was disabled or soft-deleted, we do not want to schedule a next attempt
        Ok(None)
    }
}

fn compute_next_retry_duration(
    max_fast_retries: u32,
    max_slow_retries: u32,
    retry_count: i16,
) -> Option<Duration> {
    u32::try_from(retry_count).ok().and_then(|count| {
        if count < max_fast_retries {
            Some(min(
                MINIMUM_FAST_RETRY_DELAY * count,
                MAXIMUM_FAST_RETRY_DELAY,
            ))
        } else if count < max_fast_retries + max_slow_retries {
            Some(SLOW_RETRY_DELAY)
        } else {
            None
        }
    })
}
