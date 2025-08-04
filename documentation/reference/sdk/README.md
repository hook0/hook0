# SDK Documentation

Hook0 provides official SDKs for popular programming languages to make integration simple and idiomatic. Each SDK includes type definitions, comprehensive error handling, and built-in retry logic.

## Available SDKs

### Official SDKs

| Language | Package | Documentation | Status |
|----------|---------|---------------|--------|
| [JavaScript/TypeScript](./javascript.md) | `@hook0/sdk` | [NPM](https://www.npmjs.com/package/@hook0/sdk) | ✅ Stable |
| [Rust](./rust.md) | `hook0` | [Crates.io](https://crates.io/crates/hook0) | ✅ Stable |

### Community SDKs

| Language | Package | Repository | Maintainer |
|----------|---------|------------|------------|
| Python | `hook0-python` | [GitHub](https://github.com/hook0-community/python-sdk) | Community |
| Go | `go-hook0` | [GitHub](https://github.com/hook0-community/go-sdk) | Community |
| PHP | `hook0-php` | [GitHub](https://github.com/hook0-community/php-sdk) | Community |
| Ruby | `hook0-ruby` | [GitHub](https://github.com/hook0-community/ruby-sdk) | Community |

## Quick Start

### JavaScript/TypeScript

```bash
npm install @hook0/sdk
```

```typescript
import { Hook0 } from '@hook0/sdk';

const hook0 = new Hook0({
  token: 'biscuit:YOUR_TOKEN_HERE',
  baseURL: 'https://api.hook0.com'
});

// Send an event
await hook0.events.send({
  event_type: 'user.created',
  payload: {
    user_id: 'user_123',
    email: 'john.doe@example.com'
  }
});
```

### Rust

```toml
[dependencies]
hook0 = "0.1"
tokio = { version = "1.0", features = ["full"] }
```

```rust
use hook0::{Hook0Client, Event};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Hook0Client::new("biscuit:YOUR_TOKEN_HERE");
    
    let event = Event {
        event_type: "user.created".to_string(),
        payload: serde_json::json!({
            "user_id": "user_123",
            "email": "john.doe@example.com"
        }),
        labels: None,
    };
    
    client.send_event(event).await?;
    Ok(())
}
```

## SDK Features

### Core Features

All official SDKs include:

- **Type Safety**: Full type definitions for all API operations
- **Error Handling**: Comprehensive error types and handling
- **Retry Logic**: Automatic retries with exponential backoff
- **Rate Limiting**: Built-in rate limit handling
- **Authentication**: Seamless Biscuit token authentication
- **Async Support**: Native async/await support where applicable

### Advanced Features

- **Batch Operations**: Send multiple events efficiently
- **Webhook Verification**: Verify incoming webhook signatures
- **Real-time Streaming**: WebSocket support for real-time events
- **Middleware Support**: Extensible middleware system
- **Configuration**: Flexible configuration options
- **Logging**: Structured logging integration

## Common Patterns

### Event Sending

```typescript
// Single event
await hook0.events.send({
  event_type: 'user.created',
  payload: { user_id: 'user_123' }
});

// Batch events
await hook0.events.sendBatch([
  {
    event_type: 'user.created',
    payload: { user_id: 'user_123' }
  },
  {
    event_type: 'user.updated', 
    payload: { user_id: 'user_124' }
  }
]);
```

### Application Management

```typescript
// Create application
const app = await hook0.applications.create({
  name: 'My Application',
  description: 'Application for webhook events'
});

// List applications
const apps = await hook0.applications.list();

// Get application details
const appDetails = await hook0.applications.get(app.id);
```

### Subscription Management

```typescript
// Create subscription
const subscription = await hook0.subscriptions.create(app.id, {
  event_types: ['user.created', 'user.updated'],
  target: {
    type: 'http',
    method: 'POST',
    url: 'https://api.example.com/webhooks',
    headers: {
      'Authorization': 'Bearer webhook-token'
    }
  },
  description: 'User events webhook'
});

// List subscriptions
const subscriptions = await hook0.subscriptions.list(app.id);
```

### Error Handling

```typescript
import { Hook0Error, RateLimitError, ValidationError } from '@hook0/sdk';

try {
  await hook0.events.send({
    event_type: 'user.created',
    payload: { user_id: 'user_123' }
  });
} catch (error) {
  if (error instanceof RateLimitError) {
    console.log(`Rate limited. Retry after: ${error.retryAfter}s`);
    // Implement backoff strategy
  } else if (error instanceof ValidationError) {
    console.log('Validation error:', error.details);
  } else if (error instanceof Hook0Error) {
    console.log('Hook0 API error:', error.message);
  } else {
    console.log('Unexpected error:', error);
  }
}
```

### Webhook Verification

```typescript
import { verifyWebhookSignature } from '@hook0/sdk';

// Express.js middleware
app.post('/webhook', express.raw({ type: 'application/json' }), (req, res) => {
  const signature = req.headers['hook0-signature'];
  const secret = 'your-subscription-secret';
  
  if (!verifyWebhookSignature(req.body, signature, secret)) {
    return res.status(401).json({ error: 'Invalid signature' });
  }
  
  const payload = JSON.parse(req.body);
  console.log('Verified webhook:', payload);
  
  res.json({ status: 'processed' });
});
```

## Configuration

### Client Configuration

```typescript
const hook0 = new Hook0({
  // Authentication
  token: 'biscuit:YOUR_TOKEN_HERE',
  
  // API endpoint
  baseURL: 'https://api.hook0.com',
  
  // Request timeout
  timeout: 30000,
  
  // Retry configuration
  retry: {
    attempts: 3,
    delay: 1000,
    backoff: 'exponential'
  },
  
  // Rate limiting
  rateLimit: {
    enabled: true,
    maxRequests: 100,
    windowMs: 60000
  },
  
  // Logging
  logger: {
    level: 'info',
    format: 'json'
  }
});
```

### Environment Variables

SDKs support configuration via environment variables:

```bash
HOOK0_TOKEN=biscuit:YOUR_TOKEN_HERE
HOOK0_BASE_URL=https://api.hook0.com
HOOK0_TIMEOUT=30000
HOOK0_RETRY_ATTEMPTS=3
HOOK0_LOG_LEVEL=info
```

## Testing

### Mocking

```typescript
import { Hook0, MockHook0Client } from '@hook0/sdk';

// Use mock client for testing
const mockClient = new MockHook0Client();
mockClient.events.send.mockResolvedValue({ event_id: 'evt_123' });

// Test your code
const result = await yourFunction(mockClient);
expect(result).toBe('expected_value');
```

### Test Helpers

```typescript
import { createTestEvent, createTestSubscription } from '@hook0/sdk/testing';

// Create test fixtures
const testEvent = createTestEvent({
  event_type: 'user.created',
  payload: { user_id: 'test_user' }
});

const testSubscription = createTestSubscription({
  event_types: ['user.created'],
  target: { url: 'https://test.example.com/webhook' }
});
```

## Migration Guides

### Upgrading from v0.x to v1.x

```typescript
// v0.x
import Hook0 from 'hook0-sdk';
const client = new Hook0('your-token');

// v1.x
import { Hook0 } from '@hook0/sdk';
const client = new Hook0({ token: 'biscuit:your-token' });
```

### Breaking Changes

- Token format changed to include `biscuit:` prefix
- Client constructor now takes configuration object
- Error types have been restructured
- Some method names have changed for consistency

See individual SDK documentation for detailed migration guides.

## Contributing

### Adding a New SDK

To create a new SDK for Hook0:

1. **Follow API Standards**: Implement all core API endpoints
2. **Error Handling**: Map all error codes to appropriate exceptions
3. **Type Definitions**: Provide complete type definitions
4. **Testing**: Include comprehensive test coverage
5. **Documentation**: Provide clear documentation and examples
6. **CI/CD**: Set up automated testing and publishing

### SDK Requirements

All SDKs should implement:

- [ ] Authentication with Biscuit tokens
- [ ] All core API endpoints (events, applications, subscriptions)
- [ ] Error handling with proper error types
- [ ] Retry logic with exponential backoff
- [ ] Rate limit handling
- [ ] Webhook signature verification
- [ ] Comprehensive test coverage
- [ ] Documentation with examples

### Submission Process

1. Create the SDK following our guidelines
2. Submit for review via GitHub issue
3. Community testing period
4. Official adoption (if approved)

## Support

### Getting Help

- **Documentation**: Check individual SDK documentation
- **GitHub Issues**: Report issues on respective repositories
- **Discord**: Join our community for questions
- **Support**: Contact support for critical issues

### SDK-Specific Support

Each SDK has its own support channels:

- **JavaScript/TypeScript**: [GitHub Issues](https://github.com/hook0/hook0-js/issues)
- **Rust**: [GitHub Issues](https://github.com/hook0/hook0-rust/issues)
- **Community SDKs**: Check individual repository issues

---

Choose your preferred SDK and follow the detailed documentation for implementation examples and best practices.