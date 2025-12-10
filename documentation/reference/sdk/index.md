---
title: SDKs & Client Libraries
description: Official Hook0 client libraries for multiple programming languages
---

# SDKs & Client Libraries

Hook0 provides official SDKs to make integration seamless across different programming languages and platforms. All SDKs follow consistent design patterns while remaining idiomatic to their respective languages.

## Official SDKs

### 🟨 [JavaScript/TypeScript SDK](javascript.md)
TypeScript SDK for Node.js applications with basic event sending capabilities.

**Features:**
- Event sending with type definitions
- Event type management (upsert)
- Webhook signature verification
- Node.js compatibility

**Installation:**
```bash
npm install hook0-client
```

**Quick Start:**
```typescript
import { Hook0Client, Event } from 'hook0-client';

const hook0 = new Hook0Client(
  'http://localhost:8081',
  'app_1234567890',
  '{YOUR_TOKEN}'
);

const event = new Event(
  'users.account.created',
  JSON.stringify({ user_id: 123 }),
  'application/json',
  { source: 'api' }
);

await hook0.sendEvent(event);
```

[📖 View Full Documentation →](javascript.md)

---

### 🦀 [Rust SDK](rust.md)
Rust SDK currently in development. Use the REST API directly with reqwest.

**Current Status:**
- 🚧 SDK interface designed but not yet implemented
- ✅ REST API fully accessible via HTTP clients
- 📝 Complete documentation for API usage

**Temporary Solution:**
```rust
use reqwest::Client;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let response = client
        .post("https://app.hook0.com/api/v1/event")
        .header("Authorization", "Bearer {YOUR_TOKEN}")
        .json(&json!({
            "event_type": "users.account.created",
            "payload": { "user_id": 123 }
        }))
        .send()
        .await?;
    Ok(())
}
```

[📖 View Full Documentation →](rust.md)

---

## Additional Language Support

:::info SDKs Coming Soon
Official SDKs for Python, Go, PHP, Ruby, Java, and .NET are planned for future releases. In the meantime, you can use the REST API directly with your language's HTTP client.
:::

### Using the REST API

All Hook0 functionality is available through the REST API. Here's how to send events in various languages:

**Python Example:**
```python
import requests

response = requests.post(
    'http://localhost:8081',
    headers={
        'Authorization': 'Bearer {YOUR_TOKEN}',
        'Content-Type': 'application/json'
    },
    json={
        'event_type': 'users.account.created',
        'payload': {'user_id': 123}
    }
)
```

**Go Example:**
```go
import "net/http"
import "encoding/json"

event := map[string]interface{}{
    "event_type": "users.account.created",
    "payload": map[string]interface{}{"user_id": 123},
}

data, _ := json.Marshal(event)
req, _ := http.NewRequest("POST", "https://app.hook0.com/api/v1/event", bytes.NewBuffer(data))
req.Header.Set("Authorization", "Bearer {YOUR_TOKEN}")
req.Header.Set("Content-Type", "application/json")

client := &http.Client{}
resp, _ := client.Do(req)
```

## Core SDK Features

All official Hook0 SDKs provide:

### 🔐 Authentication & Security
- Biscuit token authentication
- Webhook signature verification
- TLS/SSL support
- Secure credential management

### 📡 API Operations
- Event sending (single events)
- Application management (via REST API)
- Subscription CRUD operations (via REST API)
- Event type management
- Delivery status tracking (via REST API)

### 🔄 Reliability
- Automatic retry with exponential backoff
- Circuit breaker pattern
- Connection pooling
- Rate limit handling
- Timeout configuration

### 🛠️ Developer Experience
- Type safety and auto-completion
- Comprehensive error handling
- Structured logging
- Mock clients for testing
- Extensive documentation

## Common Usage Patterns

### Sending Events

```typescript
// JavaScript/TypeScript
const event = new Event(
  'orders.checkout.completed',
  JSON.stringify({
    order_id: 'ord_123',
    total: 99.99
  }),
  'application/json',
  {
    environment: 'production',
    region: 'us-west'
  }
);

await hook0.sendEvent(event);
```

```bash
# Using cURL
curl -X POST http://localhost:8081/api/v1/event \
  -H "Authorization: Bearer {YOUR_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{
    "event_type": "orders.checkout.completed",
    "payload": {
      "order_id": "ord_123",
      "total": 99.99
    },
    "labels": {
      "environment": "production",
      "region": "us-west"
    }
  }'
```

### Managing Subscriptions

```bash
# Using the REST API
curl -X POST http://localhost:8081/api/v1/subscriptions \
  -H "Authorization: Bearer {YOUR_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{
    "description": "Order Events",
    "event_types": ["orders.checkout.completed", "order.shipped"],
    "target": {
      "type": "http",
      "url": "https://api.example.com/webhooks",
      "method": "POST"
    }
  }'
```

### Webhook Verification

