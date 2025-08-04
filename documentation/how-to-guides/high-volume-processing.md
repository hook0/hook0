# Managing High-Volume Event Processing

This guide covers strategies for handling high-throughput webhook delivery scenarios, optimizing performance, and scaling Hook0 to process thousands of events per second reliably.

## Understanding High-Volume Requirements

### Performance Metrics to Monitor

**Throughput Metrics** 📊
- Events ingested per second
- Webhooks delivered per second
- Queue processing rate
- Database write/read operations per second

**Latency Metrics** ⏱️
- Event ingestion to storage latency
- Webhook delivery latency
- End-to-end processing time
- Database query response times

**Resource Utilization** 🔧
- CPU usage across services
- Memory consumption
- Network I/O bandwidth
- Database connection pool usage

## Step 1: Optimize Event Ingestion

### Batch Event Ingestion

```javascript
// batch-event-client.js
class BatchEventClient {
  constructor(options = {}) {
    this.apiUrl = options.apiUrl || 'https://api.hook0.com';
    this.token = options.token;
    this.batchSize = options.batchSize || 100;
    this.flushInterval = options.flushInterval || 1000; // 1 second
    this.eventBuffer = [];
    this.flushTimer = null;
    
    this.startAutoFlush();
  }
  
  async sendEvent(eventType, payload, labels = {}) {
    const event = {
      event_type: eventType,
      payload,
      labels,
      timestamp: new Date().toISOString()
    };
    
    this.eventBuffer.push(event);
    
    // Flush if buffer is full
    if (this.eventBuffer.length >= this.batchSize) {
      await this.flush();
    }
    
    return { queued: true, bufferSize: this.eventBuffer.length };
  }
  
  async flush() {
    if (this.eventBuffer.length === 0) return { sent: 0 };
    
    const eventsToSend = this.eventBuffer.splice(0, this.batchSize);
    
    try {
      const response = await fetch(`${this.apiUrl}/api/v1/events/batch`, {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${this.token}`,
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({ events: eventsToSend })
      });
      
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }
      
      const result = await response.json();
      console.log(`Batch sent: ${eventsToSend.length} events`);
      
      return { sent: eventsToSend.length, result };
    } catch (error) {
      // Re-add failed events to front of buffer for retry
      this.eventBuffer.unshift(...eventsToSend);
      console.error('Batch send failed:', error.message);
      throw error;
    }
  }
  
  startAutoFlush() {
    this.flushTimer = setInterval(async () => {
      if (this.eventBuffer.length > 0) {
        try {
          await this.flush();
        } catch (error) {
          console.error('Auto-flush failed:', error.message);
        }
      }
    }, this.flushInterval);
  }
  
  async shutdown() {
    if (this.flushTimer) {
      clearInterval(this.flushTimer);
    }
    
    // Flush remaining events
    while (this.eventBuffer.length > 0) {
      await this.flush();
    }
  }
}

// Usage
const eventClient = new BatchEventClient({
  apiUrl: 'https://api.hook0.com',
  token: 'biscuit:YOUR_TOKEN_HERE',
  batchSize: 500,
  flushInterval: 2000
});

// High-volume event generation
async function generateHighVolumeEvents() {
  const promises = [];
  
  for (let i = 0; i < 10000; i++) {
    promises.push(eventClient.sendEvent('user.action', {
      user_id: `user_${i}`,
      action: 'page_view',
      page: '/dashboard',
      timestamp: new Date().toISOString()
    }, {
      batch_id: Math.floor(i / 1000),
      source: 'high_volume_test'
    }));
    
    // Process in batches to avoid overwhelming
    if (promises.length >= 100) {
      await Promise.all(promises);
      promises.length = 0;
      
      // Small delay to prevent rate limiting
      await new Promise(resolve => setTimeout(resolve, 50));
    }
  }
  
  // Process remaining
  await Promise.all(promises);
  await eventClient.shutdown();
}

