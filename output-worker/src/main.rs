mod monitoring;
mod opentelemetry;
mod pg;
mod pulsar;
mod throughput_log;
mod work;

use ::pulsar::{Authentication, Pulsar, TokioExecutor};
use anyhow::bail;
use aws_sdk_s3::Client;
use aws_sdk_s3::config::retry::RetryConfig;
use aws_sdk_s3::config::timeout::TimeoutConfig;
use aws_sdk_s3::config::{AppName, Credentials, Region};
use chrono::{DateTime, Utc};
use clap::{ArgGroup, Parser, ValueEnum, crate_name, crate_version};
use humantime::format_duration;
use reqwest::Url;
use reqwest::header::HeaderName;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{PgConnection, PgPool, query, query_as};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use strum::{EnumString, VariantNames};
use tokio::signal::unix::{SignalKind, signal};
use tokio::sync::mpsc::channel;
use tokio::task::JoinSet;
use tokio::time::sleep;
use tokio::{select, spawn};
use tokio_util::task::TaskTracker;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use hook0_protobuf::RequestAttempt;
use work::*;

#[derive(Debug, Clone, Copy, strum::EnumString, strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum RetryStrategy {
    Exponential,
    Linear,
    Custom,
}

/// Schedule config as enum — makes invalid states unrepresentable.
#[derive(Debug, Clone)]
pub enum ScheduleConfig {
    Exponential { max_retries: i32 },
    Linear { max_retries: i32, delay_secs: i32 },
    Custom { max_retries: i32, intervals_secs: Vec<i32> },
}

