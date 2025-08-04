# Monitoring Webhook Performance

This comprehensive guide covers implementing robust monitoring and observability for webhook systems, from basic metrics collection to advanced alerting and performance analysis.

## Key Performance Indicators (KPIs)

### Primary Metrics 📊

**Delivery Success Rate**
- Percentage of successful webhook deliveries
- Target: >99% for critical events, >95% for non-critical

**Response Time**
- P50, P95, P99 percentiles of webhook delivery times
- Target: P95 < 5 seconds, P99 < 10 seconds

**Throughput**
- Events processed per second
- Webhooks delivered per second

**Error Rates**
- 4xx client errors (permanent failures)
- 5xx server errors (temporary failures)
- Network/timeout errors

### Secondary Metrics 📈

**Queue Depth**
- Number of pending webhook deliveries
- Indicates system load and bottlenecks

**Retry Rates**
- Percentage of deliveries requiring retries
- Average number of retries per delivery

**Circuit Breaker Status**
- Which endpoints are failing consistently
- Recovery time for failed endpoints

## Step 1: Implement Basic Metrics Collection

### Prometheus Metrics Setup

```javascript
// metrics-collector.js
const promClient = require('prom-client');

class WebhookMetrics {
  constructor() {
    // Create a registry for metrics
    this.register = new promClient.Registry();
    
    // Add default metrics (CPU, memory, etc.)
    promClient.collectDefaultMetrics({ register: this.register });
    
    this.setupWebhookMetrics();
  }
  
  setupWebhookMetrics() {
    // Delivery success counter
    this.deliveryCounter = new promClient.Counter({
      name: 'webhook_deliveries_total',
      help: 'Total number of webhook delivery attempts',
      labelNames: ['status', 'event_type', 'subscription_id', 'endpoint_host'],
      registers: [this.register]
    });
    
    // Delivery duration histogram
    this.deliveryDuration = new promClient.Histogram({
      name: 'webhook_delivery_duration_seconds',
      help: 'Webhook delivery duration in seconds',
      labelNames: ['status', 'event_type', 'subscription_id'],
      buckets: [0.1, 0.5, 1, 2, 5, 10, 30, 60],
      registers: [this.register]
    });
    
    // Queue depth gauge
    this.queueDepth = new promClient.Gauge({
      name: 'webhook_queue_depth',
      help: 'Number of webhooks waiting to be delivered',
      labelNames: ['priority'],
      registers: [this.register]
    });
    
    // Retry counter
    this.retryCounter = new promClient.Counter({
      name: 'webhook_retries_total',
      help: 'Total number of webhook retries',
      labelNames: ['event_type', 'subscription_id', 'attempt_number'],
      registers: [this.register]
    });
    
    // Circuit breaker status gauge
    this.circuitBreakerStatus = new promClient.Gauge({
      name: 'webhook_circuit_breaker_status',
      help: 'Circuit breaker status (0=closed, 1=half-open, 2=open)',
      labelNames: ['endpoint_host'],
      registers: [this.register]
    });
    
    // Event ingestion rate
    this.eventIngestionRate = new promClient.Counter({
      name: 'events_ingested_total',
      help: 'Total number of events ingested',
      labelNames: ['event_type', 'application_id'],
      registers: [this.register]
    });
    
    // Response time by status code
    this.responseTimeByStatus = new promClient.Histogram({
      name: 'webhook_response_time_by_status_seconds',
      help: 'Response time by HTTP status code',
      labelNames: ['status_code', 'endpoint_host'],
      buckets: [0.1, 0.5, 1, 2, 5, 10, 30],
      registers: [this.register]
    });
  }
  
  recordDeliveryAttempt(labels, duration, success) {
    const status = success ? 'success' : 'failure';
    
    this.deliveryCounter.inc({
      status,
      event_type: labels.eventType,
      subscription_id: labels.subscriptionId,
      endpoint_host: labels.endpointHost
    });
    
    this.deliveryDuration.observe({
      status,
      event_type: labels.eventType,
      subscription_id: labels.subscriptionId
    }, duration / 1000); // Convert to seconds
    
    if (labels.statusCode) {
      this.responseTimeByStatus.observe({
        status_code: labels.statusCode.toString(),
        endpoint_host: labels.endpointHost
      }, duration / 1000);
    }
  }
  
  recordRetry(eventType, subscriptionId, attemptNumber) {
    this.retryCounter.inc({
      event_type: eventType,
      subscription_id: subscriptionId,
      attempt_number: attemptNumber.toString()
    });
  }
  
  updateQueueDepth(priority, depth) {
    this.queueDepth.set({ priority }, depth);
  }
  
  updateCircuitBreakerStatus(endpointHost, status) {
    // 0 = closed, 1 = half-open, 2 = open
    const statusMap = { 'closed': 0, 'half-open': 1, 'open': 2 };
    this.circuitBreakerStatus.set({ endpoint_host: endpointHost }, statusMap[status] || 0);
  }
  
  recordEventIngestion(eventType, applicationId) {
    this.eventIngestionRate.inc({
      event_type: eventType,
      application_id: applicationId
    });
  }
  
  getMetrics() {
    return this.register.metrics();
  }
  
  getMetricsAsJson() {
    return this.register.getMetricsAsJSON();
  }
}

module.exports = WebhookMetrics;
```

