# Setting up Event Types and Subscriptions

This tutorial teaches you how to design effective event schemas, create organized event type hierarchies, and configure sophisticated subscription patterns for complex applications.

## Prerequisites

- Completed [Getting Started](./getting-started.md) tutorial
- Understanding of JSON schemas
- Familiarity with webhook concepts

## Event Type Design Principles

### Naming Conventions

Use clear, hierarchical naming:
```
resource.action
user.created
user.updated
user.deleted
order.created
order.paid
order.shipped
order.completed
```

### Hierarchical Organization
```
user.*           → All user events
user.account.*   → User account events  
user.profile.*   → User profile events
order.*          → All order events
order.payment.*  → Payment-related events
```

## Step 1: Design Your Event Schema

Before creating event types, plan your event structure:

### User Events Schema
```json
{
  "user.created": {
    "user_id": "string (required)",
    "email": "string (required)",
    "name": "string (required)",
    "plan": "string (optional)",
    "created_at": "ISO 8601 timestamp",
    "metadata": {
      "source": "web|mobile|api",
      "campaign": "string (optional)"
    }
  },
  "user.updated": {
    "user_id": "string (required)",
    "changes": ["array of changed fields"],
    "previous_values": "object (optional)",
    "updated_at": "ISO 8601 timestamp"
  }
}
```

### Order Events Schema
```json
{
  "order.created": {
    "order_id": "string (required)",
    "customer_id": "string (required)",
    "items": "array of order items",
    "total_amount": "number",
    "currency": "string",
    "created_at": "ISO 8601 timestamp"
  },
  "order.payment.processed": {
    "order_id": "string (required)",
    "payment_id": "string (required)",
    "amount": "number",
    "currency": "string",
    "payment_method": "string",
    "processed_at": "ISO 8601 timestamp"
  }
}
```

## Step 2: Create Event Types with Validation

### Basic Event Type Creation

```bash
curl -X POST "https://app.hook0.com/api/v1/event_types" \
  -H "Authorization: Bearer biscuit:YOUR_TOKEN_HERE" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "{app-id}",
    "service": "user",
    "resource_type": "account",
    "verb": "created"
  }'
```

### Event Type with Detailed Metadata

```bash
curl -X POST "https://app.hook0.com/api/v1/event_types" \
  -H "Authorization: Bearer biscuit:YOUR_TOKEN_HERE" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "{app-id}",
    "service": "order",
    "resource_type": "payment",
    "verb": "processed"
  }'
```

## Step 3: Create a Complete Event Type Hierarchy

Let's create a comprehensive set of event types for a SaaS application:

### User Lifecycle Events

```bash
# User registration
curl -X POST "https://app.hook0.com/api/v1/event_types" \
  -H "Authorization: Bearer biscuit:YOUR_TOKEN_HERE" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "{app-id}",
    "service": "user",
    "resource_type": "account",
    "verb": "registered"
  }'

# Email verification
curl -X POST "https://app.hook0.com/api/v1/event_types" \
  -H "Authorization: Bearer biscuit:YOUR_TOKEN_HERE" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "{app-id}",
    "service": "user",
    "resource_type": "email",
    "verb": "verified"
  }'

# Profile updates
curl -X POST "https://app.hook0.com/api/v1/event_types" \
  -H "Authorization: Bearer biscuit:YOUR_TOKEN_HERE" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "{app-id}",
    "service": "user",
    "resource_type": "profile",
    "verb": "updated"
  }'

# Account deactivation
curl -X POST "https://app.hook0.com/api/v1/event_types" \
  -H "Authorization: Bearer biscuit:YOUR_TOKEN_HERE" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "{app-id}",
    "service": "user",
    "resource_type": "account",
    "verb": "deactivated"
  }'
```

### Subscription Events

