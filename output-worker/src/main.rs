mod work;

use chrono::{DateTime, Utc};
use clap::{crate_name, crate_version, Parser};
use log::{debug, info, trace};
use reqwest::header::HeaderMap;
use sqlx::postgres::types::PgInterval;
use sqlx::postgres::PgConnectOptions;
use sqlx::{Connection, PgConnection};
use std::cmp::min;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::str::FromStr;
use std::time::Duration;
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

    /// Worker ID or name (if empty, will generate a random UUID)
    #[clap(long, env)]
    worker_id: Option<String>,

    /// Worker version (if empty, will use version from Cargo.toml)
    #[clap(long, env)]
    worker_version: Option<String>,

    /// Maximum number of fast retries (before doing slow retries)
    #[clap(long, env, default_value = "30")]
    max_fast_retries: u32,

    /// Maximum number of slow retries (before giving up)
    #[clap(long, env, default_value = "30")]
    max_slow_retries: u32,
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

/// How long to wait when there are no unprocessed items to pick
const POLLING_SLEEP: Duration = Duration::from_secs(1);

/// How long to wait before first fast retry
const MINIMUM_FAST_RETRY_DELAY: Duration = Duration::from_secs(5);

/// How long to wait between fast retries at maximum
const MAXIMUM_FAST_RETRY_DELAY: Duration = Duration::from_secs(5 * 60);

/// How long to wait between slow retries
const SLOW_RETRY_DELAY: Duration = Duration::from_secs(60 * 60);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::parse();

    let worker_id = config
        .worker_id
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    let worker_version = config
        .worker_version
        .unwrap_or_else(|| crate_version!().to_owned());

    // Initialize app logger as well as Sentry integration
    // Return value *must* be kept in a variable or else it will be dropped and Sentry integration won't work
    let _sentry = sentry_integration::init(crate_name!(), &config.sentry_dsn);

    info!(
        "Starting {} {} [{}]",
        crate_name!(),
        &worker_version,
        &worker_id
    );

    debug!("Connecting to database...");
    let mut conn = PgConnection::connect_with(
        &PgConnectOptions::from_str(&config.database_url)?.application_name(&format!(
            "{}-{}-{}",
            crate_name!(),
            &worker_version,
            &worker_id
        )),
    )
    .await?;
    info!("Connected to database");

    info!("Begin looking for work");
    loop {
        trace!("Fetching next unprocessed request attempt...");
        let mut tx = conn.begin().await?;
        let next_attempt = sqlx::query_as!(
            RequestAttempt,
            "
                SELECT ra.request_attempt__id, ra.event__id, ra.subscription__id, ra.created_at, ra.retry_count, t_http.method AS http_method, t_http.url AS http_url, t_http.headers AS http_headers, e.payload AS payload, e.payload_content_type AS payload_content_type, s.secret
                FROM webhook.request_attempt AS ra
                INNER JOIN webhook.subscription AS s ON s.subscription__id = ra.subscription__id
                INNER JOIN webhook.target_http AS t_http ON t_http.target__id = s.target__id
                INNER JOIN event.event AS e ON e.event__id = ra.event__id
                WHERE succeeded_at IS NULL AND failed_at IS NULL AND (delay_until IS NULL OR delay_until <= statement_timestamp())
                ORDER BY created_at ASC
                LIMIT 1
                FOR UPDATE OF ra
                SKIP LOCKED
            "
        )
        .fetch_optional(&mut tx)
        .await?;

        if let Some(attempt) = next_attempt {
            // Set picked_at
            debug!("Picking request attempt {}", &attempt.request_attempt__id);
            sqlx::query!(
                "
                    UPDATE webhook.request_attempt
                    SET picked_at = statement_timestamp(), worker_id = $1, worker_version = $2
                    WHERE request_attempt__id = $3
                ",
                &worker_id,
                &worker_version,
                attempt.request_attempt__id
            )
            .execute(&mut tx)
            .await?;
            info!("Picked request attempt {}", &attempt.request_attempt__id);

            // Work
            let response = work(&attempt).await;
            debug!(
                "Got a response for request attempt {} in {} ms",
                &attempt.request_attempt__id,
                &response.elapsed_time_ms()
            );
            trace!("{:?}", &response);

            // Store response
            debug!(
                "Storing response for request attempt {}",
                &attempt.request_attempt__id
            );
            let response_id = sqlx::query!(
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
            .fetch_one(&mut tx)
            .await?
            .response__id;

            // Associate response and request attempt
            debug!(
                "Associating response {} with request attempt {}",
                &response_id, &attempt.request_attempt__id
            );
            #[allow(clippy::suspicious_else_formatting)] // Clippy false positive
            sqlx::query!(
                "UPDATE webhook.request_attempt SET response__id = $1 WHERE request_attempt__id = $2",
                response_id, attempt.request_attempt__id
            )
            .execute(&mut tx)
            .await?;

            if response.is_success() {
                // Mark attempt as completed
                debug!(
                    "Completing request attempt {}",
                    &attempt.request_attempt__id
                );
                sqlx::query!(
                    "UPDATE webhook.request_attempt SET succeeded_at = statement_timestamp() WHERE request_attempt__id = $1",
                    attempt.request_attempt__id
                )
                .execute(&mut tx)
                .await?;

                info!(
                    "Request attempt {} was completed sucessfully",
                    &attempt.request_attempt__id
                );
            } else {
                // Mark attempt as failed
                debug!("Failing request attempt {}", &attempt.request_attempt__id);
                sqlx::query!(
                    "UPDATE webhook.request_attempt SET failed_at = statement_timestamp() WHERE request_attempt__id = $1",
                    attempt.request_attempt__id
                )
                .execute(&mut tx)
                .await?;

                // Creating a retry request or giving up
                if let Some(retry_in) = compute_next_retry(
                    config.max_fast_retries,
                    config.max_slow_retries,
                    attempt.retry_count,
                ) {
                    let next_retry_count = attempt.retry_count + 1;
                    let retry_id = sqlx::query!(
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
                    .fetch_one(&mut tx)
                    .await?
                    .request_attempt__id;

                    info!(
                        "Request attempt {} failed; retry #{} created as {} to be picked in {}s",
                        &attempt.request_attempt__id,
                        &next_retry_count,
                        &retry_id,
                        &retry_in.as_secs()
                    );
                } else {
                    info!(
                        "Request attempt {} failed after {} attempts; giving up",
                        &attempt.request_attempt__id, &attempt.retry_count,
                    );
                }
            }
        } else {
            trace!("No unprocessed attempt found");
            sleep(POLLING_SLEEP).await;
        }

        // Commit transaction
        tx.commit().await?;
    }
}

fn compute_next_retry(
    max_fast_retries: u32,
    max_slow_retries: u32,
    retry_count: i16,
) -> Option<Duration> {
    u32::try_from(retry_count).ok().and_then(|count| {
        if count <= max_fast_retries {
            Some(min(
                MINIMUM_FAST_RETRY_DELAY * count,
                MAXIMUM_FAST_RETRY_DELAY,
            ))
        } else if count <= max_fast_retries + max_slow_retries {
            Some(SLOW_RETRY_DELAY)
        } else {
            None
        }
    })
}
