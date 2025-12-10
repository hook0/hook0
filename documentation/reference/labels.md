---
sidebar_position: 6
---

# Event Labels

Labels are key-value pairs attached to events and used for filtering and routing webhooks to subscriptions. They enable powerful use cases like multi-tenancy, environment-based routing, and priority filtering.

## What Are Labels?

Labels are metadata attached to events that subscriptions can filter on. Each label consists of:
- **Key**: A string identifier (e.g., `tenant_id`, `environment`, `priority`)
- **Value**: A string value (e.g., `tenant_123`, `production`, `high`)

:::warning Minimum Label Requirement
Every event must include **at least one label**. The API will reject events with an empty `labels: {}` object. This ensures proper routing and filtering of events to subscriptions.
:::

```json
{
  "event_id": "evt_123",
  "event_type": "users.account.created",
  "labels": {
    "tenant_id": "tenant_abc",
    "environment": "production",
    "region": "us-east-1",
    "priority": "high"
  },
  "payload": { ... }
}
```

## How Labels Work

### Event Sending with Labels

When sending an event, include labels in the request:

```bash
curl -X POST "https://app.hook0.com/api/v1/event" \
  -H "Authorization: Bearer{YOUR_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "app_123",
    "event_id": "evt_456",
    "event_type": "users.account.created",
    "payload": "{\"user_id\": \"user_789\"}",
    "payload_content_type": "application/json",
    "labels": {
      "tenant_id": "acme_corp",
      "environment": "production",
      "source": "web_app"
    }
  }'
```

### Subscription Filtering

Subscriptions use `label_key` and `label_value` to filter events:

```bash
curl -X POST "https://app.hook0.com/api/v1/subscriptions" \
  -H "Authorization: Bearer{YOUR_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "app_123",
    "is_enabled": true,
    "event_types": ["users.account.created", "users.account.updated"],
    "label_key": "tenant_id",
    "label_value": "acme_corp",
    "description": "Webhooks for ACME Corp tenant",
    "target": {
      "type": "http",
      "method": "POST",
      "url": "https://acme.example.com/webhooks"
    }
  }'
```

**Filtering logic**:
- Subscription receives event if `event.labels[label_key] == label_value`
- If event does not have the specified label, subscription will not receive it
- Only one label filter per subscription (label_key/label_value pair)

## Common Use Cases

### 1. Multi-Tenancy

Route events to tenant-specific webhooks:

```javascript
// Send event with tenant label
await hook0.sendEvent({
  eventType: 'invoice.created',
  payload: { invoiceId: 'inv_123', amount: 99.99 },
  labels: {
    tenant_id: 'acme_corp',
    organization: 'org_456'
  }
});

// Tenant A subscription (only receives tenant_a events)
{
  "label_key": "tenant_id",
  "label_value": "acme_corp",
  "target": { "url": "https://acme.example.com/webhooks" }
}

// Tenant B subscription (only receives tenant_b events)
{
  "label_key": "tenant_id",
  "label_value": "globex_inc",
  "target": { "url": "https://globex.example.com/webhooks" }
}
```

**Benefits**:
- Complete data isolation between tenants
- Tenants only receive their own events
- Scalable to thousands of tenants

### 2. Environment Filtering

Separate production, staging, and development events:

```javascript
// Production event
await hook0.sendEvent({
  eventType: 'deployment.completed',
  labels: { environment: 'production' }
});

// Staging event
await hook0.sendEvent({
  eventType: 'deployment.completed',
  labels: { environment: 'staging' }
});
```

Subscriptions filter by environment:

```json
{
  "label_key": "environment",
  "label_value": "production",
  "description": "Production monitoring",
  "target": { "url": "https://monitoring.prod.example.com/webhooks" }
}
```

### 3. Priority Routing

Route high-priority events to dedicated endpoints:

```javascript
// High-priority event
await hook0.sendEvent({
  eventType: 'payments.transaction.failed',
  labels: {
    priority: 'critical',
    alert_team: 'payments'
  }
});

// Normal-priority event
await hook0.sendEvent({
  eventType: 'payments.transaction.succeeded',
  labels: { priority: 'normal' }
});
```

Subscriptions handle priorities differently:

```json
{
  "label_key": "priority",
  "label_value": "critical",
  "description": "Critical alerts to Slack",
  "target": {
    "url": "https://hooks.slack.com/services/critical-alerts",
    "headers": { "X-Priority": "urgent" }
  }
}
```

### 4. Geographic Routing

Route events based on region or data center:

```javascript
await hook0.sendEvent({
  eventType: 'orders.order.created',
  payload: { orderId: 'ord_123' },
  labels: {
    region: 'us-east-1',
    country: 'US'
  }
});
```

Region-specific subscriptions:

```json
{
  "label_key": "region",
  "label_value": "us-east-1",
  "description": "US East fulfillment center",
  "target": { "url": "https://fulfillment.us-east.example.com/webhooks" }
}
```

### 5. Feature Flags

Route events based on feature enablement:

```javascript
await hook0.sendEvent({
  eventType: 'users.account.created',
  labels: {
    feature_new_onboarding: 'enabled',
    experiment_group: 'variant_a'
  }
});
```

### 6. Source Tracking

Identify event origin:

```javascript
await hook0.sendEvent({
  eventType: 'purchase.completed',
  labels: {
    source: 'mobile_app',
    platform: 'ios',
    version: '2.1.0'
  }
});
```

## Label Structure

### JSON Format

Labels are a flat JSON object with string keys and string values:

```json
{
  "labels": {
    "key1": "value1",
    "key2": "value2",
    "key3": "value3"
  }
}
```

### Validation Rules

| Constraint | Value | Error Code |
|------------|-------|------------|
| **Max labels per event** | 10 | `labels-size` |
| **Key length** | 1-50 characters | `labels-property-length` |
| **Value length** | 1-50 characters | `labels-property-length` |
| **Value type** | String only | `labels-property-type` |

```javascript
// ✅ Valid labels
{
  "tenant": "acme",
  "env": "prod",
  "region": "us"
}

// ❌ Too many labels (max 10)
{
  "label1": "value1",
  "label2": "value2",
  // ... 11 labels total
}

// ❌ Value too long (max 50 chars)
{
  "key": "this_value_is_way_too_long_and_exceeds_the_fifty_character_limit"
}

// ❌ Non-string value
{
  "priority": 1,  // Must be "1" (string)
  "active": true  // Must be "true" (string)
}
```

## Reserved Labels

Hook0 does not strictly reserve label keys, but common conventions exist:

### Recommended Standard Labels

```json
{
  "tenant_id": "unique_tenant_identifier",
  "environment": "production|staging|development",
  "region": "aws_region_or_datacenter",
  "priority": "critical|high|normal|low",
  "source": "api|web|mobile|batch",
  "version": "api_or_schema_version"
}
```

### Label Naming Conventions

**✅ Good practices**:
- Use snake_case: `tenant_id`, `event_source`
- Be descriptive: `payment_provider` not `pp`
- Use consistent naming across events
- Keep keys short but meaningful

**❌ Avoid**:
- Camel case: `tenantId` (use `tenant_id`)
- Special characters: `tenant@id`, `tenant.id`
- Generic names: `data`, `info`, `value`
- Sensitive information: `credit_card`, `ssn`, `password`

## Advanced Patterns

### Multi-Label Filtering (Multiple Subscriptions)

Since subscriptions support only one label filter, use multiple subscriptions for AND logic:

```javascript
// Event with multiple labels
await hook0.sendEvent({
  eventType: 'orders.order.created',
  labels: {
    tenant_id: 'acme_corp',
    environment: 'production',
    priority: 'high'
  }
});
```

Create separate subscriptions for different filtering needs:

```json
// Subscription 1: All ACME Corp events
{
  "label_key": "tenant_id",
  "label_value": "acme_corp",
  "target": { "url": "https://acme.example.com/all-events" }
}

// Subscription 2: All production events (all tenants)
{
  "label_key": "environment",
  "label_value": "production",
  "target": { "url": "https://monitoring.example.com/production" }
}

// Subscription 3: All high-priority events
{
  "label_key": "priority",
  "label_value": "high",
  "target": { "url": "https://alerts.example.com/high-priority" }
}
```

