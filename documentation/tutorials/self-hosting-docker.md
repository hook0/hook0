# Self-hosting Hook0 with Docker

This tutorial guides you through setting up a complete Hook0 installation using Docker and Docker Compose. You'll learn how to deploy, configure, and manage your own Hook0 instance.

## Prerequisites

- Docker and Docker Compose installed
- Basic understanding of containerization
- Familiarity with PostgreSQL
- Domain name and SSL certificates (for production)

## Architecture Overview

Hook0 consists of three main services:
- **API Server**: Handles event ingestion and management
- **Worker Process**: Processes webhook deliveries
- **Web Dashboard**: Provides the management interface
- **PostgreSQL**: Database for persistence

## Step 1: Prepare Your Environment

### Create Project Directory

```bash
mkdir hook0-self-hosted
cd hook0-self-hosted
```

### Create Environment Configuration

```bash
# .env
# Database Configuration
POSTGRES_USER=hook0
POSTGRES_PASSWORD=your-secure-password
POSTGRES_DB=hook0
DATABASE_URL=postgresql://hook0:your-secure-password@postgres:5432/hook0

# Hook0 API Configuration
HOOK0_CLIENT_API_URL=https://your-domain.com/api
HOOK0_CLIENT_FRONTEND_URL=https://your-domain.com
RUST_LOG=info

# Security
JWT_SECRET=your-jwt-secret-key-minimum-32-characters
BISCUIT_PRIVATE_KEY=your-biscuit-private-key

# Optional: Email Configuration (for notifications)
SMTP_HOST=smtp.your-domain.com
SMTP_PORT=587
SMTP_USERNAME=noreply@your-domain.com
SMTP_PASSWORD=your-smtp-password
SMTP_FROM=Hook0 <noreply@your-domain.com>

# Optional: Monitoring
SENTRY_DSN=https://your-sentry-dsn
```

### Generate Secrets

```bash
# Generate JWT secret
openssl rand -base64 32

# Generate Biscuit private key
openssl rand -base64 32
```

## Step 2: Create Docker Compose Configuration

### Basic Docker Compose Setup

```yaml
# docker-compose.yml
version: '3.8'

services:
  postgres:
    image: postgres:15
    environment:
      POSTGRES_USER: ${POSTGRES_USER}
      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD}
      POSTGRES_DB: ${POSTGRES_DB}
    volumes:
      - postgres_data:/var/lib/postgresql/data
    ports:
      - "5432:5432"
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U ${POSTGRES_USER}"]
      interval: 30s
      retries: 3
      start_period: 30s
      timeout: 10s

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 30s
      retries: 3
      start_period: 10s
      timeout: 5s

  hook0-api:
    image: hook0/hook0:latest
    environment:
      DATABASE_URL: ${DATABASE_URL}
      HOOK0_CLIENT_API_URL: ${HOOK0_CLIENT_API_URL}
      HOOK0_CLIENT_FRONTEND_URL: ${HOOK0_CLIENT_FRONTEND_URL}
      RUST_LOG: ${RUST_LOG}
      JWT_SECRET: ${JWT_SECRET}
      BISCUIT_PRIVATE_KEY: ${BISCUIT_PRIVATE_KEY}
    ports:
      - "8000:8000"
    depends_on:
      postgres:
        condition: service_healthy
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8000/health"]
      interval: 30s
      retries: 3
      start_period: 60s
      timeout: 10s

  hook0-worker:
    image: hook0/hook0-worker:latest
    environment:
      DATABASE_URL: ${DATABASE_URL}
      RUST_LOG: ${RUST_LOG}
    depends_on:
      postgres:
        condition: service_healthy
      hook0-api:
        condition: service_healthy
    deploy:
      replicas: 2

  hook0-frontend:
    image: hook0/hook0-frontend:latest
    environment:
      VITE_API_URL: ${HOOK0_CLIENT_API_URL}
    ports:
      - "3000:80"
    depends_on:
      - hook0-api

volumes:
  postgres_data:
```

## Step 3: Production Configuration

### Production Docker Compose with Reverse Proxy

