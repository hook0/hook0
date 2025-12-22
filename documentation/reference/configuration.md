# Configuration Reference

<!--
  ⚠️  AUTO-GENERATED FILE - DO NOT EDIT MANUALLY

  This file is generated from the Hook0 API /environment_variables endpoint.
  To regenerate, run: npm run generate:config
-->

Environment variables for configuring Hook0.

:::tip Source of Truth
The authoritative reference for all configuration options is running the executable with `--help`:

```bash
hook0-api --help
hook0-output-worker --help
```

This documentation may not cover all options or reflect recent changes.
:::

## API

### Web Server

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `CORS_ALLOWED_ORIGINS` | Comma-separated allowed origins for CORS | - |  |
| `ENABLE_HSTS_HEADER` | If true, the HSTS header will be enabled | `false` |  |
| `ENABLE_SECURITY_HEADERS` | If true, the secured HTTP headers will be enabled | `true` |  |
| `IP` | IP address on which to start the HTTP server | `127.0.0.1` |  |
| `PORT` | Port on which to start the HTTP server | `8080` |  |

### Reverse Proxy

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `BEHIND_CLOUDFLARE` | Set to true if your instance is served behind Cloudflare's proxies in order to determine the correct user IP for each request | `false` |  |
| `CC_REVERSE_PROXY_IPS` | A comma-separated list of trusted IP addresses (e.g. `192.168.1.1`) or CIDRs (e.g. `192.168.0.0/16`) that are allowed to set "X-Forwarded-For" and "Forwarded" headers | - |  |
| `REVERSE_PROXY_IPS` | A comma-separated list of trusted IP addresses (e.g. `192.168.1.1`) or CIDRs (e.g. `192.168.0.0/16`) that are allowed to set "X-Forwarded-For" and "Forwarded" headers | - |  |

### Database

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `DATABASE_URL` 🔒 | Database URL (with credentials) | - | ✓ |
| `DB_STATEMENT_TIMEOUT` | Statement timeout for database queries; if `0ms` (default), no timeout will be set; this is only for API-related queries, housekeeping tasks run without timeout | `0ms` |  |
| `MAX_DB_CONNECTIONS` | Maximum number of connections to database | `5` |  |
| `NO_AUTO_DB_MIGRATION` | Disable automatic database migration | - |  |

### Auth

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `BISCUIT_PRIVATE_KEY` | Biscuit's private key, used for authentication | - |  |
| `DEBUG_AUTHORIZER` | If true, a trace log message containing authorizer context is emitted on each request; defaut is false because this feature implies a small overhead | `false` |  |
| `DISABLE_REGISTRATION` | Set to true to disable registration endpoint | - |  |
| `MASTER_API_KEY` 🔒 | A global admin API key that have almost all rights. Better left undefined, USE AT YOUR OWN RISKS! | - |  |
| `MAX_AUTHORIZATION_TIME_IN_MS` | Maximum duration (in millisecond) that can be spent running Biscuit's authorizer | `10` |  |
| `PASSWORD_MINIMUM_LENGTH` | Minimum length of user passwords. This is checked when a user registers | `12` |  |

### Email

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `EMAIL_SENDER_ADDRESS` | Sender email address | - | ✓ |
| `EMAIL_SENDER_NAME` | Sender name | `Hook0` |  |
| `SMTP_CONNECTION_URL` 🔒 | Connection URL to SMTP server; for example: `smtp://localhost:1025`, `smtps://user:password@provider.com:465` (SMTP over TLS) or `smtp://user:password@provider.com:465?tls=required` (SMTP with STARTTLS) | - | ✓ |
| `SMTP_TIMEOUT_IN_S` | Duration (in second) to use as timeout when sending emails to the SMTP server | `5` |  |

### Frontend

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `APP_URL` | Frontend application URL (used for building links in emails and pagination) | - | ✓ |
| `CLOUDFLARE_TURNSTILE_SECRET_KEY` | Cloudflare Turnstile secret key (enables Turnstile for user registration) | - |  |
| `CLOUDFLARE_TURNSTILE_SITE_KEY` | Cloudflare Turnstile site key (enables Turnstile for user registration) | - |  |
| `DISABLE_SERVING_WEBAPP` | Set to true to disable serving the web app and only serve the API | - |  |
| `EMAIL_LOGO_URL` | URL of the Hook0 logo | `https://app.hook0.com/256x256.png` |  |
| `FORMBRICKS_API_HOST` | Formbricks API host | `https://app.formbricks.com` |  |
| `FORMBRICKS_ENVIRONMENT_ID` | Formbricks API environment ID | - |  |
| `MATOMO_SITE_ID` | Matomo site ID | - |  |
| `MATOMO_URL` | Matomo URL | - |  |
| `SUPPORT_EMAIL_ADDRESS` | Support email address | `support@hook0.com` |  |
| `WEBAPP_PATH` | Path to the directory containing the web app to serve | `../frontend/dist/` |  |
| `WEBSITE_URL` | Website URL | `https://hook0.com` |  |

