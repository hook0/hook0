---
sidebar_position: 5
---

# Troubleshooting Guide

This guide helps you diagnose and resolve common issues with Hook0. Each section includes symptoms, root causes, solutions, and debug commands.

### Set Up Environment Variables

```bash
# Set your service token (from dashboard)
export HOOK0_TOKEN="YOUR_TOKEN_HERE"
export HOOK0_API="https://app.hook0.com/api/v1" # Replace by your domain (or http://localhost:8081 locally)

# Set your application ID (shown in dashboard URL or application details)
export APP_ID="YOUR_APPLICATION_ID_HERE"
```

Save these values:
```bash
# Save to .env file for later use
cat > .env <<EOF
HOOK0_TOKEN=$HOOK0_TOKEN
HOOK0_API=$HOOK0_API
APP_ID=$APP_ID
EOF
```

## Connection Issues

### Cannot Connect to Hook0 API

**Symptoms**:
- `ECONNREFUSED` or connection timeout errors
- "Cannot reach host" messages
- Network-level failures

**Possible Causes**:

1. **Incorrect API URL**

```bash
# ❌ Wrong - missing application_id
curl $HOOK0_API/events/

# ✅ Correct
curl "$HOOK0_API/events/?application_id=$APP_ID" \
  -H "Authorization: Bearer $HOOK0_TOKEN"
```

2. **Firewall blocking outbound connections**

**Solution**: Allow outbound HTTPS (port 443) to `app.hook0.com`

3. **Self-hosted instance not running**

**Debug commands**:

```bash
# Check if API is accessible (swagger endpoint is always available)
curl -s http://localhost:8081/api/v1/swagger.json | head -c 100

# Health endpoint (requires HEALTH_CHECK_KEY env var to be set)
# If configured: curl "http://localhost:8081/api/v1/health/?key={HEALTH_CHECK_KEY}"

# Test DNS resolution
nslookup app.hook0.com

# Test connectivity with verbose output
curl -v "$HOOK0_API/events/?application_id=$APP_ID" \
  -H "Authorization: Bearer $HOOK0_TOKEN"

# For self-hosted: Check container status
docker ps | grep hook0
docker logs hook0-api-1

# Check API process
ps aux | grep hook0
```

**Solutions**:
1. Verify API URL in your configuration
2. Check network connectivity and DNS
3. For self-hosted: Ensure all containers are running
4. Review firewall rules and security groups

### Database Connection Failed (Self-Hosted)

**Symptoms**:
- "Connection to database failed" in logs
- API returns 503 Service Unavailable
- `sqlx::Error` in application logs

**Possible Causes**:

1. **PostgreSQL not running**

```bash
# Check PostgreSQL status
docker logs hook0-postgres

# Verify PostgreSQL is listening
docker exec hook0-postgres pg_isready

# Check logs for errors
docker logs hook0-postgres --tail 100
```

2. **Incorrect database credentials**

```bash
# Check environment variables
docker exec hook0-api env | grep DATABASE_URL

# Test connection manually
docker exec -it hook0-postgres psql -U hook0 -d hook0
```

3. **Network connectivity between API and database**

```bash
# Check if API can reach database
docker exec hook0-api nc -zv postgres 5432

# Verify Docker network
docker network inspect hook0-network
```

**Solutions**:

1. Start PostgreSQL container:
```bash
docker-compose up -d postgres
```

2. Fix database URL in docker-compose.yaml:
```yaml
environment:
  DATABASE_URL: postgresql://hook0:hook0@postgres:5432/hook0
```

3. Run migrations:
```bash
docker exec hook0-api sqlx migrate run
```

4. Check PostgreSQL logs for specific errors:
```bash
docker logs hook0-postgres | grep ERROR
```

## Authentication Issues

### Invalid Token Error

**Symptoms**:
- HTTP 403 Forbidden
- Error: `AuthInvalidBiscuit`
- "Invalid authentication token" message

**Possible Causes**:

1. **Token expired**

```bash
# Check token expiration using biscuit-cli
biscuit inspect token "$TOKEN" --public-key "$PUBLIC_KEY"

# Look for: check if time($t), $t < [expiration]
```