### Metrics Middleware Integration

```javascript
// monitoring-middleware.js
const WebhookMetrics = require('./metrics-collector');

class MonitoringMiddleware {
  constructor(options = {}) {
    this.metrics = new WebhookMetrics();
    this.trackingEnabled = options.tracking !== false;
    this.detailedLogging = options.detailedLogging || false;
  }
  
  // Middleware for webhook delivery
  wrapWebhookDelivery(deliveryFunction) {
    return async (webhook, subscription, options = {}) => {
      const startTime = Date.now();
      const labels = {
        eventType: webhook.event_type,
        subscriptionId: subscription.id,
        endpointHost: this.extractHost(subscription.target.url)
      };
      
      try {
        // Record event ingestion
        this.metrics.recordEventIngestion(
          webhook.event_type,
          webhook.application_id
        );
        
        const result = await deliveryFunction(webhook, subscription, options);
        const duration = Date.now() - startTime;
        
        // Record successful delivery
        this.metrics.recordDeliveryAttempt(
          { ...labels, statusCode: result.statusCode },
          duration,
          true
        );
        
        if (this.detailedLogging) {
          console.log('Webhook delivery success:', {
            eventId: webhook.event_id,
            subscriptionId: subscription.id,
            duration,
            statusCode: result.statusCode
          });
        }
        
        return result;
        
      } catch (error) {
        const duration = Date.now() - startTime;
        
        // Record failed delivery
        this.metrics.recordDeliveryAttempt(
          { ...labels, statusCode: error.status },
          duration,
          false
        );
        
        if (this.detailedLogging) {
          console.error('Webhook delivery failed:', {
            eventId: webhook.event_id,
            subscriptionId: subscription.id,
            duration,
            error: error.message,
            statusCode: error.status
          });
        }
        
        throw error;
      }
    };
  }
  
  // Track retry attempts
  trackRetry(webhook, subscription, attemptNumber) {
    this.metrics.recordRetry(
      webhook.event_type,
      subscription.id,
      attemptNumber
    );
  }
  
  // Update queue metrics
  updateQueueMetrics(queueStats) {
    Object.entries(queueStats).forEach(([priority, stats]) => {
      this.metrics.updateQueueDepth(priority, stats.waiting + stats.active);
    });
  }
  
  // Update circuit breaker status
  updateCircuitBreaker(endpointHost, status) {
    this.metrics.updateCircuitBreakerStatus(endpointHost, status);
  }
  
  extractHost(url) {
    try {
      return new URL(url).hostname;
    } catch (error) {
      return 'unknown';
    }
  }
  
  // Express middleware for metrics endpoint
  getMetricsEndpoint() {
    return async (req, res) => {
      res.set('Content-Type', this.metrics.register.contentType);
      res.end(await this.metrics.getMetrics());
    };
  }
  
  // Get metrics as JSON for custom processing
  getMetricsJson() {
    return this.metrics.getMetricsAsJson();
  }
}

module.exports = MonitoringMiddleware;
```

## Step 2: Implement Real-time Dashboard

### Grafana Dashboard Configuration

```json
{
  "dashboard": {
    "title": "Hook0 Webhook Performance",
    "tags": ["webhooks", "hook0"],
    "timezone": "UTC",
    "refresh": "30s",
    "panels": [
      {
        "title": "Delivery Success Rate",
        "type": "stat",
        "targets": [
          {
            "expr": "rate(webhook_deliveries_total{status=\"success\"}[5m]) / rate(webhook_deliveries_total[5m]) * 100",
            "legendFormat": "Success Rate %"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "unit": "percent",
            "thresholds": {
              "steps": [
                {"color": "red", "value": 0},
                {"color": "yellow", "value": 95},
                {"color": "green", "value": 99}
              ]
            }
          }
        }
      },
      {
        "title": "Delivery Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(webhook_deliveries_total[1m])",
            "legendFormat": "{{status}} - {{event_type}}"
          }
        ]
      },
      {
        "title": "Response Time Percentiles",
        "type": "graph",
        "targets": [
          {
            "expr": "histogram_quantile(0.50, rate(webhook_delivery_duration_seconds_bucket[5m]))",
            "legendFormat": "P50"
          },
          {
            "expr": "histogram_quantile(0.95, rate(webhook_delivery_duration_seconds_bucket[5m]))",
            "legendFormat": "P95"
          },
          {
            "expr": "histogram_quantile(0.99, rate(webhook_delivery_duration_seconds_bucket[5m]))",
            "legendFormat": "P99"
          }
        ]
      },
      {
        "title": "Queue Depth",
        "type": "graph",
        "targets": [
          {
            "expr": "webhook_queue_depth",
            "legendFormat": "{{priority}} priority"
          }
        ]
      },
      {
        "title": "Error Rate by Status Code",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(webhook_deliveries_total{status=\"failure\"}[5m])",
            "legendFormat": "{{endpoint_host}}"
          }
        ]
      },
      {
        "title": "Circuit Breaker Status",
        "type": "table",
        "targets": [
          {
            "expr": "webhook_circuit_breaker_status",
            "format": "table"
          }
        ]
      }
    ]
  }
}
```

