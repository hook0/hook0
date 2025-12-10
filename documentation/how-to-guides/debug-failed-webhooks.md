# Debugging Failed Webhook Deliveries

This guide helps you identify, diagnose, and resolve webhook delivery failures in Hook0. Learn systematic approaches to troubleshoot common issues and prevent future failures.

## Quick Diagnosis Checklist

Before diving deep, run through this quick checklist:

- [ ] Is the webhook endpoint accessible?
- [ ] Are you returning the correct HTTP status codes?
- [ ] Is the endpoint processing requests within timeout limits?
- [ ] Have you verified the webhook signature?
- [ ] Is your subscription properly configured?

## Understanding Webhook Failures

Hook0 categorizes delivery failures to help you understand the root cause:

### HTTP Status Code Categories

:::tip 2xx - Success
Request processed successfully. No retry needed.
:::

:::warning 4xx - Client Errors
- 400-407, 409-499: Permanent failures, no retry
- 408 (Timeout), 429 (Rate Limited): Temporary failures, will retry
:::

:::danger 5xx - Server Errors
All 5xx codes: Temporary failures, will retry. Suggests issues with your webhook endpoint.
:::

:::info Network Errors
- Connection timeouts
- DNS resolution failures
- Connection refused
:::

## Step 1: Access Hook0 Dashboard Diagnostics

### Navigate to Request Attempts

1. **Go to your Hook0 Dashboard**
2. **Select your Application**
3. **Click on "Events"**
4. **Find the failed event and click "View Details"**
5. **Review "Request Attempts" section**

### Analyze Delivery Attempts

Look for these key details:

```json
{
  "attempt_number": 3,
  "status_code": 500,
  "response_body": "Internal Server Error",
  "error_message": null,
  "duration_ms": 5000,
  "created_at": "2024-01-15T10:30:00Z",
  "next_retry_at": "2024-01-15T10:34:00Z"
}
```

## Step 2: Using the API for Detailed Analysis

### Get Request Attempts via API

```bash
# Get all request attempts for an application
curl "http://localhost:8081/api/v1/request_attempts/?application_id={APP_ID}" \
  -H "Authorization: Bearer {YOUR_TOKEN}"

# Filter by event
curl "http://localhost:8081/api/v1/request_attempts/?application_id={APP_ID}&event_id={EVENT_ID}" \
  -H "Authorization: Bearer {YOUR_TOKEN}"

# Filter by subscription
curl "http://localhost:8081/api/v1/request_attempts/?application_id={APP_ID}&subscription_id={SUBSCRIPTION_ID}" \
  -H "Authorization: Bearer {YOUR_TOKEN}"
```

### Get Response Details

```bash
# Get the response body and headers for a failed attempt
curl "http://localhost:8081/api/v1/responses/{RESPONSE_ID}?application_id={APP_ID}" \
  -H "Authorization: Bearer {YOUR_TOKEN}"
```

## Step 3: Common Failure Scenarios and Solutions

### Scenario 1: Connection Timeouts

**Symptoms:**
- Error message: "Connection timeout"
- No HTTP status code
- High duration_ms values

**Diagnosis:**
```bash
# Test endpoint connectivity
curl -v --max-time 30 https://your-webhook-endpoint.com/webhook

# Check DNS resolution
nslookup your-webhook-endpoint.com

# Test from different networks
curl -v https://your-webhook-endpoint.com/webhook
```

**Solutions:**
- Optimize webhook processing to respond faster
- Increase server resources
- Check network connectivity
- Verify DNS configuration

### Scenario 2: SSL/TLS Certificate Issues

**Symptoms:**
- Error message: "SSL certificate verification failed"
- Connection errors for HTTPS endpoints

**Diagnosis:**
```bash
# Check SSL certificate
openssl s_client -connect your-domain.com:443 -servername your-domain.com

# Verify certificate chain
curl -v https://your-webhook-endpoint.com/webhook
```

**Solutions:**
```bash
# Renew expired certificates
certbot renew

# Fix certificate chain issues
# Ensure intermediate certificates are included

# Test with SSL Labs
# https://www.ssllabs.com/ssltest/
```

### Scenario 3: Webhook Signature Verification Failures

**Symptoms:**
- 401 Unauthorized responses
- Error messages about invalid signatures

**Diagnosis:**

Add logging to compare expected vs received signature. See [Implementing Webhook Authentication](../tutorials/webhook-authentication.md) for the correct verification code.