module.exports = BatchEventClient;
```

### Connection Pooling and Reuse

```javascript
// optimized-http-client.js
const fetch = require('node-fetch');
const https = require('https');

class OptimizedHttpClient {
  constructor(options = {}) {
    this.baseURL = options.baseURL;
    this.token = options.token;
    
    // HTTP Agent with connection pooling
    this.agent = new https.Agent({
      keepAlive: true,
      keepAliveMsecs: 30000,
      maxSockets: 50,
      maxFreeSockets: 10,
      timeout: 30000
    });
    
    this.defaultHeaders = {
      'Authorization': `Bearer ${this.token}`,
      'Content-Type': 'application/json',
      'Connection': 'keep-alive'
    };
  }
  
  async request(method, path, data = null) {
    const url = `${this.baseURL}${path}`;
    
    const options = {
      method,
      headers: this.defaultHeaders,
      agent: this.agent,
      timeout: 30000
    };
    
    if (data) {
      options.body = JSON.stringify(data);
    }
    
    const response = await fetch(url, options);
    
    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    }
    
    return response.json();
  }
  
  async batchEvents(events) {
    return this.request('POST', '/api/v1/events/batch', { events });
  }
  
  destroy() {
    this.agent.destroy();
  }
}

module.exports = OptimizedHttpClient;
```

## Step 2: Optimize Database Performance

### Database Configuration for High Volume

```sql
-- postgresql.conf optimizations for high volume
-- Memory settings
shared_buffers = 512MB                    -- 25% of RAM
effective_cache_size = 2GB                -- 75% of RAM
work_mem = 16MB                           -- Per operation
maintenance_work_mem = 128MB              -- For maintenance ops

-- Connection settings
max_connections = 200                     -- Adjust based on workload
superuser_reserved_connections = 3

-- Write-ahead logging
wal_buffers = 16MB
wal_writer_delay = 200ms
checkpoint_completion_target = 0.9
max_wal_size = 2GB
min_wal_size = 512MB

-- Query planner
random_page_cost = 1.1                   -- For SSD storage
effective_io_concurrency = 200           -- For SSD storage

-- Logging (disable in production for performance)
log_statement = 'none'
log_duration = off
log_lock_waits = on
log_checkpoints = on

-- Background writer
bgwriter_delay = 200ms
bgwriter_lru_maxpages = 100
bgwriter_lru_multiplier = 2.0

-- Autovacuum tuning
autovacuum_max_workers = 4
autovacuum_naptime = 30s
autovacuum_vacuum_threshold = 500
autovacuum_analyze_threshold = 250
```

### Optimized Database Schema and Indexes

```sql
-- Events table optimization
CREATE TABLE events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    application_id UUID NOT NULL,
    event_type_id UUID NOT NULL,
    payload JSONB NOT NULL,
    labels JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    processed_at TIMESTAMP WITH TIME ZONE,
    
    -- Foreign keys
    FOREIGN KEY (application_id) REFERENCES applications(id),
    FOREIGN KEY (event_type_id) REFERENCES event_types(id)
);

-- Partition events by date for better performance
CREATE TABLE events_y2024m01 PARTITION OF events
FOR VALUES FROM ('2024-01-01') TO ('2024-02-01');

CREATE TABLE events_y2024m02 PARTITION OF events
FOR VALUES FROM ('2024-02-01') TO ('2024-03-01');

-- Critical indexes for high-volume operations
CREATE INDEX CONCURRENTLY idx_events_application_created 
ON events (application_id, created_at DESC);

CREATE INDEX CONCURRENTLY idx_events_type_created 
ON events (event_type_id, created_at DESC);

CREATE INDEX CONCURRENTLY idx_events_created_processed 
ON events (created_at, processed_at) 
WHERE processed_at IS NULL;

-- GIN index for JSONB queries
CREATE INDEX CONCURRENTLY idx_events_payload_gin 
ON events USING gin (payload);

