# API Reference

Hook0 provides a comprehensive REST API for managing webhooks, events, and integrations. This reference covers all endpoints, request/response formats, and authentication methods.

## Base URL

```
Production: https://api.hook0.com/api/v1
Staging: https://api-staging.hook0.com/api/v1
```

## Authentication

Hook0 uses Biscuit tokens for API authentication. Include your token in the Authorization header:

```http
Authorization: Bearer biscuit:YOUR_TOKEN_HERE
```

### Token Types

**User Access Tokens**
- Full access to user's organizations and applications
- Used for web application authentication
- Short-lived (24 hours by default)

**Service Tokens**
- API access for server-to-server communication
- Configurable permissions and scope
- Long-lived or permanent

**Refresh Tokens**
- Used to obtain new access tokens
- Long-lived, single-use

## Rate Limiting

API requests are rate limited by organization:

- **Free tier**: 1,000 requests per hour
- **Pro tier**: 10,000 requests per hour
- **Enterprise**: Custom limits

Rate limit headers are included in responses:
```http
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 999
X-RateLimit-Reset: 1609459200
```

## Pagination

List endpoints support cursor-based pagination:

```http
GET /api/v1/events?limit=50&cursor=eyJpZCI6IjEyMyJ9
```

**Parameters:**
- `limit`: Number of items per page (max 100, default 20)
- `cursor`: Pagination cursor from previous response

**Response:**
```json
{
  "data": [...],
  "pagination": {
    "next_cursor": "eyJpZCI6IjQ1NiJ9",
    "has_more": true
  }
}
```

## Error Handling

Hook0 uses conventional HTTP status codes and returns detailed error information:

```json
{
  "error": {
    "type": "validation_error",
    "message": "The request contains invalid parameters",
    "code": "INVALID_PARAMETER",
    "details": [
      {
        "field": "event_type",
        "message": "Event type is required"
      }
    ],
    "request_id": "req_1234567890"
  }
}
```

### HTTP Status Codes

- `200` - Success
- `201` - Created
- `400` - Bad Request
- `401` - Unauthorized
- `403` - Forbidden
- `404` - Not Found
- `409` - Conflict
- `422` - Unprocessable Entity
- `429` - Too Many Requests
- `500` - Internal Server Error

## Organizations

### List Organizations

```http
GET /api/v1/organizations
```

**Response:**
```json
{
  "data": [
    {
      "id": "org_1234567890",
      "name": "Acme Corp",
      "slug": "acme-corp",
      "created_at": "2024-01-01T00:00:00Z",
      "metadata": {}
    }
  ]
}
```

### Get Organization

```http
GET /api/v1/organizations/{org_id}
```

**Response:**
```json
{
  "id": "org_1234567890",
  "name": "Acme Corp",
  "slug": "acme-corp",
  "created_at": "2024-01-01T00:00:00Z",
  "metadata": {},
  "quotas": {
    "events_per_month": 100000,
    "applications": 10,
    "subscriptions_per_application": 50
  },
  "usage": {
    "events_this_month": 45000,
    "applications": 3,
    "subscriptions": 15
  }
}
```

### Create Organization

```http
POST /api/v1/organizations
Content-Type: application/json
```

**Request:**
```json
{
  "name": "New Organization",
  "slug": "new-org",
  "metadata": {
    "industry": "technology"
  }
}
```

**Response:** `201 Created`
```json
{
  "id": "org_2345678901",
  "name": "New Organization",
  "slug": "new-org",
  "created_at": "2024-01-15T10:30:00Z",
  "metadata": {
    "industry": "technology"
  }
}
```

### Update Organization

```http
PUT /api/v1/organizations/{org_id}
Content-Type: application/json
```

**Request:**
```json
{
  "name": "Updated Organization Name",
  "metadata": {
    "industry": "fintech"
  }
}
```

## Applications

### List Applications

```http
GET /api/v1/applications
```

**Parameters:**
- `organization_id` (optional): Filter by organization
- `limit`: Page size (default 20, max 100)
- `cursor`: Pagination cursor

