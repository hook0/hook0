# Configuration Reference

Environment variables for configuring Hook0.

## Server Configuration

### Network

```bash
# IP address to bind (use 0.0.0.0 in Docker containers)
IP=127.0.0.1

# Port to listen on
PORT=8080
```

:::tip Docker deployments
When running in Docker, set `IP=0.0.0.0` to allow connections from outside the container. The self-hosting tutorial uses port 8081 by convention.
:::

### Reverse Proxy

```bash
# Trusted IPs/CIDRs that can set X-Forwarded-For and Forwarded headers
REVERSE_PROXY_IPS=192.168.0.0/16,10.0.0.0/8

# Enable if behind Cloudflare
BEHIND_CLOUDFLARE=false
```

## Database

```bash
# PostgreSQL connection string
DATABASE_URL=postgresql://user:password@localhost:5432/hook0

# Maximum database connections
MAX_DB_CONNECTIONS=5

# Disable automatic database migration on startup
NO_AUTO_DB_MIGRATION=false
```

## Authentication

```bash
# Biscuit private key for token generation
BISCUIT_PRIVATE_KEY=your-hex-encoded-private-key

# Maximum authorization time (milliseconds)
MAX_AUTHORIZATION_TIME_IN_MS=10
```

## CORS

```bash
# Comma-separated allowed origins
CORS_ALLOWED_ORIGINS=https://app.hook0.com,https://your-domain.com
```

## API Rate Limiting

```bash
# Disable all rate limiting
DISABLE_API_RATE_LIMITING=false

# Global rate limiting (across all requests)
DISABLE_API_RATE_LIMITING_GLOBAL=false
API_RATE_LIMITING_GLOBAL_BURST_SIZE=2000
API_RATE_LIMITING_GLOBAL_REPLENISH_PERIOD_IN_MS=1

# Per-IP rate limiting
DISABLE_API_RATE_LIMITING_IP=false
API_RATE_LIMITING_IP_BURST_SIZE=200
API_RATE_LIMITING_IP_REPLENISH_PERIOD_IN_MS=10

# Per-token rate limiting
DISABLE_API_RATE_LIMITING_TOKEN=false
API_RATE_LIMITING_TOKEN_BURST_SIZE=20
API_RATE_LIMITING_TOKEN_REPLENISH_PERIOD_IN_MS=100
```

## Email Configuration

```bash
# Sender information
EMAIL_SENDER_ADDRESS=noreply@hook0.com
EMAIL_SENDER_NAME=Hook0

# SMTP connection URL
# Examples:
# - smtp://localhost:1025 (plain)
# - smtps://user:password@provider.com:465 (TLS)
# - smtp://user:password@provider.com:587?tls=required (STARTTLS)
SMTP_CONNECTION_URL=smtp://localhost:1025

# SMTP timeout (seconds)
SMTP_TIMEOUT_IN_S=5

# Logo URL for emails
EMAIL_LOGO_URL=https://app.hook0.com/256x256.png

# Frontend URL for email links
APP_URL=https://app.hook0.com
```

## Frontend

```bash
# Directory containing web app to serve
WEBAPP_PATH=../frontend/dist/

# Disable serving the web app (API only)
DISABLE_SERVING_WEBAPP=false
```

## Security

```bash
# Security headers
ENABLE_SECURITY_HEADERS=true
ENABLE_HSTS_HEADER=false

# Disable user registration
DISABLE_REGISTRATION=false

# Minimum password length
PASSWORD_MINIMUM_LENGTH=12
```

## Quotas

```bash
# Enable quota enforcement
ENABLE_QUOTA_ENFORCEMENT=false

# Default quota limits (can be overridden by plans)
QUOTA_GLOBAL_MEMBERS_PER_ORGANIZATION_LIMIT=1
QUOTA_GLOBAL_APPLICATIONS_PER_ORGANIZATION_LIMIT=1
QUOTA_GLOBAL_EVENTS_PER_DAY_LIMIT=100
QUOTA_GLOBAL_DAYS_OF_EVENTS_RETENTION_LIMIT=7
QUOTA_GLOBAL_SUBSCRIPTIONS_PER_APPLICATION_LIMIT=10
QUOTA_GLOBAL_EVENT_TYPES_PER_APPLICATION_LIMIT=10

# Quota notification threshold (percentage)
QUOTA_NOTIFICATION_EVENTS_PER_DAY_THRESHOLD=80
ENABLE_QUOTA_BASED_EMAIL_NOTIFICATIONS=false
```

## Cleanup Tasks

### Materialized Views

```bash
# Refresh period (seconds)
MATERIALIZED_VIEWS_REFRESH_PERIOD_IN_S=60
```

### Old Events

```bash
# Cleanup period (seconds)
OLD_EVENTS_CLEANUP_PERIOD_IN_S=3600

# Grace period before deletion (days)
OLD_EVENTS_CLEANUP_GRACE_PERIOD_IN_DAY=30

# Actually delete (vs just report)
OLD_EVENTS_CLEANUP_REPORT_AND_DELETE=false
```

### Expired Tokens

