# Self-hosting Hook0 in production

This tutorial guides you through setting up a complete Hook0 installation using Docker and Docker Compose.

## Prerequisites

- Docker and Docker Compose installed
- Basic understanding of containerization
- Familiarity with PostgreSQL
- Domain name and SSL certificates (for production)
- SMTP server for sending emails (or use Mailpit for testing)

## Architecture Overview

Hook0 consists of four main components:
- **api**: Handles event ingestion and management
- **frontend**: Provides the management interface
- **output-worker**: Processes webhook deliveries
- **postgres**: Database for persistence (PostgreSQL 16)

:::tip No Redis Required
Hook0 uses PostgreSQL for all persistence and queuing needs. This simplifies deployment and reduces operational overhead compared to systems requiring Redis.
:::

## Step 1: Prepare Your Environment

### Create Project Directory

```bash
mkdir hook0-self-hosted
cd hook0-self-hosted
```

### Generate Biscuit Private Key

```bash
# Generate a random hex key (64 characters)
openssl rand -hex 32
```

Save this key - you'll need it in the next step.

### Create Environment Configuration

Create a `.env` file with your configuration. Replace placeholder values with your actual settings:

```bash
# .env

# Database Configuration
POSTGRES_PASSWORD=your-secure-password-here
DATABASE_URL=postgres://postgres:your-secure-password-here@postgres:5432/hook0

# API Configuration
IP=0.0.0.0
PORT=8081
CORS_ALLOWED_ORIGINS=https://your-domain.com

# Security - Use the key generated above
BISCUIT_PRIVATE_KEY=your-generated-hex-key-here

# Email Configuration - Use a real SMTP server in production
# Examples:
#   SendGrid: smtps://apikey:SG.xxxxx@smtp.sendgrid.net:465
#   Mailgun:  smtps://postmaster@mg.example.com:password@smtp.mailgun.org:465
#   AWS SES:  smtps://AKIAXXXXX:secret@email-smtp.us-east-1.amazonaws.com:465
SMTP_CONNECTION_URL=smtps://user:password@smtp.example.com:465
EMAIL_SENDER_ADDRESS=noreply@your-domain.com
EMAIL_SENDER_NAME=Hook0

# Frontend URL (used in email links)
APP_URL=https://your-domain.com

# Security
ENABLE_SECURITY_HEADERS=true
ENABLE_HSTS_HEADER=true
DISABLE_REGISTRATION=false
PASSWORD_MINIMUM_LENGTH=12

# Monitoring (optional)
HEALTH_CHECK_KEY=your-secret-health-key
# SENTRY_DSN=https://xxx@xxx.ingest.sentry.io/xxx  # Uncomment to enable Sentry
```

:::warning Password Consistency
Make sure `POSTGRES_PASSWORD` matches the password in `DATABASE_URL`.
:::

## Step 2: Create Docker Compose Configuration

Create a `docker-compose.yml` file:

```yaml
# docker-compose.yml

volumes:
  postgres-data:

services:
  postgres:
    image: postgres:16-alpine
    volumes:
      - postgres-data:/var/lib/postgresql/data
    environment:
      - POSTGRES_PASSWORD=${POSTGRES_PASSWORD}
      - POSTGRES_DB=hook0
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U postgres -d hook0"]
      timeout: 5s
      interval: 10s
      retries: 5
    restart: unless-stopped
    # Do not expose port externally in production

  api:
    # Pin to a specific version tag for reproducible builds
    build:
      context: https://github.com/hook0/hook0.git#v1.0.0
      dockerfile: api/Dockerfile
      args:
        FEATURES: "reqwest-rustls-tls-webpki-roots,application-secret-compatibility"
    environment:
      - IP=${IP}
      - PORT=${PORT}
      - CORS_ALLOWED_ORIGINS=${CORS_ALLOWED_ORIGINS}
      - DATABASE_URL=${DATABASE_URL}
      - SMTP_CONNECTION_URL=${SMTP_CONNECTION_URL}
      - EMAIL_SENDER_ADDRESS=${EMAIL_SENDER_ADDRESS}
      - EMAIL_SENDER_NAME=${EMAIL_SENDER_NAME}
      - APP_URL=${APP_URL}
      - BISCUIT_PRIVATE_KEY=${BISCUIT_PRIVATE_KEY}
      - ENABLE_SECURITY_HEADERS=${ENABLE_SECURITY_HEADERS}
      - ENABLE_HSTS_HEADER=${ENABLE_HSTS_HEADER}
      - HEALTH_CHECK_KEY=${HEALTH_CHECK_KEY}
      - RUST_LOG=info
      # - SENTRY_DSN=${SENTRY_DSN}  # Uncomment to enable Sentry
    ports:
      - "8081:8081"
    healthcheck:
      test: ["CMD-SHELL", "curl --fail http://localhost:8081/api/v1/swagger.json || exit 1"]
      timeout: 5s
      interval: 10s
      retries: 3
    depends_on:
      postgres:
        condition: service_healthy
    restart: unless-stopped

  frontend:
    build:
      context: https://github.com/hook0/hook0.git#v1.0.0
      dockerfile: frontend/Dockerfile
    ports:
      - "8001:80"
    healthcheck:
      test: ["CMD-SHELL", "curl --fail http://localhost || exit 1"]
      timeout: 5s
      interval: 10s
      retries: 3
    depends_on:
      api:
        condition: service_healthy
    restart: unless-stopped

  output-worker:
    build:
      context: https://github.com/hook0/hook0.git#v1.0.0
      dockerfile: output-worker/Dockerfile
      args:
        FEATURES: "reqwest-rustls-tls-webpki-roots"
    environment:
      - DATABASE_URL=${DATABASE_URL}
      - WORKER_NAME=prod-worker-1
      - RUST_LOG=info
      # For local testing only: allow webhooks to private IPs (localhost, Docker networks)
      # - DISABLE_TARGET_IP_CHECK=true
    depends_on:
      postgres:
        condition: service_healthy
      api:
        condition: service_healthy
    restart: unless-stopped

  # ============================================================
  # OPTIONAL: Mailpit for local email testing
  # Uncomment this section if you don't have an SMTP server yet.
  # Remember to also update SMTP_CONNECTION_URL in your .env:
  #   SMTP_CONNECTION_URL=smtp://mailpit:1025
  # ============================================================
  # mailpit:
  #   image: axllent/mailpit:latest
  #   ports:
  #     - "8025:8025"  # Web UI
  #     - "1025:1025"  # SMTP
  #   healthcheck:
  #     test: ["CMD-SHELL", "wget --no-verbose --tries=1 --spider http://localhost:8025/api/v1/info || exit 1"]
  #     timeout: 1s
  #     interval: 5s
  #   restart: unless-stopped
```

:::warning Version Pinning
Always pin to a specific Git tag (e.g., `#v1.0.0`) instead of building from the default branch. Building from `main` means your production could break if a bug is pushed minutes before your deployment.
:::

:::tip Local Email Testing with Mailpit
If you don't have an SMTP server configured yet, uncomment the `mailpit` service in the compose file and set `SMTP_CONNECTION_URL=smtp://mailpit:1025` in your `.env` file. You can then view all outgoing emails at http://localhost:8025.
:::

## Step 3: Start the Services

:::warning Build Time
The first build compiles Rust code and may take **10-15 minutes** depending on your hardware.
Subsequent builds use Docker layer caching and are much faster.
:::

```bash
# Start all services
docker compose up -d

# Check service status
docker compose ps

# View logs
docker compose logs -f api
docker compose logs -f output-worker
```

### Access the Application

- **Frontend**: http://localhost:8001
- **API**: http://localhost:8081
- **API Docs**: http://localhost:8081/api/v1/swagger.json
- **Mailpit** (if enabled): http://localhost:8025

## Step 4: Initial Setup

After starting the services, you need to bootstrap your Hook0 instance with an admin account and organization.

### 4.1: Access the Dashboard

Open your browser and navigate to:
```
http://localhost:8001
```

### 4.2: Create Your Account

On the first visit, you'll see the login page:

1. **Click "Sign up"** to create a new account
2. **Fill in your details**: Email, First Name, Last Name, Password
3. **Click "Register"**

### 4.3: Verify Your Email

After registration, you need to verify your email:

1. **Check your email** (or Mailpit at http://localhost:8025 if using local testing)
2. **Find the verification email** from Hook0
3. **Click the verification link** in the email
4. **Return to the dashboard** and log in with your credentials

:::tip Verification Required
Without email verification, you'll see an `AuthEmailNotVerified` error when trying to access the API.
:::

### 4.4: Complete the Setup Tutorial

After logging in, Hook0 displays an **interactive tutorial** that guides you through:

1. **Creating an organization** - Your team workspace
2. **Creating an application** - Where your webhooks live
3. **Creating an event type** - The events your app sends
4. **Creating a subscription** - Where webhooks are delivered
5. **Sending a test event** - Verify everything works

:::tip Skip the Tutorial?
You can skip this tutorial and set things up manually or via the API. See [Getting Started](./getting-started.md) for the programmatic approach.
:::

### 4.5: Generate API Token

Once you have an application (created via the tutorial or manually):

1. **Select your application** in the sidebar
2. **Click "API keys"** in the application menu
3. **Click "Create new API Key"**
4. **Enter a name** (e.g., "Production API Key")
5. **Copy the token** - you'll need it for API calls

### 4.6: Verify Installation

Test your setup with a simple API call. Replace the placeholder values with your actual IDs and token from the previous steps:

```bash
# Set your variables (from dashboard)
HOOK0_API="http://localhost:8081/api/v1"
HOOK0_TOKEN="your-api-token-from-step-4.5"
APP_ID="your-application-id-from-dashboard"

# Generate a UUID for the event
EVENT_UUID=$(uuidgen | tr '[:upper:]' '[:lower:]')

# Send a test event
curl -X POST "$HOOK0_API/event/" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "'"$APP_ID"'",
    "event_id": "'"$EVENT_UUID"'",
    "event_type": "system.test.completed",
    "payload": "{\"message\": \"Hook0 is working\"}",
    "payload_content_type": "application/json",
    "occurred_at": "2024-01-15T10:00:00Z",
    "labels": {
      "environment": "test"
    }
  }'
```

If successful, you'll receive a response with an event ID:
```json
{
  "application_id": "your-app-id",
  "event_id": "generated-uuid",
  "received_at": "2024-01-15T10:00:00Z"
}
```

### 4.7: Check Event in Dashboard

1. Go to **Events** in the dashboard
2. You should see your test event listed
3. Click on it to view details including:
   - Event type
   - Payload
   - Timestamp
   - Delivery attempts (if subscriptions exist)

**Note**: Without subscriptions, events are stored but not delivered. Continue to the [Getting Started tutorial](getting-started.md) to create subscriptions and webhooks.

## Step 5: Reverse Proxy Setup

For production, set up a reverse proxy with SSL termination.

### Nginx Configuration

```nginx
# /etc/nginx/sites-available/hook0
upstream hook0_api {
    server localhost:8081;
}

upstream hook0_frontend {
    server localhost:8001;
}

server {
    listen 80;
    server_name your-domain.com api.your-domain.com;
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl;
    http2 on;  # Nginx 1.25.1+ syntax (replaces deprecated 'listen 443 ssl http2')
    server_name api.your-domain.com;

    ssl_certificate /etc/letsencrypt/live/api.your-domain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/api.your-domain.com/privkey.pem;

    # Security headers
    add_header Strict-Transport-Security "max-age=63072000" always;

    location / {
        proxy_pass http://hook0_api;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_read_timeout 60s;
    }
}

server {
    listen 443 ssl;
    http2 on;
    server_name your-domain.com;

    ssl_certificate /etc/letsencrypt/live/your-domain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/your-domain.com/privkey.pem;

    add_header Strict-Transport-Security "max-age=63072000" always;

    location / {
        proxy_pass http://hook0_frontend;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

### Enable Site

```bash
sudo ln -s /etc/nginx/sites-available/hook0 /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl reload nginx
```

## Step 6: Backup and Recovery

### Database Backup Script

```bash
#!/bin/bash
# backup.sh
set -e  # Exit immediately on error (prevents silent failures)

BACKUP_DIR="./backups"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="hook0_backup_${TIMESTAMP}.sql"

# Ensure backup directory exists
mkdir -p "${BACKUP_DIR}"

# Create backup
docker compose exec -T postgres pg_dump -U postgres hook0 > "${BACKUP_DIR}/${BACKUP_FILE}"

# Compress backup
gzip "${BACKUP_DIR}/${BACKUP_FILE}"

# Remove backups older than 30 days
find ${BACKUP_DIR} -name "hook0_backup_*.sql.gz" -mtime +30 -delete

echo "Backup completed: ${BACKUP_FILE}.gz"
```

### Database Restore Script

```bash
#!/bin/bash
# restore.sh
set -e

if [ $# -eq 0 ]; then
    echo "Usage: $0 <backup_file.sql.gz>"
    exit 1
fi

BACKUP_FILE=$1

# Stop services
docker compose stop api output-worker

# Restore database
gunzip -c ${BACKUP_FILE} | docker compose exec -T postgres psql -U postgres -d hook0

# Start services
docker compose start api output-worker

echo "Restore completed from: ${BACKUP_FILE}"
```

### Automated Backup with Cron

```bash
# Add to crontab
# Backup every day at 2 AM
0 2 * * * /path/to/hook0-self-hosted/backup.sh

# Health check every 5 minutes (requires HEALTH_CHECK_KEY env var in API)
*/5 * * * * curl -f "http://localhost:8081/api/v1/health/?key=your-health-key" || echo "Health check failed"
```

## Step 7: Scaling

### Horizontal Scaling

```bash
# Scale workers
docker compose up -d --scale output-worker=5

# Scale API
docker compose up -d --scale api=3
```

:::tip Scaling Services
Use `docker compose up -d --scale api=2 --scale output-worker=3` to run multiple instances. The `deploy:` section with `replicas` is only supported by Docker Swarm mode (`docker stack deploy`), not standard Docker Compose.
:::

### PostgreSQL Performance Tuning

```bash
# postgresql.conf additions
shared_buffers = 256MB
effective_cache_size = 1GB
maintenance_work_mem = 64MB
checkpoint_completion_target = 0.7
wal_buffers = 16MB
default_statistics_target = 100
random_page_cost = 1.1
effective_io_concurrency = 200
work_mem = 4MB
min_wal_size = 1GB
max_wal_size = 4GB
max_worker_processes = 8
max_parallel_workers = 8
```

## What You've Learned

- Set up Hook0 self-hosted environment
- Configured production-ready Docker Compose setup
- Implemented reverse proxy with Nginx
- Created backup and recovery procedures
- Configured horizontal scaling

## Best Practices

### Deployment
- Use health checks for all services
- Implement proper resource limits
- Use secrets management for sensitive data
- Set up automated backups
- Monitor service health continuously

### Security
- Use HTTPS for all endpoints
- Keep containers updated
- Run containers as non-root users where possible
- Implement proper firewall rules
- Use strong Biscuit private keys

### Operations
- Set up log aggregation
- Monitor resource usage
- Implement alerting for failures
- Test backup and recovery procedures
- Document operational procedures

## Next Steps

- [Configuration Reference](../reference/configuration.md)
- [Debugging Failed Webhooks](../how-to-guides/debug-failed-webhooks.md)
- [Securing Webhook Endpoints](../how-to-guides/secure-webhook-endpoints.md)

## Troubleshooting

### Services will not start

1. Check Docker Compose syntax
2. Verify environment variables
3. Check port conflicts
4. Review service logs: `docker compose logs service-name`
5. Ensure database is accessible

### Database Connection Issues

1. Verify DATABASE_URL format
2. Check PostgreSQL is running: `docker compose ps postgres`
3. Check health: `docker compose exec postgres pg_isready`
4. Review PostgreSQL logs: `docker compose logs postgres`

### Performance Issues

1. Monitor resource usage: `docker stats`
2. Check database performance: slow query logs
3. Review worker scaling
4. Optimize connection pooling: MAX_DB_CONNECTIONS

### Webhooks Not Delivered (Invalid Target Error)

If you see `Target has an invalid URL: URL resolves to a forbidden IP` in the output-worker logs:

1. **This is expected for private IPs** - Hook0 blocks webhooks to private/local IPs by default (SSRF protection)
2. **For local testing only**, add `DISABLE_TARGET_IP_CHECK=true` to the output-worker environment
3. **In production**, webhooks must target publicly routable IPs

```bash
# Check output-worker logs for delivery errors
docker compose logs output-worker | grep -i "invalid\|forbidden\|error"
```

:::warning Security
Never enable `DISABLE_TARGET_IP_CHECK` in production. This protection prevents Server-Side Request Forgery (SSRF) attacks.
:::