impl ScheduleConfig {
    fn max_retries(&self) -> i32 {
        match self {
            Self::Exponential { max_retries } => *max_retries,
            Self::Linear { max_retries, .. } => *max_retries,
            Self::Custom { max_retries, .. } => *max_retries,
        }
    }
}

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

    /// Enable Sentry SDK debug mode
    #[clap(long, env, default_value_t = false)]
    sentry_debug: bool,

    /// Send default PII (IP addresses, cookies, etc.) to Sentry
    #[clap(long, env, default_value_t = false)]
    sentry_send_default_pii: bool,

    /// Optional OTLP endpoint that will receive metrics
    #[clap(long, env)]
    otlp_metrics_endpoint: Option<Url>,

    /// Optional OTLP endpoint that will receive traces
    #[clap(long, env)]
    otlp_traces_endpoint: Option<Url>,

    /// Optional value for OTLP `Authorization` header (for example: `Bearer mytoken`)
    #[clap(long, env, hide_env_values = true)]
    otlp_authorization: Option<String>,

    /// Database URL (with credentials)
    #[clap(long, env, hide_env_values = true)]
    database_url: String,

    /// Maximum number of connections to database (for a worker with pg queue type, it should be equal to CONCURRENT)
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

    /// Maximum number of attempts for object storage operations
    #[clap(long, env, default_value_t = 3)]
    object_storage_max_attempts: u32,

    /// Connect timeout for object storage operations (time to initiate socket connection)
    #[clap(long, env, value_parser = humantime::parse_duration, default_value = "3s")]
    object_storage_connect_timeout: Duration,

    /// Read timeout for object storage operations (time to first byte)
    #[clap(long, env, value_parser = humantime::parse_duration, default_value = "5s")]
    object_storage_read_timeout: Duration,

    /// Operation attempt timeout for object storage operations
    #[clap(long, env, value_parser = humantime::parse_duration, default_value = "10s")]
    object_storage_operation_attempt_timeout: Duration,

    /// Operation timeout for object storage operations
    #[clap(long, env, value_parser = humantime::parse_duration, default_value = "30s")]
    object_storage_operation_timeout: Duration,

    /// Bucket name of the S3-like object storage
    #[clap(long, env)]
    object_storage_bucket_name: Option<String>,

    /// If true, new response bodies and headers will be stored in object storage instead of database
    #[clap(long, env, default_value_t = false)]
    store_response_body_and_headers_in_object_storage: bool,

    /// A comma-separated list of applications ID whose response bodies and headers should be stored in object storage; if empty (default), all response bodies and headers will be stored in object storage regardless of application ID
    #[clap(long, env, use_value_delimiter = true)]
    store_response_body_and_headers_in_object_storage_only_for: Vec<Uuid>,

    /// Worker name (as defined in the infrastructure.worker table)
    #[clap(long, env)]
    worker_name: String,

    /// Worker version (if empty, will use version from Cargo.toml)
    #[clap(long, env)]
    worker_version: Option<String>,

    /// Number of request attempts to handle concurrently (for a worker with pg queue type, this means opening 1 connection to PostgreSQL per concurrent unit)
    #[clap(long, env, default_value = "1", value_parser = clap::value_parser!(u16).range(1..))]
    concurrent: u16,

    /// Maximum number of delivery retries before giving up (the effective number of retries is limited by `MAX_RETRIES`, `MAX_RETRY_WINDOW` and the retry policy)
    #[clap(long, env, default_value_t = 25)]
    max_retries: u8,

    /// Maximum time window for delivery retries before giving up (the effective number of retries is limited by `MAX_RETRIES`, `MAX_RETRY_WINDOW` and the retry policy)
    #[clap(long, env, value_parser = humantime::parse_duration, default_value = "8d")]
    max_retry_window: Duration,

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
    load_waiting_request_attempts_into_pulsar: bool,

    /// Grace period to wait for database commit before dropping unfound request attempts (only for Pulsar workers)
    #[clap(long, env, value_parser = humantime::parse_duration, default_value = "5s")]
    request_attempt_db_commit_grace_period: Duration,

    /// Period of Pulsar consumer stats collection (set to "0s" to disable) (only for Pulsar workers)
    #[clap(long, env, value_parser = humantime::parse_duration, default_value = "15s")]
    pulsar_consumer_stats_interval: Duration,

    /// Interval between periodic throughput log lines (set to "0s" to disable)
    #[clap(long, env, value_parser = humantime::parse_duration, default_value = "60s")]
    throughput_log_interval: Duration,
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
    fn is_public(&self) -> bool {
        matches!(self, Self::Public { .. })
    }

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
    store_response_body_and_headers: bool,
    store_response_body_and_headers_only_for: Vec<Uuid>,
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
    pub delay_until: Option<DateTime<Utc>>,
    pub http_method: String,
    pub http_url: String,
    pub http_headers: serde_json::Value,
    pub event_type_name: String,
    pub payload: Option<Vec<u8>>,
    pub payload_content_type: String,
    pub secret: Uuid,
    pub retry_strategy: Option<String>,
    pub retry_max_retries: Option<i32>,
    pub retry_custom_intervals: Option<Vec<i32>>,
    pub retry_linear_delay: Option<i32>,
}

impl RequestAttemptWithOptionalPayload {
    pub fn schedule_config(&self) -> Option<ScheduleConfig> {
        let strategy_str = self.retry_strategy.as_ref()?;
        let strategy = RetryStrategy::from_str(strategy_str).ok()?;
        let max_retries = self.retry_max_retries?;
        match strategy {
            RetryStrategy::Exponential => Some(ScheduleConfig::Exponential { max_retries }),
            RetryStrategy::Linear => {
                let delay_secs = self.retry_linear_delay?;
                Some(ScheduleConfig::Linear { max_retries, delay_secs })
            }
            RetryStrategy::Custom => {
                let intervals_secs = self.retry_custom_intervals.clone()?;
                if intervals_secs.len() != usize::try_from(max_retries).unwrap_or(0) {
                    warn!("Custom schedule intervals len {} != max_retries {}, falling back to default",
                        intervals_secs.len(), max_retries);
                    return None;
                }
                Some(ScheduleConfig::Custom { max_retries, intervals_secs })
            }
        }
    }
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
    let _sentry = hook0_sentry_integration::init(
        &config.sentry_dsn,
        &None,
        config.sentry_debug,
        config.sentry_send_default_pii,
        false,
    );

    // Init OpenTelemetry
    let otlp_exporters = opentelemetry::init(&config, &worker_version)?;

