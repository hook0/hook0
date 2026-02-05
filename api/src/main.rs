use ::hook0_client::Hook0Client;
use actix_cors::Cors;
use actix_files::{Files, NamedFile};
use actix_web::middleware::{Compat, Logger, NormalizePath};
use actix_web::{App, HttpServer, http, middleware};
use aws_sdk_s3::Client;
use aws_sdk_s3::config::retry::RetryConfig;
use aws_sdk_s3::config::timeout::TimeoutConfig;
use aws_sdk_s3::config::{AppName, Credentials, Region};
use biscuit_auth::{KeyPair, PrivateKey};
use clap::builder::{BoolValueParser, TypedValueParser};
use clap::{ArgGroup, Parser, crate_name, crate_version};
use ipnetwork::IpNetwork;
use lettre::Address;
use log::{debug, error, info, trace, warn};
use paperclip::actix::{OpenApiExt, web};
use pulsar::{
    Authentication, ConnectionRetryOptions, MultiTopicProducer, ProducerOptions, Pulsar,
    TokioExecutor,
};
use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, Semaphore};
use url::Url;
use uuid::Uuid;

mod cloudflare_turnstile;
mod expired_tokens_cleanup;
mod extractor_user_ip;
mod handlers;
mod hook0_client;
mod iam;
mod mailer;
mod materialized_views;
mod middleware_biscuit;
mod middleware_get_user_ip;
mod object_storage_cleanup;
mod old_events_cleanup;
mod onboarding;
mod openapi;
mod opentelemetry;
mod pagination;
mod problems;
mod quotas;
mod rate_limiting;
mod soft_deleted_applications_cleanup;
mod soft_deleted_users_cleanup;
mod unverified_users_cleanup;
mod validators;

#[cfg(feature = "migrate-users-from-keycloak")]
mod keycloak_api;

#[cfg(all(target_env = "msvc", feature = "jemalloc"))]
compile_error!("jemalloc is not supporter when compiling to msvc");

#[cfg(all(not(target_env = "msvc"), feature = "jemalloc"))]
#[global_allocator]
static ALLOC: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

#[cfg(all(not(target_env = "msvc"), feature = "profiling"))]
#[allow(non_upper_case_globals)]
#[unsafe(export_name = "malloc_conf")]
pub static malloc_conf: &[u8] = b"prof:true,prof_active:true,lg_prof_sample:19\0";

const APP_TITLE: &str = "Hook0 API";
const WEBAPP_INDEX_FILE: &str = "index.html";
const PULSAR_CONNECTION_MAX_RETRIES: u32 = 10;

#[derive(Debug, Clone, Parser)]
#[clap(author, about, version, name = APP_TITLE)]
#[clap(group(ArgGroup::new("reverse_proxy").multiple(false)))]
#[clap(group(
    ArgGroup::new("client")
        .multiple(true)
        .requires_all(&["hook0_client_api_url", "hook0_client_application_id", "hook0_client_token"]),
))]
#[clap(group(
    ArgGroup::new("cloudflare_turnstile")
        .multiple(true)
        .requires_all(&["cloudflare_turnstile_site_key", "cloudflare_turnstile_secret_key"]),
))]
#[clap(group(
    ArgGroup::new("pulsar")
        .multiple(true)
        .requires_all(&["pulsar_binary_url", "pulsar_token", "pulsar_tenant", "pulsar_namespace"]),
))]
#[clap(group(
    ArgGroup::new("object_storage")
        .multiple(true)
        .requires_all(&["object_storage_host", "object_storage_key_id", "object_storage_key_secret", "object_storage_bucket_name"]),
))]
struct Config {
    /// [Web Server] IP address on which to start the HTTP server
    #[clap(long, env, default_value = "127.0.0.1")]
    ip: String,

    /// [Web Server] Port on which to start the HTTP server
    #[clap(long, env, default_value = "8080")]
    port: String,

    /// [Reverse Proxy] A comma-separated list of trusted IP addresses (e.g. `192.168.1.1`) or CIDRs (e.g. `192.168.0.0/16`) that are allowed to set "X-Forwarded-For" and "Forwarded" headers
    #[clap(long, env, use_value_delimiter = true, group = "reverse_proxy")]
    reverse_proxy_ips: Vec<IpNetwork>,

    /// [Reverse Proxy] A comma-separated list of trusted IP addresses (e.g. `192.168.1.1`) or CIDRs (e.g. `192.168.0.0/16`) that are allowed to set "X-Forwarded-For" and "Forwarded" headers
    #[clap(long, env, use_value_delimiter = true, group = "reverse_proxy")]
    cc_reverse_proxy_ips: Vec<IpNetwork>,

    /// [Reverse Proxy] Set to true if your instance is served behind Cloudflare's proxies in order to determine the correct user IP for each request
    #[clap(long, env, default_value = "false")]
    behind_cloudflare: bool,

    /// [Monitoring] Optional Sentry DSN for error reporting
    #[clap(long, env)]
    sentry_dsn: Option<String>,

    /// [Monitoring] Optional sample rate for tracing transactions with Sentry (between 0.0 and 1.0)
    #[clap(long, env)]
    sentry_traces_sample_rate: Option<f32>,

    /// [Monitoring] Optional OTLP endpoint that will receive metrics
    #[clap(long, env)]
    otlp_metrics_endpoint: Option<Url>,

    /// [Monitoring] Optional OTLP endpoint that will receive traces
    #[clap(long, env)]
    otlp_traces_endpoint: Option<Url>,

    /// [Monitoring] Optional value for OTLP `Authorization` header (for example: `Bearer mytoken`)
    #[clap(long, env, hide_env_values = true)]
    otlp_authorization: Option<String>,

    /// [Database] Database URL (with credentials)
    #[clap(long, env, hide_env_values = true)]
    database_url: String,

    /// [Database] Maximum number of connections to database
    #[clap(long, env, default_value = "5")]
    max_db_connections: u32,

    /// [Database] Statement timeout for database queries; if `0ms` (default), no timeout will be set; this is only for API-related queries, housekeeping tasks run without timeout
    #[clap(long, env, value_parser = humantime::parse_duration, default_value = "0ms")]
    db_statement_timeout: Duration,

    /// [Pulsar] Pulsar binary URL
    #[clap(long, env, group = "pulsar")]
    pulsar_binary_url: Option<Url>,

    /// [Pulsar] Pulsar token
    #[clap(long, env, hide_env_values = true, group = "pulsar")]
    pulsar_token: Option<String>,

    /// [Pulsar] Pulsar tenant
    #[clap(long, env, group = "pulsar")]
    pulsar_tenant: Option<String>,

    /// [Pulsar] Pulsar namespace
    #[clap(long, env, group = "pulsar")]
    pulsar_namespace: Option<String>,

    /// [Object Storage] Host of the S3-like object storage (without https://)
    #[clap(long, env)]
    object_storage_host: Option<String>,

    /// [Object Storage] Force endpoint scheme to be HTTP (by default it is HTTPS)
    #[clap(long, env, default_value_t = false)]
    object_storage_force_http_scheme: bool,