CREATE INDEX CONCURRENTLY idx_events_labels_gin 
ON events USING gin (labels);

-- Request attempts optimization
CREATE TABLE request_attempts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id UUID NOT NULL,
    subscription_id UUID NOT NULL,
    attempt_number INTEGER NOT NULL,
    status_code INTEGER,
    response_body TEXT,
    error_message TEXT,
    duration_ms INTEGER,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    FOREIGN KEY (event_id) REFERENCES events(id),
    FOREIGN KEY (subscription_id) REFERENCES subscriptions(id)
);

-- Compound index for efficient queries
CREATE INDEX CONCURRENTLY idx_request_attempts_event_sub 
ON request_attempts (event_id, subscription_id, attempt_number);

CREATE INDEX CONCURRENTLY idx_request_attempts_status_created 
ON request_attempts (status_code, created_at);

-- Partial index for failed attempts only
CREATE INDEX CONCURRENTLY idx_request_attempts_failed 
ON request_attempts (subscription_id, created_at) 
WHERE status_code >= 400 OR status_code IS NULL;
```

### Database Connection Pool Optimization

```javascript
// database-pool.js
const { Pool } = require('pg');

class OptimizedDatabasePool {
  constructor(config = {}) {
    this.readPool = new Pool({
      ...config,
      host: config.readHost || config.host,
      max: config.readPoolSize || 20,
      min: config.readPoolMin || 5,
      idleTimeoutMillis: 30000,
      connectionTimeoutMillis: 2000,
      statement_timeout: 30000,
      query_timeout: 30000
    });
    
    this.writePool = new Pool({
      ...config,
      host: config.writeHost || config.host,
      max: config.writePoolSize || 10,
      min: config.writePoolMin || 2,
      idleTimeoutMillis: 30000,
      connectionTimeoutMillis: 2000,
      statement_timeout: 30000,
      query_timeout: 30000
    });
    
    this.setupMonitoring();
  }
  
  setupMonitoring() {
    // Monitor pool health
    setInterval(() => {
      console.log('DB Pool Stats:', {
        read: {
          total: this.readPool.totalCount,
          idle: this.readPool.idleCount,
          waiting: this.readPool.waitingCount
        },
        write: {
          total: this.writePool.totalCount,
          idle: this.writePool.idleCount,
          waiting: this.writePool.waitingCount
        }
      });
    }, 60000); // Every minute
  }
  
  async batchInsertEvents(events) {
    const client = await this.writePool.connect();
    
    try {
      await client.query('BEGIN');
      
      const values = [];
      const placeholders = [];
      
      events.forEach((event, index) => {
        const offset = index * 6;
        placeholders.push(
          `($${offset + 1}, $${offset + 2}, $${offset + 3}, $${offset + 4}, $${offset + 5}, $${offset + 6})`
        );
        values.push(
          event.id,
          event.application_id,
          event.event_type_id,
          JSON.stringify(event.payload),
          JSON.stringify(event.labels),
          event.created_at
        );
      });
      
      const query = `
        INSERT INTO events (id, application_id, event_type_id, payload, labels, created_at)
        VALUES ${placeholders.join(', ')}
        RETURNING id
      `;
      
      const result = await client.query(query, values);
      await client.query('COMMIT');
      
      return result.rows;
    } catch (error) {
      await client.query('ROLLBACK');
      throw error;
    } finally {
      client.release();
    }
  }
  
  async getPendingEvents(limit = 1000) {
    const query = `
      SELECT e.id, e.application_id, e.event_type_id, e.payload, e.labels, e.created_at,
             et.name as event_type, a.name as application_name
      FROM events e
      JOIN event_types et ON e.event_type_id = et.id
      JOIN applications a ON e.application_id = a.id
      WHERE e.processed_at IS NULL
      ORDER BY e.created_at ASC
      LIMIT $1
    `;
    
    const result = await this.readPool.query(query, [limit]);
    return result.rows;
  }
  
