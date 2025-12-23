# Event Payload Schemas

This reference provides standardized schemas and examples for common event types in Hook0. Following these schemas ensures consistency and compatibility across integrations.

## Event Type Naming Convention

Hook0 uses a **three-part naming convention** for event types:

```
service.resource.verb
```

- **service**: The service or domain (e.g., `user`, `order`, `payment`)
- **resource**: The specific resource or entity (e.g., `account`, `payment`, `subscription`)
- **verb**: The action in past tense (e.g., `created`, `updated`, `deleted`, `completed`)

**Examples:**
- `user.account.created` - A user account was created
- `order.payment.completed` - An order payment was completed
- `inventory.stock.updated` - Inventory stock was updated
- `subscription.plan.cancelled` - A subscription plan was cancelled

**Anti-patterns (avoid 2-part names):**
- ❌ `user.created` → ✅ `user.account.created`
- ❌ `order.paid` → ✅ `order.payment.completed`
- ❌ `file.uploaded` → ✅ `storage.file.uploaded`

## Schema Format

Hook0 events follow this structure:

```json
{
  "event_id": "string (UUID)",
  "event_type": "string",
  "payload": "object",
  "labels": "object (optional)",
  "timestamp": "string (ISO 8601)"
}
```

## User Lifecycle Events

### user.account.created

Emitted when a new user account is created.

**Schema:**
```typescript
interface UserCreatedPayload {
  user_id: string;
  email: string;
  name: string;
  username?: string;
  plan?: string;
  email_verified: boolean;
  created_at: string; // ISO 8601
  metadata?: Record<string, any>;
}
```

**Example:**
```json
{
  "event_id": "evt_user_created_123",
  "event_type": "user.account.created",
  "payload": {
    "user_id": "user_12345",
    "email": "john.doe@example.com",
    "name": "John Doe",
    "username": "johndoe",
    "plan": "free",
    "email_verified": false,
    "created_at": "2024-01-15T10:30:00Z",
    "metadata": {
      "signup_source": "web",
      "referrer": "google",
      "campaign": "winter_2024"
    }
  },
  "labels": {
    "environment": "production",
    "source": "auth_service",
    "priority": "normal"
  },
  "timestamp": "2024-01-15T10:30:01Z"
}
```

### user.account.updated

Emitted when user profile information is modified.

**Schema:**
```typescript
interface UserUpdatedPayload {
  user_id: string;
  changes: string[]; // Array of changed fields
  previous_values?: Record<string, any>;
  updated_fields: Record<string, any>;
  updated_at: string; // ISO 8601
}
```

**Example:**
```json
{
  "event_id": "evt_user_updated_456",
  "event_type": "user.account.updated",
  "payload": {
    "user_id": "user_12345",
    "changes": ["name", "plan"],
    "previous_values": {
      "name": "John D.",
      "plan": "free"
    },
    "updated_fields": {
      "name": "John Doe",
      "plan": "pro"
    },
    "updated_at": "2024-01-15T14:20:00Z"
  },
  "labels": {
    "environment": "production",
    "source": "user_service"
  },
  "timestamp": "2024-01-15T14:20:01Z"
}
```

### user.account.deleted

Emitted when a user account is deleted.

**Schema:**
```typescript
interface UserDeletedPayload {
  user_id: string;
  email: string;
  deleted_at: string; // ISO 8601
  deletion_reason?: string;
  soft_delete: boolean;
  retention_period_days?: number;
}
```

**Example:**
```json
{
  "event_id": "evt_user_deleted_789",
  "event_type": "user.account.deleted",
  "payload": {
    "user_id": "user_12345",
    "email": "john.doe@example.com",
    "deleted_at": "2024-01-15T18:45:00Z",
    "deletion_reason": "user_request",
    "soft_delete": true,
    "retention_period_days": 30
  },
  "labels": {
    "environment": "production",
    "source": "user_service",
    "priority": "high"
  },
  "timestamp": "2024-01-15T18:45:01Z"
}
```

### user.email.verified

Emitted when a user verifies their email address.

