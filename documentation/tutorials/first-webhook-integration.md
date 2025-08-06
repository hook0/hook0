# Building Your First Webhook System with Hook0

This tutorial demonstrates how to integrate Hook0 into your SaaS platform to provide webhook capabilities to your customers. We'll use curl commands to illustrate the API flow, making it easy to understand and implement in any programming language.

## What You'll Build

A complete webhook delivery system that:
- Sends events from your SaaS to Hook0
- Enables customer subscriptions to specific events  
- Handles automatic delivery, retries, and monitoring
- Supports multi-tenant filtering with labels

## Prerequisites

- Hook0 account with API token (Biscuit format)
- Application ID from Hook0
- Basic understanding of REST APIs and webhooks

## Architecture Overview

```
Your SaaS → Hook0 → Customer Endpoints
   ↓         ↓
Events    Delivery
          Management
```

Your SaaS sends events to Hook0, which manages all webhook complexity including delivery, retries, signature verification, and monitoring.

## Understanding the API Flow

The integration follows this logical sequence:
1. **Create Event Types** - Define what events your SaaS can emit
2. **Send Events** - Push events to Hook0 when actions occur
3. **Create Subscriptions** - Enable customers to receive webhooks
4. **Monitor Delivery** - Track webhook success and failures

Let's walk through each step with practical examples.

## Step 1: Define Your Event Types

Hook0 uses a three-part structure for event types: `service.resource_type.verb`. This provides clear semantics and enables powerful filtering.

### Why Define Event Types First?
Event types act as a contract between your SaaS and customer webhooks. They must be created before you can send events or create subscriptions.

### Creating a User Created Event Type

```bash
curl -X POST "https://app.hook0.com/api/v1/event_types" \
  -H "Authorization: Bearer biscuit:YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "YOUR_APP_ID",
    "service": "users",
    "resource_type": "account", 
    "verb": "created"
  }'
```

**Response:**
```json
{
  "event_type_id": "evt_123",
  "name": "users.account.created",
  "created_at": "2024-01-15T10:00:00Z"
}
```

### Creating an Order Completed Event Type

```bash
curl -X POST "https://app.hook0.com/api/v1/event_types" \
  -H "Authorization: Bearer biscuit:YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "YOUR_APP_ID",
    "service": "orders",
    "resource_type": "purchase",
    "verb": "completed"
  }'
```

## Step 2: Send Events to Hook0

Once event types are defined, your SaaS can start sending events. The `/api/v1/event` endpoint (singular) accepts individual events.

### Why This Order?
You must create event types before sending events. Hook0 validates that the event type exists before accepting the event.

### Sending a User Created Event

When a user signs up in your SaaS:

```bash
curl -X POST "https://app.hook0.com/api/v1/event" \
  -H "Authorization: Bearer biscuit:YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "YOUR_APP_ID",
    "event_id": "550e8400-e29b-41d4-a716-446655440000",
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

**Key Points:**
- `event_id`: Use a UUID for idempotency
- `payload`: Must be a JSON string (not an object)
- `labels`: Critical for multi-tenant filtering
- `tenant_id` label: Ensures events only go to the right customer

### Sending an Order Completed Event

When an order is fulfilled:

```bash
curl -X POST "https://app.hook0.com/api/v1/event" \
  -H "Authorization: Bearer biscuit:YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "YOUR_APP_ID",
    "event_id": "660e8400-e29b-41d4-a716-446655440001",
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

## Step 3: Create Customer Subscriptions

Subscriptions define where and how webhooks are delivered. Each subscription filters events based on labels, ensuring customers only receive their own events.

### Why Labels Matter for Multi-Tenancy
The `label_key` and `label_value` create a filter. Only events with matching labels are delivered to that subscription. This is how Hook0 supports multi-tenant SaaS platforms.

### Creating a Subscription for a Customer

```bash
curl -X POST "https://app.hook0.com/api/v1/subscriptions" \
  -H "Authorization: Bearer biscuit:YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "YOUR_APP_ID",
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
  "subscription_id": "sub_789",
  "secret": "whsec_abcd1234...",
  "created_at": "2024-01-15T10:00:00Z"
}
```