**Response:**
```json
{
  "data": [
    {
      "id": "app_1234567890",
      "organization_id": "org_1234567890",
      "name": "Payment Service",
      "description": "Handles payment processing events",
      "created_at": "2024-01-01T00:00:00Z",
      "metadata": {}
    }
  ],
  "pagination": {
    "next_cursor": "eyJpZCI6IjQ1NiJ9",
    "has_more": false
  }
}
```

### Get Application

```http
GET /api/v1/applications/{app_id}
```

**Response:**
```json
{
  "id": "app_1234567890",
  "organization_id": "org_1234567890",
  "name": "Payment Service",
  "description": "Handles payment processing events",
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-10T15:30:00Z",
  "metadata": {
    "team": "backend",
    "environment": "production"
  },
  "stats": {
    "total_events": 50000,
    "total_subscriptions": 5,
    "events_last_30_days": 15000
  }
}
```

### Create Application

```http
POST /api/v1/applications
Content-Type: application/json
```

**Request:**
```json
{
  "name": "User Service",
  "description": "Manages user lifecycle events",
  "metadata": {
    "team": "identity",
    "environment": "production"
  }
}
```

**Response:** `201 Created`

### Update Application

```http
PUT /api/v1/applications/{app_id}
Content-Type: application/json
```

### Delete Application

```http
DELETE /api/v1/applications/{app_id}
```

**Response:** `204 No Content`

## Event Types

### List Event Types

```http
GET /api/v1/applications/{app_id}/event_types
```

**Response:**
```json
{
  "data": [
    {
      "id": "et_1234567890",
      "application_id": "app_1234567890",
      "name": "user.created",
      "description": "User account created",
      "created_at": "2024-01-01T00:00:00Z",
      "deactivated_at": null,
      "metadata": {
        "category": "user_lifecycle"
      }
    }
  ]
}
```

### Get Event Type

```http
GET /api/v1/applications/{app_id}/event_types/{event_type_id}
```

### Create Event Type

```http
POST /api/v1/applications/{app_id}/event_types
Content-Type: application/json
```

**Request:**
```json
{
  "name": "order.completed",
  "description": "Order has been completed and is ready for fulfillment",
  "metadata": {
    "category": "order_lifecycle",
    "priority": "high"
  }
}
```

**Response:** `201 Created`

### Update Event Type

```http
PUT /api/v1/applications/{app_id}/event_types/{event_type_id}
Content-Type: application/json
```

### Deactivate Event Type

```http
PUT /api/v1/applications/{app_id}/event_types/{event_type_id}
Content-Type: application/json
```

**Request:**
```json
{
  "deactivated_at": "2024-01-15T10:30:00Z"
}
```

## Events

### Send Event

```http
POST /api/v1/events
Content-Type: application/json
```

**Request:**
```json
{
  "event_type": "user.created",
  "payload": {
    "user_id": "user_123",
    "email": "john.doe@example.com",
    "name": "John Doe",
    "plan": "pro",
    "created_at": "2024-01-15T10:30:00Z"
  },
  "labels": {
    "environment": "production",
    "source": "api",
    "priority": "high"
  }
}
```

**Response:** `201 Created`
```json
{
  "event_id": "evt_1234567890",
  "status": "accepted",
  "created_at": "2024-01-15T10:30:01Z"
}
```

### Send Multiple Events (Batch)

```http
POST /api/v1/events/batch
Content-Type: application/json
```

**Request:**
```json
{
  "events": [
    {
      "event_type": "user.created",
      "payload": { "user_id": "user_123" },
      "labels": { "source": "batch" }
    },
    {
      "event_type": "user.updated",
      "payload": { "user_id": "user_124" },
      "labels": { "source": "batch" }
    }
  ]
}
```

**Response:** `201 Created`
```json
{
  "events": [
    {
      "event_id": "evt_1234567890",
      "status": "accepted"
    },
    {
      "event_id": "evt_1234567891",
      "status": "accepted"
    }
  ],
  "total_accepted": 2,
  "total_rejected": 0
}
```

