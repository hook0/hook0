# API Reference

Hook0 provides a comprehensive REST API for managing webhooks, events, and integrations. This reference covers all endpoints, request/response formats, and authentication methods.

:::tip Interactive API Explorer
For a fully interactive experience with request/response examples, try our API in the [Hook0 App](https://app.hook0.com) or use our [Postman Collection](https://www.postman.com/hook0-team/workspace/hook0-api).
:::

## Base URL

import Tabs from '@theme/Tabs';
import TabItem from '@theme/TabItem';

<Tabs>
  <TabItem value="production" label="Production" default>

```
https://app.hook0.com/api/v1
```

  </TabItem>
  <TabItem value="staging" label="Staging">

```
https://staging.hook0.com/api/v1
```

  </TabItem>
  <TabItem value="self-hosted" label="Self-hosted">

```
https://your-domain.com/api/v1
```

  </TabItem>
</Tabs>

## Authentication

Hook0 uses Biscuit tokens for API authentication. Include your token in the Authorization header:

<Tabs>
  <TabItem value="http" label="HTTP" default>

```http
Authorization: Bearer biscuit:YOUR_TOKEN_HERE
```

  </TabItem>
  <TabItem value="curl" label="cURL">

```bash
curl -H "Authorization: Bearer biscuit:YOUR_TOKEN_HERE" \
     https://app.hook0.com/api/v1/events
```

  </TabItem>
  <TabItem value="javascript" label="JavaScript">

```javascript
const response = await fetch('https://app.hook0.com/api/v1/events', {
  headers: {
    'Authorization': 'Bearer biscuit:YOUR_TOKEN_HERE',
    'Content-Type': 'application/json'
  }
});
```

  </TabItem>
  <TabItem value="python" label="Python">

```python
import requests

headers = {
    'Authorization': 'Bearer biscuit:YOUR_TOKEN_HERE',
    'Content-Type': 'application/json'
}
response = requests.get('https://app.hook0.com/api/v1/events', headers=headers)
```

  </TabItem>
  <TabItem value="rust" label="Rust">

```rust
let client = reqwest::Client::new();
let response = client
    .get("https://app.hook0.com/api/v1/events")
    .header("Authorization", "Bearer biscuit:YOUR_TOKEN_HERE")
    .send()
    .await?;
```

  </TabItem>
</Tabs>

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
GET /api/v1/event_types
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
GET /api/v1/event_types/{event_type_id}
```

### Create Event Type

```http
POST /api/v1/event_types
Content-Type: application/json
```

**Request:**
```json
{
  "application_id": "app_1234567890",
  "service": "order",
  "resource_type": "purchase",
  "verb": "completed"
}
```

**Response:** `201 Created`

### Update Event Type

```http
PUT /api/v1/event_types/{event_type_id}
Content-Type: application/json
```

### Deactivate Event Type

```http
PUT /api/v1/event_types/{event_type_id}
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
POST /api/v1/event
Content-Type: application/json
```

**Request:**
```json
{
  "application_id": "app_1234567890",
  "event_id": "550e8400-e29b-41d4-a716-446655440000",
  "event_type": "user.created",
  "payload": "{\"user_id\":\"user_123\",\"email\":\"john.doe@example.com\",\"name\":\"John Doe\",\"plan\":\"pro\",\"created_at\":\"2024-01-15T10:30:00Z\"}",
  "payload_content_type": "application/json",
  "occurred_at": "2024-01-15T10:30:00Z",
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

:::note Batch Events Not Available
The batch events endpoint is not currently implemented. Please send events individually using the single event endpoint above.
:::

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
GET /api/v1/subscriptions
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
GET /api/v1/subscriptions/{sub_id}
```

### Create Subscription

```http
POST /api/v1/subscriptions
Content-Type: application/json
```

**Request:**
```json
{
  "application_id": "app_1234567890",
  "is_enabled": true,
  "description": "Order notifications to shipping service",
  "event_types": ["order.purchase.created", "order.purchase.completed"],
  "label_key": "environment",
  "label_value": "production",
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
PUT /api/v1/subscriptions/{sub_id}
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
DELETE /api/v1/subscriptions/{sub_id}
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

### Replay Event

```http
POST /api/v1/events/{event_id}/replay
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
  "message": "Event replay queued for processing",
  "replay_id": "replay_1234567890"
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

:::note Quotas and Usage Endpoints Not Available
The quotas and usage tracking endpoints are not currently implemented. Usage information is available in the organization details endpoint.
:::

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

:::note WebSocket API Not Available
Real-time event streaming via WebSocket is not currently implemented. Please use the REST API endpoints for event management.
:::

## SDKs and Libraries

Hook0 provides official SDKs for popular programming languages. Choose your preferred language:

<Tabs>
  <TabItem value="javascript" label="JavaScript/TypeScript" default>

**Installation**
```bash
npm install @hook0/sdk
```

**Usage**
```javascript
import { Hook0 } from '@hook0/sdk';

const hook0 = new Hook0({
  token: 'biscuit:YOUR_TOKEN_HERE',
  baseURL: 'https://app.hook0.com'
});

// Send an event
await hook0.events.send({
  event_type: 'user.created',
  payload: { 
    user_id: 'user_123',
    email: 'john@example.com'
  }
});

// List applications
const apps = await hook0.applications.list();
    
    // Create a subscription
    await hook0.subscriptions.create('app_123', {
      description: 'User events webhook',
      event_types: ['user.created', 'user.updated'],
      target: {
        type: 'http',
        method: 'POST',
        url: 'https://your-app.com/webhooks'
      }
});
```

  </TabItem>
  <TabItem value="rust" label="Rust">

**Installation**
```toml
[dependencies]
hook0 = "0.1"
tokio = { version = "1", features = ["full"] }
serde_json = "1.0"
```

**Usage**
```rust
use hook0::{Hook0Client, Event};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Hook0Client::new("biscuit:YOUR_TOKEN_HERE");

    // Send an event
    let event = Event {
        event_type: "user.created".to_string(),
        payload: json!({
            "user_id": "user_123",
            "email": "john@example.com"
        }),
        labels: Some(json!({
            "environment": "production"
        })),
    };

    client.send_event(event).await?;

    // List applications
    let apps = client.applications().list().await?;
    
    Ok(())
}
```

  </TabItem>
  <TabItem value="curl" label="cURL">
    **Send an Event**
    ```bash
    curl -X POST https://app.hook0.com/api/v1/event \
      -H "Authorization: Bearer biscuit:YOUR_TOKEN_HERE" \
      -H "Content-Type: application/json" \
      -d '{
        "event_type": "user.created",
        "payload": {
          "user_id": "user_123",
          "email": "john@example.com"
        },
        "labels": {
          "environment": "production"
        }
      }'
    ```
    
    **List Applications**
    ```bash
    curl -X GET https://app.hook0.com/api/v1/applications \
      -H "Authorization: Bearer biscuit:YOUR_TOKEN_HERE"
    ```
    
    **Create Subscription**
    ```bash
    curl -X POST https://app.hook0.com/api/v1/applications/app_123/subscriptions \
      -H "Authorization: Bearer biscuit:YOUR_TOKEN_HERE" \
      -H "Content-Type: application/json" \
      -d '{
        "description": "User events webhook",
        "event_types": ["user.created", "user.updated"],
        "target": {
          "type": "http",
          "method": "POST", 
          "url": "https://your-app.com/webhooks"
        }
      }'
    ```

  </TabItem>
</Tabs>

:::note Additional SDKs Coming Soon
Official SDKs for Python, Go, and other languages are planned for future releases. Currently, TypeScript/JavaScript and Rust SDKs are available.
:::

## OpenAPI Specification

The complete OpenAPI 3.0 specification for the Hook0 API is available at:

- **Production**: `https://app.hook0.com/api/v1/swagger.json`
- **Self-hosted**: `https://your-domain.com/api/v1/swagger.json`

You can use this specification to:
- Generate client SDKs for any language
- Import into API testing tools like Postman or Insomnia
- Integrate with API documentation tools
- Set up automated API testing

For the complete OpenAPI specification, visit `https://app.hook0.com/api/v1/swagger.json`.

## API Limits and Guidelines

### Request Size Limits
- **Event payload**: 1MB maximum
- **Request body**: 10MB maximum

### Performance Guidelines
- Send events individually (batch endpoint not yet available)
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

For more examples and advanced usage, see our [SDK Documentation](sdk/) and [Integration Guides](../how-to-guides/index.md).