  async getSubscriptionsForEvent(eventTypeId, applicationId) {
    const query = `
      SELECT s.id, s.target, s.secret, s.is_enabled
      FROM subscriptions s
      WHERE s.application_id = $1
        AND s.is_enabled = true
        AND $2 = ANY(s.event_type_ids)
        AND s.deleted_at IS NULL
    `;
    
    const result = await this.readPool.query(query, [applicationId, eventTypeId]);
    return result.rows;
  }
  
  async close() {
    await this.readPool.end();
    await this.writePool.end();
  }
}

module.exports = OptimizedDatabasePool;
```

## Step 3: Optimize Webhook Delivery

### Concurrent Webhook Delivery

```javascript
// webhook-delivery-engine.js
const pLimit = require('p-limit');
const fetch = require('node-fetch');

class WebhookDeliveryEngine {
  constructor(options = {}) {
    this.concurrencyLimit = options.concurrencyLimit || 100;
    this.timeout = options.timeout || 30000;
    this.retryAttempts = options.retryAttempts || 3;
    this.retryDelay = options.retryDelay || 1000;
    
    // Create concurrency limiter
    this.limiter = pLimit(this.concurrencyLimit);
    
    // HTTP client with optimized settings
    this.httpAgent = new (require('https').Agent)({
      keepAlive: true,
      keepAliveMsecs: 30000,
      maxSockets: this.concurrencyLimit,
      maxFreeSockets: Math.floor(this.concurrencyLimit / 4),
      timeout: this.timeout
    });
    
    this.metrics = {
      delivered: 0,
      failed: 0,
      totalDuration: 0,
      inFlight: 0
    };
  }
  
  async deliverWebhook(event, subscription) {
    return this.limiter(async () => {
      this.metrics.inFlight++;
      const startTime = Date.now();
      
      try {
        const result = await this.attemptDelivery(event, subscription);
        
        this.metrics.delivered++;
        this.metrics.totalDuration += Date.now() - startTime;
        
        return result;
      } catch (error) {
        this.metrics.failed++;
        throw error;
      } finally {
        this.metrics.inFlight--;
      }
    });
  }
  
  async attemptDelivery(event, subscription, attempt = 1) {
    const webhookPayload = {
      event_id: event.id,
      event_type: event.event_type,
      payload: event.payload,
      labels: event.labels,
      timestamp: event.created_at
    };
    
    const signature = this.generateSignature(
      JSON.stringify(webhookPayload),
      subscription.secret
    );
    
    const requestOptions = {
      method: subscription.target.method || 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Hook0-Signature': signature,
        'Hook0-Event-Type': event.event_type,
        'Hook0-Event-Id': event.id,
        'User-Agent': 'Hook0/1.0',
        ...(subscription.target.headers || {})
      },
      body: JSON.stringify(webhookPayload),
      timeout: this.timeout,
      agent: this.httpAgent
    };
    