**Solutions:**
- Use raw request body for signature verification (not parsed JSON)
- Ensure consistent character encoding (UTF-8)
- Verify you're using the correct subscription secret
- Check HMAC algorithm (SHA256)
- Include headers in signature computation (v1 format)

### Scenario 4: Rate Limiting Issues

**Symptoms:**
- 429 Too Many Requests responses
- Intermittent failures during high traffic

**Diagnosis:**
```bash
# Monitor request patterns - filter 429 responses
curl "http://localhost:8081/api/v1/request_attempts/?application_id={APP_ID}&subscription_id={SUBSCRIPTION_ID}" \
  -H "Authorization: Bearer {YOUR_TOKEN}" | \
  jq '.[] | select(.status.type == "failed")'
```

**Solutions:**
```javascript
// Implement rate limiting on your endpoint
const rateLimit = require('express-rate-limit');

const webhookLimiter = rateLimit({
  windowMs: 15 * 60 * 1000, // 15 minutes
  max: 1000, // limit each IP to 1000 requests per windowMs
  message: 'Too many webhook requests',
  standardHeaders: true,
  legacyHeaders: false,
});

app.use('/webhooks', webhookLimiter);
```

### Scenario 5: Internal Server Errors (5xx)

**Symptoms:**
- 500, 502, 503, 504 responses
- Generic error messages

**Diagnosis:**
```javascript
// Add comprehensive logging to your webhook handler
app.post('/webhook', (req, res) => {
  const startTime = Date.now();
  
  try {
    console.log('Webhook received:', {
      timestamp: new Date().toISOString(),
      headers: req.headers,
      body: req.body
    });
    
    // Your webhook processing logic
    processWebhook(req.body);
    
    const duration = Date.now() - startTime;
    console.log('Webhook processed successfully:', { duration });
    
    res.json({ status: 'processed' });
  } catch (error) {
    const duration = Date.now() - startTime;
    console.error('Webhook processing failed:', {
      error: error.message,
      stack: error.stack,
      duration
    });
    
    res.status(500).json({ error: 'Internal server error' });
  }
});
```

**Solutions:**
- Add proper error handling and logging
- Monitor application performance and resources
- Implement health checks
- Use application monitoring tools (APM)

## Step 4: Setting Up Webhook Debugging

### Create a Debug Webhook Endpoint

```javascript
// debug-webhook.js
const express = require('express');
const crypto = require('crypto');
const fs = require('fs');
const app = express();

// Middleware to log all requests
app.use((req, res, next) => {
  const timestamp = new Date().toISOString();
  const logEntry = {
    timestamp,
    method: req.method,
    url: req.url,
    headers: req.headers,
    query: req.query,
    ip: req.ip
  };
  
  console.log('Request received:', JSON.stringify(logEntry, null, 2));
  next();
});

// Parse JSON body
app.use('/webhook', express.json());

// Note: Express.js normalizes all header names to lowercase
app.post('/webhook', (req, res) => {
  const timestamp = new Date().toISOString();
  const signature = req.headers['x-hook0-signature'];
  const bodyString = JSON.stringify(req.body);

  const debugInfo = {
    timestamp,
    signature,
    bodyLength: bodyString.length,
    bodyPreview: bodyString.slice(0, 200),
    headers: req.headers
  };

  console.log('Webhook debug info:', JSON.stringify(debugInfo, null, 2));

  // Save to file for analysis
  fs.appendFileSync('webhook-debug.log', JSON.stringify({
    ...debugInfo,
    fullBody: bodyString
  }) + '\n');

  // Always respond successfully for debugging
  res.json({
    status: 'debug_received',
    timestamp,
    bodyLength: bodyString.length
  });
});

app.listen(3000, () => {
  console.log('Debug webhook server running on port 3000');
});
```

### Use ngrok for Local Testing

```bash
# Install ngrok
npm install -g ngrok

# Start your debug server
node debug-webhook.js

# In another terminal, expose it
ngrok http 3000

# Use the ngrok URL in your Hook0 subscription
# https://abc123.ngrok.io/webhook
```

## Step 5: Automated Failure Detection

### Monitor Failure Rates with Script

