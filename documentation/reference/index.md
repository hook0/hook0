# Reference

Comprehensive technical specifications for Hook0. Use this reference when you need exact details about APIs, configuration options, data formats, or error codes.

## Quick Navigation

| I want to... | Go to |
|--------------|-------|
| Integrate with the API | [API Reference](../openapi/intro) |
| Use a client library | [SDKs](#sdks) |
| Configure Hook0 | [Configuration](configuration.md) |
| Understand an error | [Error Codes](error-codes.md) |
| Filter events with labels | [Event Labels](labels.md) |
| Define event payloads | [Event Schemas](event-schemas.md) |
| Use AI assistants | [MCP Server](mcp-for-ia-assistant.md) |
| Learn terminology | [Glossary](glossary.md) |

---

## API Reference

### [REST API Documentation](../openapi/intro)

Complete API specification with all endpoints, request/response formats, and authentication details.

**Available formats:**
- [OpenAPI 3.0 Specification](../openapi/intro) — Machine-readable, import into tools
- Interactive Swagger UI — Available at `/docs` on your Hook0 instance

**Key endpoints:**

| Resource | Operations |
|----------|------------|
| Organizations | List, get |
| Applications | Create, list, get, delete |
| Event Types | Create, list, get, delete |
| Subscriptions | Create, list, get, update, delete |
| Events | Ingest, list, get |
| Request Attempts | List, get, retry |

---

## SDKs

Client libraries for integrating Hook0 into your applications.

| Language | Package | Status |
|----------|---------|--------|
| JavaScript/TypeScript | [`@hook0/sdk`](sdk/javascript.md) | Stable |
| Rust | [`hook0-client`](sdk/rust.md) | Stable |

---

## Data Specifications

### [Event Labels](labels.md)

Labels are key-value metadata for filtering and routing events to subscriptions. Essential for multi-tenancy and environment-based routing.

```json
{
  "labels": {
    "tenant_id": "customer_123",
    "environment": "production"
  }
}
```

### [Event Schemas](event-schemas.md)

Standardized payload structures and validation rules for your event types.

### [Glossary](glossary.md)

Definitions of Hook0 terminology: events, subscriptions, webhooks, request attempts, and more.

---

## Configuration

### [Configuration Reference](configuration.md)

All environment variables and configuration options for Hook0 server and worker processes.

**Key sections:**

| Category | Examples |
|----------|----------|
| Database | `DATABASE_URL`, connection pool settings |
| Server | `API_PORT`, `API_HOST`, TLS settings |
| Worker | Concurrency, retry policies |
| Security | `BISCUIT_PRIVATE_KEY`, authentication |
| Logging | Log levels, output format |

---

## Troubleshooting

### [Error Codes](error-codes.md)

Complete list of HTTP status codes and Hook0-specific error codes with resolution steps.

**Common errors:**

| Code | Meaning | Quick Fix |
|------|---------|-----------|
| 401 | Unauthorized | Check API token |
| 403 | Forbidden | Verify token permissions |
| 404 | Not Found | Check resource ID |
| 422 | Validation Error | Review request payload |

---

## AI Assistant Integration

### [MCP Server for AI Assistants](mcp-for-ia-assistant.md)

Control Hook0 using natural language with Claude, Cursor, Windsurf, or any MCP-compatible AI assistant.

**Capabilities:**
- List and inspect applications, events, subscriptions
- Create and manage webhook subscriptions
- Debug failed deliveries and retry them
- Send test events

---

## Quick Reference

### Event Delivery Status

| Status | Description |
|--------|-------------|
| `pending` | Queued for delivery |
| `inprogress` | Currently being delivered |
| `successful` | Delivered successfully (2xx response) |
| `failed` | Delivery failed, will retry |
| `waiting` | Scheduled for retry (backoff) |

### Webhook Signature Format

Hook0 signs webhooks using HMAC-SHA256. The signature header format:

```
X-Hook0-Signature: t=<timestamp>,v1=<signature>[,h=<headers>]
```

See [Webhook Authentication](../tutorials/webhook-authentication.md) for verification examples.

### Event Type Naming Convention

```
<service>.<resource>.<action>
```

Examples: `user.account.created`, `order.payment.completed`, `inventory.stock.updated`

---

## Related Documentation

- **[Tutorials](../tutorials/index.md)** — Step-by-step learning guides
- **[How-to Guides](../how-to-guides/index.md)** — Task-oriented problem solving
- **[Explanation](../explanation/index.md)** — Conceptual understanding
