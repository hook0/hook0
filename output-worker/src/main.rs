mod dns;
mod monitoring;
mod opentelemetry;
mod pg;
mod pulsar;
mod throughput_log;
mod work;

use ::pulsar::{Authentication, ConnectionRetryOptions, Pulsar, TokioExecutor};
use anyhow::bail;
use aws_sdk_s3::Client;
use aws_sdk_s3::config::retry::RetryConfig;
use aws_sdk_s3::config::timeout::TimeoutConfig;
use aws_sdk_s3::config::{AppName, Credentials, Region};
use chrono::{DateTime, Utc};
use clap::{ArgGroup, Parser, ValueEnum, crate_name, crate_version};
use hickory_resolver::config::LookupIpStrategy;
use humantime::format_duration;
use reqwest::Url;
use reqwest::header::{HeaderName, RETRY_AFTER};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{PgConnection, PgPool, query, query_as};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use strum::{EnumString, VariantNames};
use thousands::Separable;
use tokio::signal::unix::{SignalKind, signal};
use tokio::sync::mpsc::channel;
use tokio::task::JoinSet;
use tokio::time::sleep;
use tokio::{select, spawn};
use tokio_util::task::TaskTracker;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::dns::{DnsResolver, DnsResolverOptions};
use crate::pulsar::LoadMode;
use crate::work::*;
use hook0_protobuf::RequestAttempt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum SignatureVersion {
    V0,
    V1,
}

/// Which address families to ask for when resolving a webhook target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[clap(rename_all = "kebab-case")]
enum DnsIpStrategy {
    Ipv4Only,
    Ipv6Only,
    Ipv4AndIpv6,
    Ipv6AndIpv4,
}

