# Setting up Event Types and Subscriptions

This tutorial teaches you how to design effective event schemas, create organized event type hierarchies, and configure sophisticated subscription patterns for complex applications.

## Prerequisites

- Completed [Getting Started](./getting-started.md) tutorial
- Understanding of JSON schemas
- Familiarity with webhook concepts
- **Node.js 18+** (for the testing script)

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

## Event Type Design Principles

### Naming Conventions

Use the `service.resource_type.verb` format:
```
service.resource_type.verb
user.account.created
user.account.updated
user.account.deleted
order.purchase.created
order.payment.completed
order.shipment.shipped
```

### Hierarchical Organization
```
user.*            → All user events
user.account.*    → User account events
user.profile.*    → User profile events
order.*           → All order events
order.payment.*   → Payment-related events
```

## Step 1: Design Your Event Schema

Before creating event types, plan your event structure.

:::info Schema Documentation
These schemas are for your internal documentation. Hook0 does not validate payload structure - it forwards the payload as-is to webhook endpoints. It's your responsibility to ensure payload consistency.
:::

### User Events Schema
```json
{
  "user.account.created": {
    "user_id": "string (required)",
    "email": "string (required)",
    "name": "string (required)",
    "plan": "string (optional)",
    "created_at": "ISO 8601 timestamp"
  },
  "user.account.updated": {
    "user_id": "string (required)",
    "changes": ["array of changed fields"],
    "updated_at": "ISO 8601 timestamp"
  }
}
```

### Order Events Schema
```json
{
  "order.purchase.created": {
    "order_id": "string (required)",
    "customer_id": "string (required)",
    "items": "array of order items",
    "total_amount": "number",
    "currency": "string",
    "created_at": "ISO 8601 timestamp"
  },
  "order.payment.completed": {
    "order_id": "string (required)",
    "payment_id": "string (required)",
    "amount": "number",
    "currency": "string",
    "payment_method": "string",
    "processed_at": "ISO 8601 timestamp"
  }
}
```

## Step 2: Create Event Types with Validation

### Basic Event Type Creation

```bash
curl -X POST "$HOOK0_API/event_types" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "'"$APP_ID"'",
    "service": "user",
    "resource_type": "account",
    "verb": "created"
  }'
```

**Response:**
```json
{
  "service_name": "user",
  "resource_type_name": "account",
  "verb_name": "created",
  "event_type_name": "user.account.created"
}
```


## Step 3: Create a Complete Event Type Hierarchy

Let's create a comprehensive set of event types for a SaaS application:

### User Lifecycle Events

```bash
# User registration
curl -X POST "$HOOK0_API/event_types" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "'"$APP_ID"'",
    "service": "user",
    "resource_type": "account",
    "verb": "registered"
  }'

# Email verification
curl -X POST "$HOOK0_API/event_types" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "'"$APP_ID"'",
    "service": "user",
    "resource_type": "email",
    "verb": "verified"
  }'

# Profile updates
curl -X POST "$HOOK0_API/event_types" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "'"$APP_ID"'",
    "service": "user",
    "resource_type": "profile",
    "verb": "updated"
  }'

# Account deactivation
curl -X POST "$HOOK0_API/event_types" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "'"$APP_ID"'",
    "service": "user",
    "resource_type": "account",
    "verb": "deactivated"
  }'
```

### Subscription Events

```bash
# Subscription created
curl -X POST "$HOOK0_API/event_types" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "'"$APP_ID"'",
    "service": "subscription",
    "resource_type": "plan",
    "verb": "created"
  }'

# Subscription upgraded
curl -X POST "$HOOK0_API/event_types" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "'"$APP_ID"'",
    "service": "subscription",
    "resource_type": "plan",
    "verb": "upgraded"
  }'

# Payment failed
curl -X POST "$HOOK0_API/event_types" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "'"$APP_ID"'",
    "service": "subscription",
    "resource_type": "payment",
    "verb": "failed"
  }'

# Subscription cancelled
curl -X POST "$HOOK0_API/event_types" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "'"$APP_ID"'",
    "service": "subscription",
    "resource_type": "plan",
    "verb": "cancelled"
  }'
```

## Step 4: Advanced Subscription Patterns

### Single Event Type Subscription

Simple subscription for one specific event:

```bash
curl -X POST "$HOOK0_API/subscriptions" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "'"$APP_ID"'",
    "is_enabled": true,
    "event_types": ["user.account.registered"],
    "description": "Welcome email trigger",
    "labels": {
      "environment": "production"
    },
    "target": {
      "type": "http",
      "method": "POST",
      "url": "https://email-service.com/webhooks/welcome",
      "headers": {
        "Content-Type": "application/json",
        "X-Service": "welcome-emails"
      }
    }
  }'
```