    /// [Object Storage] Key ID of the S3-like object storage
    #[clap(long, env)]
    object_storage_key_id: Option<String>,

    /// [Object Storage] Key secret of the S3-like object storage
    #[clap(long, env, hide_env_values = true)]
    object_storage_key_secret: Option<String>,

    /// [Object Storage] Maximum number of attempts for object storage operations
    #[clap(long, env, default_value_t = 3)]
    object_storage_max_attempts: u32,

    /// [Object Storage] Connect timeout for object storage operations (time to initiate socket connection)
    #[clap(long, env, value_parser = humantime::parse_duration, default_value = "3s")]
    object_storage_connect_timeout: Duration,

    /// [Object Storage] Read timeout for object storage operations (time to first byte)
    #[clap(long, env, value_parser = humantime::parse_duration, default_value = "5s")]
    object_storage_read_timeout: Duration,

    /// [Object Storage] Operation attempt timeout for object storage operations
    #[clap(long, env, value_parser = humantime::parse_duration, default_value = "10s")]
    object_storage_operation_attempt_timeout: Duration,

    /// [Object Storage] Operation timeout for object storage operations
    #[clap(long, env, value_parser = humantime::parse_duration, default_value = "30s")]
    object_storage_operation_timeout: Duration,

    /// [Object Storage] Bucket name of the S3-like object storage
    #[clap(long, env)]
    object_storage_bucket_name: Option<String>,

    /// [Object Storage] If true, new event payloads will be stored in object storage instead of database
    #[clap(long, env, default_value_t = false)]
    store_event_payloads_in_object_storage: bool,

    /// [Object Storage] A comma-separated list of applications ID whose event payloads should be stored in object storage; if empty (default), all event payloads will be stored in object storage regardless of application ID
    #[clap(long, env, use_value_delimiter = true)]
    store_event_payloads_in_object_storage_only_for: Vec<Uuid>,

    /// [Frontend] Path to the directory containing the web app to serve
    #[clap(long, env, default_value = "../frontend/dist/")]
    webapp_path: String,

    /// [Frontend] Set to true to disable serving the web app and only serve the API
    #[clap(long, env)]
    disable_serving_webapp: bool,

    /// [Monitoring] Key for the health check endpoint; if not specified, endpoint is disabled; if empty, endpoint is public
    #[clap(long, env, hide_env_values = true)]
    health_check_key: Option<String>,

    /// [Monitoring] Max timeout duration for health check: if subsystems take longer to respond they will be considered unhealthy
    #[clap(long, env, value_parser = humantime::parse_duration, default_value = "5s")]
    health_check_timeout: Duration,

    /// [Deprecated] Enable Keycloak migration mode
    #[cfg(feature = "migrate-users-from-keycloak")]
    #[clap(long, env, default_value = "true")]
    enable_keycloak_migration: bool,

    /// [Deprecated] Enable application secret compatibility mode
    #[cfg(feature = "application-secret-compatibility")]
    #[clap(long, env, default_value = "true")]
    enable_application_secret_compatibility: bool,

    /// [Deprecated] Keycloak RS256 public key (with GPG delimiters)
    #[cfg(feature = "migrate-users-from-keycloak")]
    #[clap(long, env)]
    keycloak_oidc_public_key: String,

    /// [Auth] Biscuit's private key, used for authentication
    #[clap(long, env, value_parser = parse_biscuit_private_key)]
    biscuit_private_key: Option<PrivateKey>,

    /// [Database] Disable automatic database migration
    #[clap(long = "no-auto-db-migration", env = "NO_AUTO_DB_MIGRATION", value_parser = BoolValueParser::new().map(|v| !v))]
    auto_db_migration: bool,

    /// [Auth] A global admin API key that have almost all rights. Better left undefined, USE AT YOUR OWN RISKS!
    #[clap(long, env, hide_env_values = true)]
    master_api_key: Option<Uuid>,

    /// [Deprecated] URL of a Keycloak instance (example: https://my.keycloak.net/auth)
    #[cfg(feature = "migrate-users-from-keycloak")]
    #[clap(long, env)]
    keycloak_url: Url,

    /// [Deprecated] Keycloak realm
    #[clap(long, env)]
    #[cfg(feature = "migrate-users-from-keycloak")]
    keycloak_realm: String,

    /// [Deprecated] OIDC client ID (the confidential client for Hook0 API)
    #[clap(long, env)]
    #[cfg(feature = "migrate-users-from-keycloak")]
    keycloak_client_id: String,

    /// [Deprecated] OIDC client secret (the confidential client for Hook0 API)
    #[cfg(feature = "migrate-users-from-keycloak")]
    #[clap(long, env, hide_env_values = true)]
    keycloak_client_secret: String,

    /// [Auth] Set to true to disable registration endpoint
    #[clap(long, env)]
    disable_registration: bool,

    /// [Auth] Minimum length of user passwords. This is checked when a user registers.
    #[clap(long, env, default_value = "12")]
    password_minimum_length: u8,

    /// [Rate Limiting] Set to true to disable every API rate limiting
    #[clap(long, env)]
    disable_api_rate_limiting: bool,

    /// [Rate Limiting] Set to true to disable global API rate limiting
    #[clap(long, env)]
    disable_api_rate_limiting_global: bool,

    /// [Rate Limiting] Global quota of API calls before rate limiting blocks incomming requests (must be ≥ 1)
    #[clap(long, env, default_value = "2000")]
    api_rate_limiting_global_burst_size: u32,

    /// [Rate Limiting] Duration (in millisecond) after which one global API call is restored in the quota (must be ≥ 1)
    #[clap(long, env, default_value = "1")]
    api_rate_limiting_global_replenish_period_in_ms: u64,

    /// [Rate Limiting] Set to true to disable per-IP API rate limiting
    #[clap(long, env)]
    disable_api_rate_limiting_ip: bool,

    /// [Rate Limiting] Quota of API calls per IP before rate limiting blocks incomming requests (must be ≥ 1)
    #[clap(long, env, default_value = "200")]
    api_rate_limiting_ip_burst_size: u32,

    /// [Rate Limiting] Duration (in millisecond) after which one API call per IP is restored in the quota (must be ≥ 1)
    #[clap(long, env, default_value = "10")]
    api_rate_limiting_ip_replenish_period_in_ms: u64,

    /// [Rate Limiting] Set to true to disable per-token API rate limiting
    #[clap(long, env)]
    disable_api_rate_limiting_token: bool,

    /// [Rate Limiting] Quota of API calls per token before rate limiting blocks incomming requests (must be ≥ 1)
    #[clap(long, env, default_value = "20")]
    api_rate_limiting_token_burst_size: u32,

    /// [Rate Limiting] Duration (in millisecond) after which one API call per token is restored in the quota (must be ≥ 1)
    #[clap(long, env, default_value = "100")]
    api_rate_limiting_token_replenish_period_in_ms: u64,