    try {
      const response = await fetch(subscription.target.url, requestOptions);
      
      return {
        success: response.ok,
        statusCode: response.status,
        responseBody: await this.safeReadResponse(response),
        duration: Date.now() - startTime,
        attempt
      };
    } catch (error) {
      if (attempt < this.retryAttempts && this.shouldRetry(error)) {
        await this.delay(this.retryDelay * Math.pow(2, attempt - 1));
        return this.attemptDelivery(event, subscription, attempt + 1);
      }
      
      throw {
        success: false,
        error: error.message,
        attempt,
        duration: Date.now() - startTime
      };
    }
  }
  
  async safeReadResponse(response) {
    try {
      const text = await response.text();
      return text.substring(0, 1000); // Limit response body size
    } catch (error) {
      return 'Failed to read response body';
    }
  }
  
  shouldRetry(error) {
    // Retry on network errors, timeouts, and 5xx responses
    return error.code === 'ENOTFOUND' ||
           error.code === 'ECONNRESET' ||
           error.code === 'TIMEOUT' ||
           (error.status >= 500 && error.status < 600);
  }
  
  generateSignature(payload, secret) {
    return 'sha256=' + require('crypto')
      .createHmac('sha256', secret)
      .update(payload)
      .digest('hex');
  }
  
  delay(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
  }
  
  async batchDeliverWebhooks(deliveryTasks) {
    const promises = deliveryTasks.map(task => 
      this.deliverWebhook(task.event, task.subscription)
        .then(result => ({ ...task, result, success: true }))
        .catch(error => ({ ...task, error, success: false }))
    );
    
    return Promise.all(promises);
  }
  
  getMetrics() {
    const avgDuration = this.metrics.delivered > 0 
      ? this.metrics.totalDuration / this.metrics.delivered 
      : 0;
    
    return {
      ...this.metrics,
      averageDuration: Math.round(avgDuration),
      successRate: this.metrics.delivered / (this.metrics.delivered + this.metrics.failed) * 100
    };
  }
  
  resetMetrics() {
    this.metrics = {
      delivered: 0,
      failed: 0,
      totalDuration: 0,
      inFlight: 0
    };
  }
  
  destroy() {
    this.httpAgent.destroy();
  }
}

module.exports = WebhookDeliveryEngine;
```

## Step 4: Implement Queue-Based Processing

### Redis-Based Task Queue

```javascript
// task-queue.js
const Redis = require('redis');
const { Worker, Queue } = require('bullmq');

class HighVolumeTaskQueue {
  constructor(redisConfig = {}) {
    this.redis = Redis.createClient(redisConfig);
    
    // Create queues for different priorities
    this.queues = {
      high: new Queue('webhooks-high', { connection: this.redis }),
      normal: new Queue('webhooks-normal', { connection: this.redis }),
      low: new Queue('webhooks-low', { connection: this.redis })
    };
    
    this.workers = [];
    this.metrics = {
      queued: 0,
      processed: 0,
      failed: 0
    };
  }
  
  async queueWebhookDelivery(event, subscriptions, priority = 'normal') {
    const jobs = subscriptions.map(subscription => ({
      name: 'deliver-webhook',
      data: {
        event,
        subscription,
        queuedAt: new Date().toISOString()
      },
      opts: {
        attempts: 3,
        backoff: {
          type: 'exponential',
          delay: 2000
        },
        removeOnComplete: 100,
        removeOnFail: 50
      }
    }));
    
    await this.queues[priority].addBulk(jobs);
    this.metrics.queued += jobs.length;
    
    return { queued: jobs.length, priority };
  }
  
  startWorkers(concurrency = 10) {
    const workerOptions = {
      connection: this.redis,
      concurrency
    };
    
    // Start workers for each priority queue
    Object.entries(this.queues).forEach(([priority, queue]) => {
      const worker = new Worker(queue.name, async (job) => {
        return this.processWebhookJob(job.data);
      }, workerOptions);
      
      worker.on('completed', (job) => {
        this.metrics.processed++;
        console.log(`✅ Webhook delivered: ${job.data.event.id}`);
      });
      
      worker.on('failed', (job, error) => {
        this.metrics.failed++;
        console.error(`❌ Webhook failed: ${job.data.event.id}`, error.message);
      });
      
      this.workers.push(worker);
    });
  }
  
  async processWebhookJob(data) {
    const { event, subscription } = data;
    const deliveryEngine = new WebhookDeliveryEngine();
    
    try {
      const result = await deliveryEngine.deliverWebhook(event, subscription);
      return result;
    } finally {
      deliveryEngine.destroy();
    }
  }
  
  async getQueueStats() {
    const stats = {};
    
    for (const [priority, queue] of Object.entries(this.queues)) {
      stats[priority] = {
        waiting: await queue.getWaiting(),
        active: await queue.getActive(),
        completed: await queue.getCompleted(),
        failed: await queue.getFailed()
      };
    }
    
    return { queues: stats, metrics: this.metrics };
  }
  