```yaml
# docker-compose.prod.yml
version: '3.8'

services:
  traefik:
    image: traefik:v2.10
    command:
      - "--api.insecure=true"
      - "--providers.docker=true"
      - "--providers.docker.exposedbydefault=false"
      - "--entrypoints.websecure.address=:443"
      - "--entrypoints.web.address=:80"
      - "--certificatesresolvers.myresolver.acme.tlschallenge=true"
      - "--certificatesresolvers.myresolver.acme.email=admin@your-domain.com"
      - "--certificatesresolvers.myresolver.acme.storage=/letsencrypt/acme.json"
    ports:
      - "80:80"
      - "443:443"
      - "8080:8080"
    volumes:
      - "/var/run/docker.sock:/var/run/docker.sock:ro"
      - "./letsencrypt:/letsencrypt"

  postgres:
    image: postgres:15
    environment:
      POSTGRES_USER: ${POSTGRES_USER}
      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD}
      POSTGRES_DB: ${POSTGRES_DB}
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./backups:/backups
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U ${POSTGRES_USER}"]
      interval: 30s
      retries: 3
      start_period: 30s
      timeout: 10s

  redis:
    image: redis:7-alpine
    volumes:
      - redis_data:/data
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 30s
      retries: 3
      start_period: 10s
      timeout: 5s

  hook0-api:
    image: hook0/hook0:latest
    environment:
      DATABASE_URL: ${DATABASE_URL}
      HOOK0_CLIENT_API_URL: https://api.your-domain.com
      HOOK0_CLIENT_FRONTEND_URL: https://your-domain.com
      RUST_LOG: info
      JWT_SECRET: ${JWT_SECRET}
      BISCUIT_PRIVATE_KEY: ${BISCUIT_PRIVATE_KEY}
      SMTP_HOST: ${SMTP_HOST}
      SMTP_PORT: ${SMTP_PORT}
      SMTP_USERNAME: ${SMTP_USERNAME}
      SMTP_PASSWORD: ${SMTP_PASSWORD}
      SMTP_FROM: ${SMTP_FROM}
    depends_on:
      postgres:
        condition: service_healthy
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.hook0-api.rule=Host(`api.your-domain.com`)"
      - "traefik.http.routers.hook0-api.entrypoints=websecure"
      - "traefik.http.routers.hook0-api.tls.certresolver=myresolver"
      - "traefik.http.services.hook0-api.loadbalancer.server.port=8000"
    deploy:
      replicas: 2
      resources:
        limits:
          memory: 512M
        reservations:
          memory: 256M

  hook0-worker:
    image: hook0/hook0-worker:latest
    environment:
      DATABASE_URL: ${DATABASE_URL}
      RUST_LOG: info
    depends_on:
      postgres:
        condition: service_healthy
    deploy:
      replicas: 3
      resources:
        limits:
          memory: 256M
        reservations:
          memory: 128M

  hook0-frontend:
    image: hook0/hook0-frontend:latest
    environment:
      VITE_API_URL: https://api.your-domain.com
    depends_on:
      - hook0-api
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.hook0-frontend.rule=Host(`your-domain.com`)"
      - "traefik.http.routers.hook0-frontend.entrypoints=websecure"
      - "traefik.http.routers.hook0-frontend.tls.certresolver=myresolver"
      - "traefik.http.services.hook0-frontend.loadbalancer.server.port=80"

volumes:
  postgres_data:
  redis_data:
```

## Step 4: Initialize the Database

### Run Database Migrations

```bash
# Start only PostgreSQL first
docker-compose up -d postgres

# Wait for PostgreSQL to be ready
docker-compose exec postgres pg_isready -U hook0

# Run migrations (if available)
docker-compose run --rm hook0-api migrate

# Or manually create database schema
docker-compose exec postgres psql -U hook0 -d hook0 -f /path/to/schema.sql
```

### Create Initial Admin User

```bash
# Create admin user (if supported by Hook0)
docker-compose exec hook0-api create-user \
  --email admin@your-domain.com \
  --name "Admin User" \
  --role admin
```

## Step 5: Start the Services

### Development Environment

```bash
# Start all services
docker-compose up -d

# Check service status
docker-compose ps

# View logs
docker-compose logs -f hook0-api
docker-compose logs -f hook0-worker
```

### Production Environment

```bash
# Start production environment
docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d

# Check health
curl -f https://api.your-domain.com/health
curl -f https://your-domain.com
```