    /// [Rate Limiting] Duration to wait beetween rate limiters housekeeping
    #[clap(long, env, value_parser = humantime::parse_duration, default_value = "5m")]
    api_rate_limiting_housekeeping_period: Duration,

    /// [Web Server] Comma-separated allowed origins for CORS
    #[clap(long, env, use_value_delimiter = true)]
    cors_allowed_origins: Vec<String>,

    /// [Hook0 Client] Base API URL of a Hook0 instance that will receive events from this Hook0 instance
    #[clap(long, env, group = "client")]
    hook0_client_api_url: Option<Url>,

    /// [Hook0 Client] UUID of a Hook0 application that will receive events from this Hook0 instance
    #[clap(long, env, group = "client")]
    hook0_client_application_id: Option<Uuid>,

    /// [Hook0 Client] Authentication token valid for a Hook0 application that will receive events from this Hook0 instance
    #[clap(long, env, group = "client")]
    hook0_client_token: Option<String>,

    /// [Hook0 Client] Number of allowed retries when upserting event types to the linked Hook0 application fails
    #[clap(long, env, default_value = "10")]
    hook0_client_upserts_retries: u16,

    /// [Quotas] Set to true to apply quotas limits (default is not to)
    #[clap(long, env)]
    enable_quota_enforcement: bool,

    /// [Quotas] Default limit of members per organization (can be overriden by a plan)
    #[clap(long, env, default_value = "1")]
    quota_global_members_per_organization_limit: quotas::QuotaValue,

    /// [Quotas] Default limit of applications per organization (can be overriden by a plan)
    #[clap(long, env, default_value = "1")]
    quota_global_applications_per_organization_limit: quotas::QuotaValue,

    /// [Quotas] Default limit of events per day (can be overriden by a plan)
    #[clap(long, env, default_value = "100")]
    quota_global_events_per_day_limit: quotas::QuotaValue,

    /// [Quotas] Default limit of day of event's retention (can be overriden by a plan)
    #[clap(long, env, default_value = "7")]
    quota_global_days_of_events_retention_limit: quotas::QuotaValue,

    /// [Quotas] Default limit of subscriptions per application (can be overriden by a plan)
    #[clap(long, env, default_value = "10")]
    quota_global_subscriptions_per_application_limit: quotas::QuotaValue,

    /// [Quotas] Default limit of event types per application (can be overriden by a plan)
    #[clap(long, env, default_value = "10")]
    quota_global_event_types_per_application_limit: quotas::QuotaValue,

    /// [Quotas] Default threshold (in %) of events per day at which to send a warning notification
    #[clap(long, env, default_value = "80")]
    quota_notification_events_per_day_threshold: u8,

    /// [Quotas] Set to true to enable quota-based email notifications
    #[clap(long, env, default_value = "false")]
    enable_quota_based_email_notifications: bool,

    /// [Housekeeping] Duration (in second) to wait between materialized views refreshes
    #[clap(long, env, default_value = "60")]
    materialized_views_refresh_period_in_s: u64,

    /// [Housekeeping] Duration (in second) to wait between old events cleanups
    #[clap(long, env, default_value = "3600")]
    old_events_cleanup_period_in_s: u64,

    /// [Housekeeping] Duration (in day) to wait before actually deleting events that are passed retention period
    #[clap(long, env, default_value = "30")]
    old_events_cleanup_grace_period_in_day: u16,

    /// [Housekeeping] If true, old events will be reported and cleaned up; if false (default), they will only be reported
    #[clap(long, env, default_value = "false")]
    old_events_cleanup_report_and_delete: bool,

    /// [Housekeeping] Duration to wait between object storage cleanups
    #[clap(long, env, value_parser = humantime::parse_duration, default_value = "1d")]
    object_storage_cleanup_period: Duration,

    /// [Housekeeping] If true, allow to delete outdated objects from object storage; if false (default), they will only be reported
    #[clap(long, env, default_value_t = false)]
    object_storage_cleanup_report_and_delete: bool,

    /// [Housekeeping] Duration to wait between expired tokens cleanups
    #[clap(long, env, value_parser = humantime::parse_duration, default_value = "1h")]
    expired_tokens_cleanup_period: Duration,

    /// [Housekeeping] Duration to wait before actually deleting expired tokens (expired tokens cannot be used anyway, even if kept for some time)
    #[clap(long, env, value_parser = humantime::parse_duration, default_value = "7d")]
    expired_tokens_cleanup_grace_period: Duration,

    /// [Housekeeping] If true, expired tokens will be reported and cleaned up; if false (default), they will only be reported
    #[clap(long, env, default_value = "false")]
    expired_tokens_cleanup_report_and_delete: bool,

    /// [Housekeeping] If true, unverified users will be remove from database after a while
    #[clap(long, env, default_value = "false")]
    enable_unverified_users_cleanup: bool,

    /// [Housekeeping] Duration (in second) to wait between unverified users cleanups
    #[clap(long, env, default_value = "3600")]
    unverified_users_cleanup_period_in_s: u64,

    /// [Housekeeping] Duration (in day) to wait before removing a unverified user
    #[clap(long, env, default_value = "7")]
    unverified_users_cleanup_grace_period_in_days: u32,

    /// [Housekeeping] If true, unverified users will be reported and cleaned up; if false (default), they will only be reported
    #[clap(long, env, default_value = "false")]
    unverified_users_cleanup_report_and_delete: bool,

    /// [Housekeeping] If true, users who requested account deletion will be soft-deleted after 30 days (GDPR Art. 17)
    #[clap(long, env, default_value_t = false)]
    enable_deleted_users_cleanup: bool,

    /// [Housekeeping] Duration to wait between deleted users cleanups
    #[clap(long, env, value_parser = humantime::parse_duration, default_value = "1h")]
    deleted_users_cleanup_period: Duration,

    /// [Housekeeping] If true, users requesting deletion will be soft-deleted; if false (default), they will only be reported
    #[clap(long, env, default_value_t = false)]
    deleted_users_cleanup_report_and_delete: bool,

    /// [Housekeeping] If true, soft-deleted applications will be removed from database after a while; otherwise they will be kept in database forever
    #[clap(long, env, default_value = "false")]
    enable_soft_deleted_applications_cleanup: bool,

    /// [Housekeeping] Duration to wait between soft-deleted applications cleanups
    #[clap(long, env, value_parser = humantime::parse_duration, default_value = "1d")]
    soft_deleted_applications_cleanup_period: Duration,

    /// [Housekeeping] Duration to wait before removing a soft-deleted application
    #[clap(long, env, value_parser = humantime::parse_duration, default_value = "30d")]
    soft_deleted_applications_cleanup_grace_period: Duration,

    /// [Web Server] If true, the secured HTTP headers will be enabled
    #[clap(long, env, default_value = "true")]
    enable_security_headers: bool,

    /// [Web Server] If true, the HSTS header will be enabled
    #[clap(long, env, default_value = "false")]
    enable_hsts_header: bool,