  async pauseQueue(priority) {
    await this.queues[priority].pause();
  }
  
  async resumeQueue(priority) {
    await this.queues[priority].resume();
  }
  
  async shutdown() {
    // Stop all workers
    await Promise.all(this.workers.map(worker => worker.close()));
    
    // Close queues
    await Promise.all(Object.values(this.queues).map(queue => queue.close()));
    
    // Close Redis connection
    await this.redis.quit();
  }
}

module.exports = HighVolumeTaskQueue;
```

## Step 5: Implement Monitoring and Alerting

### Performance Monitoring Dashboard

```javascript
// performance-monitor.js
const EventEmitter = require('events');

class PerformanceMonitor extends EventEmitter {
  constructor(options = {}) {
    super();
    this.windowSize = options.windowSize || 60000; // 1 minute
    this.alertThresholds = {
      highLatency: options.alertThresholds?.highLatency || 5000, // 5 seconds
      lowSuccessRate: options.alertThresholds?.lowSuccessRate || 95, // 95%
      highErrorRate: options.alertThresholds?.highErrorRate || 5, // 5%
      queueBacklog: options.alertThresholds?.queueBacklog || 10000 // 10k items
    };
    
    this.metrics = {
      events: new Map(),
      webhooks: new Map(),
      errors: new Map()
    };
    
    this.startMonitoring();
  }
  
  recordEventIngestion(count, latency) {
    const now = Date.now();
    const window = Math.floor(now / this.windowSize);
    
    const existing = this.metrics.events.get(window) || {
      count: 0,
      totalLatency: 0,
      timestamp: window * this.windowSize
    };
    
    existing.count += count;
    existing.totalLatency += latency;
    
    this.metrics.events.set(window, existing);
    this.cleanupOldMetrics();
  }
  
  recordWebhookDelivery(count, successCount, totalLatency) {
    const now = Date.now();
    const window = Math.floor(now / this.windowSize);
    
    const existing = this.metrics.webhooks.get(window) || {
      count: 0,
      successCount: 0,
      totalLatency: 0,
      timestamp: window * this.windowSize
    };
    
    existing.count += count;
    existing.successCount += successCount;
    existing.totalLatency += totalLatency;
    
    this.metrics.webhooks.set(window, existing);
    this.cleanupOldMetrics();
  }
  
  recordError(errorType, count = 1) {
    const now = Date.now();
    const window = Math.floor(now / this.windowSize);
    
    const windowKey = `${window}:${errorType}`;
    this.metrics.errors.set(windowKey, 
      (this.metrics.errors.get(windowKey) || 0) + count
    );
    
    this.cleanupOldMetrics();
  }
  
  getMetrics(windowCount = 5) {
    const now = Date.now();
    const currentWindow = Math.floor(now / this.windowSize);
    
    const events = [];
    const webhooks = [];
    const errors = {};
    
    // Get last N windows
    for (let i = windowCount - 1; i >= 0; i--) {
      const window = currentWindow - i;
      
      const eventMetric = this.metrics.events.get(window);
      if (eventMetric) {
        events.push({
          timestamp: eventMetric.timestamp,
          count: eventMetric.count,
          avgLatency: eventMetric.totalLatency / eventMetric.count
        });
      }
      
      const webhookMetric = this.metrics.webhooks.get(window);
      if (webhookMetric) {
        webhooks.push({
          timestamp: webhookMetric.timestamp,
          count: webhookMetric.count,
          successCount: webhookMetric.successCount,
          successRate: (webhookMetric.successCount / webhookMetric.count) * 100,
          avgLatency: webhookMetric.totalLatency / webhookMetric.count
        });
      }
    }
    
    // Aggregate error counts
    for (const [key, count] of this.metrics.errors.entries()) {
      const [window, errorType] = key.split(':');
      if (parseInt(window) >= currentWindow - windowCount) {
        errors[errorType] = (errors[errorType] || 0) + count;
      }
    }
    
    return { events, webhooks, errors };
  }
  