2. **Token revoked**

Check Hook0 dashboard:
- Navigate to Organization → Service Tokens
- Verify token is not revoked

3. **Incorrect token format**

```bash
# ✅ Correct format
Authorization: Bearer EoQKCAoh...

# ❌ Wrong - missing "Bearer"
Authorization: EoQKCAoh...
```

4. **Wrong public key (self-hosted)**

```bash
# Verify public key in API configuration
docker exec hook0-api env | grep BISCUIT_PUBLIC_KEY

# Compare with key used to generate token
```

**Debug commands**:

```bash
# Test API with token
curl -v "$HOOK0_API/events/?application_id=$APP_ID" \
  -H "Authorization: Bearer $HOOK0_TOKEN"

# Check response headers for error details
# Look for X-Error-Id header

# Decode token (requires biscuit-cli)
biscuit inspect token "$HOOK0_TOKEN" --public-key "PUBLIC_KEY"
```

**Solutions**:

1. Generate new token via dashboard or API:
```bash
curl -X POST "$HOOK0_API/service_token" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "organization_id": "org_123",
    "name": "New API Token",
    "description": "Replacement for expired token"
  }'
```

2. Update token in your application:
```bash
export HOOK0_TOKEN="NEW_TOKEN_HERE"
```

3. Verify token works:
```bash
curl "$HOOK0_API/events/?application_id=$APP_ID" \
  -H "Authorization: Bearer $HOOK0_TOKEN"
```

### Permission Denied

**Symptoms**:
- HTTP 403 Forbidden
- "Insufficient permissions" message
- Specific operations fail

**Possible Causes**:

1. **Token lacks required permissions**

```bash
# Inspect token capabilities
biscuit inspect token "$TOKEN" --public-key "$PUBLIC_KEY"

# Look for right() facts:
# right("events", "write")
# right("subscriptions", "read")
```

2. **Application scope mismatch**

Token restricted to specific application but request targets different application.

3. **Organization membership issue**

User not member of target organization or has incorrect role.

**Debug commands**:

```bash
# List organizations user belongs to
curl "$HOOK0_API/organizations" \
  -H "Authorization: Bearer $HOOK0_TOKEN"

# Check specific organization info and roles
curl "$HOOK0_API/organizations/{ORG_ID}" \
  -H "Authorization: Bearer $HOOK0_TOKEN"
```

**Solutions**:

1. Create token with correct permissions:
```bash
# Request editor role token
curl -X POST "$HOOK0_API/service_token" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "organization_id": "org_123",
    "name": "Editor Token"
  }'
```

2. For self-hosted, update user role in database:
```sql
UPDATE iam.user_organization
SET role = 'editor'
WHERE user__id = 'user_id' AND organization__id = 'org_id';
```

3. Verify application_id matches token scope:
```javascript
// Ensure request uses correct application
const response = await hook0.sendEvent({
  applicationId: 'app_123',  // Must match token scope
  eventType: 'user.account.created',
  payload: { ... }
});
```

### No Authorization Header

**Symptoms**:
- HTTP 401 Unauthorized
- Error: `AuthNoAuthorizationHeader`
- "Authorization header required" message

**Possible Causes**:

1. **Missing Authorization header**

```javascript
// ❌ Wrong - no Authorization header
fetch('http://localhost:8081/api/v1/events', {
  method: 'POST',
  body: JSON.stringify(event)
});

// ✅ Correct
fetch('http://localhost:8081/api/v1/events', {
  method: 'POST',
  headers: {
    'Authorization': `Bearer ${token}`,
    'Content-Type': 'application/json'
  },
  body: JSON.stringify(event)
});
```

2. **Header stripped by proxy/load balancer**

Check proxy configuration to ensure Authorization headers are forwarded.

**Solutions**:

1. Add Authorization header to all requests
2. Verify header is present in request:
```bash
curl -v $HOOK0_API/events \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  2>&1 | grep Authorization
```

## Event Delivery Issues

### Events Not Being Delivered

**Symptoms**:
- Events created successfully but webhooks not triggered
- Zero delivery attempts in dashboard
- Subscriptions show no activity

**Possible Causes**:

1. **Subscription disabled**