**Schema:**
```typescript
interface UserEmailVerifiedPayload {
  user_id: string;
  email: string;
  verified_at: string; // ISO 8601
  verification_method: 'email_link' | 'code' | 'magic_link';
}
```

**Example:**
```json
{
  "event_id": "evt_email_verified_101",
  "event_type": "user.email.verified",
  "payload": {
    "user_id": "user_12345",
    "email": "john.doe@example.com",
    "verified_at": "2024-01-15T11:15:00Z",
    "verification_method": "email_link"
  },
  "labels": {
    "environment": "production",
    "source": "auth_service"
  },
  "timestamp": "2024-01-15T11:15:01Z"
}
```

## Subscription and Billing Events

### subscription.plan.created

Emitted when a new subscription is created.

**Schema:**
```typescript
interface SubscriptionCreatedPayload {
  subscription_id: string;
  user_id: string;
  customer_id?: string;
  plan: string;
  status: 'trialing' | 'active' | 'past_due' | 'canceled' | 'unpaid';
  billing_cycle: 'monthly' | 'yearly' | 'weekly' | 'daily';
  amount: number;
  currency: string;
  trial_end?: string; // ISO 8601
  current_period_start: string; // ISO 8601
  current_period_end: string; // ISO 8601
  created_at: string; // ISO 8601
}
```

**Example:**
```json
{
  "event_id": "evt_sub_created_202",
  "event_type": "subscription.plan.created",
  "payload": {
    "subscription_id": "sub_67890",
    "user_id": "user_12345",
    "customer_id": "cus_stripe_abc123",
    "plan": "pro_monthly",
    "status": "trialing",
    "billing_cycle": "monthly",
    "amount": 2999,
    "currency": "USD",
    "trial_end": "2024-01-29T10:30:00Z",
    "current_period_start": "2024-01-15T10:30:00Z",
    "current_period_end": "2024-02-15T10:30:00Z",
    "created_at": "2024-01-15T10:30:00Z"
  },
  "labels": {
    "environment": "production",
    "source": "billing_service",
    "priority": "high"
  },
  "timestamp": "2024-01-15T10:30:01Z"
}
```

### subscription.plan.updated

Emitted when subscription details change.

**Schema:**
```typescript
interface SubscriptionUpdatedPayload {
  subscription_id: string;
  user_id: string;
  changes: string[];
  previous_values: Record<string, any>;
  updated_fields: Record<string, any>;
  updated_at: string; // ISO 8601
}
```

### subscription.plan.canceled

Emitted when a subscription is canceled.

**Schema:**
```typescript
interface SubscriptionCanceledPayload {
  subscription_id: string;
  user_id: string;
  canceled_at: string; // ISO 8601
  cancellation_reason?: string;
  effective_date: string; // ISO 8601
  immediate: boolean;
}
```

### payment.transaction.succeeded

Emitted when a payment is successfully processed.

**Schema:**
```typescript
interface PaymentSucceededPayload {
  payment_id: string;
  subscription_id?: string;
  user_id: string;
  amount: number;
  currency: string;
  payment_method: string;
  provider: string;
  provider_payment_id: string;
  processed_at: string; // ISO 8601
  metadata?: Record<string, any>;
}
```

**Example:**
```json
{
  "event_id": "evt_payment_success_303",
  "event_type": "payment.transaction.succeeded",
  "payload": {
    "payment_id": "pay_12345",
    "subscription_id": "sub_67890",
    "user_id": "user_12345",
    "amount": 2999,
    "currency": "USD",
    "payment_method": "card",
    "provider": "stripe",
    "provider_payment_id": "pi_stripe_xyz789",
    "processed_at": "2024-01-15T10:30:00Z",
    "metadata": {
      "card_last4": "4242",
      "card_brand": "visa"
    }
  },
  "labels": {
    "environment": "production",
    "source": "payment_service",
    "priority": "high"
  },
  "timestamp": "2024-01-15T10:30:01Z"
}
```

### payment.transaction.failed

Emitted when a payment fails.

**Schema:**
```typescript
interface PaymentFailedPayload {
  payment_id: string;
  subscription_id?: string;
  user_id: string;
  amount: number;
  currency: string;
  payment_method: string;
  provider: string;
  error_code: string;
  error_message: string;
  failed_at: string; // ISO 8601
  retry_count: number;
  next_retry_at?: string; // ISO 8601
}
```