```javascript
// monitor-failures.js
const fetch = require('node-fetch');

const HOOK0_TOKEN = '{YOUR_TOKEN}';
const APP_ID = '{APP_ID}';
const SUBSCRIPTION_ID = '{SUBSCRIPTION_ID}';

async function getFailureRate(subscriptionId, hours = 24) {
  // Get request attempts, optionally filtered by subscription
  const url = subscriptionId
    ? `http://localhost:8081/api/v1/request_attempts/?application_id=${APP_ID}&subscription_id=${subscriptionId}`
    : `http://localhost:8081/api/v1/request_attempts/?application_id=${APP_ID}`;

  const response = await fetch(url, {
    headers: {
      'Authorization': `Bearer ${HOOK0_TOKEN}`
    }
  });

  const attempts = await response.json();

  const total = attempts.length;
  // Status is a string: "pending", "succeeded", or "failed"
  const failed = attempts.filter(a => a.failed_at !== null).length;
  const failureRate = total > 0 ? (failed / total) * 100 : 0;

  return { total, failed, failureRate, attempts: attempts.slice(0, 5) };
}

async function monitorSubscriptions() {
  try {
    const stats = await getFailureRate(SUBSCRIPTION_ID);

    console.log(`Failure Rate: ${stats.failureRate.toFixed(2)}%`);
    console.log(`Total Attempts: ${stats.total}`);
    console.log(`Failed Attempts: ${stats.failed}`);

    if (stats.failureRate > 10) {
      console.warn('⚠️ High failure rate detected!');

      // Show recent failures (failed_at is set when delivery failed)
      const recentFailures = stats.attempts.filter(a => a.failed_at !== null);
      console.log('Recent failures:', recentFailures);
    }

  } catch (error) {
    console.error('Monitoring error:', error.message);
  }
}

// Run monitoring
monitorSubscriptions();

// Schedule regular monitoring
setInterval(monitorSubscriptions, 5 * 60 * 1000); // Every 5 minutes
```

### Set Up Alerts

```javascript
// alert-system.js
const nodemailer = require('nodemailer');

const transporter = nodemailer.createTransporter({
  host: 'smtp.your-domain.com',
  port: 587,
  auth: {
    user: 'alerts@your-domain.com',
    pass: 'your-password'
  }
});

async function sendAlert(subject, message) {
  await transporter.sendMail({
    from: 'alerts@your-domain.com',
    to: 'admin@your-domain.com',
    subject: `Hook0 Alert: ${subject}`,
    text: message,
    html: `<pre>${message}</pre>`
  });
}

async function checkAndAlert() {
  const stats = await getFailureRate(SUBSCRIPTION_ID);

  if (stats.failureRate > 20) {
    const failedAttempts = stats.attempts.filter(a => a.failed_at !== null);
    await sendAlert('High Webhook Failure Rate', `
Failure Rate: ${stats.failureRate.toFixed(2)}%
Total Attempts: ${stats.total}
Failed Attempts: ${stats.failed}

Recent failures:
${JSON.stringify(failedAttempts, null, 2)}
    `);
  }
}
```

## Step 6: Recovery Strategies

### Manual Retry Failed Events

```bash
# Get events for your application
curl "http://localhost:8081/api/v1/events/?application_id={APP_ID}" \
  -H "Authorization: Bearer {YOUR_TOKEN}"

# Replay a specific event
curl -X POST "http://localhost:8081/api/v1/events/{EVENT_ID}/replay" \
  -H "Authorization: Bearer {YOUR_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "{APP_ID}"
  }'
```

### Bulk Retry Script

```javascript
// retry-failed.js
const HOOK0_TOKEN = '{YOUR_TOKEN}';
const APP_ID = '{APP_ID}';

async function retryFailedEvents(maxAge = 24) {
  // Get failed request attempts
  const response = await fetch(
    `http://localhost:8081/api/v1/request_attempts/?application_id=${APP_ID}`,
    {
      headers: { 'Authorization': `Bearer ${HOOK0_TOKEN}` }
    }
  );

  const attempts = await response.json();

  // Filter failed attempts (failed_at is set when delivery failed)
  const failedAttempts = attempts.filter(a => a.failed_at !== null);

  // Get unique event IDs
  const uniqueEvents = [...new Set(failedAttempts.map(a => a.event_id))];

  console.log(`Found ${uniqueEvents.length} failed events to retry`);

  for (const eventId of uniqueEvents) {
    try {
      const retryResponse = await fetch(
        `http://localhost:8081/api/v1/events/${eventId}/replay`,
        {
          method: 'POST',
          headers: {
            'Authorization': `Bearer ${HOOK0_TOKEN}`,
            'Content-Type': 'application/json'
          },
          body: JSON.stringify({ application_id: APP_ID })
        }
      );

      if (retryResponse.ok) {
        console.log(`✅ Replayed event: ${eventId}`);
      } else {
        const error = await retryResponse.text();
        console.log(`❌ Failed to replay event: ${eventId} - ${error}`);
      }

      // Rate limit retries
      await new Promise(resolve => setTimeout(resolve, 100));
    } catch (error) {
      console.error(`Error replaying event ${eventId}:`, error.message);
    }
  }
}

