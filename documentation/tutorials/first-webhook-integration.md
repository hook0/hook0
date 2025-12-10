# Building Your First Webhook System with Hook0

This tutorial demonstrates how to integrate Hook0 into your SaaS platform to provide webhook capabilities to your customers. It uses curl commands to illustrate the API flow, making it easy to understand and implement in any programming language.

## What You'll Build

A complete webhook delivery system that:
- Send events from your SaaS to Hook0
- Enable customer subscriptions to specific events  
- Handle automatic delivery, retries, and monitoring
- Support multi-tenant filtering with labels

## Prerequisites

- [Hook0 cloud account](https://www.hook0.com/) or [self-hosted instance running](./getting-started.md)
- [API token](./getting-started.md#step-3-get-your-api-token)
- Application ID from Hook0
- Basic understanding of REST APIs and webhooks

## Architecture Overview

```
  ┌──────────┐  POST event    ┌────────┐  POST webhook   ┌──────────┐
  │ Your     │───────────────▶│        │────────────────▶│ Customer │
  │ SaaS     │                │ Hook0  │                 │ Endpoint │
  └──────────┘                │        │                 └──────────┘
                              └────────┘
```

Your SaaS sends events to Hook0, which manages all webhook complexity including delivery, retries, signature verification, and monitoring.

## Understanding the API Flow

The integration follows this logical sequence:
1. **Create Event Types** - Define what events your SaaS can emit
2. **Create Subscriptions** - Enable customers to receive webhooks
3. **Send Events** - Push events to Hook0 when actions occur
4. **Monitor Delivery** - Track webhook success and failures

Let's walk through each step with practical examples.

## Step 1: Define Your Event Types

Hook0 uses a three-part structure for event types: `service.resource_type.verb`. This provides clear semantics and enables powerful filtering.

### Why define event types first?
Event types act as a contract between your SaaS and customer webhooks. They must be created before you can send events or create subscriptions.

### Creating a "User Created" Event Type

```bash
curl -X POST "https://app.hook0.com/api/v1/event_types" \
  -H "Authorization: Bearer {YOUR_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "{APP_ID}",
    "service": "users",
    "resource_type": "account", 
    "verb": "created"
  }'
```

**Response:**
```json
{
  "service_name": "users",
  "resource_type_name": "account",
  "verb_name": "created",
  "event_type_name": "users.account.created"
}
```

### Creating an "Order Completed" Event Type

```bash
curl -X POST "https://app.hook0.com/api/v1/event_types" \
  -H "Authorization: Bearer {YOUR_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "{APP_ID}",
    "service": "orders",
    "resource_type": "purchase",
    "verb": "completed"
  }'
```

**Response:**
```json
{
  "service_name":"orders",
  "resource_type_name":"purchase",
  "verb_name":"completed",
  "event_type_name":"orders.purchase.completed"
}
```


## Step 2: Send Events to Hook0

Once event types are defined, your SaaS can start sending events. The `/api/v1/event` endpoint (singular) accepts individual events.

### Why this order?
You must create event types before sending events. Hook0 validates that the event type exists before accepting the event.

### Sending a User Created Event

When a user signs up in your SaaS:

```bash
curl -X POST "https://app.hook0.com/api/v1/event" \
  -H "Authorization: Bearer {YOUR_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "{APP_ID}",
    "event_id": "'$(uuidgen)'",
    "event_type": "users.account.created",
    "payload": "{\"user_id\": \"usr_789\", \"email\": \"john@example.com\", \"plan\": \"premium\"}",
    "payload_content_type": "application/json",
    "occurred_at": "2024-01-15T10:30:00Z",
    "labels": {
      "tenant_id": "customer_123",
      "environment": "production",
      "region": "us-east-1"
    }
  }'
```

**Response:**
```json
{
  "application_id": "3cae6773-88ae-4489-9b30-3918624fbd49",
  "event_id": "69f69ecf-0d9e-4c92-a6e0-3b2676343940",
  "received_at": "2025-12-12T09:25:28.084734Z"
}
```

**Key Points:**
- `event_id`: Must be a valid UUID (e.g., `69f69ecf-0d9e-4c92-a6e0-3b2676343940`)
- `payload`: Must be a **JSON-encoded string** (not an object) - see warning below
- `labels`: Critical for multi-tenant filtering - must have at least one key-value pair

:::warning Payload Format
The `payload` field must be a **JSON-encoded string**, not a JSON object:

✅ **Correct**: `"payload": "{\"user_id\": \"usr_789\"}"`

❌ **Incorrect**: `"payload": {"user_id": "usr_789"}`

This allows Hook0 to forward the exact payload to webhooks without re-serialization.
:::

The `tenant_id` label ensures events only go to the right customer.

### Sending an Order Completed Event

When an order is fulfilled:

```bash
curl -X POST "https://app.hook0.com/api/v1/event" \
  -H "Authorization: Bearer {YOUR_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "{APP_ID}",
    "event_id": "'$(uuidgen)'",
    "event_type": "orders.purchase.completed",
    "payload": "{\"order_id\": \"ord_456\", \"amount\": 299.99, \"items\": 3}",
    "payload_content_type": "application/json",
    "occurred_at": "2024-01-15T11:00:00Z",
    "labels": {
      "tenant_id": "customer_123",
      "priority": "high",
      "environment": "production"
    }
  }'
```

**Response:**
```json
{
  "application_id": "3cae6773-88ae-4489-9b30-3918624fbd49",
  "event_id": "7c7a01cc-56a7-48b6-a0cb-f43457346e41",
  "received_at": "2025-12-12T09:27:27.045191Z"
}
```

## Step 3: Create Customer Subscriptions

Subscriptions define where and how webhooks are delivered. Each subscription filters events based on labels, ensuring customers only receive their own events.

### Why labels matter for multi-tenancy
The `label_key` and `label_value` create a filter. Only events with matching labels are delivered to that subscription. This is how Hook0 supports multi-tenant SaaS platforms.

### Creating a Subscription for a Customer

```bash
curl -X POST "https://app.hook0.com/api/v1/subscriptions" \
  -H "Authorization: Bearer {YOUR_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "{APP_ID}",
    "is_enabled": true,
    "event_types": [
      "users.account.created",
      "orders.purchase.completed"
    ],
    "description": "Webhook for Customer ABC Corp",
    "label_key": "tenant_id",
    "label_value": "customer_123",
    "target": {
      "type": "http",
      "method": "POST",
      "url": "https://customer-abc.com/webhooks/hook0",
      "headers": {
        "Content-Type": "application/json",
        "X-Customer-Id": "customer_123"
      }
    },
    "metadata": {
      "customer_name": "ABC Corp",
      "created_by": "api",
      "plan": "enterprise"
    }
  }'
```

**Response:**
```json
{
  "application_id": "3cae6773-88ae-4489-9b30-3918624fbd49",
  "subscription_id": "5fd32357-494b-446a-b49d-68f973e6aaaa",
  "is_enabled": true,
  "event_types": [
    "users.account.created",
    "orders.purchase.completed"
  ],
  "description": "Webhook for Customer ABC Corp",
  "secret": "d48488f1-cddc...",
  "metadata": {
    "customer_name": "ABC Corp",
    "plan": "enterprise",
    "created_by": "api"
  },
  "label_key": "tenant_id",
  "label_value": "customer_123",
  "labels": {
    "tenant_id": "customer_123"
  },
  "target": {
    "type": "http",
    "method": "POST",
    "url": "https://customer-abc.com/webhooks/hook0",
    "headers": {
      "content-type": "application/json",
      "x-customer-id": "customer_123"
    }
  },
  "created_at": "2025-12-12T09:30:13.822397Z",
  "dedicated_workers": []
}
```

**Important:** Save the `secret` - customers need it to verify webhook signatures.

### How Multi-Tenant Filtering Works

```
Event labels                 Subscription filter              Result
────────────────────────     ─────────────────────────        ────────────
tenant_id: "customer_123" →  tenant_id = "customer_123"   →   ✅ Delivered
tenant_id: "customer_456" →  tenant_id = "customer_123"   →   ❌ Skipped
```

## Step 4: Monitor Webhook Deliveries

Hook0 provides comprehensive monitoring to track webhook success and failures.

### List Recent Events

See what events have been sent:

```bash
curl -X GET "https://app.hook0.com/api/v1/events/?application_id={APP_ID}" \
  -H "Authorization: Bearer {YOUR_TOKEN}"
```

**Response:**
```json
[
  {
    "event_id": "7c7a01cc-56a7-48b6-a0cb-f43457346e41",
    "event_type_name": "orders.purchase.completed",
    "payload_content_type": "application/json",
    "ip": "192.168.97.1",
    "metadata": {},
    "occurred_at": "2024-01-15T11:00:00Z",
    "received_at": "2025-12-12T09:27:27.045191Z",
    "labels": {
      "environment": "production",
      "priority": "high",
      "tenant_id": "customer_123"
    }
  },
  {
    "event_id": "69f69ecf-0d9e-4c92-a6e0-3b2676343940",
    "event_type_name": "users.account.created",
    "payload_content_type": "application/json",
    "ip": "192.168.97.1",
    "metadata": {},
    "occurred_at": "2024-01-15T10:30:00Z",
    "received_at": "2025-12-12T09:25:28.084734Z",
    "labels": {
      "environment": "production",
      "region": "us-east-1",
      "tenant_id": "customer_123"
    }
  },
  {
    "event_id": "ae5844f0-b1ce-46fb-87f1-eddd52ace36c",
    "event_type_name": "users.account.created",
    "payload_content_type": "application/json",
    "ip": "192.168.97.1",
    "metadata": {},
    "occurred_at": "2025-12-12T08:39:19Z",
    "received_at": "2025-12-12T08:39:19.407621Z",
    "labels": {
      "environment": "tutorial"
    }
  }
]
```

### Check Delivery Attempts

Monitor webhook delivery attempts and their status:

```bash
curl -X GET "https://app.hook0.com/api/v1/request_attempts/?application_id={APP_ID}" \
  -H "Authorization: Bearer {YOUR_TOKEN}"
```

**Response:**
```json
[
  {
    "request_attempt_id": "550e8400-e29b-41d4-a716-446655440001",
    "event_id": "550e8400-e29b-41d4-a716-446655440000",
    "subscription": {
      "subscription_id": "550e8400-e29b-41d4-a716-446655440002",
      "description": "Customer webhook"
    },
    "created_at": "2024-01-15T10:30:01Z",
    "picked_at": "2024-01-15T10:30:02Z",
    "succeeded_at": "2024-01-15T10:30:03Z",
    "failed_at": null,
    "retry_count": 0,
    "response_id": "550e8400-e29b-41d4-a716-446655440003",
    "status": {
      "type": "succeeded",
      "at": "2024-01-15T10:30:03Z"
    }
  }
]
```

### Replay Failed Events

If a webhook fails, you can replay it:

```bash
curl -X POST "https://app.hook0.com/api/v1/events/{EVENT_ID}/replay" \
  -H "Authorization: Bearer {YOUR_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "{APP_ID}"
  }'
```

## Advanced Patterns

### Bulk Event Processing

When you need to send multiple events (e.g., batch import), send them individually but with correlation:

```bash
# Event 1 of batch
curl -X POST "https://app.hook0.com/api/v1/event" \
  -H "Authorization: Bearer {YOUR_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "{APP_ID}",
    "event_id": "'$(uuidgen)'",
    "event_type": "users.account.created",
    "payload": "{\"user_id\": \"usr_789\", \"email\": \"john@example.com\", \"plan\": \"premium\"}",
    "labels": {
      "tenant_id": "customer_123",
      "batch_id": "batch-001",
      "batch_size": "100"
    }
  }'

# Event 2 of batch  
curl -X POST "https://app.hook0.com/api/v1/event" \
  -H "Authorization: Bearer {YOUR_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "{APP_ID}",
    "event_id": "'$(uuidgen)'",
    "event_type": "users.account.created",
    "payload": "{\"user_id\": \"usr_7892\", \"email\": \"john2@example.com\", \"plan\": \"premium\"}",
    "labels": {
      "tenant_id": "customer_123",
      "batch_id": "batch-001",
      "batch_size": "100"
    }
  }'
```

### Environment-Based Filtering

Use labels to separate environments:

```bash
# Production event
curl -X POST "https://app.hook0.com/api/v1/event" \
  -H "Authorization: Bearer {YOUR_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "{APP_ID}",
    "event_id": "'$(uuidgen)'",
    "event_type": "users.account.created",
    "payload": "{\"user_id\": \"usr_123\"}",
    "payload_content_type": "application/json",
    "occurred_at": "'$(date -u +"%Y-%m-%dT%H:%M:%SZ")'",
    "labels": {
      "tenant_id": "customer_123",
      "environment": "production"
    }
  }'
```

**Response:**
```json
{
  "application_id": "3cae6773-88ae-4489-9b30-3918624fbd49",
  "event_id": "dad2b176-d385-4d02-8201-f5b17ba764c1",
  "received_at": "2025-12-12T09:41:39.757315Z"
}
```

```bash
# Staging subscription (will not receive production events)
curl -X POST "https://app.hook0.com/api/v1/subscriptions" \
  -H "Authorization: Bearer {YOUR_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "{APP_ID}",
    "is_enabled": true,
    "event_types": ["users.account.created"],
    "description": "Staging webhook",
    "label_key": "environment",
    "label_value": "staging",
    "target": {
      "type": "http",
      "method": "POST",
      "url": "https://staging.example.com/webhook",
      "headers": {
        "Content-Type": "application/json"
      }
    }
  }'
```

**Response:**
```json
{
  "application_id": "3cae6773-88ae-4489-9b30-3918624fbd49",
  "subscription_id": "d2e628df-fa60-42f4-ab9a-5034b4f3a333",
  "is_enabled": true,
  "event_types": ["users.account.created"],
  "description": "Staging webhook",
  "secret": "d77dfd50-477f-4a9e-b2e1-b49f7e506cf8",
  "label_key": "environment",
  "label_value": "staging",
  "target": {
    "type": "http",
    "method": "POST",
    "url": "https://staging.example.com/webhook",
    "headers": {"content-type": "application/json"}
  },
  "created_at": "2025-12-12T09:41:41.304925Z"
}
```

Since this subscription filters on `environment: "staging"`, it will **not** receive the production event above.

### Priority-Based Routing

Use labels for priority handling:

```bash
# High-priority event
curl -X POST "https://app.hook0.com/api/v1/event" \
  -H "Authorization: Bearer {YOUR_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "{APP_ID}",
    "event_id": "'$(uuidgen)'",
    "event_type": "payments.transaction.processed",
    "payload": "{\"amount\": 10000}",
    "payload_content_type": "application/json",
    "occurred_at": "'$(date -u +"%Y-%m-%dT%H:%M:%SZ")'",
    "labels": {
      "tenant_id": "customer_123",
      "priority": "high",
      "amount_tier": "enterprise"
    }
  }'
```

**Response:**
```json
{
  "application_id": "3cae6773-88ae-4489-9b30-3918624fbd49",
  "event_id": "cb00bf75-5757-4de2-a314-a37c26c230fd",
  "received_at": "2025-12-12T09:41:52.802753Z"
}
```

## Complete Integration Example

Here's a practical example showing the complete flow from event creation to webhook delivery:

### Step 1: Create Event Type (one-time setup)

```bash
curl -X POST "https://app.hook0.com/api/v1/event_types" \
  -H "Authorization: Bearer {YOUR_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "{APP_ID}",
    "service": "billing",
    "resource_type": "invoice",
    "verb": "paid"
  }'
```

**Response:**
```json
{
  "service_name": "billing",
  "resource_type_name": "invoice",
  "verb_name": "paid",
  "event_type_name": "billing.invoice.paid"
}
```

### Step 2: Create Subscription (customer setup)

```bash
curl -X POST "https://app.hook0.com/api/v1/subscriptions" \
  -H "Authorization: Bearer {YOUR_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "{APP_ID}",
    "is_enabled": true,
    "event_types": ["billing.invoice.paid"],
    "description": "Customer billing webhook",
    "label_key": "tenant_id",
    "label_value": "customer_789",
    "target": {
      "type": "http",
      "method": "POST",
      "url": "https://customer.example.com/webhooks",
      "headers": {
        "Content-Type": "application/json"
      }
    }
  }'
```

**Response:**
```json
{
  "application_id": "3cae6773-88ae-4489-9b30-3918624fbd49",
  "subscription_id": "6b15a1be-20f9-4a6b-b09b-e09c50072d2d",
  "is_enabled": true,
  "event_types": ["billing.invoice.paid"],
  "description": "Customer billing webhook",
  "secret": "af0c8a2a-00f9-42e5-afef-69ba9a66997a",
  "label_key": "tenant_id",
  "label_value": "customer_789",
  "target": {
    "type": "http",
    "method": "POST",
    "url": "https://customer.example.com/webhooks",
    "headers": {"content-type": "application/json"}
  },
  "created_at": "2025-12-12T09:43:22.425687Z"
}
```

⚠️ **Save the `secret`** - customers need it to verify webhook signatures.

### Step 3: Send Event (when invoice is paid)

```bash
curl -X POST "https://app.hook0.com/api/v1/event" \
  -H "Authorization: Bearer {YOUR_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "{APP_ID}",
    "event_id": "'$(uuidgen)'",
    "event_type": "billing.invoice.paid",
    "payload": "{\"invoice_id\": \"inv_456\", \"amount\": 1500.00}",
    "payload_content_type": "application/json",
    "occurred_at": "'$(date -u +"%Y-%m-%dT%H:%M:%SZ")'",
    "labels": {
      "tenant_id": "customer_789"
    }
  }'
```

**Response:**
```json
{
  "application_id": "3cae6773-88ae-4489-9b30-3918624fbd49",
  "event_id": "1d84fdac-1de4-424d-966b-8c6386834ef6",
  "received_at": "2025-12-12T09:43:24.351417Z"
}
```

### Step 4: Automatic Delivery

Hook0 automatically delivers the webhook to `https://customer.example.com/webhooks` with:
- `X-Hook0-Signature` header for verification
- The event payload
- Automatic retries on failure

See [Implementing Webhook Authentication](./webhook-authentication.md) for signature verification code in Node.js, Python, and Go.

## Implementation Patterns in Different Languages

While this tutorial uses curl for clarity, here's how to implement the same patterns in various languages:

### Python
```python
import requests
import uuid
import json
from datetime import datetime, timezone

# Send event
event = {
    "application_id": "{APP_ID}",
    "event_id": str(uuid.uuid4()),
    "event_type": "users.account.created",
    "payload": json.dumps({"user_id": "usr_123"}),
    "payload_content_type": "application/json",
    "occurred_at": datetime.now(timezone.utc).isoformat(),
    "labels": {"tenant_id": "customer_123"}
}

response = requests.post(
    "https://app.hook0.com/api/v1/event",
    headers={"Authorization": "Bearer {YOUR_TOKEN}"},
    json=event
)
```

### Go
```go
import (
    "bytes"
    "encoding/json"
    "net/http"
    "time"

    "github.com/google/uuid"
)

// Send event
event := map[string]interface{}{
    "application_id":       "{APP_ID}",
    "event_id":             uuid.New().String(),
    "event_type":           "users.account.created",
    "payload":              `{"user_id": "usr_123"}`,
    "payload_content_type": "application/json",
    "occurred_at":          time.Now().UTC().Format(time.RFC3339),
    "labels": map[string]string{
        "tenant_id": "customer_123",
    },
}

jsonData, _ := json.Marshal(event)
req, _ := http.NewRequest("POST",
    "https://app.hook0.com/api/v1/event",
    bytes.NewBuffer(jsonData))
req.Header.Set("Authorization", "Bearer {YOUR_TOKEN}")
req.Header.Set("Content-Type", "application/json")

client := &http.Client{}
resp, _ := client.Do(req)
defer resp.Body.Close()
```

### Ruby
```ruby
require 'net/http'
require 'json'
require 'securerandom'
require 'time'

# Send event
event = {
  application_id: "{APP_ID}",
  event_id: SecureRandom.uuid,
  event_type: "users.account.created",
  payload: {user_id: "usr_123"}.to_json,
  payload_content_type: "application/json",
  occurred_at: Time.now.utc.iso8601,
  labels: {tenant_id: "customer_123"}
}

uri = URI('https://app.hook0.com/api/v1/event')
http = Net::HTTP.new(uri.host, uri.port)

request = Net::HTTP::Post.new(uri)
request['Authorization'] = 'Bearer {YOUR_TOKEN}'
request['Content-Type'] = 'application/json'
request.body = event.to_json

response = http.request(request)
```

## Best Practices

### 1. Event Design
- **Use semantic naming**: `service.resource.verb` structure
- **Include context**: Add relevant data to help consumers
- **Be consistent**: Same structure across all events
- **Version carefully**: Plan for schema evolution

### 2. Label Strategy
- **tenant_id**: Always include for multi-tenancy
- **environment**: Separate prod/staging/dev
- **priority**: Enable priority-based processing
- **region**: Support geographic filtering

### 3. Error Handling
- **Idempotency**: Use unique event_ids to prevent duplicates
- **Retry logic**: Implement exponential backoff
- **Circuit breaker**: Fail fast when Hook0 is down
- **Local queue**: Buffer events during outages

### 4. Security
- **Never log tokens**: Keep authentication secure
- **Validate signatures**: Customers should verify webhooks
- **Encrypt sensitive data**: Do not send PII in plaintext
- **Rate limit**: Protect against abuse

## Troubleshooting Common Issues

### Event Not Delivered
```bash
# Check if event was received
curl -X GET "https://app.hook0.com/api/v1/events?application_id={APP_ID}" \
  -H "Authorization: Bearer {YOUR_TOKEN}"

# Check subscription filters
curl -X GET "https://app.hook0.com/api/v1/subscriptions?application_id={APP_ID}" \
  -H "Authorization: Bearer {YOUR_TOKEN}"
# Verify label_key and label_value match your events
```

### 401 Unauthorized
```bash
# Verify token is included
curl -H "Authorization: Bearer {YOUR_TOKEN}"  # ✅ Correct
curl -H "Authorization: {YOUR_TOKEN}"         # ❌ Wrong - missing Bearer
```

### Event Type Not Found
```bash
# List existing event types
curl -X GET "https://app.hook0.com/api/v1/event_types?application_id={APP_ID}" \
  -H "Authorization: Bearer {YOUR_TOKEN}"

# Create missing event type before sending events
```

## Summary

You've learned how to:
1. **Create event types** using the three-part structure
2. **Send events** with proper labeling for multi-tenancy
3. **Create subscriptions** with label-based filtering
4. **Monitor deliveries** and handle failures
5. **Implement patterns** for bulk processing and priority routing

The key insight is that Hook0's label system enables powerful multi-tenant webhook delivery while keeping the integration simple. By following this flow - define types, send events, create subscriptions, monitor results - you can build a robust webhook system for your SaaS platform.

## Next Steps

- [Event Types and Subscriptions Deep Dive](./event-types-subscriptions.md)
- [Webhook Authentication and Security](./webhook-authentication.md)
- [Debugging Failed Webhooks](../how-to-guides/debug-failed-webhooks.md)
- [GitLab-Style Webhook Migration](../how-to-guides/gitlab-webhook-migration.md)