### List Events

```http
GET /api/v1/events
```

**Parameters:**
- `application_id`: Filter by application
- `event_type`: Filter by event type
- `since`: ISO 8601 timestamp
- `until`: ISO 8601 timestamp
- `limit`: Page size
- `cursor`: Pagination cursor

**Response:**
```json
{
  "data": [
    {
      "id": "evt_1234567890",
      "application_id": "app_1234567890",
      "event_type": "user.created",
      "payload": {
        "user_id": "user_123",
        "email": "john.doe@example.com"
      },
      "labels": {
        "environment": "production"
      },
      "created_at": "2024-01-15T10:30:00Z",
      "processed_at": "2024-01-15T10:30:01Z"
    }
  ]
}
```

### Get Event

```http
GET /api/v1/events/{event_id}
```

**Response:**
```json
{
  "id": "evt_1234567890",
  "application_id": "app_1234567890",
  "event_type": "user.created",
  "payload": {
    "user_id": "user_123",
    "email": "john.doe@example.com",
    "name": "John Doe"
  },
  "labels": {
    "environment": "production",
    "source": "api"
  },
  "created_at": "2024-01-15T10:30:00Z",
  "processed_at": "2024-01-15T10:30:01Z",
  "delivery_stats": {
    "total_attempts": 3,
    "successful_deliveries": 2,
    "failed_deliveries": 1,
    "pending_deliveries": 0
  }
}
```

## Subscriptions

### List Subscriptions

```http
GET /api/v1/applications/{app_id}/subscriptions
```

**Response:**
```json
{
  "data": [
    {
      "id": "sub_1234567890",
      "application_id": "app_1234567890",
      "description": "Slack notifications",
      "is_enabled": true,
      "event_types": ["user.created", "user.updated"],
      "target": {
        "type": "http",
        "method": "POST",
        "url": "https://hooks.slack.com/services/...",
        "headers": {
          "Content-Type": "application/json"
        }
      },
      "secret": "sub_secret_abcdef123456",
      "metadata": {
        "team": "engineering",
        "service": "slack"
      },
      "created_at": "2024-01-01T00:00:00Z"
    }
  ]
}
```

### Get Subscription

```http
GET /api/v1/applications/{app_id}/subscriptions/{sub_id}
```

### Create Subscription

```http
POST /api/v1/applications/{app_id}/subscriptions
Content-Type: application/json
```

**Request:**
```json
{
  "description": "Order notifications to shipping service",
  "event_types": ["order.created", "order.completed"],
  "target": {
    "type": "http",
    "method": "POST",
    "url": "https://shipping.company.com/webhooks/orders",
    "headers": {
      "Content-Type": "application/json",
      "Authorization": "Bearer shipping-api-key"
    }
  },
  "metadata": {
    "service": "shipping",
    "priority": "high"
  }
}
```

**Response:** `201 Created`
```json
{
  "id": "sub_2345678901",
  "application_id": "app_1234567890",
  "description": "Order notifications to shipping service",
  "is_enabled": true,
  "event_types": ["order.created", "order.completed"],
  "target": {
    "type": "http",
    "method": "POST",
    "url": "https://shipping.company.com/webhooks/orders",
    "headers": {
      "Content-Type": "application/json",
      "Authorization": "Bearer shipping-api-key"
    }
  },
  "secret": "sub_secret_xyz789",
  "created_at": "2024-01-15T10:30:00Z"
}
```

### Update Subscription

```http
PUT /api/v1/applications/{app_id}/subscriptions/{sub_id}
Content-Type: application/json
```

**Request:**
```json
{
  "description": "Updated shipping notifications",
  "is_enabled": false,
  "event_types": ["order.completed"]
}
```

### Delete Subscription

```http
DELETE /api/v1/applications/{app_id}/subscriptions/{sub_id}
```

**Response:** `204 No Content`

## Request Attempts

### List Request Attempts

```http
GET /api/v1/events/{event_id}/request_attempts
```

