mod monitoring;
mod pg;
mod pulsar;
mod work;

use ::pulsar::{Authentication, Pulsar, TokioExecutor};
use anyhow::bail;
use aws_sdk_s3::Client;
use aws_sdk_s3::config::{AppName, Credentials, Region};
use chrono::{DateTime, Utc};
use clap::{ArgGroup, Parser, ValueEnum, crate_name, crate_version};
use log::{debug, error, info, warn};
use reqwest::Url;
use reqwest::header::HeaderName;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{PgConnection, PgPool, query, query_as};
use std::cmp::min;
use std::convert::TryFrom;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use strum::{EnumString, VariantNames};
use tokio::select;
use tokio::signal::unix::{SignalKind, signal};
use tokio::sync::mpsc::channel;
use tokio::task::JoinSet;
use tokio::time::sleep;
use tokio_util::task::TaskTracker;
use uuid::Uuid;

use hook0_protobuf::RequestAttempt;
use work::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum SignatureVersion {
    V0,
    V1,
}

#[derive(Debug, Clone, Parser)]
#[clap(author, about, version)]
#[clap(group(
    ArgGroup::new("pulsar")
        .multiple(true)
        .requires_all(&["pulsar_binary_url", "pulsar_token", "pulsar_tenant", "pulsar_namespace"]),
))]
struct Config {
    /// Optional Sentry DSN for error reporting
    #[clap(long, env)]
    sentry_dsn: Option<String>,

    /// Database URL (with credentials)
    #[clap(long, env, hide_env_values = true)]
    database_url: String,

    /// Maximum number of connections to database (for a worker with pg queue type, it should be equal to CONCURRENCY)
    #[clap(long, env, default_value = "5")]
    max_db_connections: u32,

    /// Pulsar binary URL
    #[clap(long, env, group = "pulsar")]
    pulsar_binary_url: Option<Url>,

    /// Pulsar token
    #[clap(long, env, hide_env_values = true, group = "pulsar")]
    pulsar_token: Option<String>,

    /// Pulsar tenant
    #[clap(long, env, group = "pulsar")]
    pulsar_tenant: Option<String>,

    /// Pulsar namespace
    #[clap(long, env, group = "pulsar")]
    pulsar_namespace: Option<String>,

    /// Host of the S3-like object storage (without https://)
    #[clap(long, env)]
    object_storage_host: Option<String>,

    /// Force endpoint scheme to be HTTP (by default it is HTTPS)
    #[clap(long, env, default_value_t = false)]
    object_storage_force_http_scheme: bool,

    /// Key ID of the S3-like object storage
    #[clap(long, env)]
    object_storage_key_id: Option<String>,

    /// Key secret of the S3-like object storage
    #[clap(long, env, hide_env_values = true)]
    object_storage_key_secret: Option<String>,

    /// Bucket name of the S3-like object storage
    #[clap(long, env)]
    object_storage_bucket_name: Option<String>,

    /// Worker name (as defined in the infrastructure.worker table)
    #[clap(long, env)]
    worker_name: String,

    /// Worker version (if empty, will use version from Cargo.toml)
    #[clap(long, env)]
    worker_version: Option<String>,

    /// Number of request attempts to handle concurrently (for a worker with pg queue type, this means opening 1 connection to PostgreSQL per concurrent unit)
    #[clap(long, env, default_value = "1", value_parser = clap::value_parser!(u16).range(1..))]
    concurrent: u16,

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

    /// A comma-separated list of enabled signature versions
    #[clap(long, env, default_value = "v1", value_delimiter = ',')]
    enabled_signature_versions: Vec<SignatureVersion>,

    /// If true, will load waiting request attempts (that can be picked by this worker) from DB into Pulsar before starting working; this is usefull when migrating ta a Pulsar worker; has no effect if worker has not a pulsar queue type
    #[clap(long, env, default_value_t = false)]
    load_waiting_request_attempt_into_pulsar: bool,
}

#[derive(Debug, Clone)]
struct Worker {
    name: String,
    scope: WorkerScope,
    queue_type: WorkerQueueType,
}

