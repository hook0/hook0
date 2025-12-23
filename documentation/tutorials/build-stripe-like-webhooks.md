---
sidebar_position: 6
---

# Building a Stripe-Like Webhook System

This comprehensive tutorial guides you through building a production-ready webhook system similar to Stripe's, using Hook0. You'll learn how to implement event types, webhook receivers with signature verification, multi-tenancy, and monitoring.

**Time**: 45 minutes
**Level**: Intermediate
**Prerequisites**: Docker, Node.js 18+, curl, basic webhook knowledge

## What You'll Build

A complete webhook infrastructure featuring:
- Payment event types (`payment.charge.succeeded`, `payment.charge.failed`)
- Secure webhook receiver with signature verification
- Multi-tenant event routing using labels
- Webhook testing and debugging tools
- Dashboard monitoring

**Final architecture**:
```
Payment Service → Hook0 → Customer Webhook Endpoints
                    ↓
              (retries, signatures, monitoring)
```

## Part 1: Environment Setup

### Step 1.1: Start Hook0 with Docker Compose


Clone and start Hook0:

```bash
git clone https://github.com/hook0/hook0.git
cd hook0
docker compose up -d

# Wait for services to be ready (first build takes 10-15 min)
docker compose logs -f api
# Wait until you see "Listening on 0.0.0.0:8081"

# Verify API is running
# Use http://localhost:8081 for self-hosted or https://app.hook0.com for cloud
curl http://localhost:8081/api/v1/swagger.json | head -1
# Expected: {"openapi":"3.0.3"...
```

### Step 1.2: Create Application and Get Token

### Create Organization and Application via Dashboard

1. **Access the Hook0 Dashboard** at http://localhost:8001 (self-hosted) or https://app.hook0.com (cloud)

2. **Create an Organization**: Click "Create Organization" and fill in:
   - Name: `Payment Processor Inc`
   - Description: `Tutorial organization`

3. **Create an Application**: Within your organization, create a new application:
   - Name: `Payment API`
   - Description: `Handles payment webhooks`

4. **Generate a Service Token**: Go to Organization Settings → Service Tokens → Create Token
   - Name: `Payment API Token`
   - Copy the generated token immediately (it won't be shown again)

:::tip Self-Hosted Email Verification
For self-hosted instances, check Mailpit at http://localhost:8025 to access verification emails after registration.
:::

### Set Up Environment Variables

```bash
# Set your service token (from dashboard)
export HOOK0_TOKEN="YOUR_TOKEN_HERE"
export HOOK0_API="https://app.hook0.com/api/v1" # Replace by your domain (or http://localhost:8081/api/v1 locally)

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

## Part 2: Define Payment Event Types

### Step 2.1: Create Event Types

Create event types for payment lifecycle:

```bash
# Load environment
source .env

# 1. Payment charge succeeded
curl -X POST "$HOOK0_API/event_types" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"application_id\": \"'"$APP_ID"'\",
    \"service\": \"payments\",
    \"resource_type\": \"charge\",
    \"verb\": \"succeeded\"
  }"

# 2. Payment charge failed
curl -X POST "$HOOK0_API/event_types" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"application_id\": \"'"$APP_ID"'\",
    \"service\": \"payments\",
    \"resource_type\": \"charge\",
    \"verb\": \"failed\"
  }"

# 3. Payment refund created
curl -X POST "$HOOK0_API/event_types" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"application_id\": \"'"$APP_ID"'\",
    \"service\": \"payments\",
    \"resource_type\": \"refund\",
    \"verb\": \"created\"
  }"

# 4. Customer created
curl -X POST "$HOOK0_API/event_types" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"application_id\": \"'"$APP_ID"'\",
    \"service\": \"customers\",
    \"resource_type\": \"account\",
    \"verb\": \"created\"
  }"