### Custom Real-time Dashboard

```javascript
// real-time-dashboard.js
const express = require('express');
const WebSocket = require('ws');
const MonitoringMiddleware = require('./monitoring-middleware');

class RealTimeDashboard {
  constructor(options = {}) {
    this.app = express();
    this.port = options.port || 3001;
    this.monitoring = new MonitoringMiddleware();
    this.wsClients = new Set();
    
    this.setupRoutes();
    this.setupWebSocket();
    this.startMetricsStreaming();
  }
  
  setupRoutes() {
    // Serve static dashboard files
    this.app.use(express.static('dashboard-public'));
    
    // Metrics endpoint for Prometheus
    this.app.get('/metrics', this.monitoring.getMetricsEndpoint());
    
    // JSON metrics endpoint
    this.app.get('/api/metrics', (req, res) => {
      res.json(this.monitoring.getMetricsJson());
    });
    
    // Health check
    this.app.get('/health', (req, res) => {
      res.json({
        status: 'healthy',
        timestamp: new Date().toISOString(),
        clients: this.wsClients.size
      });
    });
    
    // Dashboard summary
    this.app.get('/api/dashboard', async (req, res) => {
      const summary = await this.getDashboardSummary();
      res.json(summary);
    });
  }
  
  setupWebSocket() {
    this.wss = new WebSocket.Server({ port: this.port + 1 });
    
    this.wss.on('connection', (ws) => {
      this.wsClients.add(ws);
      
      // Send initial data
      this.sendMetricsToClient(ws);
      
      ws.on('close', () => {
        this.wsClients.delete(ws);
      });
      
      ws.on('error', (error) => {
        console.error('WebSocket error:', error);
        this.wsClients.delete(ws);
      });
    });
  }
  
  startMetricsStreaming() {
    // Stream metrics to connected clients every 5 seconds
    setInterval(async () => {
      if (this.wsClients.size > 0) {
        const summary = await this.getDashboardSummary();
        this.broadcastToClients(summary);
      }
    }, 5000);
  }
  
  async getDashboardSummary() {
    const metricsJson = this.monitoring.getMetricsJson();
    
    // Process metrics into dashboard format
    const summary = {
      timestamp: new Date().toISOString(),
      deliveryRate: this.extractMetricValue(metricsJson, 'webhook_deliveries_total'),
      successRate: this.calculateSuccessRate(metricsJson),
      averageResponseTime: this.extractMetricValue(metricsJson, 'webhook_delivery_duration_seconds'),
      queueDepth: this.extractMetricValue(metricsJson, 'webhook_queue_depth'),
      retryRate: this.extractMetricValue(metricsJson, 'webhook_retries_total'),
      circuitBreakers: this.extractCircuitBreakerStatus(metricsJson),
      topErrors: this.extractTopErrors(metricsJson),
      recentEvents: this.getRecentEvents()
    };
    
    return summary;
  }
  
  extractMetricValue(metrics, metricName) {
    const metric = metrics.find(m => m.name === metricName);
    if (!metric) return 0;
    
    if (metric.type === 'counter') {
      return metric.values.reduce((sum, v) => sum + v.value, 0);
    } else if (metric.type === 'gauge') {
      return metric.values[0]?.value || 0;
    } else if (metric.type === 'histogram') {
      // Return average from histogram
      const sum = metric.values.find(v => v.metricName?.includes('_sum'))?.value || 0;
      const count = metric.values.find(v => v.metricName?.includes('_count'))?.value || 1;
      return sum / count;
    }
    
    return 0;
  }
  
  calculateSuccessRate(metrics) {
    const deliveries = metrics.find(m => m.name === 'webhook_deliveries_total');
    if (!deliveries) return 100;
    
    const total = deliveries.values.reduce((sum, v) => sum + v.value, 0);
    const successful = deliveries.values
      .filter(v => v.labels.status === 'success')
      .reduce((sum, v) => sum + v.value, 0);
    
    return total > 0 ? (successful / total) * 100 : 100;
  }
  
  extractCircuitBreakerStatus(metrics) {
    const cbMetric = metrics.find(m => m.name === 'webhook_circuit_breaker_status');
    if (!cbMetric) return [];
    
    return cbMetric.values.map(v => ({
      endpoint: v.labels.endpoint_host,
      status: ['closed', 'half-open', 'open'][v.value] || 'unknown'
    }));
  }
  
  extractTopErrors(metrics) {
    const deliveries = metrics.find(m => m.name === 'webhook_deliveries_total');
    if (!deliveries) return [];
    
    const failures = deliveries.values
      .filter(v => v.labels.status === 'failure')
      .sort((a, b) => b.value - a.value)
      .slice(0, 5);
    
    return failures.map(f => ({
      endpoint: f.labels.endpoint_host,
      eventType: f.labels.event_type,
      count: f.value
    }));
  }
  
  getRecentEvents() {
    // This would typically come from a database or log aggregation
    // For demo purposes, return mock data
    return [
      {
        timestamp: new Date().toISOString(),
        eventType: 'user.created',
        status: 'success',
        duration: 245
      },
      {
        timestamp: new Date(Date.now() - 30000).toISOString(),
        eventType: 'order.completed',
        status: 'retry',
        duration: 5000
      }
    ];
  }
  
  sendMetricsToClient(ws) {
    this.getDashboardSummary().then(summary => {
      if (ws.readyState === WebSocket.OPEN) {
        ws.send(JSON.stringify({
          type: 'metrics',
          data: summary
        }));
      }
    });
  }
  
  broadcastToClients(data) {
    const message = JSON.stringify({
      type: 'metrics',
      data
    });
    
    this.wsClients.forEach(ws => {
      if (ws.readyState === WebSocket.OPEN) {
        ws.send(message);
      }
    });
  }
  
  start() {
    this.app.listen(this.port, () => {
      console.log(`Dashboard server running on port ${this.port}`);
      console.log(`WebSocket server running on port ${this.port + 1}`);
    });
  }
}

module.exports = RealTimeDashboard;
```