**Response:**
```json
{
  "application_id": "{APP_ID}",
  "subscription_id": "{{SUBSCRIPTION_ID}}",
  "is_enabled": true,
  "event_types": ["user.account.registered"],
  "description": "Welcome email trigger",
  "secret": "{SECRET}",
  "labels": {
    "environment": "production"
  },
  "target": {
    "type": "http",
    "method": "POST",
    "url": "https://email-service.com/webhooks/welcome",
    "headers": {
      "content-type": "application/json",
      "x-service": "welcome-emails"
    }
  },
  "created_at": "2025-12-12T10:26:24.044337Z"
}
```

### Multiple Event Type Subscription

Subscribe to multiple related events:

```bash
curl -X POST "$HOOK0_API/subscriptions" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "'"$APP_ID"'",
    "is_enabled": true,
    "event_types": [
      "user.account.registered",
      "user.email.verified",
      "user.profile.updated"
    ],
    "description": "CRM system user sync",
    "labels": {
      "environment": "production"
    },
    "target": {
      "type": "http",
      "method": "POST",
      "url": "https://crm.company.com/api/webhooks/users",
      "headers": {
        "Content-Type": "application/json",
        "Authorization": "Bearer crm-api-key"
      }
    }
  }'
```

### Pattern-Based Subscription (All User Events)

Subscribe to all events matching a pattern:

```bash
curl -X POST "$HOOK0_API/subscriptions" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "'"$APP_ID"'",
    "is_enabled": true,
    "event_types": [
      "user.account.registered",
      "user.email.verified",
      "user.profile.updated",
      "user.account.deactivated"
    ],
    "description": "Analytics tracking for all user events",
    "labels": {
      "environment": "production"
    },
    "target": {
      "type": "http",
      "method": "POST",
      "url": "https://analytics.company.com/api/events",
      "headers": {
        "Content-Type": "application/json",
        "X-Analytics-Source": "user-events"
      }
    }
  }'
```

### Critical Events Subscription

High-priority events with special handling:

```bash
curl -X POST "$HOOK0_API/subscriptions" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "'"$APP_ID"'",
    "is_enabled": true,
    "event_types": [
      "subscription.payment.failed",
      "user.account.deactivated"
    ],
    "description": "Alert system for critical events",
    "labels": {
      "environment": "production"
    },
    "target": {
      "type": "http",
      "method": "POST",
      "url": "https://alerts.company.com/api/critical",
      "headers": {
        "Content-Type": "application/json",
        "X-Priority": "high",
        "X-Alert-Channel": "slack"
      }
    }
  }'
```

## Step 5: Advanced Subscription Configuration

### Custom Headers and Authentication

```bash
curl -X POST "$HOOK0_API/subscriptions" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "'"$APP_ID"'",
    "is_enabled": true,
    "event_types": ["subscription.plan.created"],
    "description": "Billing system integration",
    "labels": {
      "environment": "production"
    },
    "target": {
      "type": "http",
      "method": "POST",
      "url": "https://billing.company.com/webhooks/subscriptions",
      "headers": {
        "Content-Type": "application/json",
        "Authorization": "Bearer billing-api-token",
        "X-Webhook-Source": "hook0",
        "X-Environment": "production"
      }
    },
    "metadata": {
      "service": "billing",
      "priority": "high",
      "team": "finance"
    }
  }'
```

### Subscription with Metadata

Add metadata to organize subscriptions in the dashboard:

:::info Metadata Purpose
The `metadata` field is for your internal organization only. It is stored with the subscription and returned in API responses, but is **not** sent in webhook payloads. Use it to tag subscriptions by team, service, priority, etc.
:::

```bash
curl -X POST "$HOOK0_API/subscriptions" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "'"$APP_ID"'",
    "is_enabled": true,
    "event_types": ["user.account.registered", "user.account.deactivated"],
    "description": "Marketing automation system",
    "labels": {
      "department": "marketing"
    },
    "target": {
      "type": "http",
      "method": "POST",
      "url": "https://marketing.company.com/api/webhooks",
      "headers": {
        "Content-Type": "application/json"
      }
    },
    "metadata": {
      "system": "automation",
      "criticality": "medium"
    }
  }'
```

## Step 6: Test Your Event Types

Create a testing script to validate your event types:

```javascript
// test-events.js (requires Node.js 18+)
const { randomUUID } = require('crypto');

const HOOK0_TOKEN = '{YOUR_TOKEN}';
const HOOK0_API = 'https://app.hook0.com/api/v1/event';
const APPLICATION_ID = '{APP_ID}';

async function sendTestEvent(eventType, payload) {
  const response = await fetch(HOOK0_API, {
    method: 'POST',
    headers: {
      'Authorization': `Bearer ${HOOK0_TOKEN}`,
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({
      application_id: APPLICATION_ID,
      event_id: randomUUID(),
      event_type: eventType,
      payload: JSON.stringify(payload),
      payload_content_type: 'application/json',
      occurred_at: new Date().toISOString(),
      labels: {
        environment: 'test',
        source: 'event-testing'
      }
    })
  });

  if (!response.ok) {
    const error = await response.text();
    console.error(`❌ ${eventType}: ${response.status} - ${error}`);
    return null;
  }

  const result = await response.json();
  console.log(`✅ ${eventType}:`, result.event_id);
  return result;
}

async function runTests() {
  console.log('🧪 Testing Event Types...\n');

  // Test user events (event types follow service.resource_type.verb format)
  await sendTestEvent('user.account.registered', {
    user_id: 'user_123',
    email: 'test@example.com',
    name: 'Test User',
    plan: 'free',
    created_at: new Date().toISOString(),
    metadata: {
      source: 'web',
      campaign: 'social_media'
    }
  });

  await sendTestEvent('user.email.verified', {
    user_id: 'user_123',
    email: 'test@example.com',
    verified_at: new Date().toISOString()
  });

  await sendTestEvent('user.profile.updated', {
    user_id: 'user_123',
    changes: ['name', 'avatar'],
    previous_values: {
      name: 'Old Name',
      avatar: null
    },
    updated_at: new Date().toISOString()
  });

  // Test subscription events
  await sendTestEvent('subscription.plan.created', {
    subscription_id: 'sub_123',
    user_id: 'user_123',
    plan: 'pro',
    amount: 29.99,
    currency: 'USD',
    billing_cycle: 'monthly',
    created_at: new Date().toISOString()
  });

  await sendTestEvent('subscription.payment.failed', {
    subscription_id: 'sub_123',
    user_id: 'user_123',
    payment_id: 'pay_123',
    amount: 29.99,
    currency: 'USD',
    error_code: 'card_declined',
    error_message: 'Insufficient funds',
    failed_at: new Date().toISOString()
  });

  console.log('\n✨ Test completed!');
}

runTests().catch(console.error);
```

Run the tests:
```bash
node test-events.js
```

## Step 7: Monitor Event Types and Subscriptions

### List All Event Types
```bash
curl "$HOOK0_API/event_types/?application_id=$APP_ID" \
  -H "Authorization: Bearer $HOOK0_TOKEN"
```

**Response:**
```json
[
  {
    "service_name": "user",
    "resource_type_name": "account",
    "verb_name": "registered",
    "event_type_name": "user.account.registered"
  },
  {
    "service_name": "user",
    "resource_type_name": "email",
    "verb_name": "verified",
    "event_type_name": "user.email.verified"
  },
  {
    "service_name": "subscription",
    "resource_type_name": "plan",
    "verb_name": "created",
    "event_type_name": "subscription.plan.created"
  }
]
```

### List All Subscriptions
```bash
curl "$HOOK0_API/subscriptions/?application_id=$APP_ID" \
  -H "Authorization: Bearer $HOOK0_TOKEN"
```

**Response:**
```json
[
  {
    "application_id": "{APP_ID}",
    "subscription_id": "{SUBSCRIPTION_ID}",
    "is_enabled": true,
    "event_types": ["user.account.registered"],
    "description": "Welcome email trigger",
    "secret": "591b414d-8cd4-...",
    "metadata": {},
    "labels": {
      "environment": "production"
    },
    "labels": {
      "environment": "production"
    },
    "target": {
      "type": "http",
      "method": "POST",
      "url": "https://email-service.com/webhooks/welcome",
      "headers": {
        "content-type": "application/json",
        "x-service": "welcome-emails"
      }
    },
    "created_at": "2025-12-12T10:26:24.044337Z",
    "dedicated_workers": []
  }
]
```

### Get Subscription Details
```bash
curl "$HOOK0_API/subscriptions/{SUBSCRIPTION_ID}?application_id=$APP_ID" \
  -H "Authorization: Bearer $HOOK0_TOKEN"
```

**Response:**
```json
{
  "application_id": "{APP_ID}",
  "subscription_id": "{SUBSCRIPTION_ID}",
  "is_enabled": true,
  "event_types": ["user.account.registered"],
  "description": "Welcome email trigger",
  "secret": "591b414d-8cd4-...",
  "metadata": {},
  "labels": {
    "environment": "production"
  },
  "target": {
    "type": "http",
    "method": "POST",
    "url": "https://email-service.com/webhooks/welcome",
    "headers": {
      "content-type": "application/json",
      "x-service": "welcome-emails"
    }
  },
  "created_at": "2025-12-12T10:26:24.044337Z",
  "dedicated_workers": []
}
```

