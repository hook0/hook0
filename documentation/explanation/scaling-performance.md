# Scaling and Performance

Understanding how Hook0 scales and performs is essential for production deployments. This document explains Hook0's architecture for horizontal scaling, performance characteristics, and resource management.

## Architecture for Scalability

Hook0 is designed for horizontal scaling through a separation of concerns between components.

### API Server and Output Worker Separation

Hook0 uses a two-component architecture:

- **API Server** (`api/`): Handles incoming HTTP requests, manages authentication, validates event ingestion, and stores events in PostgreSQL
- **Output Workers** (`output-worker/`): Pick up pending webhook delivery attempts from PostgreSQL and execute HTTP requests to target endpoints

This separation allows each component to scale independently based on workload:
- Scale API servers based on incoming event ingestion rate and API request volume
- Scale output workers based on webhook delivery throughput and retry processing needs

### Database-Centric Coordination

Unlike systems that rely on Redis or other external message queues, Hook0 uses **PostgreSQL exclusively** for:
- Event storage and persistence
- Request attempt queue management
- Worker coordination via `FOR UPDATE ... SKIP LOCKED` queries
- State management (succeeded_at, failed_at, delay_until)

This approach:
- Eliminates the need for additional infrastructure components
- Provides ACID guarantees for event processing
- Simplifies deployment and operational complexity
- Uses PostgreSQL's robust connection pooling and query optimization

### Horizontal Scaling Pattern

**Scaling API Servers:**
- Deploy multiple API server instances behind a load balancer
- Each instance connects to PostgreSQL with its own connection pool
- Stateless design allows adding/removing instances dynamically
- Configure `MAX_DB_CONNECTIONS` per instance (default: 5)

**Scaling Output Workers:**
- Deploy multiple worker instances (public or private workers)
- Each worker polls PostgreSQL for pending request attempts
- PostgreSQL row-level locking (`FOR UPDATE ... SKIP LOCKED`) prevents duplicate processing
- Configure `--concurrent` flag to control how many webhook requests each worker handles simultaneously (1-100)

## Connection Pooling

### PostgreSQL Connection Management

**API Server Configuration:**
```bash
# Maximum connections to PostgreSQL per API instance
MAX_DB_CONNECTIONS=5
```

The API server uses `sqlx::PgPoolOptions` with configurable max connections:
```rust
PgPoolOptions::new()
    .max_connections(config.max_db_connections)
    .connect_with(PgConnectOptions::from_str(&config.database_url)?)
```

**Output Worker Configuration:**
```bash
# Number of concurrent webhook deliveries per worker
--concurrent=10
```

Each worker creates a connection pool matching its concurrency level:
```rust
PgPoolOptions::new()
    .max_connections(config.concurrent.into())
    .connect_with(...)
```

### Connection Pool Sizing Guidelines

- **API Server**: 5-20 connections per instance, depending on expected API request rate
- **Output Worker**: Match `--concurrent` flag to expected webhook throughput
- **Total Connections**: Ensure PostgreSQL `max_connections` setting exceeds sum of all pools across all instances

Example for 3 API servers + 5 workers:
- 3 API servers × 10 connections = 30
- 5 workers × 10 concurrent = 50
- PostgreSQL needs: 80+ `max_connections`

## Rate Limiting

Hook0 implements multi-level rate limiting to protect the system from abuse and overload.

### Rate Limiting Layers

**1. Global Rate Limiting:**
```bash
DISABLE_API_RATE_LIMITING_GLOBAL=false
API_RATE_LIMITING_GLOBAL_BURST_SIZE=2000
API_RATE_LIMITING_GLOBAL_REPLENISH_PERIOD_IN_MS=1
```
- Applies across all API requests system-wide
- Default: 2000 requests burst, replenishes 1 request per millisecond
- Protects against total system overload

**2. Per-IP Rate Limiting:**
```bash
DISABLE_API_RATE_LIMITING_IP=false
API_RATE_LIMITING_IP_BURST_SIZE=200
API_RATE_LIMITING_IP_REPLENISH_PERIOD_IN_MS=10
```
- Limits requests per source IP address
- Default: 200 requests burst, replenishes 1 request per 10ms
- Protects against single-source abuse

