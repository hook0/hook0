# Configuration Reference

This reference covers all configuration options for Hook0, including environment variables, configuration files, and deployment settings for both self-hosted and cloud deployments.

## Environment Variables

### Core Configuration

#### Database Configuration

```bash
# PostgreSQL connection
DATABASE_URL=postgresql://user:password@localhost:5432/hook0

# Connection pool settings
DATABASE_MAX_CONNECTIONS=50
DATABASE_MIN_CONNECTIONS=5
DATABASE_ACQUIRE_TIMEOUT=30000
DATABASE_IDLE_TIMEOUT=600000
DATABASE_MAX_LIFETIME=1800000
```

#### Redis Configuration

```bash
# Redis connection for caching and queuing
REDIS_URL=redis://localhost:6379/0

# Redis connection pool
REDIS_MAX_CONNECTIONS=10
REDIS_MIN_CONNECTIONS=2
REDIS_CONNECT_TIMEOUT=5000
REDIS_COMMAND_TIMEOUT=3000
```

#### Server Configuration

```bash
# HTTP server settings
PORT=8000
HOST=0.0.0.0
REQUEST_TIMEOUT=30000

# CORS settings
CORS_ORIGINS=https://app.hook0.com,https://your-frontend.com
CORS_CREDENTIALS=true

# Rate limiting
RATE_LIMIT_WINDOW_MS=900000    # 15 minutes
RATE_LIMIT_MAX_REQUESTS=1000
RATE_LIMIT_SKIP_SUCCESSFUL=false
```

### Authentication & Security

#### Token Configuration

```bash
# Biscuit token settings
BISCUIT_PRIVATE_KEY=your-base64-encoded-private-key
BISCUIT_TOKEN_TTL=86400         # 24 hours in seconds
BISCUIT_REFRESH_TTL=2592000     # 30 days in seconds

# JWT settings (legacy support)
JWT_SECRET=your-jwt-secret-key
JWT_EXPIRES_IN=24h
JWT_ISSUER=hook0.com
```

#### Security Headers

```bash
# Security configuration
SECURITY_HSTS_MAX_AGE=31536000
SECURITY_CONTENT_TYPE_OPTIONS=nosniff
SECURITY_FRAME_OPTIONS=DENY
SECURITY_XSS_PROTECTION=1; mode=block

# CSRF protection
CSRF_SECRET=your-csrf-secret
CSRF_COOKIE_NAME=_hook0_csrf
CSRF_HEADER_NAME=X-CSRF-Token
```

### Webhook Processing

#### Worker Configuration

```bash
# Worker process settings
WORKER_CONCURRENCY=50
WORKER_BATCH_SIZE=100
WORKER_POLL_INTERVAL=1000      # milliseconds
WORKER_MAX_RETRY_ATTEMPTS=5
WORKER_INITIAL_RETRY_DELAY=1000
WORKER_MAX_RETRY_DELAY=300000
WORKER_RETRY_MULTIPLIER=2

# HTTP client settings
HTTP_CLIENT_TIMEOUT=30000
HTTP_CLIENT_MAX_CONNECTIONS=100
HTTP_CLIENT_KEEP_ALIVE=true
HTTP_CLIENT_KEEP_ALIVE_TIMEOUT=30000
```

#### Queue Configuration

```bash
# Queue settings
QUEUE_DEFAULT_PRIORITY=normal
QUEUE_HIGH_PRIORITY_WEIGHT=3
QUEUE_NORMAL_PRIORITY_WEIGHT=2
QUEUE_LOW_PRIORITY_WEIGHT=1

# Dead letter queue
DLQ_ENABLED=true
DLQ_MAX_ATTEMPTS=3
DLQ_RETENTION_DAYS=30
```

### Monitoring & Observability

#### Logging Configuration

```bash
# Logging level and format
LOG_LEVEL=info
LOG_FORMAT=json
LOG_TIMESTAMP=true
LOG_CALLER=false

# Log destinations
LOG_FILE_ENABLED=true
LOG_FILE_PATH=/var/log/hook0/app.log
LOG_FILE_MAX_SIZE=100MB
LOG_FILE_MAX_FILES=10
LOG_FILE_COMPRESS=true

# Structured logging
LOG_REQUEST_ID=true
LOG_USER_ID=true
LOG_PERFORMANCE=true
```

#### Metrics Configuration

```bash
# Prometheus metrics
METRICS_ENABLED=true
METRICS_PORT=9090
METRICS_PATH=/metrics
METRICS_NAMESPACE=hook0

# Custom metrics
METRICS_COLLECT_SYSTEM=true
METRICS_COLLECT_RUNTIME=true
METRICS_COLLECT_HTTP=true
METRICS_COLLECT_DATABASE=true
```