:::info
Each subscription filters independently. An event with labels `{tenant_id: "acme_corp", environment: "production"}` will trigger both subscriptions above.
:::

### Wildcard Simulation

Hook0 does not support wildcard labels, but you can achieve similar results:

```javascript
// Strategy 1: Use hierarchical values
labels: {
  scope: "acme_corp.us_east.production"
}

// Strategy 2: Use multiple labels
labels: {
  tenant: "acme_corp",
  region: "us_east",
  env: "production"
}

// Strategy 3: Subscribe to multiple specific values
// Create subscriptions for tenant_a, tenant_b, tenant_c separately
```

### Dynamic Label Values

Generate labels programmatically:

```javascript
function sendOrderEvent(order) {
  return hook0.sendEvent({
    eventType: 'orders.order.created',
    payload: order,
    labels: {
      tenant_id: order.tenantId,
      environment: process.env.NODE_ENV,
      region: process.env.AWS_REGION,
      order_total: categorizeOrderTotal(order.total), // "small"|"medium"|"large"
      customer_type: order.customer.isPremium ? "premium" : "standard"
    }
  });
}

function categorizeOrderTotal(total) {
  if (total < 50) return 'small';
  if (total < 500) return 'medium';
  return 'large';
}
```

### Label-Based Subscription Management

Organize subscriptions using labels:

```javascript
// Create subscription with metadata describing label usage
await hook0.createSubscription({
  eventTypes: ['payments.transaction.succeeded'],
  labelKey: 'tenant_id',
  labelValue: 'acme_corp',
  description: 'ACME Corp payment webhooks',
  metadata: {
    tenant_name: 'ACME Corporation',
    tenant_tier: 'enterprise',
    contact_email: 'webhooks@acme.com',
    label_strategy: 'tenant_isolation'
  }
});
```

## Best Practices

### Design Principles

1. **Keep labels simple**: Use for filtering, not business logic
2. **Use consistent naming**: Establish conventions and stick to them
3. **Document label schema**: Maintain documentation of label keys and allowed values
4. **Limit label count**: Use only necessary labels (remember 10 max)
5. **Avoid PII**: Do not include sensitive data in labels

### Example Label Schema Documentation

```yaml
# labels-schema.yaml
labels:
  tenant_id:
    type: string
    pattern: "^[a-z0-9_]{3,50}$"
    description: "Unique tenant identifier"
    required: true

  environment:
    type: string
    enum: [production, staging, development]
    description: "Deployment environment"
    required: true

  priority:
    type: string
    enum: [critical, high, normal, low]
    description: "Event priority level"
    required: false

  region:
    type: string
    pattern: "^[a-z]{2}-[a-z]+-[0-9]$"
    description: "AWS region identifier"
    required: false
```

### Implementation Checklist

**Event sending**:
- ✅ Validate label values before sending
- ✅ Use constants for label keys
- ✅ Include required labels (tenant_id, environment)
- ✅ Keep label count under 10
- ✅ Use string values only

**Subscription configuration**:
- ✅ Choose appropriate label_key for filtering
- ✅ Document subscription filtering logic
- ✅ Test subscription receives expected events
- ✅ Monitor subscription delivery metrics

### Code Example: Label Helper

```javascript
// labels.js
const REQUIRED_LABELS = ['tenant_id', 'environment'];
const MAX_LABELS = 10;
const MAX_LENGTH = 50;

class LabelValidator {
  static validate(labels) {
    // Check required labels
    for (const key of REQUIRED_LABELS) {
      if (!labels[key]) {
        throw new Error(`Missing required label: ${key}`);
      }
    }

    // Check label count
    if (Object.keys(labels).length > MAX_LABELS) {
      throw new Error(`Too many labels (max ${MAX_LABELS})`);
    }

    // Check lengths and types
    for (const [key, value] of Object.entries(labels)) {
      if (typeof value !== 'string') {
        throw new Error(`Label ${key} must be a string`);
      }
      if (key.length > MAX_LENGTH || value.length > MAX_LENGTH) {
        throw new Error(`Label ${key} exceeds max length ${MAX_LENGTH}`);
      }
    }

    return true;
  }

  static create(tenant, environment, extra = {}) {
    const labels = {
      tenant_id: tenant,
      environment: environment,
      ...extra
    };

    this.validate(labels);
    return labels;
  }
}

// Usage
const labels = LabelValidator.create('acme_corp', 'production', {
  region: 'us-east-1',
  priority: 'high'
});

await hook0.sendEvent({
  eventType: 'orders.order.created',
  labels
});
```