**Important:** Save the `secret` - customers need it to verify webhook signatures.

### How Multi-Tenant Filtering Works

```
Event sent with labels:          Subscription filter:
{                                {
  "tenant_id": "customer_123" →    "label_key": "tenant_id",
  "environment": "production"      "label_value": "customer_123"
}                                }
                                 ✅ Match! Webhook delivered

Event sent with labels:          Subscription filter:
{                                {
  "tenant_id": "customer_456" →    "label_key": "tenant_id",
  "environment": "production"      "label_value": "customer_123"
}                                }
                                 ❌ No match! Webhook NOT delivered
```

## Step 4: Monitor Webhook Deliveries

Hook0 provides comprehensive monitoring to track webhook success and failures.

### List Recent Events

See what events have been sent:

```bash
curl -X GET "https://app.hook0.com/api/v1/events?application_id=YOUR_APP_ID&limit=10" \
  -H "Authorization: Bearer biscuit:YOUR_TOKEN"
```

**Response:**
```json
{
  "events": [
    {
      "event_id": "550e8400-e29b-41d4-a716-446655440000",
      "event_type": "users.account.created",
      "labels": {"tenant_id": "customer_123"},
      "occurred_at": "2024-01-15T10:30:00Z",
      "delivery_status": "delivered"
    }
  ]
}
```

### Check Delivery Attempts

Monitor webhook delivery attempts and their status:

```bash
curl -X GET "https://app.hook0.com/api/v1/request_attempts?application_id=YOUR_APP_ID" \
  -H "Authorization: Bearer biscuit:YOUR_TOKEN"
```

**Response:**
```json
{
  "attempts": [
    {
      "attempt_id": "att_001",
      "event_id": "550e8400-e29b-41d4-a716-446655440000",
      "subscription_id": "sub_789",
      "status": "success",
      "response_status_code": 200,
      "attempted_at": "2024-01-15T10:30:05Z",
      "labels": {"tenant_id": "customer_123"}
    }
  ]
}
```

### Replay Failed Events

If a webhook fails, you can replay it:

```bash
curl -X POST "https://app.hook0.com/api/v1/events/{EVENT_ID}/replay" \
  -H "Authorization: Bearer biscuit:YOUR_TOKEN" \
  -H "Content-Type: application/json"
```

## Advanced Patterns

### Bulk Event Processing

When you need to send multiple events (e.g., batch import), send them individually but with correlation:

```bash
# Event 1 of batch
curl -X POST "https://app.hook0.com/api/v1/event" \
  -H "Authorization: Bearer biscuit:YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "YOUR_APP_ID",
    "event_id": "batch-001-item-1",
    "event_type": "inventory.item.updated",
    "payload": "{\"sku\": \"PROD-001\", \"quantity\": 50}",
    "labels": {
      "tenant_id": "customer_123",
      "batch_id": "batch-001",
      "batch_size": "100"
    }
  }'

# Event 2 of batch  
curl -X POST "https://app.hook0.com/api/v1/event" \
  -H "Authorization: Bearer biscuit:YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "YOUR_APP_ID",
    "event_id": "batch-001-item-2",
    "event_type": "inventory.item.updated",
    "payload": "{\"sku\": \"PROD-002\", \"quantity\": 75}",
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
  -d '{
    "labels": {
      "tenant_id": "customer_123",
      "environment": "production"
    }
  }'

# Staging subscription (won't receive production events)
curl -X POST "https://app.hook0.com/api/v1/subscriptions" \
  -d '{
    "label_key": "environment",
    "label_value": "staging"
  }'
```

### Priority-Based Routing

Use labels for priority handling:

```bash
# High-priority event
curl -X POST "https://app.hook0.com/api/v1/event" \
  -d '{
    "event_type": "payment.processed",
    "payload": "{\"amount\": 10000}",
    "labels": {
      "tenant_id": "customer_123",
      "priority": "high",
      "amount_tier": "enterprise"
    }
  }'
```

## Complete Integration Example

Here's a practical example showing the complete flow from event creation to webhook delivery:

```bash
# Step 1: Create event type (one-time setup)
curl -X POST "https://app.hook0.com/api/v1/event_types" \
  -H "Authorization: Bearer biscuit:YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "app_xyz",
    "service": "billing",
    "resource_type": "invoice",
    "verb": "paid"
  }'

# Step 2: Customer creates subscription (via your API or UI)
curl -X POST "https://app.hook0.com/api/v1/subscriptions" \
  -H "Authorization: Bearer biscuit:YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "app_xyz",
    "event_types": ["billing.invoice.paid"],
    "label_key": "tenant_id",
    "label_value": "customer_789",
    "target": {
      "type": "http",
      "url": "https://customer.com/webhooks"
    }
  }'
# Returns: {"subscription_id": "sub_123", "secret": "whsec_..."}

# Step 3: Send event when invoice is paid
curl -X POST "https://app.hook0.com/api/v1/event" \
  -H "Authorization: Bearer biscuit:YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "app_xyz",
    "event_id": "evt_unique_123",
    "event_type": "billing.invoice.paid",
    "payload": "{\"invoice_id\": \"inv_456\", \"amount\": 1500.00}",
    "labels": {
      "tenant_id": "customer_789"
    }
  }'

# Step 4: Hook0 automatically delivers to customer webhook!
# Customer receives POST to https://customer.com/webhooks with signature
```

## Implementation Patterns in Different Languages

While we've used curl for clarity, here's how to implement the same patterns in various languages:

### Python
```python
import requests
import uuid
import json

# Send event
event = {
    "application_id": "YOUR_APP_ID",
    "event_id": str(uuid.uuid4()),
    "event_type": "users.account.created",
    "payload": json.dumps({"user_id": "usr_123"}),
    "labels": {"tenant_id": "customer_123"}
}

response = requests.post(
    "https://app.hook0.com/api/v1/event",
    headers={"Authorization": "Bearer biscuit:YOUR_TOKEN"},
    json=event
)
```

### Go
```go
// Send event
event := map[string]interface{}{
    "application_id": "YOUR_APP_ID",
    "event_id": uuid.New().String(),
    "event_type": "users.account.created",
    "payload": `{"user_id": "usr_123"}`,
    "labels": map[string]string{
        "tenant_id": "customer_123",
    },
}

jsonData, _ := json.Marshal(event)
req, _ := http.NewRequest("POST", 
    "https://app.hook0.com/api/v1/event", 
    bytes.NewBuffer(jsonData))
req.Header.Set("Authorization", "Bearer biscuit:YOUR_TOKEN")
```

### Ruby
```ruby
require 'net/http'
require 'json'
require 'securerandom'

# Send event
event = {
  application_id: "YOUR_APP_ID",
  event_id: SecureRandom.uuid,
  event_type: "users.account.created",
  payload: {user_id: "usr_123"}.to_json,
  labels: {tenant_id: "customer_123"}
}

uri = URI('https://app.hook0.com/api/v1/event')
http = Net::HTTP.new(uri.host, uri.port)
http.use_ssl = true

request = Net::HTTP::Post.new(uri)
request['Authorization'] = 'Bearer biscuit:YOUR_TOKEN'
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
- **Encrypt sensitive data**: Don't send PII in plaintext
- **Rate limit**: Protect against abuse

## Troubleshooting Common Issues

### Event Not Delivered
```bash
# Check if event was received
curl -X GET "https://app.hook0.com/api/v1/events?application_id=YOUR_APP_ID" \
  -H "Authorization: Bearer biscuit:YOUR_TOKEN"

# Check subscription filters
curl -X GET "https://app.hook0.com/api/v1/subscriptions?application_id=YOUR_APP_ID" \
  -H "Authorization: Bearer biscuit:YOUR_TOKEN"
# Verify label_key and label_value match your events
```

### 401 Unauthorized
```bash
# Verify token format (must include "biscuit:" prefix)
curl -H "Authorization: Bearer biscuit:YOUR_TOKEN"  # ✅ Correct
curl -H "Authorization: Bearer YOUR_TOKEN"         # ❌ Wrong
```

### Event Type Not Found
```bash
# List existing event types
curl -X GET "https://app.hook0.com/api/v1/event_types?application_id=YOUR_APP_ID" \
  -H "Authorization: Bearer biscuit:YOUR_TOKEN"

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