**Parameters:**
- `subscription_id`: Filter by subscription
- `status`: Filter by status (`pending`, `success`, `failed`)
- `since`: ISO 8601 timestamp
- `limit`: Page size

**Response:**
```json
{
  "data": [
    {
      "id": "att_1234567890",
      "event_id": "evt_1234567890",
      "subscription_id": "sub_1234567890",
      "attempt_number": 1,
      "status_code": 200,
      "response_body": "{\"status\":\"processed\"}",
      "error_message": null,
      "duration_ms": 245,
      "created_at": "2024-01-15T10:30:01Z"
    },
    {
      "id": "att_1234567891",
      "event_id": "evt_1234567890",
      "subscription_id": "sub_2345678901",
      "attempt_number": 1,
      "status_code": 500,
      "response_body": "Internal Server Error",
      "error_message": null,
      "duration_ms": 5000,
      "created_at": "2024-01-15T10:30:01Z"
    }
  ]
}
```

### Get Request Attempt

```http
GET /api/v1/request_attempts/{attempt_id}
```

### Retry Failed Attempt

```http
POST /api/v1/events/{event_id}/retry
Content-Type: application/json
```

**Request (optional):**
```json
{
  "subscription_ids": ["sub_1234567890"]
}
```

**Response:** `202 Accepted`
```json
{
  "message": "Retry queued for processing",
  "retry_id": "retry_1234567890"
}
```

## Service Tokens

### List Service Tokens

```http
GET /api/v1/service_tokens
```

**Response:**
```json
{
  "data": [
    {
      "id": "st_1234567890",
      "name": "API Integration Token",
      "description": "Token for backend API integration",
      "permissions": [
        "application:read",
        "application:write",
        "event:send"
      ],
      "applications": ["app_1234567890"],
      "created_at": "2024-01-01T00:00:00Z",
      "last_used_at": "2024-01-15T09:15:00Z",
      "expires_at": null
    }
  ]
}
```

### Create Service Token

```http
POST /api/v1/service_tokens
Content-Type: application/json
```

**Request:**
```json
{
  "name": "Webhook Integration",
  "description": "Token for webhook sending from production API",
  "permissions": [
    "event:send",
    "application:read"
  ],
  "applications": ["app_1234567890"],
  "expires_at": null
}
```

**Response:** `201 Created`
```json
{
  "id": "st_2345678901",
  "name": "Webhook Integration",
  "token": "biscuit:EoQKCAohCiEKIH0eTOWqO...",
  "permissions": [
    "event:send",
    "application:read"
  ],
  "applications": ["app_1234567890"],
  "created_at": "2024-01-15T10:30:00Z"
}
```

### Update Service Token

```http
PUT /api/v1/service_tokens/{token_id}
Content-Type: application/json
```

### Delete Service Token

```http
DELETE /api/v1/service_tokens/{token_id}
```

## Quotas and Usage

### Get Organization Quotas

```http
GET /api/v1/organizations/{org_id}/quotas
```

**Response:**
```json
{
  "quotas": {
    "events_per_month": 100000,
    "applications": 10,
    "subscriptions_per_application": 50,
    "service_tokens": 20
  },
  "usage": {
    "events_this_month": 45000,
    "applications": 3,
    "subscriptions": 15,
    "service_tokens": 5
  },
  "usage_percentage": {
    "events_per_month": 45.0,
    "applications": 30.0,
    "subscriptions": 30.0,
    "service_tokens": 25.0
  },
  "reset_date": "2024-02-01T00:00:00Z"
}
```

### Get Application Usage

```http
GET /api/v1/applications/{app_id}/usage
```

**Parameters:**
- `period`: `day`, `week`, `month`, `year` (default: `month`)
- `since`: ISO 8601 timestamp
- `until`: ISO 8601 timestamp