```bash
# Cleanup period (humantime format: 1h, 1d, etc.)
EXPIRED_TOKENS_CLEANUP_PERIOD=1h

# Grace period before deletion
EXPIRED_TOKENS_CLEANUP_GRACE_PERIOD=7d

# Actually delete (vs just report)
EXPIRED_TOKENS_CLEANUP_REPORT_AND_DELETE=false
```

### Unverified Users

```bash
# Enable cleanup
ENABLE_UNVERIFIED_USERS_CLEANUP=false

# Cleanup period (seconds)
UNVERIFIED_USERS_CLEANUP_PERIOD_IN_S=3600

# Grace period (days)
UNVERIFIED_USERS_CLEANUP_GRACE_PERIOD_IN_DAYS=7

# Actually delete (vs just report)
UNVERIFIED_USERS_CLEANUP_REPORT_AND_DELETE=false
```

### Soft-Deleted Applications

```bash
# Enable cleanup
ENABLE_SOFT_DELETED_APPLICATIONS_CLEANUP=false

# Cleanup period
SOFT_DELETED_APPLICATIONS_CLEANUP_PERIOD=1d

# Grace period
SOFT_DELETED_APPLICATIONS_CLEANUP_GRACE_PERIOD=30d
```

## Monitoring

```bash
# Sentry error tracking
SENTRY_DSN=https://your-sentry-dsn

# Sentry traces sample rate (0.0-1.0)
SENTRY_TRACES_SAMPLE_RATE=0.1

# Health check endpoint key (empty = public, unset = disabled)
HEALTH_CHECK_KEY=your-secret-key
```

## Analytics

```bash
# Matomo analytics
MATOMO_URL=https://analytics.example.com
MATOMO_SITE_ID=1

# Formbricks feedback
FORMBRICKS_API_HOST=https://app.formbricks.com
FORMBRICKS_ENVIRONMENT_ID=your-env-id
```

## Other

```bash
# Website URL
WEBSITE_URL=https://hook0.com

# Support email
SUPPORT_EMAIL_ADDRESS=support@hook0.com

# Cloudflare Turnstile (bot protection for registration)
CLOUDFLARE_TURNSTILE_SITE_KEY=your-site-key
CLOUDFLARE_TURNSTILE_SECRET_KEY=your-secret-key

# Global admin API key (USE AT YOUR OWN RISK)
MASTER_API_KEY=uuid-goes-here
```

## Hook0 Client (Self-hosting with upstream instance)

```bash
# Base API URL of upstream Hook0 instance
HOOK0_CLIENT_API_URL=https://api.hook0.com

# Application ID on upstream instance
HOOK0_CLIENT_APPLICATION_ID=uuid-goes-here

# Authentication token for upstream instance
HOOK0_CLIENT_TOKEN=your-token

# Retry attempts for event type upserts
HOOK0_CLIENT_UPSERTS_RETRIES=10
```

## Feature Flags (compile-time)

These are set during build, not runtime:

```bash
# Enable Keycloak migration support
ENABLE_KEYCLOAK_MIGRATION=true
KEYCLOAK_OIDC_PUBLIC_KEY=-----BEGIN PUBLIC KEY-----...
KEYCLOAK_URL=https://keycloak.example.com/auth
KEYCLOAK_REALM=myrealm
KEYCLOAK_CLIENT_ID=hook0
KEYCLOAK_CLIENT_SECRET=secret

# Enable application secret compatibility mode
ENABLE_APPLICATION_SECRET_COMPATIBILITY=true
```

## Output Worker Configuration

The output-worker is a separate binary with its own configuration.

### Core Settings

```bash
# Database connection
DATABASE_URL=postgresql://user:password@localhost:5432/hook0

# Worker identification
WORKER_NAME=default
WORKER_VERSION=0.1.0  # Optional, defaults to cargo version

# Concurrency (1-100)
CONCURRENT=10
```

### Retry Strategy

```bash
# Fast retries (exponential backoff from 5s to 5min)
MAX_FAST_RETRIES=30

# Slow retries (1 hour between attempts)
MAX_SLOW_RETRIES=30
```

### Timeouts

```bash
# Connection establishment timeout
CONNECT_TIMEOUT=5s

# Total request timeout (including connect)
TIMEOUT=15s
```

### Security

```bash
# Allow webhooks to target private IPs (NOT recommended for production)
DISABLE_TARGET_IP_CHECK=false

# Signature configuration
SIGNATURE_HEADER_NAME=X-Hook0-Signature
ENABLED_SIGNATURE_VERSIONS=v1
```

### Monitoring

```bash
# Sentry error tracking
SENTRY_DSN=https://your-sentry-dsn

# Heartbeat for dead man's switch monitoring
MONITORING_HEARTBEAT_URL=https://healthchecks.io/ping/your-uuid
MONITORING_HEARTBEAT_MIN_PERIOD_IN_S=60
```

## Notes

- Boolean values: `true`, `false` (case-insensitive)
- Durations: Use humantime format (`1h`, `30m`, `7d`) where supported, otherwise seconds
- Lists: Comma-separated
- URLs: Must be valid URLs with scheme
- Hook0 uses PostgreSQL (15+) and does NOT use Redis