## Step 3: Implement Alerting System

### Multi-Channel Alerting

```javascript
// alerting-system.js
const nodemailer = require('nodemailer');
const { WebClient } = require('@slack/web-api');

class AlertingSystem {
  constructor(config = {}) {
    this.config = config;
    this.setupNotificationChannels();
    this.alertHistory = new Map();
    this.suppressionRules = new Map();
  }
  
  setupNotificationChannels() {
    // Email setup
    if (this.config.email) {
      this.emailTransporter = nodemailer.createTransporter(this.config.email);
    }
    
    // Slack setup
    if (this.config.slack?.token) {
      this.slackClient = new WebClient(this.config.slack.token);
    }
    
    // PagerDuty setup (webhook-based)
    this.pagerDutyEndpoint = this.config.pagerDuty?.endpoint;
    
    // Discord setup
    this.discordWebhook = this.config.discord?.webhookUrl;
  }
  
  async checkMetricsAndAlert(metrics) {
    const alerts = [];
    
    // Check success rate
    const successRate = this.calculateSuccessRate(metrics);
    if (successRate < 95) {
      alerts.push({
        severity: successRate < 90 ? 'critical' : 'warning',
        title: 'Low Webhook Success Rate',
        message: `Webhook success rate is ${successRate.toFixed(2)}% (threshold: 95%)`,
        metric: 'success_rate',
        value: successRate,
        threshold: 95
      });
    }
    
    // Check response time
    const avgResponseTime = this.extractAverageResponseTime(metrics);
    if (avgResponseTime > 5000) { // 5 seconds
      alerts.push({
        severity: avgResponseTime > 10000 ? 'critical' : 'warning',
        title: 'High Response Time',
        message: `Average response time is ${avgResponseTime}ms (threshold: 5000ms)`,
        metric: 'response_time',
        value: avgResponseTime,
        threshold: 5000
      });
    }
    
    // Check queue depth
    const queueDepth = this.extractQueueDepth(metrics);
    if (queueDepth > 1000) {
      alerts.push({
        severity: queueDepth > 5000 ? 'critical' : 'warning',
        title: 'High Queue Depth',
        message: `Queue depth is ${queueDepth} items (threshold: 1000)`,
        metric: 'queue_depth',
        value: queueDepth,
        threshold: 1000
      });
    }
    
    // Check circuit breaker status
    const openCircuitBreakers = this.extractOpenCircuitBreakers(metrics);
    if (openCircuitBreakers.length > 0) {
      alerts.push({
        severity: 'warning',
        title: 'Circuit Breakers Open',
        message: `${openCircuitBreakers.length} endpoints have open circuit breakers: ${openCircuitBreakers.join(', ')}`,
        metric: 'circuit_breakers',
        value: openCircuitBreakers.length,
        endpoints: openCircuitBreakers
      });
    }
    
    // Process and send alerts
    for (const alert of alerts) {
      await this.processAlert(alert);
    }
    
    return alerts;
  }
  
  async processAlert(alert) {
    const alertKey = `${alert.metric}_${alert.severity}`;
    
    // Check suppression rules
    if (this.shouldSuppressAlert(alertKey)) {
      console.log(`Alert suppressed: ${alertKey}`);
      return;
    }
    
    // Check if this is a duplicate alert
    const lastAlert = this.alertHistory.get(alertKey);
    if (lastAlert && Date.now() - lastAlert.timestamp < 300000) { // 5 minutes
      console.log(`Duplicate alert suppressed: ${alertKey}`);
      return;
    }
    
    // Send alert through configured channels
    const results = await Promise.allSettled([
      this.sendEmailAlert(alert),
      this.sendSlackAlert(alert),
      this.sendPagerDutyAlert(alert),
      this.sendDiscordAlert(alert)
    ]);
    
    // Record alert
    this.alertHistory.set(alertKey, {
      alert,
      timestamp: Date.now(),
      results: results.map(r => r.status)
    });
    
    console.log(`Alert sent: ${alert.title} [${alert.severity}]`);
  }
  
  async sendEmailAlert(alert) {
    if (!this.emailTransporter) return;
    
    const emailConfig = this.config.email;
    const isHtml = emailConfig.html !== false;
    
    const subject = `[${alert.severity.toUpperCase()}] Hook0 Alert: ${alert.title}`;
    
    const textMessage = `
