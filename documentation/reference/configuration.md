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

:::note The dashboard is not here
Everything below is read by `hook0-api` and `hook0-output-worker` at startup. The dashboard is a
separate build with variables of its own, named `VITE_*` and compiled into the bundle rather than
read when anything starts; they are listed under
[bare metal self-hosting](../self-hosting/bare-metal.md#ui-building). The `Frontend` group below is
API configuration about the dashboard, such as the URL to link to in an email, not the dashboard's
own build.
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
| `DEBUG_AUTHORIZER` | If true, a trace log message containing authorizer context is emitted on each request; default is false because this feature implies a small overhead and might expose PII in logs | `false` |  |
| `DISABLE_REGISTRATION` | Set to true to disable registration endpoint | - |  |
| `MASTER_API_KEY` 🔒 | A global admin API key that have almost all rights. Better left undefined, USE AT YOUR OWN RISKS! | - |  |
| `MAX_AUTHORIZATION_TIME` | Maximum duration that can be spent running Biscuit's authorizer | `30ms` |  |
| `PASSWORD_MINIMUM_LENGTH` | Minimum length of user passwords, between 1 and 100. Checked wherever a password is set: registration, password reset and password change | `12` |  |

### Email

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `EMAIL_COMPANY_LEGAL_NAME` | Legal name of the company publishing Hook0 (footer mention required by Art. L1463-1 CPCE) | `FGRibreau SARL` |  |
| `EMAIL_COMPANY_POSTAL_ADDRESS` | Postal address of the company publishing Hook0 (footer mention required by Art. L1463-1 CPCE + Art. 6 LCEN) | `3 rue de l'Aubépine, 85110 Chantonnay, France` |  |
| `EMAIL_COMPANY_RCS` | RCS / company registration identifier of the company publishing Hook0 (footer mention) | `RCS La Roche-sur-Yon 850 824 350` |  |
| `EMAIL_DOC_URL` | URL of the Hook0 documentation (used as a secondary CTA in welcome emails) | `https://documentation.hook0.com/` |  |
| `EMAIL_PRIVACY_POLICY_URL` | URL of the Hook0 privacy policy (linked in every email footer for GDPR Art. 13 compliance) | `https://www.hook0.com/privacy-policy` |  |
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
| `EMAIL_LOGO_URL` | URL of the Hook0 logo (must be a publicly fetchable transparent PNG; do NOT use app.hook0.com — that path serves a Git LFS pointer instead of the image). The default points to the banner-transparent variant which renders cleanly on any background | `https://www.hook0.com/mediakit/logo/512x512-banner-transparent.png` |  |
| `FORMBRICKS_API_HOST` | Formbricks API host | `https://app.formbricks.com` |  |
| `FORMBRICKS_ENVIRONMENT_ID` | Formbricks API environment ID | - |  |
| `MATOMO_SITE_ID` | Matomo site ID (also the target site of the server-side activation event; the Hook0 app site) | - |  |
| `MATOMO_URL` | Matomo URL (also the base URL of the server-side activation event tracker when MATOMO_TOKEN_AUTH is set) | - |  |
| `SUPPORT_EMAIL_ADDRESS` | Support email address | `support@hook0.com` |  |
| `WEBAPP_PATH` | Path to the directory containing the web app to serve | `../frontend/dist/` |  |
| `WEBSITE_URL` | Website URL | `https://hook0.com` |  |

### Rate Limiting

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `API_RATE_LIMITING_EMAIL_BURST_SIZE` | Quota of calls per IP to the endpoints that send an email to an address the caller names, before rate limiting blocks incomming requests (must be ≥ 1) | `5` |  |
| `API_RATE_LIMITING_EMAIL_REPLENISH_PERIOD_IN_MS` | Duration (in millisecond) after which one call per IP to the endpoints that send an email is restored in the quota (must be ≥ 1) | `60000` |  |
| `API_RATE_LIMITING_GLOBAL_BURST_SIZE` | Global quota of API calls before rate limiting blocks incomming requests (must be ≥ 1) | `2000` |  |
| `API_RATE_LIMITING_GLOBAL_REPLENISH_PERIOD_IN_MS` | Duration (in millisecond) after which one global API call is restored in the quota (must be ≥ 1) | `1` |  |
| `API_RATE_LIMITING_HOUSEKEEPING_PERIOD` | Duration to wait beetween rate limiters housekeeping | `5m` |  |
| `API_RATE_LIMITING_IP_BURST_SIZE` | Quota of API calls per IP before rate limiting blocks incomming requests (must be ≥ 1) | `200` |  |
| `API_RATE_LIMITING_IP_REPLENISH_PERIOD_IN_MS` | Duration (in millisecond) after which one API call per IP is restored in the quota (must be ≥ 1) | `10` |  |
| `API_RATE_LIMITING_TOKEN_BURST_SIZE` | Quota of API calls per token before rate limiting blocks incomming requests (must be ≥ 1) | `20` |  |
| `API_RATE_LIMITING_TOKEN_REPLENISH_PERIOD_IN_MS` | Duration (in millisecond) after which one API call per token is restored in the quota (must be ≥ 1) | `100` |  |
| `DISABLE_API_RATE_LIMITING` | Set to true to disable every API rate limiting | - |  |
| `DISABLE_API_RATE_LIMITING_EMAIL` | Set to true to disable per-IP API rate limiting of the endpoints that send an email to an address the caller names | - |  |
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
| `OBJECT_STORAGE_CLEANUP_COLLECT_CONCURRENCY` | Maximum number of applications to process concurrently during object storage cleanup prefix collection | `1` |  |
| `OBJECT_STORAGE_CLEANUP_DELETE_CONCURRENCY` | Maximum number of prefixes to delete concurrently during object storage cleanup | `1` |  |
| `OBJECT_STORAGE_CLEANUP_OPERATION_ATTEMPT_TIMEOUT` | Operation attempt timeout for object storage cleanup operations | `2m` |  |
| `OBJECT_STORAGE_CLEANUP_OPERATION_TIMEOUT` | Operation timeout for object storage cleanup operations | `7m` |  |
| `OBJECT_STORAGE_CLEANUP_PERIOD` | Duration to wait between object storage cleanups | `1d` |  |
| `OBJECT_STORAGE_CLEANUP_REPORT_AND_DELETE` | If true, allow to delete outdated objects from object storage; if false (default), they will only be reported | `false` |  |
| `OLD_EVENTS_CLEANUP_GRACE_PERIOD_IN_DAY` | Duration (in day) to wait before actually deleting events that are passed retention period | `30` |  |
| `OLD_EVENTS_CLEANUP_PERIOD_IN_S` | Duration (in second) to wait between old events cleanups; set to 0 to disable the task | `3600` |  |
| `OLD_EVENTS_CLEANUP_REPORT_AND_DELETE` | If true, old events will be reported and cleaned up; if false (default), they will only be reported | `false` |  |
| `SOFT_DELETED_APPLICATIONS_CLEANUP_GRACE_PERIOD` | Duration to wait before removing a soft-deleted application | `30d` |  |
| `SOFT_DELETED_APPLICATIONS_CLEANUP_PERIOD` | Duration to wait between soft-deleted applications cleanups | `1d` |  |
| `UNVERIFIED_USERS_CLEANUP_GRACE_PERIOD_IN_DAYS` | Duration (in day) to wait before removing a unverified user | `7` |  |
| `UNVERIFIED_USERS_CLEANUP_PERIOD_IN_S` | Duration (in second) to wait between unverified users cleanups | `3600` |  |
| `UNVERIFIED_USERS_CLEANUP_REPORT_AND_DELETE` | If true, unverified users will be reported and cleaned up; if false (default), they will only be reported | `false` |  |

### Reactivation

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `ENABLE_REACTIVATION_EMAILS` | If true (default), verified accounts that never sent an event get a bounded "0 event sent" reactivation email sequence (J+1 / J+3 / J+7); set to false to opt out | `true` |  |
| `REACTIVATION_DISCORD_URL` | URL of the Hook0 community/Discord used as the J+7 CTA | `https://www.hook0.com/community` |  |
| `REACTIVATION_EMAILS_MAX_PER_STEP_PER_RUN` | Upper bound on how many recipients a single pass processes per step (bounds work per pass) | `500` |  |
| `REACTIVATION_EMAILS_PERIOD` | Duration to wait between reactivation email passes (at least 1s) | `6h` |  |
| `REACTIVATION_PLAY_URL` | URL of the Hook0 webhook tester used as the J+3 CTA (lift the "no public URL" blocker) | `https://play.hook0.com/` |  |

### Monitoring

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `HEALTH_CHECK_KEY` 🔒 | Key for the health check endpoint; if not specified, endpoint is disabled; if empty, endpoint is public | - |  |
| `HEALTH_CHECK_TIMEOUT` | Max timeout duration for health check: if subsystems take longer to respond they will be considered unhealthy | `5s` |  |
| `OTLP_AUTHORIZATION` 🔒 | Optional value for OTLP `Authorization` header (for example: `Bearer mytoken`) | - |  |
| `OTLP_METRICS_ENDPOINT` | Optional OTLP endpoint that will receive metrics | - |  |
| `OTLP_TRACES_ENDPOINT` | Optional OTLP endpoint that will receive traces | - |  |
| `SENTRY_DEBUG` | Enable Sentry SDK debug mode | `false` |  |
| `SENTRY_DSN` | Optional Sentry DSN for error reporting | - |  |
| `SENTRY_ENABLE_SPANS` | Enable sending tracing spans to Sentry | `false` |  |
| `SENTRY_SEND_DEFAULT_PII` | Send default PII (IP addresses, cookies, etc.) to Sentry | `false` |  |
| `SENTRY_TRACES_SAMPLE_RATE` | Optional sample rate for tracing transactions with Sentry (between 0.0 and 1.0) | - |  |

### Hook0 Client

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `HOOK0_CLIENT_API_URL` | Base API URL of a Hook0 instance that will receive events from this Hook0 instance | - |  |
| `HOOK0_CLIENT_APPLICATION_ID` | UUID of a Hook0 application that will receive events from this Hook0 instance | - |  |
| `HOOK0_CLIENT_TOKEN` | Authentication token valid for a Hook0 application that will receive events from this Hook0 instance | - |  |
| `HOOK0_CLIENT_UPSERTS_RETRIES` | Number of allowed retries when upserting event types to the linked Hook0 application fails | `10` |  |

### Object Storage

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `OBJECT_STORAGE_BUCKET_NAME` | Bucket name of the S3-like object storage | - |  |
| `OBJECT_STORAGE_CONNECT_TIMEOUT` | Connect timeout for object storage operations (time to initiate socket connection) | `3s` |  |
| `OBJECT_STORAGE_FORCE_HTTP_SCHEME` | Force endpoint scheme to be HTTP (by default it is HTTPS) | `false` |  |
| `OBJECT_STORAGE_HOST` | Host of the S3-like object storage (without https://) | - |  |
| `OBJECT_STORAGE_KEY_ID` | Key ID of the S3-like object storage | - |  |
| `OBJECT_STORAGE_KEY_SECRET` 🔒 | Key secret of the S3-like object storage | - |  |
| `OBJECT_STORAGE_MAX_ATTEMPTS` | Maximum number of attempts for object storage operations | `3` |  |
| `OBJECT_STORAGE_OPERATION_ATTEMPT_TIMEOUT` | Operation attempt timeout for object storage operations | `10s` |  |
| `OBJECT_STORAGE_OPERATION_TIMEOUT` | Operation timeout for object storage operations | `30s` |  |
| `OBJECT_STORAGE_READ_TIMEOUT` | Read timeout for object storage operations (time to first byte) | `5s` |  |
| `STORE_EVENT_PAYLOADS_DB_FALLBACK` | If true (default), event payloads that cannot be written to object storage are stored in the database instead of failing ingestion; set to false to fail the request and keep payloads out of the database | `true` |  |
| `STORE_EVENT_PAYLOADS_IN_OBJECT_STORAGE` | If true, new event payloads will be stored in object storage instead of database | `false` |  |
| `STORE_EVENT_PAYLOADS_IN_OBJECT_STORAGE_ONLY_FOR` | A comma-separated list of applications ID whose event payloads should be stored in object storage; if empty (default), all event payloads will be stored in object storage regardless of application ID | - |  |

### Pulsar

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `PULSAR_BINARY_URL` | Pulsar binary URL | - |  |
| `PULSAR_NAMESPACE` | Pulsar namespace | - |  |
| `PULSAR_SEND_RECEIPT_TIMEOUT` | Maximum time to wait for the Pulsar broker to acknowledge a sent message | `10s` |  |
| `PULSAR_TENANT` | Pulsar tenant | - |  |
| `PULSAR_TOKEN` 🔒 | Pulsar token | - |  |

### Google Ads

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `GOOGLE_ADS_CUSTOMER_ID` | Customer ID (e.g. 629-941-8464) | - |  |
| `GOOGLE_ADS_DEVELOPER_TOKEN` | Developer token (enables server-side conversion upload). All Google Ads fields must be provided together; if any is missing, the conversion upload is silently disabled (build_google_ads_client emits a warn log when partially configured) | - |  |
| `GOOGLE_ADS_FIRST_EVENT_CONVERSION_ACTION_ID` | Numeric ID of the FIRST-EVENT conversion action (optional). When unset, first-event conversion tracking (and its background job) is disabled; signup is unaffected | - |  |
| `GOOGLE_ADS_FIRST_EVENT_CONVERSION_PERIOD_IN_S` | Duration (in second) between first-event conversion upload passes; set to 0 to disable the task. Only runs when a first-event conversion action is configured | `300` |  |
| `GOOGLE_ADS_FIRST_WEBHOOK_DELIVERED_CONVERSION_ACTION_ID` | Numeric ID of the FIRST-WEBHOOK-DELIVERED conversion action (optional, north-star activation signal). When unset, first-webhook- delivered conversion tracking (and its background job) is disabled; the other conversions are unaffected | - |  |
| `GOOGLE_ADS_FIRST_WEBHOOK_DELIVERED_CONVERSION_PERIOD_IN_S` | Duration (in second) between first-webhook-delivered conversion upload passes; set to 0 to disable the task. Only runs when a first-webhook-delivered conversion action is configured | `300` |  |
| `GOOGLE_ADS_LOGIN_CUSTOMER_ID` | MCC login customer ID (optional) | - |  |
| `GOOGLE_ADS_OAUTH_CLIENT_ID` | OAuth client ID (Desktop App credentials) | - |  |
| `GOOGLE_ADS_OAUTH_CLIENT_SECRET` | OAuth client secret | - |  |
| `GOOGLE_ADS_OAUTH_REFRESH_TOKEN` | OAuth refresh token | - |  |
| `GOOGLE_ADS_SIGNUP_CONVERSION_ACTION_ID` | Numeric ID of the signup conversion action (e.g. 7576442588) | - |  |
| `SIGNUP_ATTRIBUTION_CLEANUP_PERIOD_IN_S` | Duration (in second) between periodic passes that delete signup-attribution rows (gclids) older than SIGNUP_ATTRIBUTION_RETENTION_IN_DAYS; set to 0 to disable the task. This enforces the retention maximum on a timer, independent of registration traffic (registrations also prune lazily). Runs regardless of Google Ads / Matomo configuration | `3600` |  |
| `SIGNUP_ATTRIBUTION_RETENTION_IN_DAYS` | Days a signup-attribution row (its gclid) is retained before the lazy purge deletes it — the safety net for rows whose conversions never complete. Defaults to 30 to match the click-through attribution window configured on the server-side conversion actions (30 days): past that window Google rejects the upload, so retaining the gclid longer would serve no purpose (data-minimisation, art. 5.1.e GDPR). Must not exceed the Google Ads conversion attribution window. Bounded to [1, 3650] days | `30` |  |

### Deprecated

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `ENABLE_APPLICATION_SECRET_COMPATIBILITY` | Enable application secret compatibility mode | `true` |  |

### Matomo

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `MATOMO_ACTIVATION_PERIOD_IN_S` | Duration (in second) between server-side activation-event scan passes; set to 0 to disable the task. Only runs when Matomo tracking is fully configured (URL + site id + token_auth) | `300` |  |
| `MATOMO_ACTIVATION_SCAN_LIMIT` | Maximum number of organizations claimed per activation scan pass (bounds the work per pass). Bounded to [1, 10000] | `500` |  |
| `MATOMO_TOKEN_AUTH` | Server-side Tracking API token_auth secret. Enables the server-side product-activation event (category=activation, action=first-webhook-delivered) emitted once per organization on its first successful webhook delivery. Dark by default: the event is emitted only when MATOMO_URL, MATOMO_SITE_ID and this token are all set | - |  |

## Output Worker

The output-worker is a separate binary with its own configuration. Run `hook0-output-worker --help` for the authoritative reference.

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `SENTRY_DSN` | Optional Sentry DSN for error reporting | - |  |
| `SENTRY_DEBUG` | Enable Sentry SDK debug mode | `false` |  |
| `SENTRY_SEND_DEFAULT_PII` | Send default PII (IP addresses, cookies, etc.) to Sentry | `false` |  |
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
| `OBJECT_STORAGE_CONNECT_TIMEOUT` | Connect timeout for object storage operations (time to initiate socket connection) | `3s` |  |
| `OBJECT_STORAGE_READ_TIMEOUT` | Read timeout for object storage operations (time to first byte) | `5s` |  |
| `OBJECT_STORAGE_OPERATION_ATTEMPT_TIMEOUT` | Operation attempt timeout for object storage operations | `10s` |  |
| `OBJECT_STORAGE_OPERATION_TIMEOUT` | Operation timeout for object storage operations | `30s` |  |
| `OBJECT_STORAGE_BUCKET_NAME` | Bucket name of the S3-like object storage | - |  |
| `STORE_RESPONSE_BODY_AND_HEADERS_IN_OBJECT_STORAGE` | If true, new response bodies and headers will be stored in object storage instead of database | `false` |  |
| `STORE_RESPONSE_BODY_AND_HEADERS_IN_OBJECT_STORAGE_ONLY_FOR` | A comma-separated list of applications ID whose response bodies and headers should be stored in object storage; if empty (default), all response bodies and headers will be stored in object storage regardless of application ID | - |  |
| `WORKER_NAME` | Worker name (as defined in the infrastructure.worker table) | - | ✓ |
| `WORKER_VERSION` | Worker version (if empty, will use version from Cargo.toml) | - |  |
| `CONCURRENT` | Number of request attempts to handle concurrently (for a worker with pg queue type, this means opening 1 connection to PostgreSQL per concurrent unit) | `1` |  |
| `HP_RETRY_CUTOFF` | Retry count cutoff for queue priority classification: if retry_count >= cutoff, item is placed in low priority queue | `2` |  |
| `CONCURRENT_HP_RESERVED` | Number of concurrent slots reserved exclusively for high-priority jobs (first attempts and early retries) | `0` |  |
| `CONCURRENT_LP_RESERVED` | Number of concurrent slots reserved exclusively for low-priority jobs (later retries) | `0` |  |
| `MAX_RETRIES` | Maximum number of delivery retries before giving up (the effective number of retries is limited by `MAX_RETRIES`, `MAX_RETRY_WINDOW` and the retry policy) | `24` |  |
| `MAX_RETRY_WINDOW` | Maximum time window for delivery retries before giving up (the effective number of retries is limited by `MAX_RETRIES`, `MAX_RETRY_WINDOW` and the retry policy) | `8d` |  |
| `RETRY_JITTER_RATIO` | Ratio of a retry's base delay used as the width of the random jitter window added on top of it, bounded by an internal minimum and by `RETRY_JITTER_MAX_SPREAD` (set to 0 to disable jitter and get strictly deterministic retry delays) | `0.1` |  |
| `RETRY_JITTER_MAX_SPREAD` | Maximum width of the random jitter window added on top of a retry's base delay; takes precedence over the internal minimum (set to "0s" to disable jitter) | `15m` |  |
| `MONITORING_HEARTBEAT_URL` | Heartbeat URL that should be called regularly | - |  |
| `MONITORING_HEARTBEAT_MIN_PERIOD_IN_S` | Minimal duration (in second) to wait between sending two heartbeats | `60` |  |
| `DISABLE_TARGET_IP_CHECK` | If set to false (default), webhooks that target IPs that are not globally reachable (like "127.0.0.1" for example) will fail | `false` |  |
| `CONNECT_TIMEOUT` | Timeout for establishing a connection to the target (if exceeded, request attempt will fail) | `5s` |  |
| `TIMEOUT` | Timeout for obtaining a HTTP response from the target, including connect phase (if exceeded, request attempt will fail) | `15s` |  |
| `DNS_TIMEOUT` | Total wall-clock budget for resolving the target's hostname, across every name server query it takes (if exceeded, request attempt will fail); must be at least "3ms" | `5s` |  |
| `DNS_CACHE_MAX_TTL` | Maximum duration a successful DNS answer is kept in the worker's in-process DNS cache (shorter record TTLs are still honored) | `5m` |  |
| `DNS_NEGATIVE_CACHE_MAX_TTL` | Maximum duration a negative DNS answer (for example NXDOMAIN) is kept in the worker's in-process DNS cache | `30s` |  |
| `DNS_IP_STRATEGY` | Which IP address families to query when resolving a webhook target's hostname; `ipv4-only` ignores AAAA records entirely, which is useful when a target's IPv6 address is not globally reachable | `ipv4-and-ipv6` |  |
| `DNS_APPEND_SEARCH_DOMAINS` | If set to false (default), a webhook target's hostname is resolved exactly as written; if true, the worker host's resolv.conf search domains are appended to it | `false` |  |
| `SIGNATURE_HEADER_NAME` | Name of the header containing webhook's signature | `X-Hook0-Signature` |  |
| `ENABLED_SIGNATURE_VERSIONS` | A comma-separated list of enabled signature versions | `v1` |  |
| `LOAD_WAITING_REQUEST_ATTEMPTS_INTO_PULSAR` | Loads request attempts that haven't been delivered yet from the DB into Pulsar before starting work; `all` loads everything; `due-now` skips request attempts scheduled more than ~10 s in the future; this is useful when migrating to a Pulsar worker (only for Pulsar workers) | `off` |  |
| `REQUEST_ATTEMPT_DB_COMMIT_GRACE_PERIOD` | Grace period to wait for database commit before dropping unfound request attempts (only for Pulsar workers) | `10s` |  |
| `PULSAR_CONSUMER_STATS_INTERVAL` | Period of Pulsar consumer stats collection (set to "0s" to disable) (only for Pulsar workers) [this feature is unstable/unreliable] | `0` |  |
| `PULSAR_SEND_RECEIPT_TIMEOUT` | Maximum time to wait for the Pulsar broker to acknowledge a sent message (only for Pulsar workers) | `10s` |  |
| `THROUGHPUT_LOG_INTERVAL` | Interval between periodic throughput log lines (set to "0s" to disable) | `60s` |  |
| `SLOT_METRICS_INTERVAL` | Period at which free concurrency slots are sampled for the throughput log and OTel gauges (set to "0s" to disable) (only for Pulsar workers) | `15s` |  |

## Notes

- 🔒 indicates sensitive values (hidden in logs)
- Boolean values: `true`, `false` (case-insensitive)
- Durations: Use humantime format (`1h`, `30m`, `7d`) where supported, otherwise seconds
- Lists: Comma-separated
- URLs: Must be valid URLs with scheme