#[derive(Debug, Clone, Copy)]
enum WorkerScope {
    Public { worker_id: Option<Uuid> },
    Private { worker_id: Uuid },
}

impl WorkerScope {
    fn worker_id(&self) -> Option<Uuid> {
        match self {
            Self::Public {
                worker_id: Some(id),
            } => Some(*id),
            Self::Private { worker_id } => Some(*worker_id),
            Self::Public { worker_id: None } => None,
        }
    }
}

impl std::fmt::Display for WorkerScope {
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

#[derive(Debug, Clone, Copy, EnumString, strum::Display)]
#[strum(serialize_all = "snake_case")]
enum WorkerQueueType {
    Pg,
    Pulsar,
}

/// How long to wait before first fast retry
const MINIMUM_FAST_RETRY_DELAY: Duration = Duration::from_secs(5);

/// How long to wait between fast retries at maximum
const MAXIMUM_FAST_RETRY_DELAY: Duration = Duration::from_secs(5 * 60);

/// How long to wait between slow retries
const SLOW_RETRY_DELAY: Duration = Duration::from_secs(60 * 60);

#[derive(Clone)]
struct PulsarConfig {
    pulsar: Pulsar<TokioExecutor>,
    tenant: String,
    namespace: String,
}

#[derive(Debug, Clone)]
struct ObjectStorageConfig {
    client: Client,
    bucket: String,
}

#[derive(Debug, Clone)]
pub struct RequestAttemptWithOptionalPayload {
    pub application_id: Uuid,
    pub request_attempt_id: Uuid,
    pub event_id: Uuid,
    pub event_received_at: DateTime<Utc>,
    pub subscription_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub retry_count: i16,
    pub http_method: String,
    pub http_url: String,
    pub http_headers: serde_json::Value,
    pub event_type_name: String,
    pub payload: Option<Vec<u8>>,
    pub payload_content_type: String,
    pub secret: Uuid,
}

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
        .max_connections(config.max_db_connections)
        .connect_with(
            PgConnectOptions::from_str(&config.database_url)?
                .application_name(&format!("{}-{worker_version}-{worker_name}", crate_name!(),)),
        )
        .await?;
    info!("Connected to database");

    let worker = get_worker(worker_name, &pool).await?;

    if matches!(worker.queue_type, WorkerQueueType::Pg)
        && u32::from(config.concurrent) > config.max_db_connections
    {
        warn!(
            "Worker has a pg queue type with CONCURRENT={}, but MAX_DB_CONNECTIONS is smaller ({}); worker with pg queue should have MAX_DB_CONNECTIONS=CONCURRENT",
            config.concurrent, config.max_db_connections
        );
    }

    let pulsar_config = if matches!(worker.queue_type, WorkerQueueType::Pulsar) {
        if let (
            Some(pulsar_binary_url),
            Some(pulsar_token),
            Some(pulsar_tenant),
            Some(pulsar_namespace),
        ) = (
            &config.pulsar_binary_url,
            &config.pulsar_token,
            &config.pulsar_tenant,
            &config.pulsar_namespace,
        ) {
            rustls::crypto::ring::default_provider()
                .install_default()
                .unwrap();

            Some(Arc::new(PulsarConfig {
                pulsar: Pulsar::builder(pulsar_binary_url.to_owned(), TokioExecutor)
                    .with_auth(Authentication {
                        name: "token".to_owned(),
                        data: pulsar_token.to_owned().into_bytes(),
                    })
                    .build()
                    .await?,
                tenant: pulsar_tenant.to_owned(),
                namespace: pulsar_namespace.to_owned(),
            }))
        } else {
            bail!("This worker has a 'pulsar' queue type, but Pulsar's configuration is missing")
        }
    } else {
        None
    };