**Example:**
```json
{
  "event_id": "evt_payment_failed_404",
  "event_type": "payment.transaction.failed",
  "payload": {
    "payment_id": "pay_12346",
    "subscription_id": "sub_67890",
    "user_id": "user_12345",
    "amount": 2999,
    "currency": "USD",
    "payment_method": "card",
    "provider": "stripe",
    "error_code": "card_declined",
    "error_message": "Your card was declined",
    "failed_at": "2024-01-15T10:30:00Z",
    "retry_count": 1,
    "next_retry_at": "2024-01-16T10:30:00Z"
  },
  "labels": {
    "environment": "production",
    "source": "payment_service",
    "priority": "critical"
  },
  "timestamp": "2024-01-15T10:30:01Z"
}
```

## E-commerce Events

### order.purchase.created

Emitted when a new order is placed.

**Schema:**
```typescript
interface OrderCreatedPayload {
  order_id: string;
  customer_id: string;
  status: 'pending' | 'confirmed' | 'processing' | 'shipped' | 'delivered' | 'canceled';
  items: Array<{
    product_id: string;
    variant_id?: string;
    quantity: number;
    price: number;
    currency: string;
    name: string;
  }>;
  subtotal: number;
  tax: number;
  shipping: number;
  total: number;
  currency: string;
  shipping_address: {
    name: string;
    street: string;
    city: string;
    state: string;
    zip: string;
    country: string;
  };
  billing_address?: {
    name: string;
    street: string;
    city: string;
    state: string;
    zip: string;
    country: string;
  };
  created_at: string; // ISO 8601
}
```

**Example:**
```json
{
  "event_id": "evt_order_created_505",
  "event_type": "order.purchase.created",
  "payload": {
    "order_id": "ord_98765",
    "customer_id": "user_12345",
    "status": "pending",
    "items": [
      {
        "product_id": "prod_111",
        "variant_id": "var_111_xl",
        "quantity": 2,
        "price": 2999,
        "currency": "USD",
        "name": "Premium T-Shirt - XL"
      }
    ],
    "subtotal": 5998,
    "tax": 480,
    "shipping": 500,
    "total": 6978,
    "currency": "USD",
    "shipping_address": {
      "name": "John Doe",
      "street": "123 Main St",
      "city": "San Francisco",
      "state": "CA",
      "zip": "94105",
      "country": "US"
    },
    "created_at": "2024-01-15T10:30:00Z"
  },
  "labels": {
    "environment": "production",
    "source": "order_service"
  },
  "timestamp": "2024-01-15T10:30:01Z"
}
```

### order.shipment.shipped

Emitted when an order is shipped.

**Schema:**
```typescript
interface OrderShippedPayload {
  order_id: string;
  customer_id: string;
  tracking_number: string;
  carrier: string;
  tracking_url?: string;
  shipped_at: string; // ISO 8601
  estimated_delivery?: string; // ISO 8601
  shipping_address: {
    name: string;
    street: string;
    city: string;
    state: string;
    zip: string;
    country: string;
  };
}
```

**Example:**
```json
{
  "event_id": "evt_order_shipped_606",
  "event_type": "order.shipment.shipped",
  "payload": {
    "order_id": "ord_98765",
    "customer_id": "user_12345",
    "tracking_number": "1Z999AA1234567890",
    "carrier": "UPS",
    "tracking_url": "https://www.ups.com/track?tracknum=1Z999AA1234567890",
    "shipped_at": "2024-01-16T14:20:00Z",
    "estimated_delivery": "2024-01-18T17:00:00Z",
    "shipping_address": {
      "name": "John Doe",
      "street": "123 Main St",
      "city": "San Francisco",
      "state": "CA",
      "zip": "94105",
      "country": "US"
    }
  },
  "labels": {
    "environment": "production",
    "source": "fulfillment_service",
    "priority": "normal"
  },
  "timestamp": "2024-01-16T14:20:01Z"
}
```

### order.shipment.delivered

Emitted when an order is successfully delivered.