**3. Per-Token Rate Limiting:**
```bash
DISABLE_API_RATE_LIMITING_TOKEN=false
API_RATE_LIMITING_TOKEN_BURST_SIZE=20
API_RATE_LIMITING_TOKEN_REPLENISH_PERIOD_IN_MS=100
```
- Applies to authenticated requests per Biscuit token
- Default: 20 requests burst, replenishes 1 request per 100ms
- Protects per-user API usage

### Disabling Rate Limiting

For testing or trusted environments:
```bash
# Disable all rate limiting
DISABLE_API_RATE_LIMITING=true

# Or disable individual layers
DISABLE_API_RATE_LIMITING_GLOBAL=true
DISABLE_API_RATE_LIMITING_IP=true
DISABLE_API_RATE_LIMITING_TOKEN=true
```

### Rate Limiter Implementation

Hook0 uses in-memory token bucket rate limiters (not Redis):
```rust
rate_limiting::Hook0RateLimiters::new(
    config.disable_api_rate_limiting,
    config.disable_api_rate_limiting_global,
    config.api_rate_limiting_global_burst_size,
    config.api_rate_limiting_global_replenish_period_in_ms,
    // ... other parameters
)
```

Applied as Actix middleware:
```rust
web::scope("/api/v1")
    .wrap(Compat::new(rate_limiters.ip()))
    .wrap(Compat::new(rate_limiters.global()))
```

## Webhook Delivery Performance

### Request Attempt Processing

Each output worker continuously:
1. Queries PostgreSQL for next pending request attempt (with `FOR UPDATE ... SKIP LOCKED`)
2. Marks attempt as picked (sets `picked_at`, `worker_name`, `worker_version`)
3. Executes HTTP request to target webhook URL
4. Stores response in PostgreSQL
5. Updates attempt status (`succeeded_at` or `failed_at`)
6. Creates retry attempt if needed, or gives up after max retries

### Retry Strategy

**Fast Retries (default: 30 attempts):**
- Start after 5 seconds delay
- Exponential backoff up to 5 minutes max
- For temporary failures (connection errors, 5xx responses)

**Slow Retries (default: 30 attempts):**
- 1 hour delay between attempts
- After fast retries exhausted
- For persistent failures

Configuration:
```bash
--max-fast-retries=30
--max-slow-retries=30
```

### Timeout Configuration

```bash
# Connection establishment timeout
--connect-timeout=5s

# Total request timeout (including connect)
--timeout=15s
```

Shorter timeouts increase throughput but may cause premature failures. Longer timeouts reduce failures but tie up worker capacity.

### Worker Polling Strategy

When no work is available, workers use variable sleep duration to balance latency vs database load:
- **Unit 0**: 1 second sleep (fast pickup of new work)
- **Units 1-2**: 5.5 seconds sleep (medium responsiveness)
- **Units 3+**: 10 seconds sleep (minimal database polling)

This staggered approach ensures low latency for new events while minimizing unnecessary database queries.

## Monitoring and Observability

### Optional Sentry Integration

Hook0 supports Sentry for error tracking and performance monitoring:

**API Server:**
```bash
SENTRY_DSN=https://...@sentry.io/...
SENTRY_TRACES_SAMPLE_RATE=0.1  # Sample 10% of transactions
```

**Output Worker:**
```bash
--sentry-dsn=https://...@sentry.io/...
```

If `SENTRY_DSN` is not set, Sentry integration is disabled and Hook0 operates with standard logging only.

### Health Checks

API server exposes health check endpoint:
```bash
HEALTH_CHECK_KEY=secret-key  # Optional authentication
```

Access via:
```bash
curl http://api-server:8081/api/v1/health?key=secret-key
```

Returns database connectivity status and basic health metrics.

### Worker Heartbeat Monitoring

Output workers can send periodic heartbeats:
```bash
--monitoring-heartbeat-url=https://healthchecks.io/ping/...
--monitoring-heartbeat-min-period-in-s=60
```

Useful for dead man's switch monitoring (e.g., with Healthchecks.io, Cronitor, UptimeRobot).

## Database Performance Considerations

### Materialized Views Refresh