## Step 6: Configure SSL and Domain

### DNS Configuration

```
# A Records
your-domain.com       → your-server-ip
api.your-domain.com   → your-server-ip
```

### Manual SSL Setup (Alternative to Let's Encrypt)

```bash
# Create SSL directory
mkdir ssl

# Place your certificates
cp your-domain.com.crt ssl/
cp your-domain.com.key ssl/
cp api.your-domain.com.crt ssl/
cp api.your-domain.com.key ssl/
```

### Nginx Configuration (Alternative to Traefik)

```nginx
# nginx.conf
upstream hook0_api {
    server hook0-api:8000;
}

upstream hook0_frontend {
    server hook0-frontend:80;
}

server {
    listen 80;
    server_name your-domain.com api.your-domain.com;
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name api.your-domain.com;
    
    ssl_certificate /etc/ssl/certs/api.your-domain.com.crt;
    ssl_certificate_key /etc/ssl/private/api.your-domain.com.key;
    
    location / {
        proxy_pass http://hook0_api;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}

server {
    listen 443 ssl http2;
    server_name your-domain.com;
    
    ssl_certificate /etc/ssl/certs/your-domain.com.crt;
    ssl_certificate_key /etc/ssl/private/your-domain.com.key;
    
    location / {
        proxy_pass http://hook0_frontend;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

## Step 7: Monitoring and Logging

### Health Check Script

```bash
#!/bin/bash
# health-check.sh

services=("hook0-api" "hook0-worker" "hook0-frontend" "postgres" "redis")

for service in "${services[@]}"; do
    if docker-compose ps $service | grep -q "Up"; then
        echo "✅ $service is running"
    else
        echo "❌ $service is not running"
        docker-compose logs --tail=20 $service
    fi
done

# Check API health endpoint
if curl -f https://api.your-domain.com/health > /dev/null 2>&1; then
    echo "✅ API health check passed"
else
    echo "❌ API health check failed"
fi
```

### Log Aggregation with ELK Stack

```yaml
# docker-compose.monitoring.yml
version: '3.8'

services:
  elasticsearch:
    image: docker.elastic.co/elasticsearch/elasticsearch:8.8.0
    environment:
      - discovery.type=single-node
      - xpack.security.enabled=false
    volumes:
      - elasticsearch_data:/usr/share/elasticsearch/data
    ports:
      - "9200:9200"

  logstash:
    image: docker.elastic.co/logstash/logstash:8.8.0
    volumes:
      - ./logstash.conf:/usr/share/logstash/pipeline/logstash.conf
    depends_on:
      - elasticsearch

  kibana:
    image: docker.elastic.co/kibana/kibana:8.8.0
    environment:
      ELASTICSEARCH_HOSTS: http://elasticsearch:9200
    ports:
      - "5601:5601"
    depends_on:
      - elasticsearch

volumes:
  elasticsearch_data:
```

### Prometheus Monitoring

```yaml
# prometheus.yml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'hook0-api'
    static_configs:
      - targets: ['hook0-api:8000']
    metrics_path: /metrics
    scrape_interval: 30s

  - job_name: 'hook0-worker'
    static_configs:
      - targets: ['hook0-worker:8000']
    metrics_path: /metrics
    scrape_interval: 30s
```

## Step 8: Backup and Recovery

### Database Backup Script

```bash
#!/bin/bash
# backup.sh

BACKUP_DIR="/backups"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="hook0_backup_${TIMESTAMP}.sql"

# Create backup
docker-compose exec -T postgres pg_dump -U hook0 hook0 > "${BACKUP_DIR}/${BACKUP_FILE}"

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