## Troubleshooting

### Events Not Reaching Subscription

**Symptom**: Events sent but subscription not triggered

**Causes**:
1. Label mismatch: Event label value does not match subscription filter
2. Missing label: Event does not include the filtered label key
3. Case sensitivity: Label values are case-sensitive

**Solutions**:

```javascript
// Check event labels
const event = await hook0.getEvent('evt_123');
console.log('Event labels:', event.labels);

// Check subscription filter
const subscription = await hook0.getSubscription('sub_456');
console.log('Subscription filter:', {
  key: subscription.label_key,
  value: subscription.label_value
});

// Verify match
const matches = event.labels[subscription.label_key] === subscription.label_value;
console.log('Label matches:', matches);
```

### Validation Errors

**Symptom**: `400 Bad Request` with label validation error

**Error codes**:
- `labels-size`: Too many labels (max 10)
- `labels-property-length`: Key or value exceeds 50 characters
- `labels-property-type`: Non-string value detected

**Solutions**:

```javascript
// Fix: Reduce label count
// ❌ Bad (11 labels)
labels: { l1: "v1", l2: "v2", ..., l11: "v11" }

// ✅ Good (10 labels)
labels: { l1: "v1", l2: "v2", ..., l10: "v10" }

// Fix: Shorten values
// ❌ Bad (too long)
labels: { key: "this_is_a_very_long_value_that_exceeds_fifty_characters_limit" }

// ✅ Good
labels: { key: "short_value" }

// Fix: Convert to string
// ❌ Bad (wrong type)
labels: { priority: 1, active: true }

// ✅ Good
labels: { priority: "1", active: "true" }
```

### Performance Considerations

**Issue**: Too many subscriptions with different label filters

**Impact**:
- Increased database queries
- Slower event routing
- Higher resource usage

**Optimization**:

```javascript
// ❌ Inefficient: 100 subscriptions for 100 tenants
for (const tenant of tenants) {
  await createSubscription({
    labelKey: 'tenant_id',
    labelValue: tenant.id,
    target: { url: `https://${tenant.domain}/webhooks` }
  });
}

// ✅ Better: Single webhook endpoint handles routing
await createSubscription({
  labelKey: 'environment',
  labelValue: 'production',
  target: { url: 'https://webhook-router.example.com/receive' }
});

// Webhook router extracts tenant from event.labels.tenant_id
// and forwards to appropriate tenant endpoint
```

## API Reference

### Event Creation with Labels

```http
POST /api/v1/event
Content-Type: application/json
Authorization: BearerTOKEN

{
  "application_id": "app_123",
  "event_id": "evt_456",
  "event_type": "users.account.created",
  "payload": "{}",
  "labels": {
    "tenant_id": "acme_corp",
    "environment": "production"
  }
}
```

### Subscription Creation with Label Filter

```http
POST /api/v1/subscriptions
Content-Type: application/json
Authorization: BearerTOKEN

{
  "application_id": "app_123",
  "event_types": ["users.account.created"],
  "label_key": "tenant_id",
  "label_value": "acme_corp",
  "target": {
    "type": "http",
    "method": "POST",
    "url": "https://example.com/webhooks"
  }
}
```

### Querying Events by Labels

```http
GET /api/v1/events?application_id=app_123
Authorization: BearerTOKEN
```

Response includes labels for filtering client-side:

```json
{
  "events": [
    {
      "event_id": "evt_123",
      "labels": {
        "tenant_id": "acme_corp",
        "environment": "production"
      }
    }
  ]
}
```

## Next Steps

- [Event Schemas Reference](./event-schemas.md) - Learn about event structure
- [API Documentation](/openapi/intro) - Full API documentation
- [Multi-tenant Architecture](../explanation/hook0-architecture.md) - Architecture patterns
- [Event Types Tutorial](../tutorials/event-types-subscriptions.md) - Hands-on tutorial