impl From<DnsIpStrategy> for LookupIpStrategy {
    fn from(strategy: DnsIpStrategy) -> Self {
        match strategy {
            DnsIpStrategy::Ipv4Only => Self::Ipv4Only,
            DnsIpStrategy::Ipv6Only => Self::Ipv6Only,
            DnsIpStrategy::Ipv4AndIpv6 => Self::Ipv4AndIpv6,
            DnsIpStrategy::Ipv6AndIpv4 => Self::Ipv6AndIpv4,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[clap(rename_all = "kebab-case")]
enum LoadWaitingRequestAttemptsIntoPulsarMode {
    #[value(alias = "false")]
    Off,
    #[value(alias = "true")]
    All,
    DueNow,
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
    #[clap(long, env, default_value_t = 1, value_parser = clap::value_parser!(u16).range(1..))]
    concurrent: u16,

    /// Retry count cutoff for queue priority classification: if retry_count >= cutoff, item is placed in low priority queue
    #[clap(long, env, default_value_t = 2)]
    hp_retry_cutoff: i16,

    /// Number of concurrent slots reserved exclusively for high-priority jobs (first attempts and early retries)
    #[clap(long, env, default_value_t = 0)]
    concurrent_hp_reserved: u16,

    /// Number of concurrent slots reserved exclusively for low-priority jobs (later retries)
    #[clap(long, env, default_value_t = 0)]
    concurrent_lp_reserved: u16,

    /// Maximum number of delivery retries before giving up (the effective number of retries is limited by `MAX_RETRIES`, `MAX_RETRY_WINDOW` and the retry policy)
    #[clap(long, env, default_value_t = 24)]
    max_retries: u8,

    /// Maximum time window for delivery retries before giving up (the effective number of retries is limited by `MAX_RETRIES`, `MAX_RETRY_WINDOW` and the retry policy)
    #[clap(long, env, value_parser = humantime::parse_duration, default_value = "8d")]
    max_retry_window: Duration,

    /// Ratio of a retry's base delay used as the width of the random jitter window added on top of it, bounded by an internal minimum and by `RETRY_JITTER_MAX_SPREAD` (set to 0 to disable jitter and get strictly deterministic retry delays)
    #[clap(long, env, default_value_t = 0.1)]
    retry_jitter_ratio: f64,

    /// Maximum width of the random jitter window added on top of a retry's base delay; takes precedence over the internal minimum (set to "0s" to disable jitter)
    #[clap(long, env, value_parser = humantime::parse_duration, default_value = "15m")]
    retry_jitter_max_spread: Duration,

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

    /// Total wall-clock budget for resolving the target's hostname, across every name server query it takes (if exceeded, request attempt will fail); must be at least "3ms"
    #[clap(long, env, value_parser = humantime::parse_duration, default_value = "5s")]
    dns_timeout: Duration,

    /// Maximum duration a successful DNS answer is kept in the worker's in-process DNS cache (shorter record TTLs are still honored)
    #[clap(long, env, value_parser = humantime::parse_duration, default_value = "5m")]
    dns_cache_max_ttl: Duration,

    /// Maximum duration a negative DNS answer (for example NXDOMAIN) is kept in the worker's in-process DNS cache
    #[clap(long, env, value_parser = humantime::parse_duration, default_value = "30s")]
    dns_negative_cache_max_ttl: Duration,

    /// Which IP address families to query when resolving a webhook target's hostname; `ipv4-only` ignores AAAA records entirely, which is useful when a target's IPv6 address is not globally reachable
    #[clap(long, env, default_value = "ipv4-and-ipv6")]
    dns_ip_strategy: DnsIpStrategy,

    /// If set to false (default), a webhook target's hostname is resolved exactly as written; if true, the worker host's resolv.conf search domains are appended to it
    #[clap(long, env, default_value_t = false)]
    dns_append_search_domains: bool,

    /// Name of the header containing webhook's signature
    #[clap(long, env, default_value = "X-Hook0-Signature")]
    signature_header_name: HeaderName,

    /// A comma-separated list of enabled signature versions
    #[clap(long, env, default_value = "v1", value_delimiter = ',')]
    enabled_signature_versions: Vec<SignatureVersion>,

    /// Loads request attempts that haven't been delivered yet from the DB into Pulsar before starting work; `all` loads everything; `due-now` skips request attempts scheduled more than ~10 s in the future; this is useful when migrating to a Pulsar worker (only for Pulsar workers)
    #[clap(long, env, default_value = "off")]
    load_waiting_request_attempts_into_pulsar: LoadWaitingRequestAttemptsIntoPulsarMode,

    /// Grace period to wait for database commit before dropping unfound request attempts (only for Pulsar workers)
    #[clap(long, env, value_parser = humantime::parse_duration, default_value = "10s")]
    request_attempt_db_commit_grace_period: Duration,

    /// Period of Pulsar consumer stats collection (set to "0s" to disable) (only for Pulsar workers) [this feature is unstable/unreliable]
    #[clap(long, env, value_parser = humantime::parse_duration, default_value = "0")]
    pulsar_consumer_stats_interval: Duration,

    /// Maximum time to wait for the Pulsar broker to acknowledge a sent message (only for Pulsar workers)
    #[clap(long, env, value_parser = humantime::parse_duration, default_value = "10s")]
    pulsar_send_receipt_timeout: Duration,

    /// Interval between periodic throughput log lines (set to "0s" to disable)
    #[clap(long, env, value_parser = humantime::parse_duration, default_value = "60s")]
    throughput_log_interval: Duration,

    /// Period at which free concurrency slots are sampled for the throughput log and OTel gauges (set to "0s" to disable) (only for Pulsar workers)
    #[clap(long, env, value_parser = humantime::parse_duration, default_value = "15s")]
    slot_metrics_interval: Duration,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotRole {
    /// Only picks jobs with retry_count < hp_retry_cutoff
    HpReserved,
    /// Only picks jobs with retry_count >= hp_retry_cutoff
    LpReserved,
    /// Picks any job (oldest first, no filter)
    Dynamic,
}

impl std::fmt::Display for SlotRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HpReserved => write!(f, "hp-reserved"),
            Self::LpReserved => write!(f, "lp-reserved"),
            Self::Dynamic => write!(f, "dynamic"),
        }
    }
}

impl SlotRole {
    pub fn from_unit_id(unit_id: u16, hp_reserved: u16, lp_reserved: u16) -> Self {
        if unit_id < hp_reserved {
            Self::HpReserved
        } else if unit_id < hp_reserved + lp_reserved {
            Self::LpReserved
        } else {
            Self::Dynamic
        }
    }

    pub fn is_hp(retry_count: i16, cutoff: i16) -> bool {
        retry_count < cutoff
    }
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

    let retry_policy = RetryPolicy::from_config(&config)?;
    let effective_retry_policy = retry_policy.evaluate(config.max_retry_window);
    info!(
        "Configured retry policy allows a maximum of {} retries in a {} window",
        effective_retry_policy.0,
        format_duration(effective_retry_policy.1)
    );