    info!(
        "Starting {} {worker_version} [{worker_name}]",
        crate_name!(),
    );
    debug!(
        "Webhook connect timeout is set to {:?}",
        config.connect_timeout
    );
    debug!("Webhook total timeout is set to {:?}", config.timeout);
    let retry_policy = evaluate_retry_policy(config.max_retries, config.max_retry_window);
    info!(
        "Configured retry policy allows a maximum of {} retries in a {} window (default, may be overridden per-subscription)",
        retry_policy.0,
        format_duration(retry_policy.1)
    );

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

    // Periodically collect metrics from database pool
    let metrics_pool = pool.clone();
    let metrics_pool_handle = spawn(async move {
        loop {
            opentelemetry::gather_pool_metrics(&metrics_pool);
            sleep(Duration::from_secs(15)).await
        }
    });

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
            rustls::crypto::aws_lc_rs::default_provider()
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
            .behavior_version_latest()
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
            .timeout_config(
                TimeoutConfig::builder()
                    .connect_timeout(config.object_storage_connect_timeout)
                    .read_timeout(config.object_storage_read_timeout)
                    .operation_attempt_timeout(config.object_storage_operation_attempt_timeout)
                    .operation_timeout(config.object_storage_operation_timeout)
                    .build(),
            )
            .retry_config(
                RetryConfig::standard()
                    .with_max_attempts(config.object_storage_max_attempts)
                    .with_max_backoff(Duration::from_secs(2)),
            )
            .build();
        let client = Client::from_conf(s3_config);
        if let Err(e) = client
            .head_bucket()
            .bucket(object_storage_bucket_name)
            .send()
            .await
        {
            if let Some(se) = e.as_service_error() {
                error!("Could not connect to object storage: (service error) {se}");
            } else {
                error!("Could not connect to object storage: {e}");
            }
            warn!("Continuing without object storage support (restart to try again)");
            None
        } else {
            info!("Object storage support is enabled");
            Some(ObjectStorageConfig {
                client,
                bucket: object_storage_bucket_name.to_owned(),
                store_response_body_and_headers: config
                    .store_response_body_and_headers_in_object_storage,
                store_response_body_and_headers_only_for: config
                    .store_response_body_and_headers_in_object_storage_only_for
                    .to_owned(),
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

    // Create throughput stats and spawn periodic log task
    let stats = Arc::new(throughput_log::ThroughputStats::new(config.concurrent));
    if !config.throughput_log_interval.is_zero() {
        let stats_clone = stats.clone();
        let interval = config.throughput_log_interval;
        let tt = task_tracker.clone();
        tasks.spawn(async move {
            throughput_log::run_throughput_log(&stats_clone, interval, &tt).await;
        });
    }

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
            let os = Arc::new(object_storage_config);
            let wid = Arc::new(worker_id);
            let wn = Arc::new(worker.name.to_owned());
            let wv = Arc::new(worker_version.to_owned());
            let pu = pulsar.clone();

            if c.load_waiting_request_attempts_into_pulsar {
                let po_clone = po.clone();
                let wid_clone = wid.clone();
                let pu_clone = pu.clone();
                let os_clone = os.clone();
                spawn(async move {
                    info!("Loading waiting request attempts from database into Pulsar...");
                    match pulsar::load_waiting_request_attempts_from_db(
                        &po_clone, &wid_clone, &pu_clone, &os_clone,
                    )
                    .await
                    {
                        Ok(c) => {
                            info!("Loaded {c} waiting request attempts from database into Pulsar")
                        }
                        Err(e) => error!(
                            "Error while loading waiting request attempts from database into Pulsar: {e}"
                        ),
                    }
                });
            }

            let stats_pulsar = stats.clone();
            tasks.spawn(async move {
                loop {
                    let result = pulsar::look_for_work(
                        &c,
                        &po,
                        &os,
                        &wid,
                        &wn,
                        &wv,
                        &pu,
                        heartbeat_tx.clone(),
                        &task_tracker_main,
                        &stats_pulsar,
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
            let stats_pg = stats.clone();
            task_tracker_main.spawn(async move {
                // Start units progressively
                sleep(Duration::from_millis(u64::from(unit_id) * 100)).await;

                loop {
                    let t = pg::look_for_work(
                        &cfg,
                        unit_id,
                        &p,
                        &os,
                        &w,
                        &wv,
                        tx.clone(),
                        &tt,
                        &stats_pg,
                    )
                    .await;
                    if let Err(ref e) = t {
                        error!(unit_id, "Unit crashed: {e}");
                    }

                    if tt.is_closed() {
                        break;
                    }

                    sleep(Duration::from_secs(1)).await;
                    info!(unit_id, "Restarting unit...");
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

    // Ensure all OpenTelemetry entities have been reported
    metrics_pool_handle.abort();
    otlp_exporters.shutdown()?;

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
    attempt: &RequestAttempt,
    response: &Response,
    max_retries: u8,
    schedule: Option<&ScheduleConfig>,
) -> Result<Option<Duration>, sqlx::Error> {
    match response.response_error {
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
            // Temporary warning message; this will be replaced by actual actions at some point
            if let Some(ResponseError::InvalidTarget) = response.response_error {
                let msg = response
                    .body
                    .as_ref()
                    .and_then(|bytes| str::from_utf8(bytes).ok())
                    .unwrap_or("???");
                warn!(request_attempt_id = %attempt.request_attempt_id, "Invalid target ({msg}); continuing as normal");
            }

            let sub = query!(
                "
                    SELECT true AS whatever
                    FROM webhook.subscription AS s
                    INNER JOIN event.application AS a ON a.application__id = s.application__id
                    WHERE s.subscription__id = $1
                        AND s.deleted_at IS NULL
                        AND s.is_enabled
                        AND a.deleted_at IS NULL
                ",
                attempt.subscription_id
            )
            .fetch_optional(conn)
            .await?;

            if sub.is_some() {
                match schedule {
                    Some(config) => Ok(compute_delay_from_schedule(config, attempt.retry_count)),
                    None => Ok(compute_next_retry_duration(max_retries, attempt.retry_count)),
                }
            } else {
                // If the subscription was disabled or soft-deleted (or its application was deleted), we do not schedule a next attempt
                Ok(None)
            }
        }
    }
}

fn compute_delay_from_schedule(config: &ScheduleConfig, retry_count: i16) -> Option<Duration> {
    let count = i32::from(retry_count);
    if count >= config.max_retries() {
        return None;
    }
    match config {
        ScheduleConfig::Exponential { max_retries } => {
            compute_next_retry_duration(
                u8::try_from(*max_retries).unwrap_or(100).min(100),
                retry_count,
            )
        }
        ScheduleConfig::Linear { delay_secs, .. } => {
            Some(Duration::from_secs(u64::try_from(*delay_secs).unwrap_or(0)))
        }
        ScheduleConfig::Custom { intervals_secs, .. } => {
            let idx = usize::try_from(count).unwrap_or(0);
            let delay = intervals_secs.get(idx)?;
            Some(Duration::from_secs(u64::try_from(*delay).unwrap_or(0)))
        }
    }
}

fn compute_next_retry_duration(max_retries: u8, retry_count: i16) -> Option<Duration> {
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

fn evaluate_retry_policy(max_retries: u8, max_retry_window: Duration) -> (u8, Duration) {
    let mut cumulative = Duration::ZERO;
    let mut effective_retries = 0;

    for i in 0..max_retries {
        match compute_next_retry_duration(max_retries, i.into()) {
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
    fn test_compute_next_retry_duration_exceeds_max() {
        assert_eq!(compute_next_retry_duration(5, 5), None);
        assert_eq!(compute_next_retry_duration(5, 6), None);
        assert_eq!(compute_next_retry_duration(0, 0), None);
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

    // --- Exponential schedule tests (7) ---

    #[test]
    fn test_exponential_first_retry() {
        let config = ScheduleConfig::Exponential { max_retries: 10 };
        assert_eq!(compute_delay_from_schedule(&config, 0), Some(Duration::from_secs(3)));
    }

    #[test]
    fn test_exponential_mid_table() {
        let config = ScheduleConfig::Exponential { max_retries: 10 };
        assert_eq!(compute_delay_from_schedule(&config, 3), Some(Duration::from_secs(30 * 60)));
    }

    #[test]
    fn test_exponential_past_table() {
        let config = ScheduleConfig::Exponential { max_retries: 10 };
        assert_eq!(compute_delay_from_schedule(&config, 9), Some(Duration::from_hours(10)));
    }

    #[test]
    fn test_exponential_at_max() {
        let config = ScheduleConfig::Exponential { max_retries: 5 };
        assert_eq!(compute_delay_from_schedule(&config, 5), None);
    }

    #[test]
    fn test_exponential_past_max() {
        let config = ScheduleConfig::Exponential { max_retries: 5 };
        assert_eq!(compute_delay_from_schedule(&config, 6), None);
    }

    #[test]
    fn test_exponential_max_1() {
        let config = ScheduleConfig::Exponential { max_retries: 1 };
        assert_eq!(compute_delay_from_schedule(&config, 0), Some(Duration::from_secs(3)));
    }

    #[test]
    fn test_exponential_max_100() {
        let config = ScheduleConfig::Exponential { max_retries: 100 };
        assert_eq!(compute_delay_from_schedule(&config, 99), Some(Duration::from_hours(10)));
    }

    // --- Linear schedule tests (5) ---

    #[test]
    fn test_linear_first_retry() {
        let config = ScheduleConfig::Linear { max_retries: 5, delay_secs: 300 };
        assert_eq!(compute_delay_from_schedule(&config, 0), Some(Duration::from_secs(300)));
    }

    #[test]
    fn test_linear_every_retry_same() {
        let config = ScheduleConfig::Linear { max_retries: 10, delay_secs: 300 };
        assert_eq!(compute_delay_from_schedule(&config, 4), Some(Duration::from_secs(300)));
    }

    #[test]
    fn test_linear_at_max() {
        let config = ScheduleConfig::Linear { max_retries: 5, delay_secs: 300 };
        assert_eq!(compute_delay_from_schedule(&config, 5), None);
    }

    #[test]
    fn test_linear_delay_1s() {
        let config = ScheduleConfig::Linear { max_retries: 3, delay_secs: 1 };
        assert_eq!(compute_delay_from_schedule(&config, 0), Some(Duration::from_secs(1)));
    }

    #[test]
    fn test_linear_delay_max() {
        let config = ScheduleConfig::Linear { max_retries: 3, delay_secs: 604800 };
        assert_eq!(compute_delay_from_schedule(&config, 0), Some(Duration::from_secs(604800)));
    }

    // --- Custom schedule tests (6) ---

    #[test]
    fn test_custom_first_interval() {
        let config = ScheduleConfig::Custom { max_retries: 3, intervals_secs: vec![3, 30, 300] };
        assert_eq!(compute_delay_from_schedule(&config, 0), Some(Duration::from_secs(3)));
    }

    #[test]
    fn test_custom_last_interval() {
        let config = ScheduleConfig::Custom { max_retries: 3, intervals_secs: vec![3, 30, 300] };
        assert_eq!(compute_delay_from_schedule(&config, 2), Some(Duration::from_secs(300)));
    }

    #[test]
    fn test_custom_at_max() {
        let config = ScheduleConfig::Custom { max_retries: 3, intervals_secs: vec![3, 30, 300] };
        assert_eq!(compute_delay_from_schedule(&config, 3), None);
    }

    #[test]
    fn test_custom_single() {
        let config = ScheduleConfig::Custom { max_retries: 1, intervals_secs: vec![60] };
        assert_eq!(compute_delay_from_schedule(&config, 0), Some(Duration::from_secs(60)));
    }

    #[test]
    fn test_custom_empty_intervals() {
        let config = ScheduleConfig::Custom { max_retries: 0, intervals_secs: vec![] };
        assert_eq!(compute_delay_from_schedule(&config, 0), None);
    }

    #[test]
    fn test_custom_all_same() {
        let config = ScheduleConfig::Custom { max_retries: 3, intervals_secs: vec![10, 10, 10] };
        assert_eq!(compute_delay_from_schedule(&config, 0), Some(Duration::from_secs(10)));
        assert_eq!(compute_delay_from_schedule(&config, 1), Some(Duration::from_secs(10)));
        assert_eq!(compute_delay_from_schedule(&config, 2), Some(Duration::from_secs(10)));
    }

    // --- schedule_config() tests (6) ---

    fn make_attempt(
        strategy: Option<&str>,
        max_retries: Option<i32>,
        intervals: Option<Vec<i32>>,
        linear_delay: Option<i32>,
    ) -> RequestAttemptWithOptionalPayload {
        RequestAttemptWithOptionalPayload {
            application_id: Uuid::nil(),
            request_attempt_id: Uuid::nil(),
            event_id: Uuid::nil(),
            event_received_at: chrono::Utc::now(),
            subscription_id: Uuid::nil(),
            created_at: chrono::Utc::now(),
            retry_count: 0,
            delay_until: None,
            http_method: "POST".to_string(),
            http_url: "http://example.com".to_string(),
            http_headers: serde_json::json!({}),
            event_type_name: "test".to_string(),
            payload: None,
            payload_content_type: "application/json".to_string(),
            secret: Uuid::nil(),
            retry_strategy: strategy.map(|s| s.to_string()),
            retry_max_retries: max_retries,
            retry_custom_intervals: intervals,
            retry_linear_delay: linear_delay,
        }
    }

    #[test]
    fn test_config_none_no_strategy() {
        let a = make_attempt(None, None, None, None);
        assert!(a.schedule_config().is_none());
    }

    #[test]
    fn test_config_exponential() {
        let a = make_attempt(Some("exponential"), Some(5), None, None);
        assert!(matches!(a.schedule_config(), Some(ScheduleConfig::Exponential { max_retries: 5 })));
    }

    #[test]
    fn test_config_linear() {
        let a = make_attempt(Some("linear"), Some(3), None, Some(60));
        assert!(matches!(a.schedule_config(), Some(ScheduleConfig::Linear { max_retries: 3, delay_secs: 60 })));
    }

    #[test]
    fn test_config_custom() {
        let a = make_attempt(Some("custom"), Some(2), Some(vec![10, 20]), None);
        match a.schedule_config() {
            Some(ScheduleConfig::Custom { max_retries, intervals_secs }) => {
                assert_eq!(max_retries, 2);
                assert_eq!(intervals_secs, vec![10, 20]);
            }
            other => panic!("Expected Custom, got {:?}", other),
        }
    }

    #[test]
    fn test_config_unknown_strategy() {
        let a = make_attempt(Some("fibonacci"), Some(5), None, None);
        assert!(a.schedule_config().is_none());
    }

    #[test]
    fn test_config_missing_max_retries() {
        let a = make_attempt(Some("exponential"), None, None, None);
        assert!(a.schedule_config().is_none());
    }

    // --- Fallback tests (2) ---

    #[test]
    fn test_no_schedule_default() {
        // Without schedule, uses default exponential: count=0 -> 3s
        assert_eq!(compute_next_retry_duration(25, 0), Some(Duration::from_secs(3)));
    }

    #[test]
    fn test_schedule_overrides() {
        // With Exponential max=2, count=2 -> None (exhausted)
        let config = ScheduleConfig::Exponential { max_retries: 2 };
        assert_eq!(compute_delay_from_schedule(&config, 2), None);
    }

    // --- Edge case tests (2) ---

    #[test]
    fn test_negative_retry_count() {
        let exp = ScheduleConfig::Exponential { max_retries: 5 };
        // Negative retry_count: i32::from(-1) = -1 < max_retries, so it proceeds
        // compute_next_retry_duration with negative i16 also works (retry_count < max_retries)
        assert!(compute_delay_from_schedule(&exp, -1).is_some());
    }

    #[test]
    fn test_zero_all_strategies() {
        let exp = ScheduleConfig::Exponential { max_retries: 5 };
        let lin = ScheduleConfig::Linear { max_retries: 5, delay_secs: 10 };
        let cust = ScheduleConfig::Custom { max_retries: 3, intervals_secs: vec![1, 2, 3] };
        assert_eq!(compute_delay_from_schedule(&exp, 0), Some(Duration::from_secs(3)));
        assert_eq!(compute_delay_from_schedule(&lin, 0), Some(Duration::from_secs(10)));
        assert_eq!(compute_delay_from_schedule(&cust, 0), Some(Duration::from_secs(1)));
    }
}