```bash
# Subscription created
curl -X POST "https://app.hook0.com/api/v1/event_types" \
  -H "Authorization: Bearer biscuit:YOUR_TOKEN_HERE" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "{app-id}",
    "service": "subscription",
    "resource_type": "plan",
    "verb": "created"
  }'

# Subscription upgraded
curl -X POST "https://app.hook0.com/api/v1/event_types" \
  -H "Authorization: Bearer biscuit:YOUR_TOKEN_HERE" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "{app-id}",
    "service": "subscription",
    "resource_type": "plan",
    "verb": "upgraded"
  }'

# Payment failed
curl -X POST "https://app.hook0.com/api/v1/event_types" \
  -H "Authorization: Bearer biscuit:YOUR_TOKEN_HERE" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "{app-id}",
    "service": "subscription",
    "resource_type": "payment",
    "verb": "failed"
  }'

# Subscription cancelled
curl -X POST "https://app.hook0.com/api/v1/event_types" \
  -H "Authorization: Bearer biscuit:YOUR_TOKEN_HERE" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "{app-id}",
    "service": "subscription",
    "resource_type": "plan",
    "verb": "cancelled"
  }'
```

## Step 4: Advanced Subscription Patterns

### Single Event Type Subscription

Simple subscription for one specific event:

```bash
curl -X POST "https://app.hook0.com/api/v1/subscriptions" \
  -H "Authorization: Bearer biscuit:YOUR_TOKEN_HERE" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "{app-id}",
    "is_enabled": true,
    "event_types": ["user.account.registered"],
    "description": "Welcome email trigger",
    "label_key": "environment",
    "label_value": "production",
    "target": {
      "type": "http",
      "method": "POST",
      "url": "https://email-service.com/webhooks/welcome",
      "headers": {
        "Content-Type": "application/json",
        "X-Service": "welcome-emails"
      }
    }
  }'
```

### Multiple Event Type Subscription

Subscribe to multiple related events:

```bash
curl -X POST "https://app.hook0.com/api/v1/subscriptions" \
  -H "Authorization: Bearer biscuit:YOUR_TOKEN_HERE" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "{app-id}",
    "is_enabled": true,
    "event_types": [
      "user.account.registered",
      "user.email.verified", 
      "user.profile.updated"
    ],
    "description": "CRM system user sync",
    "label_key": "environment",
    "label_value": "production",
    "target": {
      "type": "http",
      "method": "POST",
      "url": "https://crm.company.com/api/webhooks/users",
      "headers": {
        "Content-Type": "application/json",
        "Authorization": "Bearer crm-api-key"
      }
    }
  }'
```

### Pattern-Based Subscription (All User Events)

Subscribe to all events matching a pattern:

```bash
curl -X POST "https://app.hook0.com/api/v1/subscriptions" \
  -H "Authorization: Bearer biscuit:YOUR_TOKEN_HERE" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "{app-id}",
    "is_enabled": true,
    "event_types": [
      "user.account.registered",
      "user.email.verified",
      "user.profile.updated",
      "user.account.deactivated"
    ],
    "description": "Analytics tracking for all user events",
    "label_key": "environment",
    "label_value": "production",
    "target": {
      "type": "http",
      "method": "POST",
      "url": "https://analytics.company.com/api/events",
      "headers": {
        "Content-Type": "application/json",
        "X-Analytics-Source": "user-events"
      }
    }
  }'
```

### Critical Events Subscription

High-priority events with special handling:

```bash
curl -X POST "https://app.hook0.com/api/v1/subscriptions" \
  -H "Authorization: Bearer biscuit:YOUR_TOKEN_HERE" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "{app-id}",
    "is_enabled": true,
    "event_types": [
      "subscription.payment.failed",
      "user.account.deactivated"
    ],
    "description": "Alert system for critical events",
    "label_key": "environment",
    "label_value": "production",
    "target": {
      "type": "http",
      "method": "POST",
      "url": "https://alerts.company.com/api/critical",
      "headers": {
        "Content-Type": "application/json",
        "X-Priority": "high",
        "X-Alert-Channel": "slack"
      }
    }
  }'
```

## Step 5: Advanced Subscription Configuration

### Custom Headers and Authentication