    /// [Email] Sender email address
    #[clap(long, env)]
    email_sender_address: Address,

    /// [Email] Sender name
    #[clap(long, env, default_value = "Hook0")]
    email_sender_name: String,

    /// [Email] Connection URL to SMTP server; for example: `smtp://localhost:1025`, `smtps://user:password@provider.com:465` (SMTP over TLS) or `smtp://user:password@provider.com:465?tls=required` (SMTP with STARTTLS)
    #[clap(long, env, hide_env_values = true)]
    smtp_connection_url: String,

    /// [Email] Duration (in second) to use as timeout when sending emails to the SMTP server
    #[clap(long, env, default_value = "5")]
    smtp_timeout_in_s: u64,

    /// [Frontend] URL of the Hook0 logo
    #[clap(long, env, default_value = "https://app.hook0.com/256x256.png")]
    email_logo_url: Url,

    /// [Frontend] Frontend application URL (used for building links in emails and pagination)
    #[clap(long, env)]
    app_url: Url,

    /// [Auth] Maximum duration (in millisecond) that can be spent running Biscuit's authorizer
    #[clap(long, env, default_value = "10")]
    max_authorization_time_in_ms: u64,

    /// [Auth] If true, a trace log message containing authorizer context is emitted on each request; default is false because this feature implies a small overhead
    #[clap(long, env, default_value_t = false)]
    debug_authorizer: bool,

    /// [Frontend] Matomo URL
    #[clap(long, env)]
    matomo_url: Option<Url>,

    /// [Frontend] Matomo site ID
    #[clap(long, env)]
    matomo_site_id: Option<u16>,

    /// [Frontend] Formbricks API host
    #[clap(long, env, default_value = "https://app.formbricks.com")]
    formbricks_api_host: String,

    /// [Frontend] Formbricks API environment ID
    #[clap(long, env)]
    formbricks_environment_id: Option<String>,

    /// [Frontend] Website URL
    #[clap(long, env, default_value = "https://hook0.com")]
    website_url: Url,

    /// [Frontend] Support email address
    #[clap(long, env, default_value = "support@hook0.com")]
    support_email_address: Address,

    /// [Frontend] Cloudflare Turnstile site key (enables Turnstile for user registration)
    #[clap(long, env, group = "cloudflare_turnstile")]
    cloudflare_turnstile_site_key: Option<String>,

    /// [Frontend] Cloudflare Turnstile secret key (enables Turnstile for user registration)
    #[clap(long, env, group = "cloudflare_turnstile")]
    cloudflare_turnstile_secret_key: Option<String>,
}

fn parse_biscuit_private_key(input: &str) -> Result<PrivateKey, String> {
    PrivateKey::from_bytes_hex(input, biscuit_auth::Algorithm::Ed25519)
        .map_err(|e| format!("Value of BISCUIT_PRIVATE_KEY is invalid ({e}). Re-run this app without the environment variable set to get a randomly generated key."))
}

/// The app state
#[derive(Debug, Clone)]
pub struct State {
    db: PgPool,
    pulsar: Option<Arc<PulsarConfig>>,
    object_storage: Option<ObjectStorageConfig>,
    biscuit_private_key: PrivateKey,
    mailer: mailer::Mailer,
    app_url: Url,
    #[cfg(feature = "migrate-users-from-keycloak")]
    enable_keycloak_migration: bool,
    #[cfg(feature = "migrate-users-from-keycloak")]
    keycloak_url: Url,
    #[cfg(feature = "migrate-users-from-keycloak")]
    keycloak_realm: String,
    #[cfg(feature = "migrate-users-from-keycloak")]
    keycloak_client_id: String,
    #[cfg(feature = "migrate-users-from-keycloak")]
    keycloak_client_secret: String,
    application_secret_compatibility: bool,
    registration_disabled: bool,
    password_minimum_length: u8,
    auto_db_migration: bool,
    hook0_client: Option<Hook0Client>,
    quotas: quotas::Quotas,
    health_check_key: Option<String>,
    health_check_timeout: Duration,
    max_authorization_time_in_ms: u64,
    debug_authorizer: bool,
    enable_quota_enforcement: bool,
    matomo_url: Option<Url>,
    matomo_site_id: Option<u16>,
    formbricks_api_host: String,
    formbricks_environment_id: Option<String>,
    quota_notification_events_per_day_threshold: u8,
    enable_quota_based_email_notifications: bool,
    support_email_address: Address,
    cloudflare_turnstile_site_key: Option<String>,
    cloudflare_turnstile_secret_key: Option<String>,
}

#[derive(Clone)]
struct PulsarConfig {
    pulsar: Pulsar<TokioExecutor>,
    tenant: String,
    namespace: String,
    request_attempts_producer: Arc<Mutex<MultiTopicProducer<TokioExecutor>>>,
}

#[derive(Debug, Clone)]
struct ObjectStorageConfig {
    client: Client,
    bucket: String,
    store_event_payloads: bool,
    store_event_only_for: Vec<Uuid>,
}