  checkAlerts() {
    const metrics = this.getMetrics(1);
    const alerts = [];
    
    // Check webhook latency
    if (metrics.webhooks.length > 0) {
      const latest = metrics.webhooks[metrics.webhooks.length - 1];
      
      if (latest.avgLatency > this.alertThresholds.highLatency) {
        alerts.push({
          type: 'high_latency',
          severity: 'warning',
          message: `High webhook latency: ${latest.avgLatency.toFixed(2)}ms`,
          value: latest.avgLatency,
          threshold: this.alertThresholds.highLatency
        });
      }
      
      if (latest.successRate < this.alertThresholds.lowSuccessRate) {
        alerts.push({
          type: 'low_success_rate',
          severity: 'critical',
          message: `Low success rate: ${latest.successRate.toFixed(2)}%`,
          value: latest.successRate,
          threshold: this.alertThresholds.lowSuccessRate
        });
      }
    }
    
    // Check error rates
    const totalWebhooks = metrics.webhooks.reduce((sum, w) => sum + w.count, 0);
    const totalErrors = Object.values(metrics.errors).reduce((sum, count) => sum + count, 0);
    
    if (totalWebhooks > 0) {
      const errorRate = (totalErrors / totalWebhooks) * 100;
      if (errorRate > this.alertThresholds.highErrorRate) {
        alerts.push({
          type: 'high_error_rate',
          severity: 'critical',
          message: `High error rate: ${errorRate.toFixed(2)}%`,
          value: errorRate,
          threshold: this.alertThresholds.highErrorRate
        });
      }
    }
    
    // Emit alerts
    alerts.forEach(alert => this.emit('alert', alert));
    
    return alerts;
  }
  
  cleanupOldMetrics() {
    const now = Date.now();
    const cutoff = Math.floor(now / this.windowSize) - 60; // Keep 1 hour of data
    
    // Cleanup events
    for (const [window, _] of this.metrics.events.entries()) {
      if (window < cutoff) {
        this.metrics.events.delete(window);
      }
    }
    
    // Cleanup webhooks
    for (const [window, _] of this.metrics.webhooks.entries()) {
      if (window < cutoff) {
        this.metrics.webhooks.delete(window);
      }
    }
    
    // Cleanup errors
    for (const [key, _] of this.metrics.errors.entries()) {
      const [window, _errorType] = key.split(':');
      if (parseInt(window) < cutoff) {
        this.metrics.errors.delete(key);
      }
    }
  }
  
  startMonitoring() {
    // Check for alerts every minute
    setInterval(() => {
      this.checkAlerts();
    }, 60000);
    
    // Log metrics every 5 minutes
    setInterval(() => {
      const metrics = this.getMetrics();
      console.log('Performance Metrics:', JSON.stringify(metrics, null, 2));
    }, 300000);
  }
}

module.exports = PerformanceMonitor;
```

## Step 6: Load Testing and Capacity Planning

### Load Testing Script

```javascript
// load-test.js
const { performance } = require('perf_hooks');
const BatchEventClient = require('./batch-event-client');

class LoadTester {
  constructor(options = {}) {
    this.baseURL = options.baseURL || 'https://api.hook0.com';
    this.token = options.token;
    this.concurrency = options.concurrency || 10;
    this.duration = options.duration || 60000; // 1 minute
    this.eventTypes = options.eventTypes || ['user.action', 'order.created'];
    
    this.clients = [];
    this.results = {
      totalEvents: 0,
      totalErrors: 0,
      startTime: 0,
      endTime: 0,
      latencies: []
    };
  }
  