Hook0 maintains materialized views for performance:
```bash
MATERIALIZED_VIEWS_REFRESH_PERIOD_IN_S=60  # Default: 1 minute
```

Background task refreshes views containing aggregated metrics. Adjust frequency based on data freshness requirements vs refresh overhead.

### Old Event Cleanup

Automatic cleanup of old events based on retention period:
```bash
OLD_EVENTS_CLEANUP_PERIOD_IN_S=3600  # Check every hour
OLD_EVENTS_CLEANUP_GRACE_PERIOD_IN_DAY=30  # Keep 30 days past retention
OLD_EVENTS_CLEANUP_REPORT_AND_DELETE=true  # Actually delete (vs report only)
```

Helps manage database growth over time. Grace period provides safety buffer before deletion.

### Indexes and Query Optimization

Hook0's database schema includes indexes on:
- `request_attempt.created_at` for FIFO queue ordering
- `request_attempt.succeeded_at` and `failed_at` for filtering pending attempts
- `request_attempt.delay_until` for retry scheduling
- Foreign keys for join performance

PostgreSQL's query planner optimizes the critical `FOR UPDATE ... SKIP LOCKED` queries used by workers.

## Performance Characteristics

### Throughput Expectations

**Event Ingestion (API Server):**
- Thousands of events per second per API instance
- Limited primarily by database write throughput
- Batch ingestion recommended for high-volume scenarios

**Webhook Delivery (Output Worker):**
- 10-100+ concurrent webhooks per worker instance
- Throughput depends on target endpoint response times
- Fast responding endpoints (< 100ms): higher throughput
- Slow endpoints (> 1s): lower throughput, consider more workers

### Latency Characteristics

**API Response Time:**
- Event ingestion: < 50ms (database write time)
- Query endpoints: < 100ms (database query time)
- Token refresh: < 20ms (in-memory token generation)

**Webhook Delivery Latency:**
- First delivery attempt: typically within 1-10 seconds after event creation
- Depends on worker polling cycle and available capacity
- Priority given to oldest pending attempts (FIFO)

### Resource Usage

**Memory:**
- API Server: 50-200 MB per instance (varies with request volume)
- Output Worker: 20-100 MB per instance (varies with concurrency and payload sizes)

**CPU:**
- API Server: Moderate, spikes during request bursts
- Output Worker: Low to moderate, depends on cryptographic signature generation and HTTP request processing

**Disk:**
- PostgreSQL database storage grows with:
  - Number of events (based on retention policy)
  - Response storage (headers, body)
  - Audit logs and attempt history

## Scaling Recommendations

### Small Deployment (< 100 events/minute)

- 1 API server instance
- 1-2 output worker instances with `--concurrent=5`
- PostgreSQL with 20 max connections
- Standard VPS or cloud instance

### Medium Deployment (100-10,000 events/minute)

- 2-3 API server instances behind load balancer
- 3-5 output worker instances with `--concurrent=10`
- PostgreSQL with 100+ max connections
- Managed PostgreSQL recommended (e.g., AWS RDS, Google Cloud SQL)

### Large Deployment (> 10,000 events/minute)

- 5+ API server instances with auto-scaling
- 10+ output worker instances with `--concurrent=20-50`
- PostgreSQL with high-performance storage and read replicas (for analytics)
- Consider dedicated PostgreSQL connection pooler (e.g., PgBouncer)
- Sentry integration for comprehensive monitoring

## No Redis Required

Hook0 deliberately **does not use Redis** for:
- Queue management
- Job distribution
- Worker coordination
- Cache layers

All coordination happens through PostgreSQL, which provides:
- Stronger consistency guarantees than Redis
- Transactional safety for event processing
- Simplified deployment (one less component)
- Proven scalability with proper indexing

This design choice prioritizes operational simplicity and reliability over raw throughput. For most webhook use cases, PostgreSQL-based coordination provides sufficient performance while eliminating an entire class of failure modes.

---

*For self-hosting details, see [Self-hosting with Docker](../tutorials/self-hosting-docker.md). For monitoring setup, see [Monitoring Webhook Performance](../how-to-guides/monitor-webhook-performance.md).*