// A Debug implementation that gets around Pulsar<TokioExecutor> not implementing Debug
impl std::fmt::Debug for PulsarConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        #[allow(dead_code)]
        #[derive(Debug)]
        struct PulsarConfigForDebug<'a> {
            pulsar: &'a str,
            tenant: &'a str,
            namespace: &'a str,
        }

        std::fmt::Debug::fmt(
            &PulsarConfigForDebug {
                pulsar: "[Pulsar connection]",
                tenant: &self.tenant,
                namespace: &self.namespace,
            },
            f,
        )
    }
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::parse();

    if let Some(biscuit_private_key) = config.biscuit_private_key {
        // Initialize app logger as well as Sentry integration
        // Return value *must* be kept in a variable or else it will be dropped and Sentry integration won't work
        let _sentry = hook0_sentry_integration::init(
            crate_name!(),
            &config.sentry_dsn,
            &config.sentry_traces_sample_rate,
        );

        // Init OpenTelemetry
        opentelemetry::init(
            crate_version!(),
            &config.otlp_authorization,
            &config.otlp_metrics_endpoint,
            &config.otlp_traces_endpoint,
        )?;

        trace!("Starting {APP_TITLE}");

        // Prepare trusted reverse proxies CIDRs
        let reverse_proxy_cidrs = if config.reverse_proxy_ips.is_empty() {
            config.cc_reverse_proxy_ips
        } else {
            config.reverse_proxy_ips
        };
        if reverse_proxy_cidrs.is_empty() {
            warn!(
                "No trusted reverse proxy CIDRs were set; if this is a production instance this is a problem"
            );
        } else {
            debug!(
                "The following CIDRs will be considered as trusted reverse proxies: {}",
                reverse_proxy_cidrs
                    .iter()
                    .map(|ip| ip.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
        info!(
            "{} reverse proxy CIDRs are trusted{}",
            reverse_proxy_cidrs.len(),
            if config.behind_cloudflare {
                " (in addition to Cloudflare's)"
            } else {
                ""
            }
        );

        // Prepare rate limiting configuration
        let rate_limiters = rate_limiting::Hook0RateLimiters::new(
            config.disable_api_rate_limiting,
            config.disable_api_rate_limiting_global,
            config.api_rate_limiting_global_burst_size,
            config.api_rate_limiting_global_replenish_period_in_ms,
            config.disable_api_rate_limiting_ip,
            config.api_rate_limiting_ip_burst_size,
            config.api_rate_limiting_ip_replenish_period_in_ms,
            config.disable_api_rate_limiting_token,
            config.api_rate_limiting_token_burst_size,
            config.api_rate_limiting_token_replenish_period_in_ms,
        );

        // Create a DB connection pool for housekeeping tasks (no timeout)
        let housekeeping_pool = PgPoolOptions::new()
            .max_connections(3)
            .connect_with(
                PgConnectOptions::from_str(&config.database_url)?
                    .application_name(&format!("{}-housekeeping", crate_name!())),
            )
            .await?;

        // Create a DB connection pool for API
        let statement_timeout = config.db_statement_timeout;
        let pool = PgPoolOptions::new()
            .max_connections(config.max_db_connections)
            .after_connect(move |conn, _meta| {
                Box::pin(async move {
                    if !statement_timeout.is_zero() {
                        sqlx::Executor::execute(
                            conn,
                            format!("SET statement_timeout = {}", statement_timeout.as_millis())
                                .as_str(),
                        )
                        .await?;
                    }
                    Ok(())
                })
            })
            .connect_with(
                PgConnectOptions::from_str(&config.database_url)?.application_name(crate_name!()),
            )
            .await?;
        info!(
            "Started a pool of maximum {} DB connections",
            &config.max_db_connections
        );

        // Periodically collect metrics from database pools
        let metrics_pools = [
            ("housekeeping", housekeeping_pool.clone()),
            ("main", pool.clone()),
        ];
        actix_web::rt::spawn(async move {
            loop {
                opentelemetry::gather_pools_metrics(&metrics_pools);
                actix_web::rt::time::sleep(Duration::from_secs(15)).await
            }
        });

        // Run migrations
        if config.auto_db_migration {
            info!("Checking/running DB migrations");
            sqlx::migrate!("./migrations")
                .run(&housekeeping_pool)
                .await?;
        }

        // Create Pulsar client
        let pulsar_config = if let (
            Some(pulsar_binary_url),
            Some(pulsar_token),
            Some(pulsar_tenant),
            Some(pulsar_namespace),
        ) = (
            config.pulsar_binary_url,
            config.pulsar_token,
            config.pulsar_tenant,
            config.pulsar_namespace,
        ) {
            rustls::crypto::aws_lc_rs::default_provider()
                .install_default()
                .unwrap();

            match Pulsar::builder(pulsar_binary_url, TokioExecutor)
                .with_connection_retry_options(ConnectionRetryOptions {
                    max_retries: PULSAR_CONNECTION_MAX_RETRIES,
                    ..Default::default()
                })
                .with_auth(Authentication {
                    name: "token".to_owned(),
                    data: pulsar_token.into_bytes(),
                })
                .build()
                .await
            {
                Ok(pulsar) => {
                    let request_attempts_producer = pulsar
                        .producer()
                        .with_name(format!(
                            "hook0-api.request-attempts-producer.{}",
                            Uuid::now_v7()
                        ))
                        .with_options(ProducerOptions {
                            block_queue_if_full: true,
                            ..Default::default()
                        })
                        .build_multi_topic();

                    info!("Pulsar support is enabled");
                    Some(Arc::new(PulsarConfig {
                        pulsar,
                        tenant: pulsar_tenant,
                        namespace: pulsar_namespace,
                        request_attempts_producer: Arc::new(Mutex::new(request_attempts_producer)),
                    }))
                }
                Err(e) => {
                    error!(
                        "Could not connect to Pulsar after {PULSAR_CONNECTION_MAX_RETRIES} attempts: {e}"
                    );
                    warn!("Continuing without Pulsar support (restart to try again)");
                    None
                }
            }
        } else {
            None
        };

        // Create object storage client
        let object_storage_config = if let (
            Some(object_storage_host),
            Some(object_storage_key_id),
            Some(object_storage_key_secret),
            Some(object_storage_bucket_name),
        ) = (
            config.object_storage_host,
            config.object_storage_key_id,
            config.object_storage_key_secret,
            config.object_storage_bucket_name,
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
                    "{}://{object_storage_host}",
                    if config.object_storage_force_http_scheme {
                        "http"
                    } else {
                        "https"
                    },
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
                .bucket(&object_storage_bucket_name)
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
                    bucket: object_storage_bucket_name,
                    store_event_payloads: config.store_event_payloads_in_object_storage,
                    store_event_only_for: config.store_event_payloads_in_object_storage_only_for,
                })
            }
        } else {
            None
        };

        // Initialize Hook0 client
        let hook0_client = hook0_client::initialize(
            config.hook0_client_api_url.clone(),
            config.hook0_client_application_id,
            config.hook0_client_token,
        );
        if let Some(client) = &hook0_client {
            let upsert_client = client.clone();
            let upserts_retries = config.hook0_client_upserts_retries;
            actix_web::rt::spawn(async move {
                trace!("Starting Hook0 client upsert task");
                hook0_client::upsert_event_types(
                    &upsert_client,
                    hook0_client::EVENT_TYPES,
                    upserts_retries,
                )
                .await;
            });
        }

        // Create an instance of QuotaLimits
        let quota_limits = quotas::QuotaLimits {
            global_members_per_organization_limit: config
                .quota_global_members_per_organization_limit,
            global_applications_per_organization_limit: config
                .quota_global_applications_per_organization_limit,
            global_events_per_day_limit: config.quota_global_events_per_day_limit,
            global_days_of_events_retention_limit: config
                .quota_global_days_of_events_retention_limit,
            global_subscriptions_per_application_limit: config
                .quota_global_subscriptions_per_application_limit,
            global_event_types_per_application_limit: config
                .quota_global_event_types_per_application_limit,
        };

        // Initialize quotas manager
        let quotas = quotas::Quotas::new(config.enable_quota_enforcement, quota_limits);

        if config.enable_quota_enforcement {
            info!("Quota enforcement is enabled");
        } else {
            info!("Quota enforcement is disabled");
        }

        // Prepare master API key
        let master_api_key = config.master_api_key;
        if master_api_key.is_some() {
            warn!(
                "The master API key is defined in the current configuration; THIS MAY BE A SECURITY ISSUE IN PRODUCTION"
            );
        }

        // Spawn tasks to do regular rate limiters housekeeping and metrics reporting
        rate_limiters.spawn_housekeeping_task(config.api_rate_limiting_housekeeping_period);
        rate_limiters.spawn_metrics_task();

        // This semaphore is used to ensure we do not run multiple housekeeping tasks at the same because it can create unnecessary load on the database
        let housekeeping_semaphore = Arc::new(Semaphore::new(1));

        // Spawn task to refresh materialized views
        let refresh_db = housekeeping_pool.clone();
        let refresh_housekeeping_semaphore = housekeeping_semaphore.clone();
        actix_web::rt::spawn(async move {
            materialized_views::periodically_refresh_materialized_views(
                &refresh_housekeeping_semaphore,
                &refresh_db,
                Duration::from_secs(config.materialized_views_refresh_period_in_s),
            )
            .await;
        });

        // Spawn task to clean up soft deleted applications
        if config.enable_soft_deleted_applications_cleanup {
            let clean_soft_deleted_applications_db = housekeeping_pool.clone();
            let clean_soft_deleted_applications_housekeeping_semaphore =
                housekeeping_semaphore.clone();
            actix_web::rt::spawn(async move {
                soft_deleted_applications_cleanup::periodically_clean_up_soft_deleted_applications(
                    &clean_soft_deleted_applications_housekeeping_semaphore,
                    &clean_soft_deleted_applications_db,
                    config.soft_deleted_applications_cleanup_period,
                    config.soft_deleted_applications_cleanup_grace_period,
                )
                .await;
            });
        }

        // Spawn task to clean up old events
        let cleanup_db = housekeeping_pool.clone();
        let cleanup_semaphore = housekeeping_semaphore.clone();
        actix_web::rt::spawn(async move {
            old_events_cleanup::periodically_clean_up_old_events(
                &cleanup_semaphore,
                &cleanup_db,
                Duration::from_secs(config.old_events_cleanup_period_in_s),
                config.quota_global_days_of_events_retention_limit,
                config.old_events_cleanup_grace_period_in_day,
                config.old_events_cleanup_report_and_delete,
            )
            .await;
        });

        // Spawn task to clean up expired tokens
        let cleanup_db = housekeeping_pool.clone();
        let cleanup_semaphore = housekeeping_semaphore.clone();
        actix_web::rt::spawn(async move {
            expired_tokens_cleanup::periodically_clean_up_expired_tokens(
                &cleanup_semaphore,
                &cleanup_db,
                config.expired_tokens_cleanup_period,
                config.expired_tokens_cleanup_grace_period,
                config.expired_tokens_cleanup_report_and_delete,
            )
            .await;
        });

        // Spawn task to clean unverified users if enabled
        if config.enable_unverified_users_cleanup {
            let clean_unverified_users_db = housekeeping_pool.clone();
            let clean_unverified_users_semaphore = housekeeping_semaphore.clone();
            actix_web::rt::spawn(async move {
                unverified_users_cleanup::periodically_clean_up_unverified_users(
                    &clean_unverified_users_semaphore,
                    &clean_unverified_users_db,
                    Duration::from_secs(config.unverified_users_cleanup_period_in_s),
                    config.unverified_users_cleanup_grace_period_in_days,
                    config.unverified_users_cleanup_report_and_delete,
                )
                .await;
            });
        }

        // Spawn task to soft-delete users who requested account deletion (GDPR Art. 17)
        if config.enable_deleted_users_cleanup {
            let clean_deleted_users_db = housekeeping_pool.clone();
            let clean_deleted_users_semaphore = housekeeping_semaphore.clone();
            actix_web::rt::spawn(async move {
                soft_deleted_users_cleanup::periodically_clean_up_soft_deleted_users(
                    &clean_deleted_users_semaphore,
                    &clean_deleted_users_db,
                    config.deleted_users_cleanup_period,
                    config.deleted_users_cleanup_report_and_delete,
                )
                .await;
            });
        }

        // Spawn task to clean up object storage
        // No housekeeping semaphore here because this task is not database-intensive and should be able to run for a long time without keeping other tasks from running
        if let Some(os) = &object_storage_config {
            let cleanup_db = housekeeping_pool.clone();
            let cleanup_object_storage = os.clone();
            actix_web::rt::spawn(async move {
                object_storage_cleanup::periodically_clean_up_object_storage(
                    &cleanup_db,
                    &cleanup_object_storage,
                    config.object_storage_cleanup_period,
                    config.object_storage_cleanup_report_and_delete,
                )
                .await;
            });
        }

        // Create Mailer
        let smtp_config = mailer::MailerSmtpConfig {
            smtp_connection_url: config.smtp_connection_url,
            smtp_timeout: Duration::from_secs(config.smtp_timeout_in_s),
            sender_name: config.email_sender_name,
            sender_address: config.email_sender_address,
        };
        let mailer = mailer::Mailer::new(
            smtp_config,
            config.email_logo_url,
            config.website_url,
            config.app_url.clone(),
            config.support_email_address.clone(),
        )
        .await
        .expect("Could not initialize mailer; check SMTP configuration");

        // Initialize state
        let initial_state = State {
            db: pool,
            pulsar: pulsar_config,
            object_storage: object_storage_config,
            app_url: config.app_url.clone(),
            biscuit_private_key,
            mailer,
            #[cfg(feature = "migrate-users-from-keycloak")]
            enable_keycloak_migration: config.enable_keycloak_migration,
            #[cfg(feature = "migrate-users-from-keycloak")]
            keycloak_url: config.keycloak_url,
            #[cfg(feature = "migrate-users-from-keycloak")]
            keycloak_realm: config.keycloak_realm,
            #[cfg(feature = "migrate-users-from-keycloak")]
            keycloak_client_id: config.keycloak_client_id,
            #[cfg(feature = "migrate-users-from-keycloak")]
            keycloak_client_secret: config.keycloak_client_secret,
            application_secret_compatibility: {
                #[cfg(feature = "application-secret-compatibility")]
                {
                    config.enable_application_secret_compatibility
                }
                #[cfg(not(feature = "application-secret-compatibility"))]
                {
                    false
                }
            },
            registration_disabled: config.disable_registration,
            password_minimum_length: config.password_minimum_length,
            auto_db_migration: config.auto_db_migration,
            hook0_client,
            quotas,
            health_check_key: config.health_check_key,
            health_check_timeout: config.health_check_timeout,
            max_authorization_time_in_ms: config.max_authorization_time_in_ms,
            debug_authorizer: config.debug_authorizer,
            enable_quota_enforcement: config.enable_quota_enforcement,
            matomo_url: config.matomo_url,
            matomo_site_id: config.matomo_site_id,
            formbricks_api_host: config.formbricks_api_host,
            formbricks_environment_id: config.formbricks_environment_id,
            quota_notification_events_per_day_threshold: config
                .quota_notification_events_per_day_threshold,
            enable_quota_based_email_notifications: config.enable_quota_based_email_notifications,
            support_email_address: config.support_email_address,
            cloudflare_turnstile_site_key: config.cloudflare_turnstile_site_key,
            cloudflare_turnstile_secret_key: config.cloudflare_turnstile_secret_key,
        };

        // Run web server
        let webapp_path = config.webapp_path.clone();
        let app_url = config.app_url;
        HttpServer::new(move || {
            // Compute default OpenAPI spec
            let spec = openapi::default_spec(&app_url);

            // Prepare user IP extraction middleware
            let get_user_ip = middleware_get_user_ip::GetUserIp {
                reverse_proxy_cidrs: reverse_proxy_cidrs.clone(),
                behind_cloudflare: config.behind_cloudflare,
            };

            // Prepare CORS configuration
            let cors = {
                let mut c = Cors::default()
                    .allowed_headers([
                        http::header::ACCEPT,
                        http::header::AUTHORIZATION,
                        http::header::CONTENT_TYPE,
                    ])
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
                    .max_age(3600);

                for origin in &config.cors_allowed_origins {
                    c = c.allowed_origin(origin);
                }

                c
            };

            // Prepare auth middleware
            let biscuit_auth = middleware_biscuit::BiscuitAuth {
                db: initial_state.db.clone(),
                biscuit_private_key: initial_state.biscuit_private_key.clone(),
                master_api_key,
                #[cfg(feature = "application-secret-compatibility")]
                enable_application_secret_compatibility: config
                    .enable_application_secret_compatibility,
            };

            let security_headers = middleware::DefaultHeaders::new()
                .add(("X-Content-Type-Options", "nosniff"))
                .add(("Referrer-Policy", "strict-origin-when-cross-origin"))
                .add(("X-XSS-Protection", "1; mode=block"))
                .add(("Referrer-Policy", "SAMEORIGIN"))
                .add(("X-Frame-Options", "DENY"));

            let hsts_header = middleware::DefaultHeaders::new()
                .add(("Strict-Transport-Security", "max-age=63072000"));

            let security_headers_condition =
                middleware::Condition::new(config.enable_security_headers, security_headers);

            let hsts_header_condition =
                middleware::Condition::new(config.enable_hsts_header, hsts_header);

            let mut app = App::new()
                .app_data(web::Data::new(initial_state.clone()))
                .app_data(web::JsonConfig::default().error_handler(|e, _req| {
                    let problem =
                        problems::Hook0Problem::JsonPayload(problems::JsonPayloadProblem::from(e));
                    actix_web::error::Error::from(problem)
                }))
                .wrap(get_user_ip)
                .wrap(hsts_header_condition)
                .wrap(security_headers_condition)
                .wrap(cors)
                .wrap(Logger::default())
                .wrap(NormalizePath::trim())
                .wrap(sentry_actix::Sentry::new())
                .wrap_api_with_spec(spec)
                .with_json_spec_v3_at("/api/v1/swagger.json")
                .service(
                    web::scope("/api/v1")
                        .wrap(Compat::new(rate_limiters.ip()))
                        .wrap(Compat::new(rate_limiters.global()))
                        .service(
                            web::scope("/auth")
                                .service(
                                    web::resource("/verify-email")
                                        .route(web::post().to(handlers::auth::verify_email)),
                                )
                                .service(
                                    web::resource("/login")
                                        .route(web::post().to(handlers::auth::login)),
                                )
                                .service(
                                    web::resource("/refresh")
                                        .wrap(Compat::new(rate_limiters.token())) // Middleware order is counter intuitive: this is executed second
                                        .wrap(biscuit_auth.clone()) // Middleware order is counter intuitive: this is executed first
                                        .route(web::post().to(handlers::auth::refresh)),
                                )
                                .service(
                                    web::resource("/logout")
                                        .wrap(Compat::new(rate_limiters.token())) // Middleware order is counter intuitive: this is executed second
                                        .wrap(biscuit_auth.clone()) // Middleware order is counter intuitive: this is executed first
                                        .route(web::post().to(handlers::auth::logout)),
                                )
                                .service(
                                    web::resource("/begin-reset-password").route(
                                        web::post().to(handlers::auth::begin_reset_password),
                                    ),
                                )
                                .service(
                                    web::resource("/reset-password")
                                        .route(web::post().to(handlers::auth::reset_password)),
                                )
                                .service(
                                    web::resource("/password")
                                        .wrap(Compat::new(rate_limiters.token())) // Middleware order is counter intuitive: this is executed second
                                        .wrap(biscuit_auth.clone()) // Middleware order is counter intuitive: this is executed first
                                        .route(web::post().to(handlers::auth::change_password)),
                                ),
                        )
                        .service(
                            web::scope("/account")
                                .service(
                                    web::resource("")
                                        .wrap(Compat::new(rate_limiters.token()))
                                        .wrap(biscuit_auth.clone())
                                        .route(
                                            web::delete().to(handlers::account::request_deletion),
                                        ),
                                )
                                .service(
                                    web::resource("/deletion-status")
                                        .wrap(Compat::new(rate_limiters.token()))
                                        .wrap(biscuit_auth.clone())
                                        .route(
                                            web::get().to(handlers::account::get_deletion_status),
                                        ),
                                )
                                .service(
                                    web::resource("/cancel-deletion")
                                        .wrap(Compat::new(rate_limiters.token()))
                                        .wrap(biscuit_auth.clone())
                                        .route(web::post().to(handlers::account::cancel_deletion)),
                                ),
                        )
                        // no auth
                        .service(web::scope("/instance").service(
                            web::resource("").route(web::get().to(handlers::instance::get)),
                        ))
                        .service(
                            web::scope("/quotas")
                                .service(web::resource("").route(web::get().to(quotas::get))),
                        )
                        .service(
                            web::scope("/environment_variables").service(
                                web::resource("")
                                    .route(web::get().to(handlers::environment_variables::get)),
                            ),
                        )
                        .service({
                            let srv = web::scope("/health").service(
                                web::resource("").route(web::get().to(handlers::instance::health)),
                            );

                            #[cfg(feature = "profiling")]
                            {
                                srv.service(
                                    web::resource("/profiling/heap")
                                        .route(web::get().to(handlers::instance::pprof_heap)),
                                )
                                .service(
                                    web::resource("/profiling/cpu")
                                        .route(web::get().to(handlers::instance::pprof_cpu)),
                                )
                            }

                            #[cfg(not(feature = "profiling"))]
                            srv
                        })
                        .service(web::scope("/errors").service(
                            web::resource("").route(web::get().to(handlers::errors::list)),
                        ))
                        .service(
                            web::scope("/payload_content_types").service(
                                web::resource("")
                                    .route(web::get().to(handlers::events::payload_content_types)),
                            ),
                        )
                        .service(
                            web::scope("/register").service(
                                web::resource("")
                                    .route(web::post().to(handlers::registrations::register)),
                            ),
                        )
                        // with authentication
                        .service(
                            web::scope("/organizations")
                                .wrap(Compat::new(rate_limiters.token())) // Middleware order is counter intuitive: this is executed second
                                .wrap(biscuit_auth.clone()) // Middleware order is counter intuitive: this is executed first
                                .service(
                                    web::resource("")
                                        .route(web::get().to(handlers::organizations::list))
                                        .route(web::post().to(handlers::organizations::create)),
                                )
                                .service(
                                    web::scope("/{organization_id}")
                                        .service(
                                            web::resource("")
                                                .route(web::get().to(handlers::organizations::get))
                                                .route(web::put().to(handlers::organizations::edit))
                                                .route(
                                                    web::delete()
                                                        .to(handlers::organizations::delete),
                                                ),
                                        )
                                        .service(
                                            web::resource("/invite")
                                                .route(
                                                    web::post().to(handlers::organizations::invite),
                                                )
                                                .route(
                                                    web::delete()
                                                        .to(handlers::organizations::revoke),
                                                )
                                                .route(
                                                    web::put()
                                                        .to(handlers::organizations::edit_role),
                                                ),
                                        ),
                                ),
                        )
                        .service(
                            web::scope("/applications")
                                .wrap(Compat::new(rate_limiters.token())) // Middleware order is counter intuitive: this is executed second
                                .wrap(biscuit_auth.clone()) // Middleware order is counter intuitive: this is executed first
                                .service(
                                    web::resource("")
                                        .route(web::get().to(handlers::applications::list))
                                        .route(web::post().to(handlers::applications::create)),
                                )
                                .service(
                                    web::resource("/{application_id}")
                                        .route(web::get().to(handlers::applications::get))
                                        .route(web::put().to(handlers::applications::edit))
                                        .route(web::delete().to(handlers::applications::delete)),
                                ),
                        )
                        .service(
                            web::scope("/event_types")
                                .wrap(Compat::new(rate_limiters.token())) // Middleware order is counter intuitive: this is executed second
                                .wrap(biscuit_auth.clone()) // Middleware order is counter intuitive: this is executed first/ Middleware order is counter intuitive: this is executed first
                                .service(
                                    web::resource("")
                                        .route(web::get().to(handlers::event_types::list))
                                        .route(web::post().to(handlers::event_types::create)),
                                )
                                .service(
                                    web::resource("/{event_type_name}")
                                        .route(web::get().to(handlers::event_types::get))
                                        .route(web::delete().to(handlers::event_types::delete)),
                                ),
                        )
                        .service(
                            #[cfg(feature = "application-secret-compatibility")]
                            web::scope("/application_secrets")
                                .wrap(Compat::new(rate_limiters.token())) // Middleware order is counter intuitive: this is executed second
                                .wrap(biscuit_auth.clone()) // Middleware order is counter intuitive: this is executed first/ Middleware order is counter intuitive: this is executed first
                                .service(
                                    web::resource("")
                                        .route(web::get().to(handlers::application_secrets::list))
                                        .route(
                                            web::post().to(handlers::application_secrets::create),
                                        ),
                                )
                                .service(
                                    web::resource("/{application_secret_token}")
                                        .route(web::put().to(handlers::application_secrets::edit))
                                        .route(
                                            web::delete().to(handlers::application_secrets::delete),
                                        ),
                                ),
                            #[cfg(not(feature = "application-secret-compatibility"))]
                            web::resource("/"),
                        )
                        .service(
                            web::scope("/service_token")
                                .wrap(Compat::new(rate_limiters.token())) // Middleware order is counter intuitive: this is executed second
                                .wrap(biscuit_auth.clone()) // Middleware order is counter intuitive: this is executed first
                                .service(
                                    web::resource("")
                                        .route(web::get().to(handlers::service_token::list))
                                        .route(web::post().to(handlers::service_token::create)),
                                )
                                .service(
                                    web::resource("/{service_token_id}")
                                        .route(web::get().to(handlers::service_token::get))
                                        .route(web::put().to(handlers::service_token::edit))
                                        .route(web::delete().to(handlers::service_token::delete)),
                                ),
                        )
                        .service(
                            web::scope("/events")
                                .wrap(Compat::new(rate_limiters.token())) // Middleware order is counter intuitive: this is executed second
                                .wrap(biscuit_auth.clone()) // Middleware order is counter intuitive: this is executed first/ Middleware order is counter intuitive: this is executed first
                                .service(
                                    web::resource("").route(web::get().to(handlers::events::list)),
                                )
                                .service(
                                    web::resource("/{event_id}")
                                        .route(web::get().to(handlers::events::get)),
                                )
                                .service(
                                    web::resource("/{event_id}/replay")
                                        .route(web::post().to(handlers::events::replay)),
                                ),
                        )
                        .service(
                            web::scope("/event")
                                .wrap(Compat::new(rate_limiters.token()))
                                .wrap(Compat::new(rate_limiters.token())) // Middleware order is counter intuitive: this is executed second
                                .wrap(biscuit_auth.clone()) // Middleware order is counter intuitive: this is executed first/ Middleware order is counter intuitive: this is executed first
                                .service(
                                    web::resource("")
                                        .route(web::post().to(handlers::events::ingest)),
                                ),
                        )
                        .service(
                            web::scope("/subscriptions")
                                .wrap(Compat::new(rate_limiters.token())) // Middleware order is counter intuitive: this is executed second
                                .wrap(biscuit_auth.clone()) // Middleware order is counter intuitive: this is executed first
                                .service(
                                    web::resource("")
                                        .route(web::get().to(handlers::subscriptions::list))
                                        .route(web::post().to(handlers::subscriptions::create)),
                                )
                                .service(
                                    web::resource("/{subscription_id}")
                                        .route(web::get().to(handlers::subscriptions::get))
                                        .route(web::put().to(handlers::subscriptions::edit))
                                        .route(web::delete().to(handlers::subscriptions::delete)),
                                ),
                        )
                        .service(
                            web::scope("/request_attempts")
                                .wrap(Compat::new(rate_limiters.token())) // Middleware order is counter intuitive: this is executed second
                                .wrap(biscuit_auth.clone()) // Middleware order is counter intuitive: this is executed first/ Middleware order is counter intuitive: this is executed first
                                .service(
                                    web::resource("")
                                        .route(web::get().to(handlers::request_attempts::list)),
                                ),
                        )
                        .service(
                            web::scope("/responses")
                                .wrap(Compat::new(rate_limiters.token())) // Middleware order is counter intuitive: this is executed second
                                .wrap(biscuit_auth.clone()) // Middleware order is counter intuitive: this is executed first/ Middleware order is counter intuitive: this is executed first
                                .service(
                                    web::resource("/{response_id}")
                                        .route(web::get().to(handlers::responses::get)),
                                ),
                        ),
                );

            if !config.disable_serving_webapp {
                app = app.default_service(
                    Files::new("/", webapp_path.as_str())
                        .index_file(WEBAPP_INDEX_FILE)
                        .default_handler(
                            NamedFile::open(format!("{}/{}", &webapp_path, WEBAPP_INDEX_FILE))
                                .expect("Cannot open SPA main file"),
                        ),
                );
            }
            app.build()
        })
        .bind(&format!("{}:{}", config.ip, config.port))?
        .run()
        .await
        .map_err(|e| e.into())
    } else {
        let keypair = KeyPair::new();
        println!(
            "The BISCUIT_PRIVATE_KEY environnement variable is required for authentication and authorization. You can use this randomly generated value: {}",
            keypair.private().to_bytes_hex()
        );
        std::process::exit(1);
    }
}
