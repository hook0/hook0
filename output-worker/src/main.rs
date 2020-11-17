use async_std::task::sleep;
use chrono::{DateTime, Utc};
use clap::{ArgSettings::HideEnvValues, Clap};
use log::{debug, info, trace};
use reqwest::header::{HeaderMap, HeaderValue};
use sqlx::postgres::types::PgInterval;
use sqlx::{Connection, PgConnection};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::time::{Duration, Instant};
use uuid::Uuid;

#[derive(Debug, Clone, Clap)]
#[clap(author, about, version)]
struct Config {
    /// Database URL (with credentials)
    #[clap(long, env, setting = HideEnvValues)]
    database_url: String,
}

#[derive(Debug, Clone)]
#[allow(non_snake_case)]
struct RequestAttempt {
    pub request_attempt__id: Uuid,
    pub event__id: Uuid,
    pub subscription__id: Uuid,
    pub created_at: DateTime<Utc>,
    pub retry_count: i16,
    pub http_method: String,
    pub http_url: String,
    pub http_headers: serde_json::Value,
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

/// How long to wait between retries
const RETRY_TIMEOUT: Duration = Duration::from_secs(5);

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::parse();
    env_logger::init();

    debug!("Connecting to database...");
    let mut conn = PgConnection::connect(&config.database_url).await?;
    info!("Connected to database");

    info!("Begin looking for work");
    loop {
        trace!("Fetching next unprocessed request attempt...");
        let mut tx = conn.begin().await?;
        let next_attempt = sqlx::query_as!(RequestAttempt, "
            SELECT ra.request_attempt__id, ra.event__id, ra.subscription__id, ra.created_at, ra.retry_count, t_http.method AS http_method, t_http.url AS http_url, t_http.headers AS http_headers
            FROM webhook.request_attempt AS ra
            NATURAL INNER JOIN webhook.subscription AS s
            NATURAL INNER JOIN webhook.target_http AS t_http
            WHERE succeeded_at IS NULL AND failed_at IS NULL AND (delay_until IS NULL OR delay_until <= statement_timestamp())
            ORDER BY created_at ASC
            LIMIT 1
            FOR UPDATE OF ra
            SKIP LOCKED
        ")
        .fetch_optional(&mut tx)
        .await?;

        if let Some(attempt) = next_attempt {
            // Set picked_at
            debug!("Picking request attempt {}", &attempt.request_attempt__id);
            sqlx::query!(
                "UPDATE webhook.request_attempt SET picked_at = statement_timestamp() WHERE request_attempt__id = $1",
                attempt.request_attempt__id
            )
            .execute(&mut tx)
            .await?;
            info!("Picked request attempt {}", &attempt.request_attempt__id);

            // TODO: remove debug output
            dbg!(&attempt);
            dbg!(&attempt.headers());

            // Work
            let response = work(&attempt).await;
            debug!(
                "Got a response for request attempt {} in {} ms",
                &attempt.request_attempt__id,
                &response.elapsed_time_ms()
            );

            // Store response
            debug!(
                "Storing response for request attempt {}",
                &attempt.request_attempt__id
            );
            let response_id = sqlx::query!("
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

                // Creating a retry request
                // TODO: implement smarter retry using a "retry_count" column and a function that generates a time-based sequence
                let next_retry_count = attempt.retry_count + 1;
                let retry_id = sqlx::query!(
                    "
                    INSERT INTO webhook.request_attempt (event__id, subscription__id, delay_until, retry_count)
                    VALUES ($1, $2, statement_timestamp() + $3, $4)
                    RETURNING request_attempt__id
                ",
                    attempt.event__id,
                    attempt.subscription__id,
                    PgInterval::try_from(RETRY_TIMEOUT).unwrap(),
                    next_retry_count,
                )
                .fetch_one(&mut tx)
                .await?
                .request_attempt__id;

                info!(
                    "Request attempt {} failed; retry #{} created as {}",
                    &attempt.request_attempt__id, &next_retry_count, &retry_id
                );
            }
        } else {
            trace!("No unprocessed attempt found");
            sleep(POLLING_SLEEP).await;
        }

        // Commit transaction
        tx.commit().await?;
    }
}

#[derive(Debug, Clone, Copy)]
enum ResponseError {
    Dns,
    Timeout,
    Http,
}

impl std::fmt::Display for ResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Dns => write!(f, "E_DNS"),
            Self::Timeout => write!(f, "E_TIMEOUT"),
            Self::Http => write!(f, "E_HTTP"),
        }
    }
}

#[derive(Debug, Clone)]
struct Response {
    pub response_error: Option<ResponseError>,
    pub http_code: Option<u8>,
    pub headers: Option<HeaderMap>,
    pub body: Option<String>,
    pub elapsed_time: Duration,
}

impl Response {
    fn is_success(&self) -> bool {
        self.response_error.is_none()
    }

    #[allow(non_snake_case)]
    fn response_error__name(&self) -> Option<String> {
        self.response_error.map(|re| re.to_string())
    }

    fn http_code(&self) -> Option<i16> {
        self.http_code.map(|c| c.into())
    }

    fn headers(&self) -> Option<serde_json::Value> {
        use std::iter::FromIterator;

        self.headers.as_ref().and_then(|hm| {
            let iter = hm
                .iter()
                .map(|(k, v)| {
                    let key = k.to_string();
                    let value = v
                        .to_str()
                        .expect("Failed to extract a HTTP header value (there might be invisible characters)")
                        .to_owned();
                    (key, value)
                });
            let hashmap: HashMap<String, String> = HashMap::from_iter(iter);
            serde_json::to_value(&hashmap).ok()
        })
    }

    fn elapsed_time_ms(&self) -> i32 {
        use std::convert::TryInto;

        self.elapsed_time.as_millis().try_into().unwrap_or(0)
    }
}

async fn work(attempt: &RequestAttempt) -> Response {
    debug!(
        "Processing request attempt {}",
        &attempt.request_attempt__id
    );
    let start = Instant::now();

    // TODO: implement actual work here

    // Actually for now we simulate working
    sleep(Duration::from_secs(5)).await;

    // Let's simulate failing for one of my test items
    let cursed_item = Uuid::parse_str("8536a6a6-e7ec-4cea-b984-d7f377f394e4").unwrap();
    if attempt.request_attempt__id == cursed_item {
        Response {
            response_error: Some(ResponseError::Dns),
            http_code: None,
            headers: None,
            body: None,
            elapsed_time: start.elapsed(),
        }
    } else {
        let mut fake_headers = HeaderMap::new();
        fake_headers.insert("X-Test", HeaderValue::from_static("Test"));

        Response {
            response_error: None,
            http_code: Some(200),
            headers: Some(fake_headers),
            body: Some("TEST".to_owned()),
            elapsed_time: start.elapsed(),
        }
    }
}