    let object_storage_config = if let (
        Some(object_storage_host),
        Some(object_storage_key_id),
        Some(object_storage_key_secret),
        Some(object_storage_bucket_name),
    ) = (
        &config.object_storage_host,
        &config.object_storage_key_id,
        &config.object_storage_key_secret,
        &config.object_storage_bucket_name,
    ) {
        let app_name = AppName::new(crate_name!()).unwrap();
        let credentials = Credentials::new(
            object_storage_key_id,
            object_storage_key_secret,
            None,
            None,
            crate_name!(),
        );
        let region = Region::from_static("none");
        let s3_config = aws_sdk_s3::Config::builder()
            .region(region)
            .credentials_provider(credentials)
            .app_name(app_name)
            .endpoint_url(format!(
                "{}://{}",
                if config.object_storage_force_http_scheme {
                    "http"
                } else {
                    "https"
                },
                object_storage_host
            ))
            .force_path_style(true)
            .build();
        let client = Client::from_conf(s3_config);
        if let Err(e) = client
            .head_bucket()
            .bucket(object_storage_bucket_name)
            .send()
            .await
        {
            error!("Could not connect to object storage: {e}");
            warn!("Continuing without object storage support (restart to try again)");
            None
        } else {
            info!("Object storage support is enabled");
            Some(ObjectStorageConfig {
                client,
                bucket: object_storage_bucket_name.to_owned(),
            })
        }
    } else {
        None
    };