**Response:**
```json
{
  "period": "month",
  "since": "2024-01-01T00:00:00Z",
  "until": "2024-01-31T23:59:59Z",
  "events": {
    "total": 15000,
    "by_type": {
      "user.created": 5000,
      "user.updated": 7000,
      "user.deleted": 3000
    },
    "by_day": [
      { "date": "2024-01-01", "count": 450 },
      { "date": "2024-01-02", "count": 520 }
    ]
  },
  "deliveries": {
    "total_attempts": 45000,
    "successful": 43500,
    "failed": 1500,
    "success_rate": 96.67
  }
}
```

## Webhook Payload Format

When Hook0 delivers webhooks to your endpoints, it sends a standardized payload:

```json
{
  "event_id": "evt_1234567890",
  "event_type": "user.created",
  "payload": {
    "user_id": "user_123",
    "email": "john.doe@example.com",
    "name": "John Doe"
  },
  "labels": {
    "environment": "production",
    "source": "api"
  },
  "timestamp": "2024-01-15T10:30:00Z"
}
```

### Webhook Headers

Hook0 includes these headers with every webhook delivery:

```http
Content-Type: application/json
Hook0-Signature: sha256=a1b2c3d4e5f6...
Hook0-Event-Type: user.created
Hook0-Event-Id: evt_1234567890
Hook0-Attempt: 1
User-Agent: Hook0/1.0
```

### Signature Verification

Verify webhook authenticity using the `Hook0-Signature` header:

```javascript
const crypto = require('crypto');

function verifySignature(payload, signature, secret) {
  const expectedSignature = crypto
    .createHmac('sha256', secret)
    .update(payload)
    .digest('hex');
  
  return signature === `sha256=${expectedSignature}`;
}
```

## WebSocket API

Hook0 provides a WebSocket API for real-time event streaming:

```javascript
const ws = new WebSocket('wss://api.hook0.com/ws');

ws.on('open', () => {
  // Authenticate
  ws.send(JSON.stringify({
    type: 'auth',
    token: 'biscuit:YOUR_TOKEN_HERE'
  }));
  
  // Subscribe to events
  ws.send(JSON.stringify({
    type: 'subscribe',
    application_id: 'app_1234567890',
    event_types: ['user.created', 'user.updated']
  }));
});

ws.on('message', (data) => {
  const message = JSON.parse(data);
  
  if (message.type === 'event') {
    console.log('Real-time event:', message.data);
  }
});
```

## SDKs and Libraries

Hook0 provides official SDKs for popular programming languages:

### JavaScript/TypeScript

```bash
npm install @hook0/sdk
```

```javascript
import { Hook0 } from '@hook0/sdk';

const hook0 = new Hook0({
  token: 'biscuit:YOUR_TOKEN_HERE',
  baseURL: 'https://api.hook0.com'
});

// Send an event
await hook0.events.send({
  event_type: 'user.created',
  payload: { user_id: 'user_123' }
});
```

### Rust

```toml
[dependencies]
hook0 = "0.1"
```

```rust
use hook0::{Hook0Client, Event};

let client = Hook0Client::new("biscuit:YOUR_TOKEN_HERE");

let event = Event {
    event_type: "user.created".to_string(),
    payload: serde_json::json!({
        "user_id": "user_123"
    }),
    labels: None,
};

client.send_event(event).await?;
```

### Python (Community)

```bash
pip install hook0-python
```

```python
from hook0 import Hook0Client

client = Hook0Client(token="biscuit:YOUR_TOKEN_HERE")

client.send_event(
    event_type="user.created",
    payload={"user_id": "user_123"}
)
```

## API Limits and Guidelines

### Request Size Limits
- **Event payload**: 1MB maximum
- **Batch events**: 100 events per request, 10MB total
- **Request body**: 10MB maximum

### Performance Guidelines
- Use batch endpoints for sending multiple events
- Implement exponential backoff for retries
- Cache application and subscription data
- Use pagination for large result sets

### Best Practices
- Always verify webhook signatures
- Implement idempotent webhook handlers
- Use meaningful event types and structured payloads
- Include relevant labels for filtering and debugging
- Monitor your webhook endpoint performance

---

For more examples and advanced usage, see our [SDK Documentation](./sdk/) and [Integration Guides](../how-to-guides/).