    // Built once and shared by every request attempt, so its cache is shared too.
    let resolver = Arc::new(DnsResolver::new(DnsResolverOptions {
        budget: config.dns_timeout,
        positive_max_ttl: config.dns_cache_max_ttl,
        negative_max_ttl: config.dns_negative_cache_max_ttl,
        ip_strategy: config.dns_ip_strategy.into(),
        append_search_domains: config.dns_append_search_domains,
    })?);
    debug!(
        "DNS budget is set to {:?} (per name server pool round: {:?})",
        config.dns_timeout,
        dns::pool_deadline(config.dns_timeout)
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

    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .unwrap();

    let worker = get_worker(worker_name, &pool).await?;

    if matches!(worker.queue_type, WorkerQueueType::Pg)
        && u32::from(config.concurrent) > config.max_db_connections
    {
        warn!(
            "Worker has a pg queue type with CONCURRENT={}, but MAX_DB_CONNECTIONS is smaller ({}); worker with pg queue should have MAX_DB_CONNECTIONS=CONCURRENT",
            config.concurrent, config.max_db_connections
        );
    }

    match config
        .concurrent_hp_reserved
        .checked_add(config.concurrent_lp_reserved)
    {
        Some(reserved) if reserved > config.concurrent => bail!(
            "CONCURRENT_HP_RESERVED ({}) + CONCURRENT_LP_RESERVED ({}) exceeds CONCURRENT ({})",
            config.concurrent_hp_reserved,
            config.concurrent_lp_reserved,
            config.concurrent
        ),
        Some(_) => {
            // All good
        }
        None => bail!(
            "CONCURRENT_HP_RESERVED ({}) + CONCURRENT_LP_RESERVED ({}) overflows; use lower values",
            config.concurrent_hp_reserved,
            config.concurrent_lp_reserved,
        ),
    }
    let priority_enabled = config.concurrent_hp_reserved > 0 || config.concurrent_lp_reserved > 0;
    let dynamic_slots =
        config.concurrent - config.concurrent_hp_reserved - config.concurrent_lp_reserved;
    if priority_enabled {
        info!(
            "Priority queue enabled: {} HP-reserved, {} LP-reserved, {dynamic_slots} dynamic, cutoff={}",
            config.concurrent_hp_reserved, config.concurrent_lp_reserved, config.hp_retry_cutoff
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
            Some(Arc::new(PulsarConfig {
                pulsar: Pulsar::builder(pulsar_binary_url.to_owned(), TokioExecutor)
                    .with_auth(Authentication {
                        name: "token".to_owned(),
                        data: pulsar_token.to_owned().into_bytes(),
                    })
                    .with_connection_retry_options(ConnectionRetryOptions::default())
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
            warn!(
                "Object storage connection test failed; reads/writes will still be attempted (restart to re-run the connection test)"
            );
        } else {
            info!("Object storage support is enabled");
        }
        Some(ObjectStorageConfig {
            client,
            bucket: object_storage_bucket_name.to_owned(),
            store_response_body_and_headers: config
                .store_response_body_and_headers_in_object_storage,
            store_response_body_and_headers_only_for: config
                .store_response_body_and_headers_in_object_storage_only_for
                .to_owned(),
        })
    } else {
        None
    };

    if config.disable_target_ip_check {
        warn!(
            "Webhook's target IP check is disabled: this allows the worker to send HTTP requests that target local IP addresses (for example: loopback, LAN, ...); THIS MAY BE A SECURITY ISSUE IN PRODUCTION"
        )
    }

    if config.dns_append_search_domains {
        warn!(
            "Webhook target hostnames will be expanded using this host's resolv.conf search domains: each resolution can then cost one DNS round trip per search domain, so DNS_TIMEOUT is no longer enforced per name server query and only bounds the resolution as a whole"
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
    let stats = Arc::new(throughput_log::ThroughputStats::new(
        config.concurrent,
        config.concurrent_hp_reserved,
        config.concurrent_lp_reserved,
    ));
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

            let load_mode = match c.load_waiting_request_attempts_into_pulsar {
                LoadWaitingRequestAttemptsIntoPulsarMode::Off => None,
                LoadWaitingRequestAttemptsIntoPulsarMode::All => Some(LoadMode::All),
                LoadWaitingRequestAttemptsIntoPulsarMode::DueNow => Some(LoadMode::DueNow),
            };
            if let Some(mode) = load_mode {
                let po_clone = po.clone();
                let wid_clone = wid.clone();
                let pu_clone = pu.clone();
                let os_clone = os.clone();
                let hp_cutoff = c.hp_retry_cutoff;
                let send_receipt_timeout = c.pulsar_send_receipt_timeout;
                spawn(async move {
                    info!(
                        ?mode,
                        "Loading waiting request attempts from database into Pulsar..."
                    );
                    match pulsar::load_waiting_request_attempts_from_db(
                        &po_clone,
                        &wid_clone,
                        &pu_clone,
                        &os_clone,
                        hp_cutoff,
                        send_receipt_timeout,
                        mode,
                    )
                    .await
                    {
                        Ok(c) => info!(
                            ?mode,
                            "Loaded {} waiting request attempts from database into Pulsar",
                            c.separate_with_commas(),
                        ),
                        Err(e) => error!(
                            ?mode,
                            "Error while loading waiting request attempts from database into Pulsar: {e}"
                        ),
                    }
                });
            }

            let stats_pulsar = stats.clone();
            let dr = resolver.clone();
            tasks.spawn(async move {
                loop {
                    let result = pulsar::look_for_work(
                        &c,
                        retry_policy,
                        &po,
                        &os,
                        &wid,
                        &wn,
                        &wv,
                        &pu,
                        heartbeat_tx.clone(),
                        &task_tracker_main,
                        &stats_pulsar,
                        &dr,
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
            let role = SlotRole::from_unit_id(
                unit_id,
                config.concurrent_hp_reserved,
                config.concurrent_lp_reserved,
            );
            let p = pool.clone();
            let os = object_storage_config.clone();
            let w = worker.to_owned();
            let wv = worker_version.to_owned();
            let tx = heartbeat_tx.to_owned();
            let cfg = config.to_owned();
            let tt = task_tracker_main.clone();
            let stats_pg = stats.clone();
            let dr = resolver.clone();
            task_tracker_main.spawn(async move {
                // Start units progressively
                sleep(Duration::from_millis(u64::from(unit_id) * 100)).await;

                loop {
                    let t = pg::look_for_work(
                        &cfg,
                        retry_policy,
                        unit_id,
                        role,
                        &p,
                        &os,
                        &w,
                        &wv,
                        tx.clone(),
                        &tt,
                        &stats_pg,
                        &dr,
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

#[derive(Debug, Clone, Copy)]
pub struct RetryPolicy {
    max_retries: u8,
    jitter_ratio: f64,
    jitter_max_spread: Duration,
}

impl RetryPolicy {
    /// Minimum width of the random jitter window.
    const JITTER_MIN_SPREAD: Duration = Duration::from_secs(2);

    /// Longest delay a target can ask for through `Retry-After`.
    ///
    /// Deliberately equal to the schedule's own longest step, so a target can never park an
    /// attempt for longer than we already would on our own — a header saying "come back in a
    /// month" costs at most one extra step, not a month.
    const RETRY_AFTER_MAX: Duration = Duration::from_hours(10);

    fn from_config(config: &Config) -> anyhow::Result<Self> {
        if !config.retry_jitter_ratio.is_finite() || config.retry_jitter_ratio < 0.0 {
            bail!(
                "RETRY_JITTER_RATIO ({}) must be a finite non-negative number (use 0 to disable jitter)",
                config.retry_jitter_ratio
            );
        }
        Ok(Self {
            max_retries: config.max_retries,
            jitter_ratio: config.retry_jitter_ratio,
            jitter_max_spread: config.retry_jitter_max_spread,
        })
    }

    /// Deterministic base delay from the retry schedule, or `None` once retries are exhausted.
    fn base_delay(&self, retry_count: i16) -> Option<Duration> {
        if retry_count < self.max_retries.into() {
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

    /// Width of the random window added on top of `base`; zero when jitter is disabled.
    fn jitter_spread(&self, base: Duration) -> Duration {
        if !self.jitter_ratio.is_finite() || self.jitter_ratio <= 0.0 {
            Duration::ZERO
        } else {
            Duration::try_from_secs_f64(self.jitter_ratio * base.as_secs_f64())
                .unwrap_or(Duration::MAX)
                .max(Self::JITTER_MIN_SPREAD)
                .min(self.jitter_max_spread)
        }
    }

    /// Base delay plus jitter, or `None` once retries are exhausted.
    ///
    /// The result is truncated to whole microseconds: both queue backends store it in
    /// PostgreSQL (as an `INTERVAL` or a `timestamptz`), neither of which keeps
    /// sub-microsecond precision. Truncating here keeps the two paths in agreement and
    /// keeps `PgInterval::try_from` from rejecting the value.
    fn next_delay(&self, retry_count: i16, factor: f64) -> Option<Duration> {
        self.base_delay(retry_count).map(|base| {
            let spread = self.jitter_spread(base);
            let delay = base.saturating_add(spread.mul_f64(factor.clamp(0.0, 1.0)));
            Duration::new(delay.as_secs(), delay.subsec_micros() * 1000)
        })
    }

    /// How long a rate-limiting target asked us to wait, bounded by `RETRY_AFTER_MAX`.
    ///
    /// Only `429 Too Many Requests` is honoured. Other statuses may carry `Retry-After` too, but
    /// only a rate limit tells us the target is healthy and merely asking for a slower pace;
    /// on a 503 the header is a guess about a recovery we have no reason to trust.
    fn retry_after_hint(&self, response: &Response, now: DateTime<Utc>) -> Option<Duration> {
        if response.http_code != Some(429) {
            return None;
        }

        response
            .headers
            .as_ref()
            .and_then(|headers| headers.get(RETRY_AFTER))
            .and_then(|value| value.to_str().ok())
            .and_then(|value| parse_retry_after(value, now))
            .map(|hint| hint.min(Self::RETRY_AFTER_MAX))
    }

    /// The scheduled delay, never earlier than what a rate-limiting target asked for.
    ///
    /// Taking the maximum is what makes this safe to ship: honouring `Retry-After` can only ever
    /// push an attempt further away, never bring it closer, and it leaves the number of retries
    /// untouched — `None` still means "exhausted", for exactly the same retry counts as before.
    ///
    /// The flip side, and it is real: a run of honoured hints stretches the wall-clock life of an
    /// attempt beyond the `max_retry_window` estimate logged at startup, which only accounts for
    /// the base schedule and its jitter.
    fn next_delay_honouring(
        &self,
        retry_count: i16,
        factor: f64,
        response: &Response,
        now: DateTime<Utc>,
    ) -> Option<Duration> {
        let hint = self
            .retry_after_hint(response, now)
            .unwrap_or(Duration::ZERO);

        self.next_delay(retry_count, factor)
            .map(|delay| delay.max(hint))
    }

    /// Worst-case number of retries and cumulative delay that fit in `max_retry_window`.
    ///
    /// Each step is charged its maximum jitter so the result is an upper bound on the retry
    /// window rather than an under-estimate.
    fn evaluate(&self, max_retry_window: Duration) -> (u8, Duration) {
        let mut cumulative = Duration::ZERO;
        let mut effective_retries = 0;

        for i in 0..self.max_retries {
            match self.base_delay(i.into()) {
                Some(base) => {
                    let d = base.saturating_add(self.jitter_spread(base));
                    if cumulative.saturating_add(d) > max_retry_window {
                        break;
                    }
                    cumulative = cumulative.saturating_add(d);
                    effective_retries = i + 1;
                }
                None => break,
            }
        }

        (effective_retries, cumulative)
    }
}

/// A `Retry-After` value as a delay from `now`, or `None` when it is not one we can trust.
///
/// RFC 9110 allows two forms: a count of seconds, or an HTTP-date. A date already in the past
/// means "you may retry now" and yields a zero delay rather than being discarded. Anything we
/// cannot parse is ignored, which falls back to the base schedule — a malformed header must
/// never cost a delivery.
fn parse_retry_after(value: &str, now: DateTime<Utc>) -> Option<Duration> {
    let value = value.trim();

    if let Ok(seconds) = value.parse::<u64>() {
        return Some(Duration::from_secs(seconds));
    }

    DateTime::parse_from_rfc2822(value).ok().map(|date| {
        (date.with_timezone(&Utc) - now)
            .to_std()
            .unwrap_or_default()
    })
}

async fn compute_next_retry(
    conn: &mut PgConnection,
    attempt: &RequestAttempt,
    response: &Response,
    policy: RetryPolicy,
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
                Ok(policy.next_delay_honouring(
                    attempt.retry_count,
                    rand::random::<f64>(),
                    response,
                    Utc::now(),
                ))
            } else {
                // If the subscription was disabled or soft-deleted (or its application was deleted), we do not schedule a next attempt
                Ok(None)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use reqwest::header::{HeaderMap, HeaderValue};

    /// A policy with jitter disabled, so tests of the base schedule assert exact values
    fn no_jitter(max_retries: u8) -> RetryPolicy {
        RetryPolicy {
            max_retries,
            jitter_ratio: 0.0,
            jitter_max_spread: Duration::ZERO,
        }
    }

    /// A policy using the shipped defaults
    fn default_jitter(max_retries: u8) -> RetryPolicy {
        RetryPolicy {
            max_retries,
            jitter_ratio: 0.1,
            jitter_max_spread: Duration::from_secs(15 * 60),
        }
    }

    #[test]
    fn test_evaluate_retry_policy_zero_retries() {
        let (retries, cumulative) = no_jitter(0).evaluate(Duration::from_hours(1));
        assert_eq!(retries, 0);
        assert_eq!(cumulative, Duration::ZERO);
    }

    #[test]
    fn test_evaluate_retry_policy_zero_window() {
        let (retries, cumulative) = no_jitter(30).evaluate(Duration::ZERO);
        assert_eq!(retries, 0);
        assert_eq!(cumulative, Duration::ZERO);
    }

    #[test]
    fn test_base_delay_exceeds_max() {
        assert_eq!(no_jitter(5).base_delay(5), None);
        assert_eq!(no_jitter(5).base_delay(6), None);
        assert_eq!(no_jitter(0).base_delay(0), None);
    }

    #[test]
    fn test_evaluate_retry_policy_unlimited_window() {
        let window = Duration::from_hours(365 * 24);
        let (retries, cumulative) = no_jitter(30).evaluate(window);
        assert_eq!(retries, 30);
        assert!(cumulative < window / 10); // Duration is not just the window but the actual cumulative duration
    }

    #[test]
    fn test_evaluate_retry_policy_tight_window() {
        let window = Duration::from_secs(15);
        let (retries, cumulative) = no_jitter(30).evaluate(window);
        assert_eq!(retries, 2);
        assert!(cumulative < window);
    }

    #[test]
    fn test_jitter_spread_is_disabled_by_a_non_positive_or_invalid_ratio() {
        let base = Duration::from_secs(3);
        for ratio in [0.0, -0.5, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let policy = RetryPolicy {
                max_retries: 30,
                jitter_ratio: ratio,
                jitter_max_spread: Duration::from_secs(15 * 60),
            };
            assert_eq!(
                policy.jitter_spread(base),
                Duration::ZERO,
                "ratio {ratio} should disable jitter"
            );
        }
    }

    #[test]
    fn test_jitter_spread_is_disabled_by_a_zero_max_spread() {
        let policy = RetryPolicy {
            max_retries: 30,
            jitter_ratio: 0.1,
            jitter_max_spread: Duration::ZERO,
        };
        assert_eq!(policy.jitter_spread(Duration::from_secs(3)), Duration::ZERO);
        assert_eq!(
            policy.jitter_spread(Duration::from_hours(10)),
            Duration::ZERO
        );
    }

    #[test]
    fn test_jitter_spread_applies_floor_and_cap() {
        let policy = default_jitter(30);

        // Below the floor: 10% of 3s and of 10s are both under the 2s minimum
        assert_eq!(
            policy.jitter_spread(Duration::from_secs(3)),
            RetryPolicy::JITTER_MIN_SPREAD
        );
        assert_eq!(
            policy.jitter_spread(Duration::from_secs(10)),
            RetryPolicy::JITTER_MIN_SPREAD
        );

        // Between floor and cap: proportional
        assert_eq!(
            policy.jitter_spread(Duration::from_secs(3 * 60)),
            Duration::from_secs(18)
        );
        assert_eq!(
            policy.jitter_spread(Duration::from_hours(1)),
            Duration::from_secs(6 * 60)
        );

        // Above the cap
        assert_eq!(
            policy.jitter_spread(Duration::from_hours(10)),
            policy.jitter_max_spread
        );
    }

    #[test]
    fn test_jitter_spread_cap_beats_floor_without_panicking() {
        let policy = RetryPolicy {
            max_retries: 30,
            jitter_ratio: 0.1,
            jitter_max_spread: Duration::from_secs(1),
        };
        assert!(policy.jitter_max_spread < RetryPolicy::JITTER_MIN_SPREAD);
        assert_eq!(
            policy.jitter_spread(Duration::from_secs(3)),
            Duration::from_secs(1)
        );
    }

    #[test]
    fn test_jitter_spread_saturates_on_an_overflowing_ratio() {
        // A ratio this large overflows `Duration`; it must saturate to the cap, not panic
        let policy = RetryPolicy {
            max_retries: 30,
            jitter_ratio: f64::MAX,
            jitter_max_spread: Duration::from_secs(15 * 60),
        };
        let base = Duration::from_hours(10);

        assert_eq!(policy.jitter_spread(base), policy.jitter_max_spread);
        assert_eq!(
            policy.next_delay(7, 1.0),
            Some(base + policy.jitter_max_spread)
        );

        // The advertised window stays finite too
        let (retries, cumulative) = policy.evaluate(Duration::from_hours(24));
        assert!(retries > 0);
        assert!(cumulative <= Duration::from_hours(24));
    }

    #[test]
    fn test_next_delay_only_ever_adds_to_the_base_delay() {
        let policy = default_jitter(30);
        let base = policy.base_delay(0).unwrap();
        let spread = policy.jitter_spread(base);

        assert_eq!(policy.next_delay(0, 0.0), Some(base));
        assert_eq!(policy.next_delay(0, 1.0), Some(base + spread));

        // Out-of-range factors are clamped, so a delay is never shorter than its base
        assert_eq!(policy.next_delay(0, -1.0), Some(base));
        assert_eq!(policy.next_delay(0, 2.0), Some(base + spread));

        for factor in [0.0, 0.25, 0.5, 0.75, 1.0] {
            let delay = policy.next_delay(0, factor).unwrap();
            assert!(delay >= base && delay <= base + spread);
        }
    }

    #[test]
    fn test_next_delay_is_always_a_valid_postgresql_interval() {
        use sqlx::postgres::types::PgInterval;

        let policy = default_jitter(30);

        // This factor is what a `rand::random::<f64>()` draw looks like: without truncation
        // it yields sub-microsecond nanoseconds, which PostgreSQL's INTERVAL type rejects
        let factor = 0.6172839455_f64;
        let base = policy.base_delay(0).unwrap();
        let spread = policy.jitter_spread(base);
        assert_ne!(
            base.saturating_add(spread.mul_f64(factor)).subsec_nanos() % 1000,
            0,
            "the pinned factor must exercise the sub-microsecond case"
        );

        for retry_count in 0..9 {
            for f in [0.0, factor, 1.0] {
                let delay = policy.next_delay(retry_count, f).unwrap();
                assert_eq!(delay.subsec_nanos() % 1000, 0, "{delay:?}");
                assert!(PgInterval::try_from(delay).is_ok(), "{delay:?}");
            }

            // The property must hold for any random factor, not just the pinned ones
            for _ in 0..200 {
                let delay = policy
                    .next_delay(retry_count, rand::random::<f64>())
                    .unwrap();
                assert!(
                    PgInterval::try_from(delay).is_ok(),
                    "delay {delay:?} is not a valid PostgreSQL interval"
                );
            }
        }
    }

    #[test]
    fn test_next_delay_is_deterministic_when_jitter_is_disabled() {
        let policy = no_jitter(30);
        for factor in [0.0, 0.5, 1.0] {
            assert_eq!(policy.next_delay(0, factor), Some(Duration::from_secs(3)));
            assert_eq!(policy.next_delay(1, factor), Some(Duration::from_secs(10)));
        }
    }

    #[test]
    fn test_evaluate_accounts_for_the_worst_case_jitter() {
        let window = Duration::from_hours(365 * 24);
        let (_, without_jitter) = no_jitter(30).evaluate(window);
        let (_, with_jitter) = default_jitter(30).evaluate(window);

        assert!(with_jitter > without_jitter);
        assert!(with_jitter <= window);

        // A tight window fits fewer retries once each step is charged its maximum jitter:
        // 3s + 2s = 5s fits, but the next step needs 10s + 2s = 12s (17s total > 15s)
        let tight = Duration::from_secs(15);
        assert_eq!(no_jitter(30).evaluate(tight).0, 2);
        assert_eq!(default_jitter(30).evaluate(tight).0, 1);
    }

    #[test]
    fn test_slot_role_assignment() {
        let hp_reserved = 2;
        let lp_reserved = 1;
        assert_eq!(
            SlotRole::from_unit_id(0, hp_reserved, lp_reserved),
            SlotRole::HpReserved
        );
        assert_eq!(
            SlotRole::from_unit_id(1, hp_reserved, lp_reserved),
            SlotRole::HpReserved
        );
        assert_eq!(
            SlotRole::from_unit_id(2, hp_reserved, lp_reserved),
            SlotRole::LpReserved
        );
        assert_eq!(
            SlotRole::from_unit_id(3, hp_reserved, lp_reserved),
            SlotRole::Dynamic
        );
        assert_eq!(
            SlotRole::from_unit_id(4, hp_reserved, lp_reserved),
            SlotRole::Dynamic
        );
    }

    #[test]
    fn test_slot_role_all_dynamic() {
        let no_reserved_role = 0;
        assert_eq!(
            SlotRole::from_unit_id(0, no_reserved_role, no_reserved_role),
            SlotRole::Dynamic
        );
        assert_eq!(
            SlotRole::from_unit_id(5, no_reserved_role, no_reserved_role),
            SlotRole::Dynamic
        );
    }

    #[test]
    fn test_is_hp() {
        let cutoff = 2;
        assert!(SlotRole::is_hp(0, cutoff));
        assert!(SlotRole::is_hp(1, cutoff));
        assert!(!SlotRole::is_hp(2, cutoff));
        assert!(!SlotRole::is_hp(3, cutoff));
        assert!(!SlotRole::is_hp(10, cutoff));

        let cutoff = 1;
        assert!(SlotRole::is_hp(0, cutoff));
        assert!(!SlotRole::is_hp(1, cutoff));
    }

    /// A response as a target would send it back: a status, and optionally a `Retry-After`.
    fn response(http_code: u16, retry_after: Option<&str>) -> Response {
        let headers = retry_after.map(|value| {
            let mut headers = HeaderMap::new();
            headers.insert(
                RETRY_AFTER,
                HeaderValue::from_str(value).expect("valid header value"),
            );
            headers
        });

        Response {
            response_error: (http_code >= 400).then_some(ResponseError::Http),
            http_code: Some(http_code),
            headers,
            body: None,
            elapsed_time: Duration::ZERO,
        }
    }

    fn a_moment() -> DateTime<Utc> {
        DateTime::from_timestamp(1_800_000_000, 0).expect("valid timestamp")
    }

    #[test]
    fn test_parse_retry_after_delay_seconds() {
        let now = a_moment();
        assert_eq!(parse_retry_after("0", now), Some(Duration::ZERO));
        assert_eq!(
            parse_retry_after("120", now),
            Some(Duration::from_secs(120))
        );
        assert_eq!(
            parse_retry_after("  30  ", now),
            Some(Duration::from_secs(30))
        );
    }

    #[test]
    fn test_parse_retry_after_http_date() {
        let now = a_moment();
        let in_five_minutes = (now + chrono::TimeDelta::minutes(5)).to_rfc2822();
        assert_eq!(
            parse_retry_after(&in_five_minutes, now),
            Some(Duration::from_secs(5 * 60))
        );

        // The IMF-fixdate form RFC 9110 asks servers to send, spelled with GMT
        assert_eq!(
            parse_retry_after("Sun, 06 Nov 1994 08:49:37 GMT", now),
            Some(Duration::ZERO),
            "a date already past means the target is ready now, not that the header is unusable"
        );
    }

    #[test]
    fn test_parse_retry_after_rejects_what_it_cannot_trust() {
        let now = a_moment();
        for value in ["", "soon", "-5", "12.5", "300s", "Tomorrow"] {
            assert_eq!(
                parse_retry_after(value, now),
                None,
                "{value:?} must fall back to the base schedule"
            );
        }
    }

    #[test]
    fn test_retry_after_hint_only_reads_rate_limits() {
        let now = a_moment();
        let policy = no_jitter(30);

        assert_eq!(
            policy.retry_after_hint(&response(429, Some("120")), now),
            Some(Duration::from_secs(120))
        );
        assert_eq!(policy.retry_after_hint(&response(429, None), now), None);
        assert_eq!(
            policy.retry_after_hint(&response(429, Some("nope")), now),
            None
        );

        for other_status in [200, 400, 500, 503] {
            assert_eq!(
                policy.retry_after_hint(&response(other_status, Some("120")), now),
                None,
                "only 429 is honoured, got a hint on {other_status}"
            );
        }
    }

    #[test]
    fn test_retry_after_hint_is_bounded() {
        let now = a_moment();
        let policy = no_jitter(30);
        let a_month = 30 * 24 * 3600;

        assert_eq!(
            policy.retry_after_hint(&response(429, Some(&a_month.to_string())), now),
            Some(RetryPolicy::RETRY_AFTER_MAX)
        );
        assert_eq!(
            policy.retry_after_hint(&response(429, Some("99999999999999999999")), now),
            None,
            "a value too large to even be a number is not a delay we can trust"
        );
    }

    #[test]
    fn test_next_delay_honouring_waits_for_what_the_target_asked() {
        let now = a_moment();
        let policy = no_jitter(30);
        let hinted = response(429, Some("600"));

        // The first steps are the ones that hammer: 3s and 10s both give way to the hint
        assert_eq!(
            policy.next_delay_honouring(0, 0.0, &hinted, now),
            Some(Duration::from_secs(600))
        );
        assert_eq!(
            policy.next_delay_honouring(1, 0.0, &hinted, now),
            Some(Duration::from_secs(600))
        );

        // Once the schedule is already slower than the hint, the schedule wins
        assert_eq!(
            policy.next_delay_honouring(3, 0.0, &hinted, now),
            Some(Duration::from_secs(30 * 60))
        );

        // No hint, no change
        assert_eq!(
            policy.next_delay_honouring(0, 0.0, &response(429, None), now),
            policy.next_delay(0, 0.0)
        );
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(512))]

        /// Honouring `Retry-After` can only ever push an attempt further away, and it never
        /// changes how many retries an attempt gets. Both halves matter: the first is what makes
        /// the change safe to ship, the second is what keeps `evaluate` honest about the retry
        /// count.
        #[test]
        fn honouring_retry_after_only_ever_delays(
            retry_count in 0i16..40,
            factor in 0.0f64..=1.0,
            hint_secs in 0u64..1_000_000,
            status in prop::sample::select(vec![200u16, 400, 429, 500, 503]),
            jitter_ratio in 0.0f64..1.0,
        ) {
            let now = a_moment();
            let policy = RetryPolicy {
                max_retries: 24,
                jitter_ratio,
                jitter_max_spread: Duration::from_secs(15 * 60),
            };
            let answer = response(status, Some(&hint_secs.to_string()));

            let base = policy.next_delay(retry_count, factor);
            let honoured = policy.next_delay_honouring(retry_count, factor, &answer, now);

            prop_assert_eq!(
                base.is_some(),
                honoured.is_some(),
                "the number of retries must not depend on a response header"
            );

            if let (Some(base), Some(honoured)) = (base, honoured) {
                prop_assert!(
                    honoured >= base,
                    "honouring Retry-After brought the attempt closer: {honoured:?} < {base:?}"
                );
                prop_assert!(
                    honoured <= base.max(RetryPolicy::RETRY_AFTER_MAX),
                    "honouring Retry-After went past its own bound: {honoured:?}"
                );
            }
        }
    }
}