  async runLoadTest() {
    console.log(`Starting load test: ${this.concurrency} concurrent clients for ${this.duration}ms`);
    
    this.results.startTime = performance.now();
    
    // Create multiple client instances
    for (let i = 0; i < this.concurrency; i++) {
      this.clients.push(new BatchEventClient({
        apiUrl: this.baseURL,
        token: this.token,
        batchSize: 50,
        flushInterval: 1000
      }));
    }
    
    // Start load generation
    const promises = this.clients.map((client, index) => 
      this.generateLoadForClient(client, index)
    );
    
    // Wait for test duration
    await Promise.all(promises);
    
    this.results.endTime = performance.now();
    
    // Shutdown clients
    await Promise.all(this.clients.map(client => client.shutdown()));
    
    return this.getResults();
  }
  
  async generateLoadForClient(client, clientIndex) {
    const endTime = Date.now() + this.duration;
    let eventCount = 0;
    
    while (Date.now() < endTime) {
      try {
        const startTime = performance.now();
        
        await client.sendEvent(
          this.eventTypes[Math.floor(Math.random() * this.eventTypes.length)],
          {
            user_id: `user_${clientIndex}_${eventCount}`,
            action: 'load_test',
            timestamp: new Date().toISOString(),
            client_id: clientIndex,
            event_number: eventCount
          },
          {
            load_test: true,
            client_id: clientIndex
          }
        );
        
        const latency = performance.now() - startTime;
        this.results.latencies.push(latency);
        this.results.totalEvents++;
        eventCount++;
        
        // Small delay to prevent overwhelming
        await new Promise(resolve => setTimeout(resolve, 10));
        
      } catch (error) {
        this.results.totalErrors++;
        console.error(`Client ${clientIndex} error:`, error.message);
      }
    }
  }
  
  getResults() {
    const duration = this.results.endTime - this.results.startTime;
    const eventsPerSecond = (this.results.totalEvents / duration) * 1000;
    
    const latencies = this.results.latencies.sort((a, b) => a - b);
    const percentiles = {
      p50: latencies[Math.floor(latencies.length * 0.5)],
      p95: latencies[Math.floor(latencies.length * 0.95)],
      p99: latencies[Math.floor(latencies.length * 0.99)]
    };
    
    return {
      duration: Math.round(duration),
      totalEvents: this.results.totalEvents,
      totalErrors: this.results.totalErrors,
      eventsPerSecond: Math.round(eventsPerSecond),
      errorRate: ((this.results.totalErrors / this.results.totalEvents) * 100).toFixed(2),
      latencies: {
        min: Math.min(...latencies),
        max: Math.max(...latencies),
        avg: latencies.reduce((a, b) => a + b, 0) / latencies.length,
        ...percentiles
      }
    };
  }
}

// Run load test
async function runTest() {
  const tester = new LoadTester({
    baseURL: 'https://api.hook0.com',
    token: 'biscuit:YOUR_TOKEN_HERE',
    concurrency: 20,
    duration: 120000, // 2 minutes
    eventTypes: ['user.created', 'user.updated', 'order.completed']
  });
  
  try {
    const results = await tester.runLoadTest();
    console.log('Load Test Results:', JSON.stringify(results, null, 2));
  } catch (error) {
    console.error('Load test failed:', error);
  }
}

if (require.main === module) {
  runTest();
}

module.exports = LoadTester;
```

## Performance Optimization Checklist

### Infrastructure
- ✅ Use connection pooling for HTTP clients
- ✅ Implement database connection pooling
- ✅ Configure PostgreSQL for high throughput
- ✅ Use Redis for caching and queuing
- ✅ Implement horizontal scaling

### Application
- ✅ Batch event processing where possible
- ✅ Use async/await properly
- ✅ Implement proper error handling
- ✅ Use streaming for large datasets
- ✅ Optimize database queries and indexes

### Monitoring
- ✅ Track key performance metrics
- ✅ Set up alerting for critical thresholds
- ✅ Monitor resource utilization
- ✅ Log performance-related events
- ✅ Regular performance testing

Ready to handle high-volume webhook processing? Start with the optimization strategies that match your current bottlenecks and gradually implement the full solution.