Alert: ${alert.title}
Severity: ${alert.severity}
Message: ${alert.message}
Timestamp: ${new Date().toISOString()}
Metric: ${alert.metric}
Current Value: ${alert.value}
${alert.threshold ? `Threshold: ${alert.threshold}` : ''}
`;
    
    const htmlMessage = `
<h2 style="color: ${alert.severity === 'critical' ? 'red' : 'orange'}">
  [${alert.severity.toUpperCase()}] ${alert.title}
</h2>
<p><strong>Message:</strong> ${alert.message}</p>
<p><strong>Timestamp:</strong> ${new Date().toISOString()}</p>
<p><strong>Metric:</strong> ${alert.metric}</p>
<p><strong>Current Value:</strong> ${alert.value}</p>
${alert.threshold ? `<p><strong>Threshold:</strong> ${alert.threshold}</p>` : ''}
${alert.endpoints ? `<p><strong>Affected Endpoints:</strong> ${alert.endpoints.join(', ')}</p>` : ''}
`;
    
    await this.emailTransporter.sendMail({
      from: emailConfig.from,
      to: emailConfig.to,
      subject,
      text: textMessage,
      html: isHtml ? htmlMessage : undefined
    });
  }
  
  async sendSlackAlert(alert) {
    if (!this.slackClient) return;
    
    const color = alert.severity === 'critical' ? 'danger' : 'warning';
    const emoji = alert.severity === 'critical' ? '🚨' : '⚠️';
    
    await this.slackClient.chat.postMessage({
      channel: this.config.slack.channel,
      text: `${emoji} Hook0 Alert: ${alert.title}`,
      attachments: [
        {
          color,
          fields: [
            {
              title: 'Severity',
              value: alert.severity.toUpperCase(),
              short: true
            },
            {
              title: 'Metric',
              value: alert.metric,
              short: true
            },
            {
              title: 'Current Value',
              value: alert.value.toString(),
              short: true
            },
            {
              title: 'Threshold',
              value: alert.threshold?.toString() || 'N/A',
              short: true
            },
            {
              title: 'Message',
              value: alert.message,
              short: false
            }
          ],
          footer: 'Hook0 Monitoring',
          ts: Math.floor(Date.now() / 1000)
        }
      ]
    });
  }
  
  async sendPagerDutyAlert(alert) {
    if (!this.pagerDutyEndpoint || alert.severity !== 'critical') return;
    
    const payload = {
      routing_key: this.config.pagerDuty.routingKey,
      event_action: 'trigger',
      dedup_key: `hook0_${alert.metric}`,
      payload: {
        summary: alert.title,
        severity: alert.severity,
        source: 'Hook0 Monitoring',
        component: alert.metric,
        group: 'webhooks',
        class: 'performance',
        custom_details: {
          message: alert.message,
          current_value: alert.value,
          threshold: alert.threshold,
          endpoints: alert.endpoints
        }
      }
    };
    
    await fetch(this.pagerDutyEndpoint, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload)
    });
  }
  
  async sendDiscordAlert(alert) {
    if (!this.discordWebhook) return;
    
    const color = alert.severity === 'critical' ? 0xff0000 : 0xffa500;
    const emoji = alert.severity === 'critical' ? '🚨' : '⚠️';
    
    const embed = {
      title: `${emoji} Hook0 Alert: ${alert.title}`,
      description: alert.message,
      color,
      fields: [
        {
          name: 'Severity',
          value: alert.severity.toUpperCase(),
          inline: true
        },
        {
          name: 'Metric',
          value: alert.metric,
          inline: true
        },
        {
          name: 'Current Value',
          value: alert.value.toString(),
          inline: true
        }
      ],
      timestamp: new Date().toISOString(),
      footer: {
        text: 'Hook0 Monitoring'
      }
    };
    
    if (alert.threshold) {
      embed.fields.push({
        name: 'Threshold',
        value: alert.threshold.toString(),
        inline: true
      });
    }
    
    await fetch(this.discordWebhook, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        embeds: [embed]
      })
    });
  }
  
  shouldSuppressAlert(alertKey) {
    const rule = this.suppressionRules.get(alertKey);
    if (!rule) return false;
    
    return Date.now() < rule.suppressUntil;
  }
  
  suppressAlert(alertKey, durationMs) {
    this.suppressionRules.set(alertKey, {
      suppressUntil: Date.now() + durationMs
    });
  }
  
  // Helper methods for metric extraction
  calculateSuccessRate(metrics) {
    const deliveries = metrics.find(m => m.name === 'webhook_deliveries_total');
    if (!deliveries) return 100;
    
    const total = deliveries.values.reduce((sum, v) => sum + v.value, 0);
    const successful = deliveries.values
      .filter(v => v.labels.status === 'success')
      .reduce((sum, v) => sum + v.value, 0);
    
    return total > 0 ? (successful / total) * 100 : 100;
  }
  
  extractAverageResponseTime(metrics) {
    const duration = metrics.find(m => m.name === 'webhook_delivery_duration_seconds');
    if (!duration) return 0;
    
    const sum = duration.values.find(v => v.metricName?.includes('_sum'))?.value || 0;
    const count = duration.values.find(v => v.metricName?.includes('_count'))?.value || 1;
    
    return (sum / count) * 1000; // Convert to milliseconds
  }
  
  extractQueueDepth(metrics) {
    const queue = metrics.find(m => m.name === 'webhook_queue_depth');
    if (!queue) return 0;
    
    return queue.values.reduce((sum, v) => sum + v.value, 0);
  }
  
  extractOpenCircuitBreakers(metrics) {
    const cb = metrics.find(m => m.name === 'webhook_circuit_breaker_status');
    if (!cb) return [];
    
    return cb.values
      .filter(v => v.value === 2) // 2 = open
      .map(v => v.labels.endpoint_host);
  }
}

module.exports = AlertingSystem;
```