```

### Step 2.2: Verify Event Types

```bash
# List all event types
curl "$HOOK0_API/event_types?application_id=$APP_ID" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  | jq '.[] | {event_type_name, service_name, resource_type_name, verb_name}'
```

Expected output:
```json
{
  "event_type_name": "payment.charge.succeeded",
  "service_name": "payment",
  "resource_type_name": "charge",
  "verb_name": "succeeded"
}
{
  "event_type_name": "payment.charge.failed",
  "service_name": "payment",
  "resource_type_name": "charge",
  "verb_name": "failed"
}
{
  "event_type_name": "payment.refund.created",
  "service_name": "payment",
  "resource_type_name": "refund",
  "verb_name": "created"
}
{
  "event_type_name": "customer.account.created",
  "service_name": "customer",
  "resource_type_name": "account",
  "verb_name": "created"
}
```

## Part 3: Build Webhook Receiver

### Step 3.1: Create Express.js Webhook Server

Create `webhook-receiver/package.json`:

```json
{
  "name": "hook0-webhook-receiver",
  "version": "1.0.0",
  "type": "module",
  "dependencies": {
    "express": "^4.18.2",
    "uuid": "^9.0.0"
  },
  "scripts": {
    "start": "node server.js"
  }
}
```

Install dependencies:
```bash
mkdir webhook-receiver
cd webhook-receiver
npm install
```

### Step 3.2: Implement Webhook Handler with Signature Verification

Create `webhook-receiver/server.js`:

```javascript
import express from 'express';
import crypto from 'crypto';

const app = express();
const processedEvents = new Set();

const WEBHOOK_SECRETS = {
  'acme_corp': process.env.WEBHOOK_SECRET_ACME || 'secret_acme_123',
  'globex_inc': process.env.WEBHOOK_SECRET_GLOBEX || 'secret_globex_456'
};

// See webhook-authentication tutorial for full signature verification details
function verifySignature(rawBody, signature, headers, secret) {
  const parts = Object.fromEntries(signature.split(',').map(p => p.split('=')));
  const headerNames = parts.h ? parts.h.split(' ') : [];
  const headerValues = headerNames.map(h => headers[h] || '').join('.');
  // Use raw body string directly, not JSON.stringify(parsedBody)
  const signedData = parts.h
    ? `${parts.t}.${parts.h}.${headerValues}.${rawBody}`
    : `${parts.t}.${rawBody}`;
  const expected = crypto.createHmac('sha256', secret).update(signedData).digest('hex');
  return parts.v1 === expected;
}

// Capture raw body for signature verification
app.use('/webhook/:tenant', express.json({
  verify: (req, res, buf) => { req.rawBody = buf; }
}));

app.post('/webhook/:tenant', (req, res) => {
  const tenant = req.params.tenant;
  const signature = req.headers['x-hook0-signature'];
  const secret = WEBHOOK_SECRETS[tenant];
  const rawBodyString = req.rawBody.toString('utf8');

  if (!secret) return res.status(404).json({ error: 'Tenant not found' });

  if (!verifySignature(rawBodyString, signature, req.headers, secret)) {
    return res.status(401).json({ error: 'Invalid signature' });
  }

  const event = req.body;

  // Idempotency check
  if (processedEvents.has(event.event_id)) {
    return res.json({ status: 'already_processed' });
  }
  processedEvents.add(event.event_id);

  // Parse payload (can be string or object)
  const payload = typeof event.payload === 'string' ? JSON.parse(event.payload) : event.payload;

  console.log(`[${tenant}] ${event.event_type}:`, payload);
  res.json({ status: 'ok', event_id: event.event_id });
});

app.get('/health', (req, res) => res.json({ status: 'ok' }));

app.listen(3000, () => console.log('Webhook receiver on http://localhost:3000'));
```

:::tip Signature Verification
For detailed explanation of signature verification including timestamp validation and secret rotation, see [Implementing Webhook Authentication](./webhook-authentication.md).
:::

### Step 3.3: Start Webhook Receiver

```bash
# In webhook-receiver directory
node server.js
```

Output:
```
🚀 Webhook receiver running on http://localhost:3000
📝 Configured tenants: acme_corp, globex_inc
✓ Signature verification enabled
✓ Idempotency protection enabled
```

Test health endpoint:
```bash
curl http://localhost:3000/health
```

## Part 4: Create Subscriptions

### Step 4.1: Create Subscription for ACME Corp

```bash
# Load environment
source .env

# Create subscription
SUB_ACME=$(curl -s -X POST "$HOOK0_API/subscriptions" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"application_id\": \"'"$APP_ID"'\",
    \"is_enabled\": true,
    \"description\": \"ACME Corp payment webhooks\",
    \"event_types\": [
      \"payment.charge.succeeded\",
      \"payment.charge.failed\",
      \"payment.refund.created\"
    ],
    \"labels\": {
      \"tenant_id\": \"acme_corp\"
    },
    \"target\": {
      \"type\": \"http\",
      \"method\": \"POST\",
      \"url\": \"http://host.docker.internal:3000/webhook/acme_corp\",
      \"headers\": {
        \"X-Tenant\": \"acme_corp\"
      }
    }
  }")