**Debug commands**:
```bash
# Check subscription status
curl "$HOOK0_API/subscriptions/{sub-id}?application_id=$APP_ID" \
  -H "Authorization: Bearer $HOOK0_TOKEN"

# Look for: "is_enabled": false
```

**Solution**:

First, get the current subscription configuration:
```bash
curl "$HOOK0_API/subscriptions/{sub-id}?application_id=$APP_ID" \
  -H "Authorization: Bearer $HOOK0_TOKEN" > subscription.json
```

:::warning PUT Requires ALL Fields
The PUT endpoint replaces the entire subscription. You must include ALL fields (`application_id`, `is_enabled`, `event_types`, `label_key`, `label_value`, `target`), not just the ones you want to change. Missing fields will cause validation errors.
:::

Then update with all required fields:
```bash
# Enable subscription (include all required fields)
curl -X PUT "$HOOK0_API/subscriptions/{sub-id}" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "'"$APP_ID"'",
    "is_enabled": true,
    "event_types": ["your.event.type"],
    "label_key": "environment",
    "label_value": "production",
    "target": {
      "type": "http",
      "method": "POST",
      "url": "https://your-webhook.com/webhook",
      "headers": {"Content-Type": "application/json"}
    }
  }'
```

2. **Event type mismatch**

```bash
# List subscription event types
curl "$HOOK0_API/subscriptions/{sub-id}?application_id=$APP_ID" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  | jq '.event_types'

# Compare with event sent
# Event: "user.created"
# Subscription: ["user.registered"]  ← mismatch!
```

**Solution**: Update subscription event types or send correct event type.

3. **Label filter not matching**

```bash
# Check event labels
curl "$HOOK0_API/events/{event-id}?application_id=$APP_ID" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  | jq '.labels'

# Check subscription label filter
curl "$HOOK0_API/subscriptions/{sub-id}?application_id=$APP_ID" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  | jq '{label_key, label_value}'

# Labels must match exactly (case-sensitive)
```

**Solution**: Ensure event labels match subscription filter:
```javascript
// Event
labels: { environment: "production" }

// Subscription
label_key: "environment"
label_value: "production"  // Must match exactly
```

4. **Output worker not running (self-hosted)**

```bash
# Check output worker status
docker logs hook0-output-worker

# Should see: "Worker started" messages
# If not running:
docker-compose up -d output-worker
```

5. **Event dispatch trigger disabled**

For self-hosted, verify database trigger exists:
```sql
SELECT tgname FROM pg_trigger WHERE tgname = 'event_dispatch_trigger';
```

### Webhook Signature Verification Failing

**Symptoms**:
- Webhook endpoint receiving requests but signature invalid
- "Invalid signature" errors in webhook logs
- Requests rejected by webhook handler

**Possible Causes**:

1. **Using wrong secret**

```bash
# Get subscription secret
curl "$HOOK0_API/subscriptions/{sub-id}?application_id=$APP_ID" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  | jq -r '.secret'

# Verify it matches your webhook code
```

2. **Incorrect signature verification logic**

```javascript
// ❌ Wrong - not using raw body
const computed = crypto.createHmac('sha256', secret)
  .update(JSON.stringify(req.body))  // Wrong if body already parsed
  .digest('hex');

// ✅ Correct - use raw body
const computed = crypto.createHmac('sha256', secret)
  .update(rawBody)  // Raw request body as received
  .digest('hex');
```

**Solutions**:

See [Implementing Webhook Authentication](../tutorials/webhook-authentication.md) for correct signature verification code and [Debugging Failed Webhooks](./debug-failed-webhooks.md#scenario-3-webhook-signature-verification-failures) for debugging tips.

2. Use `JSON.stringify(req.body)` for signature verification:

```javascript
// Express - use express.json() and stringify for verification
app.use('/webhook', express.json());

// In handler, stringify the body for signature verification
const payload = JSON.stringify(req.body);
const isValid = verifySignature(payload, signature, secret);
```

3. Rotate subscription secret if compromised:

To rotate a subscription secret, delete the subscription and recreate it, or update it via PUT (the secret is regenerated on update):
```bash
# Delete and recreate subscription to get new secret
curl -X DELETE "$HOOK0_API/subscriptions/{sub-id}?application_id=$APP_ID" \
  -H "Authorization: Bearer $HOOK0_TOKEN"

# Then create a new one (see subscriptions API documentation)
```

### Webhook Endpoint Timing Out

**Symptoms**:
- Delivery attempts show high duration_ms (>30 seconds)
- "Connection timeout" errors
- Retries exhausted

**Possible Causes**:

1. **Slow webhook processing**

```javascript
// ❌ Slow - blocking operations in webhook handler
app.post('/webhook', (req, res) => {
  const event = req.body;

  // Synchronous processing (slow)
  processEvent(event);  // 30+ seconds
  updateDatabase(event);
  sendNotifications(event);

  res.sendStatus(200);  // Takes too long
});

// ✅ Fast - respond immediately, process async
app.post('/webhook', (req, res) => {
  const event = req.body;

  // Respond immediately
  res.sendStatus(200);

  // Process asynchronously
  processEventAsync(event).catch(console.error);
});
```

2. **Database queries timing out**

**Debug**:
```bash
# Monitor slow queries
docker exec -it postgres \
  psql -U hook0 -c "SELECT query, state, wait_event_type, wait_event
                    FROM pg_stat_activity
                    WHERE state != 'idle' AND query_start < now() - interval '10 seconds';"
```

3. **External API calls blocking**

```javascript
// ❌ Blocking external API call
app.post('/webhook', async (req, res) => {
  await fetch('https://slow-external-api.com/notify', {
    method: 'POST',
    body: JSON.stringify(req.body)
  });  // May take 30+ seconds

  res.sendStatus(200);
});

// ✅ Use queue for external calls
app.post('/webhook', async (req, res) => {
  await queue.push({ type: 'external_notify', data: req.body });
  res.sendStatus(200);
});
```

**Solutions**:

1. Respond within 30 seconds:
```javascript
const WEBHOOK_TIMEOUT = 25000;  // 25 seconds (leave buffer)

app.post('/webhook', (req, res) => {
  // Set response timeout
  const timeout = setTimeout(() => {
    res.sendStatus(200);
  }, WEBHOOK_TIMEOUT);

  // Process with timeout
  processEvent(req.body)
    .then(() => {
      clearTimeout(timeout);
      res.sendStatus(200);
    })
    .catch(err => {
      clearTimeout(timeout);
      console.error('Processing failed:', err);
      res.sendStatus(200);  // Still return 200 to avoid retries
    });
});
```

2. Use background jobs:
```javascript
// Queue-based processing
app.post('/webhook', async (req, res) => {
  try {
    await queue.add('process-webhook', req.body, {
      attempts: 3,
      backoff: { type: 'exponential', delay: 1000 }
    });
    res.sendStatus(200);
  } catch (error) {
    res.sendStatus(500);
  }
});
```

3. Optimize database queries:
```sql
-- Add indexes for common queries
CREATE INDEX idx_events_created_at ON events(created_at);
CREATE INDEX idx_events_type ON events(event_type);
```

### High Failure Rate

**Symptoms**:
- Many failed delivery attempts in dashboard
- Subscription showing mostly 4xx/5xx responses
- Alerts for high failure rate

**Debug commands**:

```bash
# Get request attempts for a subscription
curl "$HOOK0_API/request_attempts/?application_id=$APP_ID&subscription_id={sub-id}" \
  -H "Authorization: Bearer $HOOK0_TOKEN"

# Count failed attempts (failed_at is set for failures)
curl "$HOOK0_API/request_attempts/?application_id=$APP_ID&subscription_id={sub-id}" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  | jq '[.[] | select(.failed_at != null)] | length'

# Get response details for failed attempts
# First get request_attempts, then fetch response details using response_id
curl "$HOOK0_API/request_attempts/?application_id=$APP_ID&subscription_id={sub-id}" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  | jq '.[] | select(.failed_at != null) | .response_id'

# Then for each response_id:
curl "$HOOK0_API/responses/{RESPONSE_ID}?application_id=$APP_ID" \
  -H "Authorization: Bearer $HOOK0_TOKEN"
```

**Solutions**:

1. Fix endpoint issues based on error codes:

```javascript
// Handle errors properly
app.post('/webhook', async (req, res) => {
  try {
    // Validate request
    if (!req.body.event_type) {
      return res.status(400).json({ error: 'event_type required' });
    }

    // Process event
    await processEvent(req.body);

    res.sendStatus(200);
  } catch (error) {
    console.error('Webhook processing error:', error);

    // Return appropriate status
    if (error.name === 'ValidationError') {
      res.status(400).json({ error: error.message });
    } else {
      res.status(500).json({ error: 'Internal server error' });
    }
  }
});
```

2. Monitor webhook endpoint health:

```bash
# Check endpoint availability
while true; do
  curl -o /dev/null -s -w "%{http_code}\n" https://your-webhook.com/webhook
  sleep 60
done
```

## Performance Issues

### Slow Event Delivery

**Symptoms**:
- Long delay between event creation and webhook delivery
- Dashboard shows delivery attempts created minutes after event
- Growing queue backlog

**Possible Causes (Self-Hosted)**:

1. **Output worker overloaded**

```bash
# Check worker logs
docker logs hook0-output-worker --tail 100

# Look for: High queue processing times

# Check worker resource usage
docker stats hook0-output-worker
```

**Solution**: Scale output workers:
```yaml
# docker-compose.yaml
services:
  output-worker:
    image: hook0/output-worker
    deploy:
      replicas: 3  # Run 3 workers
```

2. **Database performance issues**

```bash
# Check slow queries
docker exec postgres psql -U hook0 -d hook0 \
  -c "SELECT query, calls, mean_exec_time FROM pg_stat_statements ORDER BY mean_exec_time DESC LIMIT 10;"

# Check database size
docker exec postgres psql -U hook0 -d hook0 \
  -c "SELECT pg_size_pretty(pg_database_size('hook0'));"
```

**Solution**: Optimize database:
```sql
-- Vacuum and analyze
VACUUM ANALYZE;

-- Add missing indexes
CREATE INDEX CONCURRENTLY idx_event_created_at ON event.event(created_at);
CREATE INDEX CONCURRENTLY idx_request_attempt_event_id ON webhook.request_attempt(event__id);
```

3. **Too many retries for failed endpoints**

```bash
# List subscriptions for application
curl "$HOOK0_API/subscriptions/?application_id=$APP_ID" \
  -H "Authorization: Bearer $HOOK0_TOKEN"

# Check request attempts for each subscription to identify problematic ones
curl "$HOOK0_API/request_attempts/?application_id=$APP_ID&subscription_id={sub-id}" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  | jq '[.[] | select(.failed_at != null)] | length'
```

**Solution**: Disable problematic subscriptions (first fetch current config, then update with `is_enabled: false`):

:::warning PUT Requires ALL Fields
Remember: PUT replaces the entire subscription. Include all fields, not just `is_enabled`.
:::

```bash
# Get current subscription config
SUB=$(curl -s "$HOOK0_API/subscriptions/{sub-id}?application_id=$APP_ID" \
  -H "Authorization: Bearer $HOOK0_TOKEN")

# Disable it (all fields required)
curl -X PUT "$HOOK0_API/subscriptions/{sub-id}" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "'"$APP_ID"'",
    "is_enabled": false,
    "event_types": ["your.event.type"],
    "label_key": "environment",
    "label_value": "production",
    "target": {
      "type": "http",
      "method": "POST",
      "url": "https://your-webhook.com/webhook",
      "headers": {"Content-Type": "application/json"}
    }
  }'
```

### High Memory Usage (Self-Hosted)

**Symptoms**:
- Docker container using excessive memory
- OOMKilled in container logs
- System slowness

**Debug commands**:

```bash
# Check memory usage
docker stats --no-stream

# Check API memory
docker exec hook0-api ps aux | grep hook0-api

# Check for memory leaks in logs
docker logs hook0-api | grep -i "memory\|oom\|allocation"
```

**Solutions**:

1. Increase memory limits:
```yaml
# docker-compose.yaml
services:
  api:
    mem_limit: 2g
    mem_reservation: 1g
```

2. Configure connection pools:
```yaml
environment:
  DATABASE_MAX_CONNECTIONS: 20
```

3. Enable event cleanup:
```sql
-- Clean old events (adjust retention)
DELETE FROM event.event
WHERE created_at < NOW() - INTERVAL '90 days';

-- Clean old request attempts
DELETE FROM webhook.request_attempt
WHERE created_at < NOW() - INTERVAL '30 days';
```

### Rate Limiting

**Symptoms**:
- HTTP 429 Too Many Requests
- "Rate limit exceeded" errors
- Requests being throttled

**Debug commands**:

```bash
# Check rate limit headers
curl -I $HOOK0_API/events \
  -H "Authorization: Bearer $HOOK0_TOKEN"

# Look for:
# X-RateLimit-Limit: 1000
# X-RateLimit-Remaining: 0
# X-RateLimit-Reset: 1704294600
```

**Solutions**:

1. Implement retry with backoff:
```javascript
async function sendEventWithRetry(event, maxRetries = 3) {
  for (let i = 0; i < maxRetries; i++) {
    try {
      return await hook0.sendEvent(event);
    } catch (error) {
      if (error.status === 429) {
        const retryAfter = error.headers['retry-after'] || Math.pow(2, i);
        console.log(`Rate limited, waiting ${retryAfter}s`);
        await sleep(retryAfter * 1000);
      } else {
        throw error;
      }
    }
  }
  throw new Error('Max retries exceeded');
}
```

2. Batch events:
```javascript
// Send events in batches instead of one-by-one
const events = [...];  // Array of events
const batchSize = 100;

for (let i = 0; i < events.length; i += batchSize) {
  const batch = events.slice(i, i + batchSize);
  await Promise.all(batch.map(e => hook0.sendEvent(e)));
  await sleep(1000);  // Delay between batches
}
```

3. For self-hosted, adjust rate limits:
```yaml
environment:
  RATE_LIMIT_PER_MINUTE: 10000
  RATE_LIMIT_PER_HOUR: 100000
```

## Still Stuck?

If you're still experiencing issues after trying these solutions:

### Check System Status

```bash
# Cloud version
curl https://status.hook0.com

# Self-hosted: check API is responding
curl -s http://localhost:8081/api/v1/swagger.json | head -c 100

# Self-hosted health check (requires HEALTH_CHECK_KEY env var)
# curl "http://localhost:8081/api/v1/health/?key={HEALTH_CHECK_KEY}"
```

### Enable Debug Logging

```yaml
# docker-compose.yaml
environment:
  RUST_LOG: debug
  LOG_LEVEL: debug
```

View detailed logs:
```bash
docker logs -f hook0-api
docker logs -f hook0-output-worker
```

### Gather Diagnostic Information

Before contacting support, collect:

1. **Error details**:
```bash
# Recent errors from logs
docker logs hook0-api --since 1h | grep ERROR > errors.log
```

2. **System information**:
```bash
# Docker version and stats
docker --version
docker-compose --version
docker stats --no-stream > docker-stats.txt
```

3. **Configuration** (redact secrets):
```bash
# Docker compose config
docker-compose config > config.yml
```

4. **Request/response examples** (with headers):
```bash
curl -v $HOOK0_API/events \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  > request-response.log 2>&1
```

### Get Help

- **GitHub Issues**: [github.com/hook0/hook0/issues](https://github.com/hook0/hook0/issues)
- **Discord Community**: [discord.gg/hook0](https://discord.gg/hook0)
- **Documentation**: [docs.hook0.com](https://docs.hook0.com)
- **Email Support**: support@hook0.com (for cloud customers)

When reporting issues, include:
- Hook0 version (for self-hosted)
- Error messages and codes
- Steps to reproduce
- Relevant logs (redacted)
- System configuration

## Next Steps

- [Debugging Failed Webhooks](./debug-failed-webhooks.md) - Deep dive into webhook failures
- [Monitor Webhook Performance](./monitor-webhook-performance.md) - Performance monitoring
- [Error Codes Reference](../reference/error-codes.md) - Complete error code list
- [Security Model](../explanation/security-model.md) - Security best practices