### Rate Limiting

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `API_RATE_LIMITING_GLOBAL_BURST_SIZE` | Global quota of API calls before rate limiting blocks incomming requests (must be ≥ 1) | `2000` |  |
| `API_RATE_LIMITING_GLOBAL_REPLENISH_PERIOD_IN_MS` | Duration (in millisecond) after which one global API call is restored in the quota (must be ≥ 1) | `1` |  |
| `API_RATE_LIMITING_HOUSEKEEPING_PERIOD` | Duration to wait beetween rate limiters housekeeping | `5m` |  |
| `API_RATE_LIMITING_IP_BURST_SIZE` | Quota of API calls per IP before rate limiting blocks incomming requests (must be ≥ 1) | `200` |  |
| `API_RATE_LIMITING_IP_REPLENISH_PERIOD_IN_MS` | Duration (in millisecond) after which one API call per IP is restored in the quota (must be ≥ 1) | `10` |  |
| `API_RATE_LIMITING_TOKEN_BURST_SIZE` | Quota of API calls per token before rate limiting blocks incomming requests (must be ≥ 1) | `20` |  |
| `API_RATE_LIMITING_TOKEN_REPLENISH_PERIOD_IN_MS` | Duration (in millisecond) after which one API call per token is restored in the quota (must be ≥ 1) | `100` |  |
| `DISABLE_API_RATE_LIMITING` | Set to true to disable every API rate limiting | - |  |
| `DISABLE_API_RATE_LIMITING_GLOBAL` | Set to true to disable global API rate limiting | - |  |
| `DISABLE_API_RATE_LIMITING_IP` | Set to true to disable per-IP API rate limiting | - |  |
| `DISABLE_API_RATE_LIMITING_TOKEN` | Set to true to disable per-token API rate limiting | - |  |

### Quotas

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `ENABLE_QUOTA_BASED_EMAIL_NOTIFICATIONS` | Set to true to enable quota-based email notifications | `false` |  |
| `ENABLE_QUOTA_ENFORCEMENT` | Set to true to apply quotas limits (default is not to) | - |  |
| `QUOTA_GLOBAL_APPLICATIONS_PER_ORGANIZATION_LIMIT` | Default limit of applications per organization (can be overriden by a plan) | `1` |  |
| `QUOTA_GLOBAL_DAYS_OF_EVENTS_RETENTION_LIMIT` | Default limit of day of event's retention (can be overriden by a plan) | `7` |  |
| `QUOTA_GLOBAL_EVENT_TYPES_PER_APPLICATION_LIMIT` | Default limit of event types per application (can be overriden by a plan) | `10` |  |
| `QUOTA_GLOBAL_EVENTS_PER_DAY_LIMIT` | Default limit of events per day (can be overriden by a plan) | `100` |  |
| `QUOTA_GLOBAL_MEMBERS_PER_ORGANIZATION_LIMIT` | Default limit of members per organization (can be overriden by a plan) | `1` |  |
| `QUOTA_GLOBAL_SUBSCRIPTIONS_PER_APPLICATION_LIMIT` | Default limit of subscriptions per application (can be overriden by a plan) | `10` |  |
| `QUOTA_NOTIFICATION_EVENTS_PER_DAY_THRESHOLD` | Default threshold (in %) of events per day at which to send a warning notification | `80` |  |