# Extract subscription ID and secret
export SUB_ACME_ID=$(echo $SUB_ACME | jq -r '.subscription_id')
export SUB_ACME_SECRET=$(echo $SUB_ACME | jq -r '.secret')

echo "ACME Subscription ID: $SUB_ACME_ID"
echo "ACME Secret: $SUB_ACME_SECRET"
```

:::info
`host.docker.internal` allows Docker containers to access host machine. For production, use public URLs.
:::

### Step 4.2: Update Webhook Receiver with Real Secret

Update `server.js` to use the real subscription secret:

```javascript
const WEBHOOK_SECRETS = {
  'acme_corp': process.env.SUB_ACME_SECRET || 'secret_acme_123',
  'globex_inc': process.env.SUB_GLOBEX_SECRET || 'secret_globex_456'
};
```

Restart with real secret:
```bash
export SUB_ACME_SECRET="<secret_from_above>"
cd webhook-receiver
node server.js
```

### Step 4.3: Create Subscription for Globex Inc

```bash
source .env

SUB_GLOBEX=$(curl -s -X POST "$HOOK0_API/subscriptions" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"application_id\": \"'"$APP_ID"'\",
    \"is_enabled\": true,
    \"description\": \"Globex Inc payment webhooks\",
    \"event_types\": [
      \"payment.charge.succeeded\",
      \"customer.account.created\"
    ],
    \"labels\": {
      \"tenant_id\": \"globex_inc\"
    },
    \"target\": {
      \"type\": \"http\",
      \"method\": \"POST\",
      \"url\": \"http://host.docker.internal:3000/webhook/globex_inc\",
      \"headers\": {
        \"X-Tenant\": \"globex_inc\"
      }
    }
  }")

export SUB_GLOBEX_ID=$(echo $SUB_GLOBEX | jq -r '.subscription_id')
export SUB_GLOBEX_SECRET=$(echo $SUB_GLOBEX | jq -r '.secret')

echo "Globex Subscription ID: $SUB_GLOBEX_ID"
echo "Globex Secret: $SUB_GLOBEX_SECRET"
```

## Part 5: Send Test Events

### Step 5.1: Send Payment Success Event (ACME Corp)

```bash
source .env

curl -X POST "$HOOK0_API/event" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"application_id\": \"'"$APP_ID"'\",
    \"event_id\": \"$(uuidgen)\",
    \"event_type\": \"payment.charge.succeeded\",
    \"payload\": \"{\\\"charge_id\\\":\\\"ch_123\\\",\\\"amount\\\":4999,\\\"currency\\\":\\\"USD\\\",\\\"customer_id\\\":\\\"cus_acme_001\\\",\\\"description\\\":\\\"Pro plan subscription\\\"}\",
    \"payload_content_type\": \"application/json\",
    \"occurred_at\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\",
    \"labels\": {
      \"tenant_id\": \"acme_corp\",
      \"environment\": \"production\",
      \"priority\": \"high\"
    }
  }"
```

**Check webhook receiver logs**:
```
[req_xxx] Received webhook for tenant: acme_corp
[req_xxx] Signature verified ✓
[req_xxx] 💰 Payment succeeded: {
  charge_id: 'ch_123',
  amount: 4999,
  currency: 'USD',
  customer: 'cus_acme_001'
}
[req_xxx] Event processed successfully in 15ms
```

### Step 5.2: Send Payment Failed Event (ACME Corp)

```bash
curl -X POST "$HOOK0_API/event" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"application_id\": \"'"$APP_ID"'\",
    \"event_id\": \"$(uuidgen)\",
    \"event_type\": \"payment.charge.failed\",
    \"payload\": \"{\\\"charge_id\\\":\\\"ch_124\\\",\\\"amount\\\":4999,\\\"currency\\\":\\\"USD\\\",\\\"customer_id\\\":\\\"cus_acme_001\\\",\\\"error_code\\\":\\\"card_declined\\\",\\\"error_message\\\":\\\"Insufficient funds\\\"}\",
    \"payload_content_type\": \"application/json\",
    \"occurred_at\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\",
    \"labels\": {
      \"tenant_id\": \"acme_corp\",
      \"environment\": \"production\",
      \"priority\": \"critical\"
    }
  }"
```

### Step 5.3: Send Customer Created Event (Globex Inc)

```bash
curl -X POST "$HOOK0_API/event" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"application_id\": \"'"$APP_ID"'\",
    \"event_id\": \"$(uuidgen)\",
    \"event_type\": \"customer.account.created\",
    \"payload\": \"{\\\"customer_id\\\":\\\"cus_globex_001\\\",\\\"email\\\":\\\"customer@globex.com\\\",\\\"name\\\":\\\"John Doe\\\",\\\"plan\\\":\\\"enterprise\\\"}\",
    \"payload_content_type\": \"application/json\",
    \"occurred_at\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\",
    \"labels\": {
      \"tenant_id\": \"globex_inc\",
      \"environment\": \"production\"
    }
  }"