```bash
curl -X POST "https://app.hook0.com/api/v1/subscriptions" \
  -H "Authorization: Bearer biscuit:YOUR_TOKEN_HERE" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "{app-id}",
    "is_enabled": true,
    "event_types": ["subscription.plan.created"],
    "description": "Billing system integration",
    "label_key": "environment",
    "label_value": "production",
    "target": {
      "type": "http",
      "method": "POST",
      "url": "https://billing.company.com/webhooks/subscriptions",
      "headers": {
        "Content-Type": "application/json",
        "Authorization": "Bearer billing-api-token",
        "X-Webhook-Source": "hook0",
        "X-Environment": "production"
      }
    },
    "metadata": {
      "service": "billing",
      "priority": "high",
      "team": "finance"
    }
  }'
```

### Subscription with Labels

Add metadata to organize subscriptions:

```bash
curl -X POST "https://app.hook0.com/api/v1/subscriptions" \
  -H "Authorization: Bearer biscuit:YOUR_TOKEN_HERE" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "{app-id}",
    "is_enabled": true,
    "event_types": ["user.account.registered", "user.account.deactivated"],
    "description": "Marketing automation system",
    "label_key": "department",
    "label_value": "marketing",
    "target": {
      "type": "http",
      "method": "POST",
      "url": "https://marketing.company.com/api/webhooks",
      "headers": {
        "Content-Type": "application/json"
      }
    },
    "metadata": {
      "system": "automation", 
      "criticality": "medium"
    }
  }'
```

## Step 6: Test Your Event Types

Create a testing script to validate your event types:

```javascript
// test-events.js
const fetch = require('node-fetch');
const { v4: uuidv4 } = require('uuid');

const HOOK0_TOKEN = 'biscuit:YOUR_TOKEN_HERE';
const HOOK0_API = 'https://app.hook0.com/api/v1/event';
const APPLICATION_ID = '{app-id}';

async function sendTestEvent(eventType, payload) {
  try {
    const response = await fetch(HOOK0_API, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${HOOK0_TOKEN}`,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        application_id: APPLICATION_ID,
        event_id: uuidv4(),
        event_type: eventType,
        payload: JSON.stringify(payload),
        payload_content_type: 'application/json',
        occurred_at: new Date().toISOString(),
        labels: {
          environment: 'test',
          source: 'event-testing'
        }
      })
    });
    
    const result = await response.json();
    console.log(`✅ ${eventType}:`, result.event_id);
    return result;
  } catch (error) {
    console.error(`❌ ${eventType}:`, error.message);
  }
}

async function runTests() {
  console.log('🧪 Testing Event Types...\n');
  
  // Test user events
  await sendTestEvent('user.registered', {
    user_id: 'user_123',
    email: 'test@example.com',
    name: 'Test User',
    plan: 'free',
    created_at: new Date().toISOString(),
    metadata: {
      source: 'web',
      campaign: 'social_media'
    }
  });
  
  await sendTestEvent('user.email.verified', {
    user_id: 'user_123',
    email: 'test@example.com',
    verified_at: new Date().toISOString()
  });
  
  await sendTestEvent('user.profile.updated', {
    user_id: 'user_123',
    changes: ['name', 'avatar'],
    previous_values: {
      name: 'Old Name',
      avatar: null
    },
    updated_at: new Date().toISOString()
  });
  
  // Test subscription events
  await sendTestEvent('subscription.created', {
    subscription_id: 'sub_123',
    user_id: 'user_123',
    plan: 'pro',
    amount: 29.99,
    currency: 'USD',
    billing_cycle: 'monthly',
    created_at: new Date().toISOString()
  });
  
  await sendTestEvent('subscription.payment.failed', {
    subscription_id: 'sub_123',
    user_id: 'user_123',
    payment_id: 'pay_123',
    amount: 29.99,
    currency: 'USD',
    error_code: 'card_declined',
    error_message: 'Insufficient funds',
    failed_at: new Date().toISOString()
  });
  
  console.log('\n✨ Test completed!');
}

runTests().catch(console.error);
```

Run the tests:
```bash
node test-events.js
```

## Step 7: Monitor Event Types and Subscriptions

### List All Event Types
```bash
curl "https://app.hook0.com/api/v1/event_types" \
  -H "Authorization: Bearer biscuit:YOUR_TOKEN_HERE"