## Step 8: Update and Manage Subscriptions

### Update Subscription Configuration

:::warning Required Fields
The PUT endpoint requires all subscription fields, not just the ones you want to change. You must include `application_id`, `is_enabled`, `event_types`, `labels`, and `target`.
:::

```bash
curl -X PUT "$HOOK0_API/subscriptions/{SUBSCRIPTION_ID}" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "'"$APP_ID"'",
    "is_enabled": true,
    "event_types": ["user.account.registered", "user.email.verified"],
    "description": "Updated CRM integration - removed profile updates",
    "labels": {
      "environment": "production"
    },
    "target": {
      "type": "http",
      "method": "POST",
      "url": "https://new-crm.company.com/api/webhooks/users",
      "headers": {
        "Content-Type": "application/json",
        "Authorization": "Bearer new-crm-api-key"
      }
    }
  }'
```

## Step 9: Delete Event Types and Subscriptions

### Delete Event Type

To permanently delete an event type:

```bash
curl -X DELETE "$HOOK0_API/event_types/{EVENT_TYPE_NAME}?application_id=$APP_ID" \
  -H "Authorization: Bearer $HOOK0_TOKEN"
```

:::warning Permanent Deletion
This permanently deletes the event type. Existing events are preserved, but you won't be able to send new events of this type. This action cannot be undone - you'll need to recreate the event type if needed.
:::

### Disable Subscription

To disable a subscription, set `is_enabled` to `false`:

```bash
curl -X PUT "$HOOK0_API/subscriptions/{SUBSCRIPTION_ID}" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "'"$APP_ID"'",
    "is_enabled": false,
    "event_types": ["user.account.created"],
    "labels": {
      "environment": "production"
    },
    "target": {
      "type": "http",
      "method": "POST",
      "url": "https://your-endpoint.com/webhook",
      "headers": {
        "Content-Type": "application/json"
      }
    }
  }'
```

## Production Best Practices

Designing for webhooks requires a different mindset than standard REST APIs. Follow these rules for a robust integration:

### 1. Consumer Idempotency is Mandatory

Webhooks implement "At-Least-Once" delivery. In case of network errors, Hook0 may retry sending the same event.

- **Rule:** Consumers must verify the `event_id`.
- **Implementation:** Store processed `event_id`s in a database. If an ID is seen again, return `200 OK` immediately without re-processing logic.

### 2. Async Processing (The 200 OK Rule)

Your webhook endpoint must be fast. If you perform long operations (sending emails, generating PDFs) synchronously, you risk timeouts.

- **Rule:** Acknowledge receipt first, process later.
- **Implementation:** Receive the webhook → Push to an internal queue (e.g., SQS, RabbitMQ, Redis) → Return `200 OK`. Workers will process the actual job.

### 3. Thin Payloads vs. Fat Payloads

Sending massive objects in webhooks increases latency and failure rates.

- **Recommendation:** Prefer "Thin Payloads" (IDs + changed fields) over "Fat Payloads" (full database record).
- **Why:** Large payloads may be rejected by intermediate proxies. It also ensures the consumer fetches the *freshest* data via your API using the ID provided in the webhook.

### 4. Security Verification

Never trust the payload blindly. Any public endpoint can be flooded with fake data.

- **Rule:** Verify the HMAC signature.
- **Implementation:** Use the signing secret provided by Hook0 to hash the incoming body and compare it with the `X-Hook0-Signature` header. Reject any request where the signature does not match. See [Webhook Authentication](./webhook-authentication.md) for implementation examples.

## What You've Learned

✅ Designed hierarchical event type schemas  
✅ Created comprehensive event type sets  
✅ Built sophisticated subscription patterns  
✅ Configured advanced subscription options  
✅ Implemented testing strategies  
✅ Managed event types and subscriptions lifecycle  

## Next Steps

- [Implementing Webhook Authentication](./webhook-authentication.md)
- [Debugging Failed Webhook Deliveries](../how-to-guides/debug-failed-webhooks.md)

## Troubleshooting

### Event Not Triggering Webhooks
1. Check event type exists in application
2. Verify subscription is enabled
3. Confirm event type matches subscription
4. Check webhook endpoint accessibility

### Subscription Not Receiving Events
1. Verify subscription configuration
2. Check event type matching logic
3. Review webhook endpoint logs
4. Monitor Hook0 dashboard for delivery attempts
