---
title: Events
description: Understanding events in Hook0
---

# Events

An event is a notification sent from your applications to Hook0 when a specific action occurs. Examples range from repository commits to user creation events.

## Key Points

- Events have associated event types serving as identifiers
- Event types enable webhook consumers to configure subscriptions based on interests
- Payloads support JSON, plain text, or base64-encoded binary formats
- Events trigger subscriber actions like emails or database updates

## Example

### Sending an Event

```bash
curl -X POST "$HOOK0_API/event" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "'"$APP_ID"'",
    "event_id": "'$(uuidgen)'",
    "event_type": "user.account.created",
    "payload": "{\"user_id\": \"usr_789\", \"email\": \"john@example.com\", \"plan\": \"premium\"}",
    "payload_content_type": "application/json",
    "occurred_at": "2024-01-15T10:30:00Z",
    "labels": {
      "tenant_id": "customer_123",
      "environment": "production"
    }
  }'
```

**Key Points:**
- `event_id`: Must be a valid UUID (e.g., `550e8400-e29b-41d4-a716-446655440000`)
- `payload`: Must be a **JSON-encoded string** (not an object) - see below
- `labels`: Required for routing events to subscriptions - must have at least one key-value pair

### Payload Formats

Hook0 supports three payload formats. **Important:** When using JSON, the `payload` field must be a **JSON-encoded string**, not a raw object.

**JSON Format** (structured data):
```json
{
  "event_type": "order.purchase.completed",
  "payload": "{\"order_id\": \"ord_456\", \"amount\": 99.99, \"currency\": \"EUR\"}",
  "payload_content_type": "application/json"
}
```

:::warning Why String Format?
Hook0 forwards payloads exactly as received without re-serialization. This ensures no data transformation, preserved formatting, and support for any content type.

**✅ Correct**: `"payload": "{\"user_id\": \"123\"}"`

**❌ Incorrect**: `"payload": {"user_id": "123"}`
:::

**Plain Text Format** (simple string data):
```json
{
  "event_type": "system.log.created",
  "payload": "User logged in successfully at 2025-01-15T10:30:00Z",
  "payload_content_type": "text/plain"
}
```

**Base64 Format** (binary data):
```json
{
  "event_type": "storage.document.uploaded",
  "payload": "SGVsbG8gV29ybGQhIFRoaXMgaXMgYmluYXJ5IGRhdGE=",
  "payload_content_type": "application/octet-stream"
}
```

## What's Next?

- [Applications](/explanation/what-is-hook0#applications)
- [Event Types](event-types.md)
- [Subscriptions](subscriptions.md)