retryFailedEvents();
```

## Step 7: Prevention Strategies

### Implement Circuit Breaker Pattern

```javascript
// circuit-breaker.js
class CircuitBreaker {
  constructor(threshold = 5, timeout = 60000) {
    this.threshold = threshold;
    this.timeout = timeout;
    this.failureCount = 0;
    this.lastFailure = null;
    this.state = 'CLOSED'; // CLOSED, OPEN, HALF_OPEN
  }
  
  async call(fn) {
    if (this.state === 'OPEN') {
      if (Date.now() - this.lastFailure < this.timeout) {
        throw new Error('Circuit breaker is OPEN');
      }
      this.state = 'HALF_OPEN';
    }
    
    try {
      const result = await fn();
      this.onSuccess();
      return result;
    } catch (error) {
      this.onFailure();
      throw error;
    }
  }
  
  onSuccess() {
    this.failureCount = 0;
    this.state = 'CLOSED';
  }
  
  onFailure() {
    this.failureCount++;
    this.lastFailure = Date.now();
    
    if (this.failureCount >= this.threshold) {
      this.state = 'OPEN';
    }
  }
}

// Use in webhook handler
const circuitBreaker = new CircuitBreaker(3, 30000);

app.post('/webhook', async (req, res) => {
  try {
    await circuitBreaker.call(async () => {
      await processWebhook(req.body);
    });
    
    res.json({ status: 'processed' });
  } catch (error) {
    console.error('Circuit breaker prevented processing:', error.message);
    res.status(503).json({ error: 'Service temporarily unavailable' });
  }
});
```

### Implement Graceful Degradation

```javascript
// graceful-degradation.js
app.post('/webhook', async (req, res) => {
  try {
    // Try primary processing
    await processPrimary(req.body);
    res.json({ status: 'processed' });
  } catch (primaryError) {
    console.warn('Primary processing failed, trying fallback:', primaryError.message);
    
    try {
      // Try fallback processing
      await processFallback(req.body);
      res.json({ status: 'processed_fallback' });
    } catch (fallbackError) {
      console.error('Both primary and fallback failed:', fallbackError.message);
      
      // Store for later processing
      await storeForRetry(req.body);
      res.status(202).json({ status: 'queued_for_retry' });
    }
  }
});
```

## Best Practices for Webhook Reliability

### Endpoint design
- ✅ Return appropriate HTTP status codes
- ✅ Respond within 30 seconds
- ✅ Implement idempotency
- ✅ Use structured error responses
- ✅ Add comprehensive logging

### Error Handling
- ✅ Distinguish between temporary and permanent failures
- ✅ Implement exponential backoff
- ✅ Use circuit breakers for external dependencies
- ✅ Monitor and alert on high failure rates
- ✅ Provide detailed error messages

### Testing
- ✅ Test webhook endpoints thoroughly
- ✅ Simulate failure scenarios
- ✅ Verify signature validation
- ✅ Test with various payload sizes
- ✅ Monitor performance under load

## Troubleshooting Checklist

When webhook deliveries fail, work through this systematic checklist:

### Infrastructure
- [ ] Endpoint is accessible from internet
- [ ] SSL certificate is valid and not expired
- [ ] DNS resolution works correctly
- [ ] Firewall allows incoming connections
- [ ] Load balancer is configured correctly

### Application
- [ ] Webhook handler is properly implemented
- [ ] Signature verification is working
- [ ] Response times are under 30 seconds
- [ ] Error handling returns appropriate status codes
- [ ] Logging captures enough detail for debugging

### Hook0 Configuration
- [ ] Subscription is enabled and active
- [ ] Event types match what you're sending
- [ ] Target URL is correct
- [ ] Custom headers are properly configured
- [ ] Retry configuration is appropriate

Ready to implement these debugging strategies? Start with the Hook0 dashboard analysis and work your way through the systematic approaches outlined above.