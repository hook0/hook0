---
title: "Hook0 SDKs — JavaScript, Python, Rust"
description: "Client libraries that wrap the Hook0 REST API. Each SDK handles auth and sends webhook events in under 10 lines of code. Available for JS/TS, Python, and Rust."
keywords: [Hook0 SDK, webhook SDK, JavaScript webhook library, Python webhook client, Rust webhook SDK, webhook API client]
---

# SDKs & client libraries

Hook0 has official SDKs for several programming languages. They share the same design patterns but stay idiomatic to each language.

## Set up environment variables

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

## Official SDKs

### [JavaScript/TypeScript SDK](javascript.md)
TypeScript SDK for Node.js with event sending.

**Features:**
- Event sending with type definitions
- Event type management (upsert)
- Webhook signature verification
- Node.js compatibility

**Installation:**
```bash
npm install hook0-client
```

**Quick start:**
```typescript
import { Hook0Client, Event } from 'hook0-client';

const hook0 = new Hook0Client(
  'http://localhost:8081',
  'app_1234567890',
  '{YOUR_TOKEN}'
);

const event = new Event(
  'user.account.created',
  JSON.stringify({ user_id: 123 }),
  'application/json',
  { source: 'api' }
);

await hook0.sendEvent(event);
```

[View full documentation](javascript.md)

---

### [Rust SDK](rust.md)
Native Rust SDK for sending events and verifying webhook signatures.

**Features:**
- Event sending with typed payloads
- Event type management (upsert)
- Webhook signature verification (v0 and v1)
- Optional feature flags (`producer`, `consumer`)

**Installation:**
```toml
[dependencies]
hook0-client = "1"
```

**Quick start:**
```rust
use hook0_client::{Hook0Client, Event};
use reqwest::Url;
use uuid::Uuid;
use std::borrow::Cow;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_url = Url::parse("http://localhost:8081/api/v1")?;
    let application_id = Uuid::parse_str("your-app-id-here")?;
    let client = Hook0Client::new(api_url, application_id, "{YOUR_TOKEN}")?;

    let event = Event {
        event_id: &None,
        event_type: "user.account.created",
        payload: Cow::Borrowed(r#"{"user_id": "123"}"#),
        payload_content_type: "application/json",
        metadata: None,
        occurred_at: None,
        labels: vec![("environment".to_string(), "production".to_string())],
    };

    let event_id = client.send_event(&event).await?;
    println!("Event sent: {}", event_id);
    Ok(())
}
```

[View full documentation](rust.md)

---

## Additional language support

:::info SDKs coming soon
Official SDKs for Python, Go, PHP, Ruby, Java, and .NET are planned. In the meantime, use the REST API directly with your language's HTTP client.
:::

### Using the REST API

Everything Hook0 does is available through the REST API. Here is how to send events in a few languages:

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
        'event_type': 'user.account.created',
        'payload': {'user_id': 123}
    }
)
```

**Go Example:**
```go
import "net/http"
import "encoding/json"

event := map[string]interface{}{
    "event_type": "user.account.created",
    "payload": map[string]interface{}{"user_id": 123},
}

data, _ := json.Marshal(event)
req, _ := http.NewRequest("POST", "https://app.hook0.com/api/v1/event", bytes.NewBuffer(data))
req.Header.Set("Authorization", "Bearer {YOUR_TOKEN}")
req.Header.Set("Content-Type", "application/json")

client := &http.Client{}
resp, _ := client.Do(req)
```

## Core SDK features

All official Hook0 SDKs provide:

### Authentication and security
- Authentication via Biscuit tokens (user sessions) and Service tokens (programmatic access)
- Webhook signature verification
- TLS/SSL support
- Secure credential management

### API operations
- Event sending (single events)
- Application management (via REST API)
- Subscription CRUD operations (via REST API)
- Event type management
- Delivery status tracking (via REST API)

### Developer experience
- Type safety and auto-completion
- Error handling with typed errors
- Structured logging
- Documentation with examples

## Common usage patterns

### Sending events

```typescript
// JavaScript/TypeScript
const event = new Event(
  'order.checkout.completed',
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
curl -X POST $HOOK0_API/event \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "event_type": "order.checkout.completed",
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

### Managing subscriptions

```bash
# Using the REST API
curl -X POST $HOOK0_API/subscriptions \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "description": "Order Events",
    "event_types": ["order.checkout.completed", "order.shipped"],
    "target": {
      "type": "http",
      "url": "https://api.example.com/webhooks",
      "method": "POST"
    }
  }'
```

### Webhook verification

```typescript
// JavaScript/TypeScript
import { verifyWebhookSignature } from 'hook0-client';

// Note: Express.js normalizes all header names to lowercase
// Capture raw body for signature verification
app.post('/webhook', express.json({
  verify: (req, res, buf) => { req.rawBody = buf; }
}), (req, res) => {
  const signature = req.headers['x-hook0-signature'];
  // Use raw body for signature verification, not JSON.stringify(req.body)
  const rawBodyString = req.rawBody.toString('utf8');
  const headers = new Headers();
  Object.entries(req.headers).forEach(([key, value]) => {
    if (typeof value === 'string') headers.set(key, value);
  });

  try {
    const isValid = verifyWebhookSignature(
      signature,
      rawBodyString,
      headers,
      secret,
      300 // 5-minute tolerance
    );

    if (!isValid) {
      return res.status(401).json({ error: 'Invalid signature' });
    }
    // Process webhook (already parsed via req.body)...
  } catch (error) {
    return res.status(401).json({ error: 'Invalid signature' });
  }
});
```

## Error handling

SDKs return typed errors you can match on:

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

### TypeScript SDK configuration
```typescript
const hook0 = new Hook0Client(
  'http://localhost:8081',     // API URL
  'app_1234567890',            // Application ID
  '{YOUR_TOKEN}',   // Authentication token
  false                        // Debug mode (optional)
);
```

### REST API configuration
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

## SDK development guidelines

Want to contribute an SDK? Here is what we expect:

### Requirements checklist

- [ ] All essential endpoints implemented
- [ ] Biscuit token and Service token support
- [ ] Typed error messages
- [ ] >80% test coverage
- [ ] API docs with examples
- [ ] Working example applications
- [ ] Automated testing and publishing

### Best practices

1. Write idiomatic code for your language
2. Provide type definitions where possible
3. Implement async/await or promises
4. Handle connection pooling and cleanup
5. Follow semantic versioning
6. Maintain a changelog

## Getting help

### Documentation
- [API Reference](../../openapi/intro) - REST API documentation
- [Tutorials](../../tutorials/) - Step-by-step guides
- [How-to Guides](../../how-to-guides/) - Problem-solving guides

### Support channels
- GitHub Issues - SDK-specific issue tracking
- [Discord](https://www.hook0.com/community) - Community support
- [Stack Overflow](https://stackoverflow.com/questions/tagged/hook0) - #hook0 tag
- support@hook0.com - For critical issues

### Contributing

To contribute to our SDK:

1. Open an issue to discuss your proposal
2. Follow the requirements checklist above
3. Submit for review
4. Enjoy :)