```

### List All Subscriptions  
```bash
curl "https://app.hook0.com/api/v1/subscriptions" \
  -H "Authorization: Bearer biscuit:YOUR_TOKEN_HERE"
```

### Get Subscription Details
```bash
curl "https://app.hook0.com/api/v1/subscriptions/{sub-id}" \
  -H "Authorization: Bearer biscuit:YOUR_TOKEN_HERE"
```

## Step 8: Update and Manage Event Types

### Update Event Type Description
```bash
curl -X PUT "https://app.hook0.com/api/v1/event_types/{event-type-id}" \
  -H "Authorization: Bearer biscuit:YOUR_TOKEN_HERE" \
  -H "Content-Type: application/json" \
  -d '{
    "description": "Updated description for user registration event",
    "metadata": {
      "category": "user-lifecycle",
      "schema_version": "1.1"
    }
  }'
```

### Update Subscription Configuration
```bash
curl -X PUT "https://app.hook0.com/api/v1/subscriptions/{sub-id}" \
  -H "Authorization: Bearer biscuit:YOUR_TOKEN_HERE" \
  -H "Content-Type: application/json" \
  -d '{
    "event_types": ["user.registered", "user.email.verified"],
    "description": "Updated CRM integration - removed profile updates",
    "target": {
      "type": "http",
      "method": "POST",
      "url": "https://new-crm.company.com/api/webhooks/users",
      "headers": {
        "Content-Type": "application/json",
        "Authorization": "Bearer new-crm-api-key"
      }
    }
  }'
```

## Step 9: Deactivate Event Types and Subscriptions

### Deactivate Event Type
```bash
curl -X PUT "https://app.hook0.com/api/v1/event_types/{event-type-id}" \
  -H "Authorization: Bearer biscuit:YOUR_TOKEN_HERE" \
  -H "Content-Type: application/json" \
  -d '{
    "deactivated_at": "'$(date -u +%Y-%m-%dT%H:%M:%SZ)'"
  }'
```

### Disable Subscription
```bash
curl -X PUT "https://app.hook0.com/api/v1/subscriptions/{sub-id}" \
  -H "Authorization: Bearer biscuit:YOUR_TOKEN_HERE" \
  -H "Content-Type: application/json" \
  -d '{
    "is_enabled": false
  }'
```

## Best Practices

### Event Type Design
- ✅ Use consistent naming conventions
- ✅ Include all necessary data in payloads
- ✅ Add timestamps to all events
- ✅ Use semantic versioning for schema changes
- ❌ Don't include sensitive data in event payloads
- ❌ Don't create too many granular event types

### Subscription Management
- ✅ Group related subscriptions logically
- ✅ Use descriptive names and descriptions
- ✅ Include metadata for organization
- ✅ Monitor subscription health regularly
- ❌ Don't create duplicate subscriptions
- ❌ Don't subscribe to events you don't need

### Testing
- ✅ Test all event types regularly
- ✅ Validate webhook endpoint responses
- ✅ Monitor delivery success rates
- ✅ Test failure scenarios
- ❌ Don't test in production without safeguards

## What You've Learned

✅ Designed hierarchical event type schemas  
✅ Created comprehensive event type sets  
✅ Built sophisticated subscription patterns  
✅ Configured advanced subscription options  
✅ Implemented testing strategies  
✅ Managed event types and subscriptions lifecycle  

## Next Steps

- [Implementing Webhook Authentication](./webhook-authentication.md)
- [Self-hosting Hook0 with Docker](./self-hosting-docker.md)
- [Debugging Failed Webhook Deliveries](../how-to-guides/debug-failed-webhooks.md)

## Troubleshooting

### Event Not Triggering Webhooks
1. Check event type exists in application
2. Verify subscription is enabled
3. Confirm event type matches subscription
4. Check webhook endpoint accessibility

### Subscription Not Receiving Events
1. Verify subscription configuration
2. Check event type matching logic
3. Review webhook endpoint logs
4. Monitor Hook0 dashboard for delivery attempts