### Housekeeping

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `ENABLE_SOFT_DELETED_APPLICATIONS_CLEANUP` | If true, soft-deleted applications will be removed from database after a while; otherwise they will be kept in database forever | `false` |  |
| `ENABLE_UNVERIFIED_USERS_CLEANUP` | If true, unverified users will be remove from database after a while | `false` |  |
| `EXPIRED_TOKENS_CLEANUP_GRACE_PERIOD` | Duration to wait before actually deleting expired tokens (expired tokens cannot be used anyway, even if kept for some time) | `7d` |  |
| `EXPIRED_TOKENS_CLEANUP_PERIOD` | Duration to wait between expired tokens cleanups | `1h` |  |
| `EXPIRED_TOKENS_CLEANUP_REPORT_AND_DELETE` | If true, expired tokens will be reported and cleaned up; if false (default), they will only be reported | `false` |  |
| `MATERIALIZED_VIEWS_REFRESH_PERIOD_IN_S` | Duration (in second) to wait between materialized views refreshes | `60` |  |
| `OBJECT_STORAGE_CLEANUP_PERIOD` | Duration to wait between object storage cleanups | `1d` |  |
| `OBJECT_STORAGE_CLEANUP_REPORT_AND_DELETE` | If true, allow to delete outdated objects from object storage; if false (default), they will only be reported | `false` |  |
| `OLD_EVENTS_CLEANUP_GRACE_PERIOD_IN_DAY` | Duration (in day) to wait before actually deleting events that are passed retention period | `30` |  |
| `OLD_EVENTS_CLEANUP_PERIOD_IN_S` | Duration (in second) to wait between old events cleanups | `3600` |  |
| `OLD_EVENTS_CLEANUP_REPORT_AND_DELETE` | If true, old events will be reported and cleaned up; if false (default), they will only be reported | `false` |  |
| `SOFT_DELETED_APPLICATIONS_CLEANUP_GRACE_PERIOD` | Duration to wait before removing a soft-deleted application | `30d` |  |
| `SOFT_DELETED_APPLICATIONS_CLEANUP_PERIOD` | Duration to wait between soft-deleted applications cleanups | `1d` |  |
| `UNVERIFIED_USERS_CLEANUP_GRACE_PERIOD_IN_DAYS` | Duration (in day) to wait before removing a unverified user | `7` |  |
| `UNVERIFIED_USERS_CLEANUP_PERIOD_IN_S` | Duration (in second) to wait between unverified users cleanups | `3600` |  |
| `UNVERIFIED_USERS_CLEANUP_REPORT_AND_DELETE` | If true, unverified users will be reported and cleaned up; if false (default), they will only be reported | `false` |  |

### Monitoring

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `HEALTH_CHECK_KEY` 🔒 | Key for the health check endpoint; if not specified, endpoint is disabled; if empty, endpoint is public | - |  |
| `HEALTH_CHECK_TIMEOUT` | Max timeout duration for health check: if subsystems take longer to respond they will be considered unhealthy | `5s` |  |
| `OTLP_AUTHORIZATION` 🔒 | Optional value for OTLP `Authorization` header (for example: `Bearer mytoken`) | - |  |
| `OTLP_METRICS_ENDPOINT` | Optional OTLP endpoint that will receive metrics | - |  |
| `OTLP_TRACES_ENDPOINT` | Optional OTLP endpoint that will receive traces | - |  |
| `SENTRY_DSN` | Optional Sentry DSN for error reporting | - |  |
| `SENTRY_TRACES_SAMPLE_RATE` | Optional sample rate for tracing transactions with Sentry (between 0.0 and 1.0) | - |  |

### Hook0 Client

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `HOOK0_CLIENT_API_URL` | Base API URL of a Hook0 instance that will receive events from this Hook0 instance | - |  |
| `HOOK0_CLIENT_APPLICATION_ID` | UUID of a Hook0 application that will receive events from this Hook0 instance | - |  |
| `HOOK0_CLIENT_TOKEN` | Authentifcation token valid for a Hook0 application that will receive events from this Hook0 instance | - |  |
| `HOOK0_CLIENT_UPSERTS_RETRIES` | Number of allowed retries when upserting event types to the linked Hook0 application fails | `10` |  |