```typescript
// JavaScript/TypeScript
import { verifyWebhookSignature } from 'hook0-client';

// Note: Express.js normalizes all header names to lowercase
app.post('/webhook', express.json(), (req, res) => {
  const signature = req.headers['x-hook0-signature'];
  const headers = new Headers();
  Object.entries(req.headers).forEach(([key, value]) => {
    if (typeof value === 'string') headers.set(key, value);
  });

  try {
    const isValid = verifyWebhookSignature(
      signature,
      JSON.stringify(req.body),
      headers,
      secret,
      300 // 5-minute tolerance
    );

    if (!isValid) {
      return res.status(401).json({ error: 'Invalid signature' });
    }
    // Process webhook (already parsed)...
  } catch (error) {
    return res.status(401).json({ error: 'Invalid signature' });
  }
});
```

## Error Handling

SDKs provide error handling capabilities:

```typescript
// JavaScript/TypeScript
import { Hook0ClientError } from 'hook0-client';

try {
  const eventId = await hook0.sendEvent(event);
} catch (error) {
  if (error instanceof Hook0ClientError) {
    console.error('Hook0 error:', error.message);
    
    if (error.message.includes('Invalid event type')) {
      // Handle invalid event type format
    } else if (error.message.includes('failed')) {
      // Retry logic
    }
  }
}
```

```javascript
// REST API error handling
fetch('http://localhost:8081', {
  method: 'POST',
  headers: {
    'Authorization': 'Bearer {YOUR_TOKEN}',
    'Content-Type': 'application/json'
  },
  body: JSON.stringify(event)
})
.then(response => {
  if (response.status === 429) {
    // Rate limited - check X-RateLimit-Reset header
    const resetTime = response.headers.get('X-RateLimit-Reset');
    // Implement retry logic
  } else if (!response.ok) {
    // Handle other errors
  }
});
```

## Configuration

### TypeScript SDK Configuration
```typescript
const hook0 = new Hook0Client(
  'http://localhost:8081',     // API URL
  'app_1234567890',            // Application ID
  '{YOUR_TOKEN}',   // Authentication token
  false                        // Debug mode (optional)
);
```

### REST API Configuration
When using the REST API directly, configure your HTTP client:

```javascript
// Example with axios
const apiClient = axios.create({
  baseURL: 'http://localhost:8081',
  headers: {
    'Authorization': 'Bearer {YOUR_TOKEN}',
    'Content-Type': 'application/json'
  },
  timeout: 30000
});
```

## Testing

### Testing with TypeScript SDK

```typescript
// Mock fetch for testing
import { Hook0Client, Event } from 'hook0-client';
import { jest } from '@jest/globals';

test('should send event', async () => {
  global.fetch = jest.fn().mockResolvedValueOnce({
    ok: true,
    text: async () => ''
  });
  
  const client = new Hook0Client(
    'http://localhost:8081',
    'app_test',
    'test_token'
  );
  
  const event = new Event(
    'test.event',
    JSON.stringify({ test: true }),
    'application/json',
    {}
  );
  
  await client.sendEvent(event);
  
  expect(fetch).toHaveBeenCalledWith(
    'http://localhost:8081',
    expect.objectContaining({
      method: 'POST'
    })
  );
});
```

## SDK Development Guidelines

Want to contribute an SDK? Follow these guidelines:

### Requirements Checklist

- [ ] **Core API Coverage** - All essential endpoints implemented
- [ ] **Authentication** - Biscuit token support
- [ ] **Error Handling** - Proper error types and messages
- [ ] **Retry Logic** - Exponential backoff implementation
- [ ] **Testing** - >80% test coverage
- [ ] **Documentation** - Complete API docs with examples
- [ ] **Examples** - Working example applications
- [ ] **CI/CD** - Automated testing and publishing

### Best Practices

1. **Follow Language Idioms** - Write idiomatic code for your language
2. **Type Safety** - Provide type definitions where possible
3. **Async Support** - Implement async/await or promises
4. **Resource Management** - Proper connection pooling and cleanup
5. **Versioning** - Follow semantic versioning
6. **Changelog** - Maintain detailed changelog

## Getting Help

### Documentation
- [API Reference](../../openapi/intro) - Complete REST API documentation
- [Tutorials](../../tutorials/) - Step-by-step guides
- [How-to Guides](../../how-to-guides/) - Problem-solving guides

### Support Channels
- **GitHub Issues** - SDK-specific issue tracking
- **Discord** - [Community support](https://www.hook0.com/community)
- **Stack Overflow** - [#hook0 tag](https://stackoverflow.com/questions/tagged/hook0)
- **Email** - support@hook0.com for critical issues

### Contributing

We welcome SDK contributions! To get started:

1. Read our SDK Development Guide
2. Open an issue to discuss your SDK proposal
3. Follow the requirements checklist above
4. Submit your SDK for review

---

*Choose your preferred language and start integrating Hook0 today!*