#### Tracing Configuration

```bash
# OpenTelemetry tracing
TRACING_ENABLED=true
TRACING_ENDPOINT=http://jaeger:14268/api/traces
TRACING_SERVICE_NAME=hook0-api
TRACING_SAMPLE_RATE=0.1

# Trace exporters
TRACING_JAEGER_ENABLED=true
TRACING_ZIPKIN_ENABLED=false
TRACING_OTLP_ENABLED=false
```

### External Services

#### Email Configuration

```bash
# SMTP settings
SMTP_HOST=smtp.sendgrid.net
SMTP_PORT=587
SMTP_USERNAME=apikey
SMTP_PASSWORD=your-sendgrid-api-key
SMTP_FROM_EMAIL=noreply@hook0.com
SMTP_FROM_NAME=Hook0

# Email templates
EMAIL_TEMPLATE_ENGINE=mjml
EMAIL_TEMPLATE_DIR=/app/templates
EMAIL_BASE_URL=https://app.hook0.com
```

#### Cloud Storage

```bash
# S3-compatible storage
STORAGE_PROVIDER=s3
STORAGE_BUCKET=hook0-events
STORAGE_REGION=us-east-1
STORAGE_ACCESS_KEY=your-access-key
STORAGE_SECRET_KEY=your-secret-key
STORAGE_ENDPOINT=https://s3.amazonaws.com

# Local storage (development)
STORAGE_PROVIDER=local
STORAGE_LOCAL_PATH=/var/lib/hook0/storage
```

#### External APIs

```bash
# Notification services
SLACK_WEBHOOK_URL=https://hooks.slack.com/services/...
DISCORD_WEBHOOK_URL=https://discord.com/api/webhooks/...
PAGERDUTY_INTEGRATION_KEY=your-pagerduty-key

# Analytics
ANALYTICS_PROVIDER=mixpanel
ANALYTICS_TOKEN=your-mixpanel-token
ANALYTICS_ENABLED=true
```

### Feature Flags

```bash
# Feature toggles
FEATURE_WEBSOCKET_API=true
FEATURE_BATCH_EVENTS=true
FEATURE_CIRCUIT_BREAKER=true
FEATURE_RATE_LIMITING=true
FEATURE_QUOTA_ENFORCEMENT=true
FEATURE_WEBHOOK_SIGNING=true
FEATURE_PAYLOAD_VALIDATION=true
FEATURE_REAL_TIME_METRICS=true
```

## Configuration Files

### YAML Configuration

Create `config/hook0.yml` for structured configuration:

```yaml
# config/hook0.yml
server:
  port: 8000
  host: "0.0.0.0"
  timeout: 30000
  cors:
    origins:
      - "https://app.hook0.com"
      - "https://your-frontend.com"
    credentials: true

database:
  url: "postgresql://user:password@localhost:5432/hook0"
  pool:
    max_connections: 50
    min_connections: 5
    acquire_timeout: 30000
    idle_timeout: 600000
    max_lifetime: 1800000

redis:
  url: "redis://localhost:6379/0"
  pool:
    max_connections: 10
    min_connections: 2

authentication:
  biscuit:
    private_key: "your-base64-encoded-private-key"
    token_ttl: 86400
    refresh_ttl: 2592000

workers:
  concurrency: 50
  batch_size: 100
  poll_interval: 1000
  retry:
    max_attempts: 5
    initial_delay: 1000
    max_delay: 300000
    multiplier: 2

monitoring:
  logging:
    level: "info"
    format: "json"
    file:
      enabled: true
      path: "/var/log/hook0/app.log"
      max_size: "100MB"
      max_files: 10
  
  metrics:
    enabled: true
    port: 9090
    path: "/metrics"
    namespace: "hook0"
  
  tracing:
    enabled: true
    endpoint: "http://jaeger:14268/api/traces"
    service_name: "hook0-api"
    sample_rate: 0.1

email:
  smtp:
    host: "smtp.sendgrid.net"
    port: 587
    username: "apikey"
    password: "your-sendgrid-api-key"
  from:
    email: "noreply@hook0.com"
    name: "Hook0"

features:
  websocket_api: true
  batch_events: true
  circuit_breaker: true
  rate_limiting: true
  quota_enforcement: true
  webhook_signing: true
  payload_validation: true
```

### JSON Configuration

Alternative JSON format in `config/hook0.json`:

```json
{
  "server": {
    "port": 8000,
    "host": "0.0.0.0",
    "timeout": 30000
  },
  "database": {
    "url": "postgresql://user:password@localhost:5432/hook0",
    "pool": {
      "max_connections": 50,
      "min_connections": 5,
      "acquire_timeout": 30000
    }
  },
  "workers": {
    "concurrency": 50,
    "batch_size": 100,
    "retry": {
      "max_attempts": 5,
      "initial_delay": 1000,
      "max_delay": 300000
    }
  }
}
```

## Docker Configuration

### Docker Compose Example

```yaml
# docker-compose.yml
version: '3.8'

services:
  hook0-api:
    image: hook0/hook0:latest
    environment:
      - DATABASE_URL=postgresql://hook0:password@postgres:5432/hook0
      - REDIS_URL=redis://redis:6379/0
      - LOG_LEVEL=info
      - WORKER_CONCURRENCY=20
    ports:
      - "8000:8000"
    depends_on:
      - postgres
      - redis
    volumes:
      - ./config:/app/config:ro
      - ./logs:/var/log/hook0
    restart: unless-stopped
    
  hook0-worker:
    image: hook0/hook0-worker:latest
    environment:
      - DATABASE_URL=postgresql://hook0:password@postgres:5432/hook0
      - REDIS_URL=redis://redis:6379/0
      - WORKER_CONCURRENCY=50
    depends_on:
      - postgres
      - redis
    restart: unless-stopped
    deploy:
      replicas: 3

  postgres:
    image: postgres:15
    environment:
      - POSTGRES_DB=hook0
      - POSTGRES_USER=hook0
      - POSTGRES_PASSWORD=password
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./config/postgres.conf:/etc/postgresql/postgresql.conf
    ports:
      - "5432:5432"
    command: postgres -c config_file=/etc/postgresql/postgresql.conf

  redis:
    image: redis:7-alpine
    volumes:
      - redis_data:/data
      - ./config/redis.conf:/usr/local/etc/redis/redis.conf
    ports:
      - "6379:6379"
    command: redis-server /usr/local/etc/redis/redis.conf

volumes:
  postgres_data:
  redis_data:
```

## Kubernetes Configuration

### ConfigMap Example

```yaml
# k8s/configmap.yml
apiVersion: v1
kind: ConfigMap
metadata:
  name: hook0-config
  namespace: hook0
data:
  DATABASE_URL: "postgresql://hook0:password@postgres:5432/hook0"
  REDIS_URL: "redis://redis:6379/0"
  LOG_LEVEL: "info"
  WORKER_CONCURRENCY: "50"
  METRICS_ENABLED: "true"
  TRACING_ENABLED: "true"
  
  hook0.yml: |
    server:
      port: 8000
      timeout: 30000
    
    workers:
      concurrency: 50
      batch_size: 100
    
    monitoring:
      metrics:
        enabled: true
        port: 9090
```

### Secret Example

```yaml
# k8s/secret.yml
apiVersion: v1
kind: Secret
metadata:
  name: hook0-secrets
  namespace: hook0
type: Opaque
stringData:
  BISCUIT_PRIVATE_KEY: "your-base64-encoded-private-key"
  SMTP_PASSWORD: "your-sendgrid-api-key"
  JWT_SECRET: "your-jwt-secret"
```

### Deployment Example

```yaml
# k8s/deployment.yml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: hook0-api
  namespace: hook0
spec:
  replicas: 3
  selector:
    matchLabels:
      app: hook0-api
  template:
    metadata:
      labels:
        app: hook0-api
    spec:
      containers:
      - name: hook0-api
        image: hook0/hook0:latest
        ports:
        - containerPort: 8000
        - containerPort: 9090
        envFrom:
        - configMapRef:
            name: hook0-config
        - secretRef:
            name: hook0-secrets
        volumeMounts:
        - name: config
          mountPath: /app/config
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "1Gi"
            cpu: "1000m"
        readinessProbe:
          httpGet:
            path: /health
            port: 8000
          initialDelaySeconds: 10
          periodSeconds: 5
        livenessProbe:
          httpGet:
            path: /health
            port: 8000
          initialDelaySeconds: 30
          periodSeconds: 10
      volumes:
      - name: config
        configMap:
          name: hook0-config
```

## Database Configuration

### PostgreSQL Settings

```sql
-- postgresql.conf optimizations
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

-- Connection settings
max_connections = 200
superuser_reserved_connections = 3

-- Logging (disable in production)
log_statement = 'none'
log_duration = off
log_lock_waits = on
log_checkpoints = on

-- Autovacuum
autovacuum_max_workers = 4
autovacuum_naptime = 30s
```

### Redis Configuration