if [ $# -eq 0 ]; then
    echo "Usage: $0 <backup_file.sql.gz>"
    exit 1
fi

BACKUP_FILE=$1

# Stop services
docker-compose stop hook0-api hook0-worker

# Restore database
gunzip -c ${BACKUP_FILE} | docker-compose exec -T postgres psql -U hook0 -d hook0

# Start services
docker-compose start hook0-api hook0-worker

echo "Restore completed from: ${BACKUP_FILE}"
```

### Automated Backup with Cron

```bash
# Add to crontab
# Backup every day at 2 AM
0 2 * * * /path/to/hook0-self-hosted/backup.sh

# Health check every 5 minutes
*/5 * * * * /path/to/hook0-self-hosted/health-check.sh
```

## Step 9: Scaling and Performance Tuning

### Horizontal Scaling Configuration

```yaml
# docker-compose.scale.yml
version: '3.8'

services:
  hook0-api:
    deploy:
      replicas: 3
      resources:
        limits:
          cpus: '1.0'
          memory: 1G
        reservations:
          cpus: '0.5'
          memory: 512M

  hook0-worker:
    deploy:
      replicas: 5
      resources:
        limits:
          cpus: '0.5'
          memory: 512M
        reservations:
          cpus: '0.25'
          memory: 256M

  postgres:
    environment:
      POSTGRES_SHARED_BUFFERS: 256MB
      POSTGRES_EFFECTIVE_CACHE_SIZE: 1GB
      POSTGRES_MAINTENANCE_WORK_MEM: 64MB
      POSTGRES_CHECKPOINT_COMPLETION_TARGET: 0.7
      POSTGRES_WAL_BUFFERS: 16MB
    volumes:
      - ./postgresql.conf:/etc/postgresql/postgresql.conf
    command: postgres -c config_file=/etc/postgresql/postgresql.conf
```

### PostgreSQL Performance Tuning

```bash
# postgresql.conf
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
max_parallel_workers_per_gather = 2
max_parallel_workers = 8
max_parallel_maintenance_workers = 2
```

## Step 10: Security Hardening

### Security Configuration

```yaml
# docker-compose.security.yml
version: '3.8'

services:
  hook0-api:
    security_opt:
      - no-new-privileges:true
    user: "1000:1000"
    read_only: true
    tmpfs:
      - /tmp:noexec,nosuid,size=100m

  hook0-worker:
    security_opt:
      - no-new-privileges:true
    user: "1000:1000"
    read_only: true
    tmpfs:
      - /tmp:noexec,nosuid,size=100m

  postgres:
    security_opt:
      - no-new-privileges:true
    environment:
      POSTGRES_INITDB_ARGS: "--auth-host=scram-sha-256"
```

### Firewall Configuration

```bash
# UFW firewall rules
ufw allow 22/tcp    # SSH
ufw allow 80/tcp    # HTTP
ufw allow 443/tcp   # HTTPS
ufw deny 5432/tcp   # PostgreSQL (internal only)
ufw deny 6379/tcp   # Redis (internal only)
ufw enable
```

## What You've Learned

✅ Set up complete Hook0 self-hosted environment  
✅ Configured production-ready Docker Compose setup  
✅ Implemented SSL/TLS with automatic certificate management  
✅ Set up monitoring and logging infrastructure  
✅ Created backup and recovery procedures  
✅ Configured horizontal scaling  
✅ Implemented security hardening measures  

## Best Practices

### Deployment
- ✅ Use health checks for all services
- ✅ Implement proper resource limits
- ✅ Use secrets management for sensitive data
- ✅ Set up automated backups
- ✅ Monitor service health continuously

### Security
- ✅ Use HTTPS for all endpoints
- ✅ Keep containers updated
- ✅ Run containers as non-root users
- ✅ Use read-only filesystems where possible
- ✅ Implement proper firewall rules

### Operations
- ✅ Set up log aggregation
- ✅ Monitor resource usage
- ✅ Implement alerting for failures
- ✅ Test backup and recovery procedures
- ✅ Document operational procedures

## Next Steps

- [Managing High-Volume Event Processing](../how-to-guides/high-volume-processing.md)
- [Monitoring Webhook Performance](../how-to-guides/monitor-performance.md)
- [Configuration Reference](../reference/configuration.md)

## Troubleshooting

### Services Won't Start
1. Check Docker Compose syntax
2. Verify environment variables
3. Check port conflicts
4. Review service logs
5. Ensure database is accessible

### SSL Certificate Issues
1. Verify DNS resolution
2. Check certificate expiration
3. Validate certificate chain
4. Ensure port 80/443 are accessible
5. Review Let's Encrypt rate limits

### Performance Issues
1. Monitor resource usage
2. Check database performance
3. Review worker scaling
4. Analyze slow queries
5. Optimize connection pooling