```

### Step 5.4: Test Label Filtering

Send event with wrong tenant label (should not trigger ACME subscription):

```bash
curl -X POST "$HOOK0_API/event" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"application_id\": \"'"$APP_ID"'\",
    \"event_id\": \"$(uuidgen)\",
    \"event_type\": \"payment.charge.succeeded\",
    \"payload\": \"{\\\"charge_id\\\":\\\"ch_999\\\"}\",
    \"payload_content_type\": \"application/json\",
    \"occurred_at\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\",
    \"labels\": {
      \"tenant_id\": \"unknown_tenant\"
    }
  }"
```

No webhook should be triggered (no subscription matches label).

## Part 6: Verify in Dashboard

### Step 6.1: Check Event Delivery

Query events via API:

```bash
# List recent events
curl "$HOOK0_API/events?application_id=$APP_ID&limit=10" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  | jq '.[] | {event_id, event_type, labels, created_at}'
```

### Step 6.2: Check Delivery Attempts

```bash
# Get request attempts for subscription
curl "$HOOK0_API/subscriptions/$SUB_ACME_ID/request_attempts?limit=10" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  | jq '.[] | {
      event_type,
      status_code,
      duration_ms,
      created_at,
      next_retry_at
    }'
```

Expected output:
```json
{
  "event_type": "payment.charge.succeeded",
  "status_code": 200,
  "duration_ms": 15,
  "created_at": "2025-12-10T12:00:00Z",
  "next_retry_at": null
}
```

### Step 6.3: Test Signature Verification Failure

Temporarily break signature verification to see failure:

```javascript
// In server.js, comment out signature verification
// if (!verifySignature(rawBody, signature, secret)) {
//   return res.status(401).json({ error: 'Invalid signature' });
// }
```

Send event and observe 401 response in request attempts.

## Part 7: Add Multi-Tenant Support

### Step 7.1: Create Test Script with Multiple Tenants

Create `test-events.js`:

```javascript
import fetch from 'node-fetch';
import { v4 as uuidv4 } from 'uuid';

const HOOK0_TOKEN = process.env.HOOK0_TOKEN;
const HOOK0_API = process.env.HOOK0_API;
const APP_ID = process.env.APP_ID;

const tenants = ['acme_corp', 'globex_inc', 'wayne_enterprises'];

async function sendEvent(eventType, payload, tenantId) {
  const event = {
    application_id: APP_ID,
    event_id: uuidv4(),
    event_type: eventType,
    payload: JSON.stringify(payload),
    payload_content_type: 'application/json',
    occurred_at: new Date().toISOString(),
    labels: {
      tenant_id: tenantId,
      environment: 'production'
    }
  };

  const response = await fetch(`${HOOK0_API}/event`, {
    method: 'POST',
    headers: {
      'Authorization': `Bearer ${HOOK0_TOKEN}`,
      'Content-Type': 'application/json'
    },
    body: JSON.stringify(event)
  });

  if (!response.ok) {
    throw new Error(`Failed to send event: ${response.statusText}`);
  }

  console.log(`✓ Sent ${eventType} for ${tenantId}`);
  return response.json();
}

async function simulatePayments() {
  console.log('🚀 Simulating multi-tenant payment events...\n');

  for (const tenant of tenants) {
    // Successful payment
    await sendEvent('payment.charge.succeeded', {
      charge_id: `ch_${uuidv4()}`,
      amount: Math.floor(Math.random() * 10000) + 1000,
      currency: 'USD',
      customer_id: `cus_${tenant}_${Math.floor(Math.random() * 1000)}`
    }, tenant);

    await sleep(100);

    // Failed payment (10% chance)
    if (Math.random() < 0.1) {
      await sendEvent('payment.charge.failed', {
        charge_id: `ch_${uuidv4()}`,
        amount: 4999,
        currency: 'USD',
        customer_id: `cus_${tenant}_${Math.floor(Math.random() * 1000)}`,
        error_code: 'card_declined',
        error_message: 'Insufficient funds'
      }, tenant);
    }

    await sleep(100);
  }

  console.log('\n✨ Simulation complete!');
}

function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