```conf
# redis.conf
maxmemory 1gb
maxmemory-policy allkeys-lru

# Persistence
save 900 1
save 300 10
save 60 10000

# Network
tcp-keepalive 300
timeout 0

# Clients
maxclients 10000

# Append only file
appendonly yes
appendfsync everysec
```

## Performance Tuning

### High-Throughput Configuration

```bash
# For high-volume deployments
WORKER_CONCURRENCY=200
WORKER_BATCH_SIZE=500
HTTP_CLIENT_MAX_CONNECTIONS=500
DATABASE_MAX_CONNECTIONS=100
REDIS_MAX_CONNECTIONS=50

# Aggressive retry settings
WORKER_MAX_RETRY_ATTEMPTS=3
WORKER_INITIAL_RETRY_DELAY=500
WORKER_MAX_RETRY_DELAY=60000

# Queue optimization
QUEUE_HIGH_PRIORITY_WEIGHT=5
QUEUE_NORMAL_PRIORITY_WEIGHT=3
QUEUE_LOW_PRIORITY_WEIGHT=1
```

### Low-Resource Configuration

```bash
# For small deployments
WORKER_CONCURRENCY=10
WORKER_BATCH_SIZE=20
HTTP_CLIENT_MAX_CONNECTIONS=25
DATABASE_MAX_CONNECTIONS=20
REDIS_MAX_CONNECTIONS=10

# Conservative retry settings
WORKER_MAX_RETRY_ATTEMPTS=5
WORKER_INITIAL_RETRY_DELAY=2000
WORKER_MAX_RETRY_DELAY=300000
```

## Security Configuration

### Production Security Settings

```bash
# Security hardening
SECURITY_HSTS_MAX_AGE=31536000
SECURITY_HSTS_INCLUDE_SUBDOMAINS=true
SECURITY_HSTS_PRELOAD=true
SECURITY_CONTENT_TYPE_OPTIONS=nosniff
SECURITY_FRAME_OPTIONS=DENY
SECURITY_XSS_PROTECTION=1; mode=block
SECURITY_REFERRER_POLICY=strict-origin-when-cross-origin

# Rate limiting
RATE_LIMIT_ENABLED=true
RATE_LIMIT_WINDOW_MS=900000
RATE_LIMIT_MAX_REQUESTS=1000
RATE_LIMIT_STORE=redis

# IP filtering
IP_WHITELIST_ENABLED=false
IP_BLACKLIST_ENABLED=true
IP_BLACKLIST_FILE=/etc/hook0/blocked-ips.txt

# Request validation
PAYLOAD_MAX_SIZE=10485760    # 10MB
REQUEST_TIMEOUT=30000
BODY_PARSER_LIMIT=10mb
```

## Configuration Validation

Hook0 validates configuration on startup. Here's how to check your configuration:

### Environment Validation

```bash
# Check configuration
hook0 config validate

# Show current configuration
hook0 config show

# Test database connection
hook0 config test-db

# Test Redis connection
hook0 config test-redis
```

### Configuration Schema

Hook0 uses JSON Schema to validate configuration:

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "properties": {
    "server": {
      "type": "object",
      "properties": {
        "port": {
          "type": "integer",
          "minimum": 1,
          "maximum": 65535
        },
        "host": {
          "type": "string",
          "format": "hostname"
        },
        "timeout": {
          "type": "integer",
          "minimum": 1000,
          "maximum": 300000
        }
      },
      "required": ["port"]
    },
    "database": {
      "type": "object",
      "properties": {
        "url": {
          "type": "string",
          "pattern": "^postgresql://"
        }
      },
      "required": ["url"]
    }
  },
  "required": ["server", "database"]
}
```

## Configuration Best Practices

### Security
- Store sensitive values in environment variables or secrets
- Use separate configurations for different environments
- Regularly rotate secrets and API keys
- Enable security headers in production
- Use strong, unique passwords and keys

### Performance
- Tune worker concurrency based on your load
- Monitor and adjust retry settings
- Configure appropriate timeouts
- Use connection pooling effectively
- Monitor resource usage and scale accordingly

### Monitoring
- Enable comprehensive logging in production
- Configure metrics collection
- Set up alerting for critical issues
- Use structured logging with correlation IDs
- Monitor configuration drift

### Maintenance
- Version your configuration files
- Document configuration changes
- Test configuration changes in staging
- Use Infrastructure as Code when possible
- Backup configuration files regularly

For more advanced configuration options and deployment scenarios, see our [Self-hosting Guide](../tutorials/self-hosting-docker.md) and [Security Best Practices](../how-to-guides/secure-webhook-endpoints.md) guides.