    if config.disable_target_ip_check {
        warn!(
            "Webhook's target IP check is disabled: this allows the worker to send HTTP requests that target local IP addresses (for example: loopback, LAN, ...); THIS MAY BE A SECURITY ISSUE IN PRODUCTION"
        )
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

    // Create a JoinSet to make it easy to wait multiple tasks at once (and crash the whole program if one of the tasks crash)
    let mut tasks = JoinSet::new();

    // Create a TaskTracker to be able to track inflight webhook tasks so it is possible to gracefully shutdown when required
    let task_tracker = TaskTracker::new();

    // This task waits for a soft termination signal
    let task_tracker_signal = task_tracker.clone();
    tasks.spawn(async move {
        let mut hangup = signal(SignalKind::hangup()).expect("Could not listen to SIGHUP");
        let mut interrupt = signal(SignalKind::interrupt()).expect("Could not listen to SIGINT");
        let mut terminate = signal(SignalKind::terminate()).expect("Could not listen to SIGTERM");

        select! {
            Some(_) = hangup.recv() => shutdown(&task_tracker_signal),
            Some(_) = interrupt.recv() => shutdown(&task_tracker_signal),
            Some(_) = terminate.recv() => shutdown(&task_tracker_signal),
        }

        fn shutdown(task_tracker: &TaskTracker) {
            info!("Finishing work before terminating...");
            task_tracker.close();
        }
    });

    // This tasks displays the number of inflight webhook tasks when graceful shutdown has been asked
    let task_tracker_indicator = task_tracker.clone();
    tasks.spawn(async move {
        loop {
            if task_tracker_indicator.is_closed() {
                if task_tracker_indicator.is_empty() {
                    break;
                } else {
                    info!(
                        "Waiting for {} task(s) to finish...",
                        task_tracker_indicator.len()
                    );
                }
            }
            sleep(Duration::from_secs(1)).await
        }
    });

    // This task is used to send HTTP hearbeat requests to monitoring (optional)
    let monitoring_heartbeat_url = config.monitoring_heartbeat_url.to_owned();
    let heartbeat_tx = if let Some(url) = monitoring_heartbeat_url {
        let task_tracker_monitoring_heartbeat = task_tracker.clone();
        let heartbeat_min_period = Duration::from_secs(config.monitoring_heartbeat_min_period_in_s);
        let (tx, rx) = channel(10);
        let wn = worker.name.to_owned();
        let wv = worker_version.to_owned();
        tasks.spawn(async move {
            let mut rx = rx;
            loop {
                let t = monitoring::heartbeat_sender(heartbeat_min_period, &url, &mut rx, &wn, &wv)
                    .await;

                if task_tracker_monitoring_heartbeat.is_closed() {
                    break;
                }

                if let Err(ref e) = t {
                    error!("Monitoring task crashed: {e}");
                }
                sleep(Duration::from_secs(1)).await;
                info!("Restarting monitoring task...");
            }
            debug!("Monitoring task terminated");
        });
        Some(tx)
    } else {
        None
    };

    // This task is the main control tasks around webhooks sending
    let task_tracker_main = task_tracker.clone();
    if let Some(ref pulsar) = pulsar_config {
        // This worker has a 'pulsar' queue type

        if let Some(worker_id) = worker.scope.worker_id() {
            let c = Arc::new(config);
            let po = pool.clone();
            let wid = Arc::new(worker_id);
            let wn = Arc::new(worker.name.to_owned());
            let wv = Arc::new(worker_version.to_owned());
            let pu = pulsar.clone();
            let os = object_storage_config.clone();
            tasks.spawn(async move {
                if c.load_waiting_request_attempt_into_pulsar {
                    info!("Loading waiting request attempts from database into Pulsar...");
                    match pulsar::load_waiting_request_attempts_from_db(&po, &wid, &pu, &os).await {
                        Ok(c) => info!("Loaded {c} waiting request attempts from database into Pulsar"),
                        Err(e) => error!("Error while loading waiting request attempts from database into Pulsar: {e}"),
                    }
                }

                loop {
                    let result = pulsar::look_for_work(
                        &c,
                        &po,
                        &wid,
                        &wn,
                        &wv,
                        &pu,
                        heartbeat_tx.clone(),
                        &task_tracker_main,
                    )
                    .await;
                    if let Err(ref e) = result {
                        error!("Pulsar consumer task failed: {e}");
                    }

                    if task_tracker_main.is_closed() {
                        break;
                    }

                    sleep(Duration::from_secs(1)).await;
                    info!("Restarting Pulsar consumer task...");
                }
                debug!("Main worker task terminated");
            });
        }
    } else {
        // This worker has a 'pg' queue type

        for unit_id in 0..config.concurrent {
            let p = pool.clone();
            let os = object_storage_config.clone();
            let w = worker.to_owned();
            let wv = worker_version.to_owned();
            let tx = heartbeat_tx.to_owned();
            let cfg = config.to_owned();
            let tt = task_tracker_main.clone();
            task_tracker_main.spawn(async move {
                // Start units progressively
                sleep(Duration::from_millis(u64::from(unit_id) * 100)).await;

                loop {
                    let t =
                        pg::look_for_work(&cfg, unit_id, &p, &os, &w, &wv, tx.clone(), &tt).await;
                    if let Err(ref e) = t {
                        error!("Unit {unit_id} crashed: {e}");
                    }

                    if tt.is_closed() {
                        break;
                    }

                    sleep(Duration::from_secs(1)).await;
                    info!("Restarting unit {unit_id}...");
                }

                debug!("Main worker task terminated");
            });
        }

        // Ensure that we do not keep a heartbeat TX so that the heartbeat task will crash if there are no more PG worker tasks
        // This allows to gracefully terminate the program
        drop(heartbeat_tx);
    }

    // We wait for all tasks to terminate or one of them to return an error
    tasks.join_all().await;

    if task_tracker.is_closed() {
        info!("Worker gracefully terminated");
        Ok(())
    } else {
        Err(anyhow::anyhow!("Fatal error"))
    }
}

async fn get_worker(name: String, conn: &PgPool) -> anyhow::Result<Worker> {
    #[allow(non_snake_case)]
    struct RawWorker {
        worker__id: Uuid,
        public: bool,
        queue_type: String,
    }
    let worker = query_as!(
        RawWorker,
        "
            SELECT worker__id, public, queue_type
            FROM infrastructure.worker
            WHERE name = $1
        ",
        &name,
    )
    .fetch_optional(conn)
    .await?;

    if let Some(w) = worker {
        let scope = if w.public {
            WorkerScope::Public {
                worker_id: Some(w.worker__id),
            }
        } else {
            WorkerScope::Private {
                worker_id: w.worker__id,
            }
        };
        let queue_type = WorkerQueueType::from_str(&w.queue_type)?;

        info!("Worker of type {queue_type} is running as '{name}' which is {scope}");
        Ok(Worker {
            name,
            scope,
            queue_type,
        })
    } else {
        warn!(
            "Worker name '{name}' was not found in database; worker is running as a public pg worker"
        );
        Ok(Worker {
            name,
            scope: WorkerScope::Public { worker_id: None },
            queue_type: WorkerQueueType::Pg,
        })
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