**Schema:**
```typescript
interface OrderDeliveredPayload {
  order_id: string;
  customer_id: string;
  tracking_number: string;
  carrier: string;
  delivered_at: string; // ISO 8601
  signature_required: boolean;
  signed_by?: string;
  delivery_notes?: string;
}
```

## Product and Inventory Events

### shop.product.created

Emitted when a new product is added to the catalog.

**Schema:**
```typescript
interface ProductCreatedPayload {
  product_id: string;
  name: string;
  description: string;
  category: string;
  brand?: string;
  sku: string;
  price: number;
  currency: string;
  inventory_quantity?: number;
  status: 'active' | 'draft' | 'archived';
  images?: string[];
  attributes?: Record<string, any>;
  created_at: string; // ISO 8601
}
```

### shop.inventory.low

Emitted when product inventory falls below threshold.

**Schema:**
```typescript
interface InventoryLowPayload {
  product_id: string;
  variant_id?: string;
  sku: string;
  current_quantity: number;
  threshold: number;
  location?: string;
  supplier?: string;
  reorder_point: number;
  checked_at: string; // ISO 8601
}
```

## System and Infrastructure Events

### system.maintenance.started

Emitted when system maintenance begins.

**Schema:**
```typescript
interface MaintenanceStartedPayload {
  maintenance_id: string;
  title: string;
  description: string;
  affected_services: string[];
  started_at: string; // ISO 8601
  estimated_end_at?: string; // ISO 8601
  severity: 'low' | 'medium' | 'high' | 'critical';
  maintenance_type: 'scheduled' | 'emergency';
}
```

### system.error.emmitted

Emitted when system errors occur.

**Schema:**
```typescript
interface SystemErrorPayload {
  error_id: string;
  service: string;
  error_type: string;
  message: string;
  stack_trace?: string;
  user_id?: string;
  request_id?: string;
  severity: 'low' | 'medium' | 'high' | 'critical';
  occurred_at: string; // ISO 8601
  metadata?: Record<string, any>;
}
```

## Custom Event Schemas

### Creating Custom Schemas

When creating custom event types, follow these guidelines:

1. **Use descriptive names**: `resource.action` format (e.g., `document.shared`, `meeting.scheduled`)
2. **Include timestamps**: Always include relevant timestamp fields
3. **Provide context**: Include sufficient context for downstream processing
4. **Use consistent data types**: Maintain consistency across similar events
5. **Version your schemas**: Plan for schema evolution

**Example Custom Event:**
```typescript
interface DocumentSharedPayload {
  document_id: string;
  document_title: string;
  owner_id: string;
  shared_with: Array<{
    user_id: string;
    email: string;
    permission: 'read' | 'write' | 'admin';
  }>;
  shared_at: string; // ISO 8601
  expiry_date?: string; // ISO 8601
  access_link?: string;
  message?: string;
}
```

## Schema Validation

Hook0 supports JSON Schema validation for event payloads:

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "properties": {
    "user_id": {
      "type": "string",
      "pattern": "^user_[a-zA-Z0-9]{10,}$"
    },
    "email": {
      "type": "string",
      "format": "email"
    },
    "created_at": {
      "type": "string",
      "format": "date-time"
    }
  },
  "required": ["user_id", "email", "created_at"],
  "additionalProperties": false
}
```

## Best Practices

### Payload Design
- Keep payloads focused and avoid unnecessary data
- Use ISO 8601 format for all timestamps
- Include unique identifiers for all entities
- Use consistent field naming conventions
- Provide meaningful descriptions and context

### Data Types
- Use appropriate data types (strings for IDs, numbers for amounts)
- Include currency information with monetary amounts
- Use enums for status fields with known values
- Include units for measurements

### Backwards Compatibility
- Add new fields as optional
- Never remove existing required fields
- Use semantic versioning for major schema changes
- Provide migration guides for breaking changes

### Documentation
- Document all fields and their purposes
- Provide example payloads
- Explain business context and use cases
- Include validation rules and constraints

For more information on implementing these schemas in your applications, see our [Integration Guides](../how-to-guides/index.md) and [SDK Documentation](sdk/).