### Object Storage

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `OBJECT_STORAGE_BUCKET_NAME` | Bucket name of the S3-like object storage | - |  |
| `OBJECT_STORAGE_FORCE_HTTP_SCHEME` | Force endpoint scheme to be HTTP (by default it is HTTPS) | `false` |  |
| `OBJECT_STORAGE_HOST` | Host of the S3-like object storage (without https://) | - |  |
| `OBJECT_STORAGE_KEY_ID` | Key ID of the S3-like object storage | - |  |
| `OBJECT_STORAGE_KEY_SECRET` 🔒 | Key secret of the S3-like object storage | - |  |
| `OBJECT_STORAGE_MAX_ATTEMPTS` | Maximum number of attempts for object storage operations | `3` |  |
| `STORE_EVENT_PAYLOADS_IN_OBJECT_STORAGE` | If true, new event payloads will be stored in object storage instead of database | `false` |  |
| `STORE_EVENT_PAYLOADS_IN_OBJECT_STORAGE_ONLY_FOR` | A comma-separated list of applications ID whose event payloads should be stored in object storage; if empty (default), all event payloads will be stored in object storage regardless of application ID | - |  |

### Pulsar

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `PULSAR_BINARY_URL` | Pulsar binary URL | - |  |
| `PULSAR_NAMESPACE` | Pulsar namespace | - |  |
| `PULSAR_TENANT` | Pulsar tenant | - |  |
| `PULSAR_TOKEN` 🔒 | Pulsar token | - |  |

### Deprecated

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `ENABLE_APPLICATION_SECRET_COMPATIBILITY` | Enable application secret compatibility mode | `true` |  |

## Output Worker

The output-worker is a separate binary with its own configuration. Run `hook0-output-worker --help` for the authoritative reference.

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `SENTRY_DSN` | Optional Sentry DSN for error reporting | - |  |
| `OTLP_METRICS_ENDPOINT` | Optional OTLP endpoint that will receive metrics | - |  |
| `OTLP_TRACES_ENDPOINT` | Optional OTLP endpoint that will receive traces | - |  |
| `OTLP_AUTHORIZATION` 🔒 | Optional value for OTLP `Authorization` header (for example: `Bearer mytoken`) | - |  |
| `DATABASE_URL` 🔒 | Database URL (with credentials) | - | ✓ |
| `MAX_DB_CONNECTIONS` | Maximum number of connections to database (for a worker with pg queue type, it should be equal to CONCURRENT) | `5` |  |
| `PULSAR_BINARY_URL` | Pulsar binary URL | - |  |
| `PULSAR_TOKEN` 🔒 | Pulsar token | - |  |
| `PULSAR_TENANT` | Pulsar tenant | - |  |
| `PULSAR_NAMESPACE` | Pulsar namespace | - |  |
| `OBJECT_STORAGE_HOST` | Host of the S3-like object storage (without https://) | - |  |
| `OBJECT_STORAGE_FORCE_HTTP_SCHEME` | Force endpoint scheme to be HTTP (by default it is HTTPS) | `false` |  |
| `OBJECT_STORAGE_KEY_ID` | Key ID of the S3-like object storage | - |  |
| `OBJECT_STORAGE_KEY_SECRET` 🔒 | Key secret of the S3-like object storage | - |  |
| `OBJECT_STORAGE_MAX_ATTEMPTS` | Maximum number of attempts for object storage operations | `3` |  |
| `OBJECT_STORAGE_BUCKET_NAME` | Bucket name of the S3-like object storage | - |  |
| `STORE_RESPONSE_BODY_AND_HEADERS_IN_OBJECT_STORAGE` | If true, new response bodies and headers will be stored in object storage instead of database | `false` |  |
| `STORE_RESPONSE_BODY_AND_HEADERS_IN_OBJECT_STORAGE_ONLY_FOR` | A comma-separated list of applications ID whose response bodies and headers should be stored in object storage | - |  |
| `WORKER_NAME` | Worker name (as defined in the infrastructure.worker table) | - | ✓ |
| `WORKER_VERSION` | Worker version (if empty, will use version from Cargo.toml) | - |  |
| `CONCURRENT` | Number of request attempts to handle concurrently | `1` |  |
| `MAX_FAST_RETRIES` | Maximum number of fast retries (before doing slow retries) | `30` |  |
| `MAX_SLOW_RETRIES` | Maximum number of slow retries (before giving up) | `30` |  |
| `MONITORING_HEARTBEAT_URL` | Heartbeat URL that should be called regularly | - |  |
| `MONITORING_HEARTBEAT_MIN_PERIOD_IN_S` | Minimal duration (in second) to wait between sending two heartbeats | `60` |  |
| `DISABLE_TARGET_IP_CHECK` | If set to false (default), webhooks targeting non-globally-reachable IPs will fail | `false` |  |
| `CONNECT_TIMEOUT` | Timeout for establishing a connection to the target | `5s` |  |
| `TIMEOUT` | Timeout for obtaining a HTTP response from the target, including connect phase | `15s` |  |
| `SIGNATURE_HEADER_NAME` | Name of the header containing webhook's signature | `X-Hook0-Signature` |  |
| `ENABLED_SIGNATURE_VERSIONS` | A comma-separated list of enabled signature versions | `v1` |  |
| `LOAD_WAITING_REQUEST_ATTEMPT_INTO_PULSAR` | If true, will load waiting request attempts from DB into Pulsar before starting | `false` |  |

## Notes

- 🔒 indicates sensitive values (hidden in logs)
- Boolean values: `true`, `false` (case-insensitive)
- Durations: Use humantime format (`1h`, `30m`, `7d`) where supported, otherwise seconds
- Lists: Comma-separated
- URLs: Must be valid URLs with scheme