## Step 4: Implement Performance Analysis

### Automated Performance Analyzer

```javascript
// performance-analyzer.js
class PerformanceAnalyzer {
  constructor(options = {}) {
    this.analysisWindow = options.analysisWindow || 3600000; // 1 hour
    this.significanceThreshold = options.significanceThreshold || 0.1; // 10%
    this.minSampleSize = options.minSampleSize || 100;
  }
  
  async analyzePerformanceMetrics(metrics) {
    const analysis = {
      timestamp: new Date().toISOString(),
      overall: await this.analyzeOverallPerformance(metrics),
      endpoints: await this.analyzeEndpointPerformance(metrics),
      trends: await this.analyzeTrends(metrics),
      anomalies: await this.detectAnomalies(metrics),
      recommendations: []
    };
    
    analysis.recommendations = this.generateRecommendations(analysis);
    
    return analysis;
  }
  
  async analyzeOverallPerformance(metrics) {
    const successRate = this.calculateSuccessRate(metrics);
    const avgResponseTime = this.extractAverageResponseTime(metrics);
    const throughput = this.calculateThroughput(metrics);
    const retryRate = this.calculateRetryRate(metrics);
    
    return {
      successRate: {
        value: successRate,
        grade: this.gradeSuccessRate(successRate),
        trend: await this.calculateTrend('success_rate')
      },
      responseTime: {
        value: avgResponseTime,
        grade: this.gradeResponseTime(avgResponseTime),
        trend: await this.calculateTrend('response_time')
      },
      throughput: {
        value: throughput,
        grade: this.gradeThroughput(throughput),
        trend: await this.calculateTrend('throughput')
      },
      retryRate: {
        value: retryRate,
        grade: this.gradeRetryRate(retryRate),
        trend: await this.calculateTrend('retry_rate')
      }
    };
  }
  
  async analyzeEndpointPerformance(metrics) {
    const endpointStats = new Map();
    
    // Group metrics by endpoint
    const deliveries = metrics.find(m => m.name === 'webhook_deliveries_total');
    if (!deliveries) return [];
    
    deliveries.values.forEach(value => {
      const endpoint = value.labels.endpoint_host;
      if (!endpointStats.has(endpoint)) {
        endpointStats.set(endpoint, {
          endpoint,
          total: 0,
          successful: 0,
          failed: 0,
          responseTime: 0,
          retries: 0
        });
      }
      
      const stats = endpointStats.get(endpoint);
      stats.total += value.value;
      
      if (value.labels.status === 'success') {
        stats.successful += value.value;
      } else {
        stats.failed += value.value;
      }
    });
    
    // Calculate performance metrics for each endpoint
    const endpointAnalysis = Array.from(endpointStats.values()).map(stats => {
      const successRate = (stats.successful / stats.total) * 100;
      
      return {
        endpoint: stats.endpoint,
        successRate,
        totalRequests: stats.total,
        grade: this.gradeEndpointPerformance(successRate, stats.total),
        issues: this.identifyEndpointIssues(stats),
        recommendations: this.getEndpointRecommendations(stats)
      };
    });
    
    return endpointAnalysis.sort((a, b) => a.grade.localeCompare(b.grade));
  }
  
  async analyzeTrends(metrics) {
    // This would typically analyze historical data
    // For this example, we'll simulate trend analysis
    
    return {
      successRateTrend: {
        direction: 'stable', // up, down, stable
        change: 0.2, // percentage change
        confidence: 0.85
      },
      responseTimeTrend: {
        direction: 'up',
        change: 15.3,
        confidence: 0.92
      },
      throughputTrend: {
        direction: 'up',
        change: 8.7,
        confidence: 0.78
      }
    };
  }
  
  async detectAnomalies(metrics) {
    const anomalies = [];
    
    // Detect sudden spikes in response time
    const responseTime = this.extractAverageResponseTime(metrics);
    const historicalAvg = await this.getHistoricalAverage('response_time');
    
    if (responseTime > historicalAvg * 2) {
      anomalies.push({
        type: 'response_time_spike',
        severity: 'high',
        description: `Response time is ${Math.round(responseTime)}ms, significantly higher than the historical average of ${Math.round(historicalAvg)}ms`,
        value: responseTime,
        baseline: historicalAvg
      });
    }
    
    // Detect unusual error patterns
    const errorRate = 100 - this.calculateSuccessRate(metrics);
    const historicalErrorRate = await this.getHistoricalAverage('error_rate');
    
    if (errorRate > historicalErrorRate * 3) {
      anomalies.push({
        type: 'error_rate_spike',
        severity: 'critical',
        description: `Error rate is ${errorRate.toFixed(2)}%, much higher than typical ${historicalErrorRate.toFixed(2)}%`,
        value: errorRate,
        baseline: historicalErrorRate
      });
    }
    
    return anomalies;
  }
  
  generateRecommendations(analysis) {
    const recommendations = [];
    
    // Success rate recommendations
    if (analysis.overall.successRate.value < 99) {
      recommendations.push({
        category: 'reliability',
        priority: 'high',
        title: 'Improve Success Rate',
        description: 'Success rate is below optimal threshold',
        actions: [
          'Review failed endpoints and implement circuit breakers',
          'Increase retry attempts for transient failures',
          'Investigate and fix recurring error patterns'
        ]
      });
    }
    
    // Response time recommendations
    if (analysis.overall.responseTime.value > 5000) {
      recommendations.push({
        category: 'performance',
        priority: 'medium',
        title: 'Optimize Response Times',
        description: 'Average response time exceeds 5 seconds',
        actions: [
          'Implement connection pooling',
          'Add timeout configurations',
          'Consider implementing async webhook processing'
        ]
      });
    }
    
    // Endpoint-specific recommendations
    const poorPerformingEndpoints = analysis.endpoints.filter(e => e.grade === 'F');
    if (poorPerformingEndpoints.length > 0) {
      recommendations.push({
        category: 'endpoints',
        priority: 'high',
        title: 'Address Failing Endpoints',
        description: `${poorPerformingEndpoints.length} endpoints have poor performance`,
        actions: [
          'Implement circuit breakers for consistently failing endpoints',
          'Contact endpoint owners to resolve issues',
          'Consider reducing retry frequency for these endpoints'
        ],
        endpoints: poorPerformingEndpoints.map(e => e.endpoint)
      });
    }
    
    return recommendations;
  }
  
  // Grading methods
  gradeSuccessRate(rate) {
    if (rate >= 99.5) return 'A';
    if (rate >= 99) return 'B';
    if (rate >= 95) return 'C';
    if (rate >= 90) return 'D';
    return 'F';
  }
  
  gradeResponseTime(time) {
    if (time <= 1000) return 'A';
    if (time <= 3000) return 'B';
    if (time <= 5000) return 'C';
    if (time <= 10000) return 'D';
    return 'F';
  }
  
  gradeThroughput(throughput) {
    if (throughput >= 1000) return 'A';
    if (throughput >= 500) return 'B';
    if (throughput >= 100) return 'C';
    if (throughput >= 50) return 'D';
    return 'F';
  }
  
  gradeRetryRate(rate) {
    if (rate <= 5) return 'A';
    if (rate <= 10) return 'B';
    if (rate <= 20) return 'C';
    if (rate <= 30) return 'D';
    return 'F';
  }
  
  gradeEndpointPerformance(successRate, totalRequests) {
    if (totalRequests < this.minSampleSize) return 'N/A';
    return this.gradeSuccessRate(successRate);
  }
  
  // Helper methods (would be implemented with actual data sources)
  async getHistoricalAverage(metric) {
    // Mock implementation - would query actual historical data
    const mockHistoricals = {
      response_time: 2500,
      error_rate: 2.5,
      throughput: 150
    };
    
    return mockHistoricals[metric] || 0;
  }
  
  async calculateTrend(metric) {
    // Mock implementation - would calculate actual trends
    return Math.random() > 0.5 ? 'up' : 'down';
  }
  
  calculateThroughput(metrics) {
    const deliveries = metrics.find(m => m.name === 'webhook_deliveries_total');
    if (!deliveries) return 0;
    
    // Calculate deliveries per second (mock calculation)
    const total = deliveries.values.reduce((sum, v) => sum + v.value, 0);
    return total / 60; // Assuming metrics represent 1 minute of data
  }
  
  calculateRetryRate(metrics) {
    const retries = metrics.find(m => m.name === 'webhook_retries_total');
    const deliveries = metrics.find(m => m.name === 'webhook_deliveries_total');
    
    if (!retries || !deliveries) return 0;
    
    const totalRetries = retries.values.reduce((sum, v) => sum + v.value, 0);
    const totalDeliveries = deliveries.values.reduce((sum, v) => sum + v.value, 0);
    
    return totalDeliveries > 0 ? (totalRetries / totalDeliveries) * 100 : 0;
  }
  
  identifyEndpointIssues(stats) {
    const issues = [];
    
    if (stats.total < this.minSampleSize) {
      issues.push('Insufficient data for analysis');
    }
    
    const successRate = (stats.successful / stats.total) * 100;
    if (successRate < 90) {
      issues.push('Low success rate');
    }
    
    if (stats.retries / stats.total > 0.3) {
      issues.push('High retry rate');
    }
    
    return issues;
  }
  
  getEndpointRecommendations(stats) {
    const recommendations = [];
    const successRate = (stats.successful / stats.total) * 100;
    
    if (successRate < 50) {
      recommendations.push('Consider disabling this endpoint temporarily');
      recommendations.push('Implement circuit breaker');
    } else if (successRate < 90) {
      recommendations.push('Investigate error patterns');
      recommendations.push('Increase retry delays');
    }
    
    return recommendations;
  }
  
  // Metric calculation helpers (reusing from previous examples)
  calculateSuccessRate(metrics) {
    const deliveries = metrics.find(m => m.name === 'webhook_deliveries_total');
    if (!deliveries) return 100;
    
    const total = deliveries.values.reduce((sum, v) => sum + v.value, 0);
    const successful = deliveries.values
      .filter(v => v.labels.status === 'success')
      .reduce((sum, v) => sum + v.value, 0);
    
    return total > 0 ? (successful / total) * 100 : 100;
  }
  
  extractAverageResponseTime(metrics) {
    const duration = metrics.find(m => m.name === 'webhook_delivery_duration_seconds');
    if (!duration) return 0;
    
    const sum = duration.values.find(v => v.metricName?.includes('_sum'))?.value || 0;
    const count = duration.values.find(v => v.metricName?.includes('_count'))?.value || 1;
    
    return (sum / count) * 1000; // Convert to milliseconds
  }
}

module.exports = PerformanceAnalyzer;
```

## Performance Monitoring Best Practices

### Metrics Collection
- ✅ Use standardized metrics (Prometheus format)
- ✅ Include relevant labels for filtering and grouping
- ✅ Set appropriate histogram buckets for latency metrics
- ✅ Collect both technical and business metrics
- ✅ Implement proper metric cardinality management

### Dashboard Design
- ✅ Show the most important metrics prominently
- ✅ Use appropriate visualizations for different metric types
- ✅ Include both current and historical views
- ✅ Provide drill-down capabilities
- ✅ Make dashboards accessible to different stakeholders

### Alerting Strategy
- ✅ Set meaningful thresholds based on business impact
- ✅ Implement alert suppression to prevent spam
- ✅ Use multiple notification channels
- ✅ Include enough context for quick resolution
- ✅ Regularly review and tune alert rules

### Performance Analysis
- ✅ Regularly analyze performance trends
- ✅ Identify patterns and anomalies
- ✅ Provide actionable recommendations
- ✅ Track improvements over time
- ✅ Share insights with relevant teams

Ready to implement comprehensive webhook performance monitoring? Start with basic metrics collection and gradually add advanced features like real-time dashboards and automated alerting.