// Run simulation
simulatePayments().catch(console.error);
```

Run simulation:
```bash
npm install node-fetch
node test-events.js
```

### Step 7.2: Create Webhook Router (Advanced)

For production with many tenants, create a router:

```javascript
// webhook-router.js
import express from 'express';

const app = express();
const PORT = 4000;

// Tenant configuration (from database in production)
const TENANT_CONFIG = {
  'acme_corp': {
    webhookUrl: 'https://acme.example.com/webhooks/hook0',
    authToken: 'acme_token_123',
    retryPolicy: { maxRetries: 3, backoff: 'exponential' }
  },
  'globex_inc': {
    webhookUrl: 'https://globex.example.com/api/webhooks',
    authToken: 'globex_token_456',
    retryPolicy: { maxRetries: 5, backoff: 'exponential' }
  }
};

app.use(express.json());

app.post('/webhook/:tenant', async (req, res) => {
  const tenant = req.params.tenant;
  const event = req.body;

  const config = TENANT_CONFIG[tenant];
  if (!config) {
    return res.status(404).json({ error: 'Tenant not found' });
  }

  try {
    // Forward to tenant's webhook endpoint
    const response = await fetch(config.webhookUrl, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${config.authToken}`,
        'X-Forwarded-Event-Id': event.event_id
      },
      body: JSON.stringify(event)
    });

    if (!response.ok) {
      throw new Error(`Tenant webhook returned ${response.status}`);
    }

    res.status(200).json({ status: 'forwarded' });
  } catch (error) {
    console.error(`Failed to forward to ${tenant}:`, error);
    res.status(500).json({ error: 'Forward failed' });
  }
});

app.listen(PORT, () => {
  console.log(`📡 Webhook router running on port ${PORT}`);
});
```

## Part 8: Troubleshooting

### Common Issues

#### Events Not Delivered

**Check subscription is enabled**:
```bash
curl "$HOOK0_API/subscriptions/$SUB_ACME_ID" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  | jq '.is_enabled'
```

**Check label matching**:
```bash
# Event labels
curl "$HOOK0_API/events/{event-id}" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  | jq '.labels'

# Subscription filter
curl "$HOOK0_API/subscriptions/$SUB_ACME_ID" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  | jq '.labels'
```

#### Signature Verification Failing

See [Debugging Failed Webhooks](../how-to-guides/debug-failed-webhooks.md#scenario-3-webhook-signature-verification-failures) for debugging signature issues.

#### Webhook Endpoint Timeout

**Respond quickly**:
```javascript
app.post('/webhook/:tenant', async (req, res) => {
  // Respond immediately
  res.status(200).json({ status: 'received' });

  // Process asynchronously
  processWebhookAsync(req.body).catch(console.error);
});
```

### Monitoring

**Track delivery success rate**:
```bash
curl "$HOOK0_API/subscriptions/$SUB_ACME_ID/request_attempts?limit=100" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  | jq '[.[] | select(.status_code >= 200 and .status_code < 300)] | length / 100'
```

**Find slow deliveries**:
```bash
curl "$HOOK0_API/subscriptions/$SUB_ACME_ID/request_attempts?limit=100" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  | jq '.[] | select(.duration_ms > 1000) | {event_type, duration_ms}'
```

## What You've Learned

✅ Set up Hook0 with Docker Compose
✅ Created payment event types (Stripe-like)
✅ Built secure webhook receiver with signature verification
✅ Implemented multi-tenant routing with labels
✅ Created and tested subscriptions
✅ Verified webhook deliveries
✅ Debugged common issues

## Next Steps

- **Advanced authentication**: [Security Model](../explanation/security-model.md)
- **Monitoring**: [Monitor Webhook Performance](../how-to-guides/monitor-webhook-performance.md)

## Production Checklist

Before going to production:

- [ ] Use real authentication tokens (not master key)
- [ ] Store webhook secrets securely (env vars, secrets manager)
- [ ] Implement idempotency with persistent storage (PostgreSQL)
- [ ] Add structured logging with request IDs
- [ ] Set up monitoring and alerting
- [ ] Configure retry policies per tenant
- [ ] Enable TLS for webhook endpoints
- [ ] Implement rate limiting on webhook receiver
- [ ] Set up database backups
- [ ] Document webhook integration for customers
- [ ] Test failure scenarios (network errors, timeouts)
- [ ] Configure CORS if needed

## Resources

- [Hook0 API Reference](/openapi/intro)
- [Webhook Best Practices](../how-to-guides/secure-webhook-endpoints.md)
- [GitHub Repository](https://github.com/hook0/hook0)
- [Example Code](https://github.com/hook0/examples)
