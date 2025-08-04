# Building Your First Webhook Integration

This tutorial walks you through building a complete webhook integration between a simple e-commerce application and external services using Hook0. You'll learn how to handle different event types and build robust webhook receivers.

## What You'll Build

A mini e-commerce system that:
- Sends events when orders are created, updated, and completed
- Notifies a shipping service when orders are ready
- Updates a customer communication system
- Handles webhook retries and failures

## Prerequisites

- Completed [Getting Started](./getting-started.md) tutorial
- Node.js installed (for webhook receiver examples)
- Basic understanding of REST APIs

## Project Overview

```
E-commerce App → Hook0 → Shipping Service
                     → Email Service  
                     → Analytics Service
```

## Step 1: Set Up the Project Structure

Create a new application for this tutorial:

```bash
curl -X POST "https://api.hook0.com/api/v1/applications" \
  -H "Authorization: Bearer biscuit:YOUR_TOKEN_HERE" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "E-commerce Tutorial",
    "description": "Complete webhook integration example"
  }'
```

## Step 2: Define Event Types

Create event types for different order lifecycle events:

### Order Created Event
```bash
curl -X POST "https://api.hook0.com/api/v1/applications/{app-id}/event_types" \
  -H "Authorization: Bearer biscuit:YOUR_TOKEN_HERE" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "order.created",
    "description": "New order placed by customer"
  }'
```

### Order Updated Event
```bash
curl -X POST "https://api.hook0.com/api/v1/applications/{app-id}/event_types" \
  -H "Authorization: Bearer biscuit:YOUR_TOKEN_HERE" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "order.updated",
    "description": "Order details modified"
  }'
```

### Order Completed Event
```bash
curl -X POST "https://api.hook0.com/api/v1/applications/{app-id}/event_types" \
  -H "Authorization: Bearer biscuit:YOUR_TOKEN_HERE" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "order.completed",
    "description": "Order ready for shipping"
  }'
```

## Step 3: Create Webhook Receivers

Let's build webhook receivers for each service. First, create a simple Node.js server:

### Basic Webhook Server

```javascript
// webhook-server.js
const express = require('express');
const crypto = require('crypto');
const app = express();

// Middleware to capture raw body for signature verification
app.use('/webhooks', express.raw({ type: 'application/json' }));

// Signature verification middleware
function verifySignature(secret) {
  return (req, res, next) => {
    const signature = req.headers['hook0-signature'];
    if (!signature) {
      return res.status(401).json({ error: 'Missing signature' });
    }

    const expectedSignature = crypto
      .createHmac('sha256', secret)
      .update(req.body)
      .digest('hex');

    if (signature !== `sha256=${expectedSignature}`) {
      return res.status(401).json({ error: 'Invalid signature' });
    }

    // Parse JSON after verification
    req.body = JSON.parse(req.body);
    next();
  };
}

// Shipping service webhook
app.post('/webhooks/shipping', verifySignature('SHIPPING_SECRET'), (req, res) => {
  const { event_type, payload } = req.body;
  
  console.log(`Shipping webhook received: ${event_type}`);
  
  if (event_type === 'order.completed') {
    // Process shipping logic
    console.log(`Creating shipment for order ${payload.order_id}`);
    
    // Simulate shipping API call
    const trackingNumber = `TRACK-${Date.now()}`;
    console.log(`Tracking number assigned: ${trackingNumber}`);
  }
  
  res.json({ status: 'processed' });
});

// Email service webhook  
app.post('/webhooks/email', verifySignature('EMAIL_SECRET'), (req, res) => {
  const { event_type, payload } = req.body;
  
  console.log(`Email webhook received: ${event_type}`);
  
  switch (event_type) {
    case 'order.created':
      console.log(`Sending order confirmation to ${payload.customer_email}`);
      break;
    case 'order.updated':
      console.log(`Sending order update to ${payload.customer_email}`);
      break;
    case 'order.completed':
      console.log(`Sending shipping notification to ${payload.customer_email}`);
      break;
  }
  
  res.json({ status: 'email_queued' });
});

// Analytics service webhook
app.post('/webhooks/analytics', verifySignature('ANALYTICS_SECRET'), (req, res) => {
  const { event_type, payload, labels } = req.body;
  
  console.log(`Analytics webhook received: ${event_type}`);
  
  // Track different metrics based on event type
  const metrics = {
    'order.created': { metric: 'orders_created', value: 1 },
    'order.updated': { metric: 'orders_updated', value: 1 },
    'order.completed': { metric: 'orders_completed', value: payload.total_amount }
  };
  
  const metric = metrics[event_type];
  if (metric) {
    console.log(`Recording metric: ${metric.metric} = ${metric.value}`);
  }
  
  res.json({ status: 'recorded' });
});

const PORT = process.env.PORT || 3000;
app.listen(PORT, () => {
  console.log(`Webhook server running on port ${PORT}`);
});
```

### Install Dependencies

```bash
npm init -y
npm install express
```

### Run the Server

```bash
node webhook-server.js
```

## Step 4: Create Subscriptions

Now create subscriptions for each service. For testing, you can use ngrok to expose your local server:

```bash
# Install ngrok if you haven't already
ngrok http 3000
```

Use the ngrok URL for your subscriptions:

### Shipping Service Subscription
```bash
curl -X POST "https://api.hook0.com/api/v1/applications/{app-id}/subscriptions" \
  -H "Authorization: Bearer biscuit:YOUR_TOKEN_HERE" \
  -H "Content-Type: application/json" \
  -d '{
    "event_types": ["order.completed"],
    "description": "Shipping service notifications",
    "target": {
      "type": "http",
      "method": "POST",
      "url": "https://your-ngrok-url.ngrok.io/webhooks/shipping",
      "headers": {
        "Content-Type": "application/json"
      }
    }
  }'
```

### Email Service Subscription
```bash
curl -X POST "https://api.hook0.com/api/v1/applications/{app-id}/subscriptions" \
  -H "Authorization: Bearer biscuit:YOUR_TOKEN_HERE" \
  -H "Content-Type: application/json" \
  -d '{
    "event_types": ["order.created", "order.updated", "order.completed"],
    "description": "Email service notifications",
    "target": {
      "type": "http",
      "method": "POST", 
      "url": "https://your-ngrok-url.ngrok.io/webhooks/email",
      "headers": {
        "Content-Type": "application/json"
      }
    }
  }'
```

### Analytics Service Subscription
```bash
curl -X POST "https://api.hook0.com/api/v1/applications/{app-id}/subscriptions" \
  -H "Authorization: Bearer biscuit:YOUR_TOKEN_HERE" \
  -H "Content-Type: application/json" \
  -d '{
    "event_types": ["order.created", "order.updated", "order.completed"],
    "description": "Analytics tracking",
    "target": {
      "type": "http",
      "method": "POST",
      "url": "https://your-ngrok-url.ngrok.io/webhooks/analytics", 
      "headers": {
        "Content-Type": "application/json"
      }
    }
  }'
```

Note the subscription IDs returned - you'll need them to get the secrets.

## Step 5: Get Subscription Secrets

Get the secrets for signature verification:

```bash
# Get shipping subscription secret
curl "https://api.hook0.com/api/v1/applications/{app-id}/subscriptions/{shipping-sub-id}" \
  -H "Authorization: Bearer biscuit:YOUR_TOKEN_HERE"

# Get email subscription secret  
curl "https://api.hook0.com/api/v1/applications/{app-id}/subscriptions/{email-sub-id}" \
  -H "Authorization: Bearer biscuit:YOUR_TOKEN_HERE"

# Get analytics subscription secret
curl "https://api.hook0.com/api/v1/applications/{app-id}/subscriptions/{analytics-sub-id}" \
  -H "Authorization: Bearer biscuit:YOUR_TOKEN_HERE"
```

Update your webhook server with the actual secrets from the responses.

## Step 6: Simulate E-commerce Events

Create a script to simulate your e-commerce application:

```javascript
// ecommerce-simulator.js
const fetch = require('node-fetch');

const HOOK0_TOKEN = 'biscuit:YOUR_TOKEN_HERE';
const HOOK0_API = 'https://api.hook0.com/api/v1/events';

async function sendEvent(eventType, payload, labels = {}) {
  const response = await fetch(HOOK0_API, {
    method: 'POST',
    headers: {
      'Authorization': `Bearer ${HOOK0_TOKEN}`,
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({
      event_type: eventType,
      payload,
      labels
    })
  });
  
  const result = await response.json();
  console.log(`Sent ${eventType}:`, result);
  return result;
}

// Simulate order lifecycle
async function simulateOrder() {
  const orderId = `order_${Date.now()}`;
  const customerEmail = 'customer@example.com';
  
  console.log('=== Starting Order Simulation ===');
  
  // 1. Order Created
  await sendEvent('order.created', {
    order_id: orderId,
    customer_email: customerEmail,
    items: [
      { sku: 'SHIRT-001', quantity: 2, price: 29.99 },
      { sku: 'PANTS-002', quantity: 1, price: 59.99 }
    ],
    total_amount: 119.97,
    currency: 'USD',
    created_at: new Date().toISOString()
  }, {
    environment: 'tutorial',
    source: 'web_checkout'
  });
  
  // Wait a bit
  await new Promise(resolve => setTimeout(resolve, 2000));
  
  // 2. Order Updated (address change)
  await sendEvent('order.updated', {
    order_id: orderId,
    customer_email: customerEmail,
    changes: ['shipping_address'],
    shipping_address: {
      street: '123 New Street',
      city: 'New City',
      state: 'NY',
      zip: '10001'
    },
    updated_at: new Date().toISOString()
  }, {
    environment: 'tutorial',
    change_type: 'shipping_address'
  });
  
  // Wait a bit more
  await new Promise(resolve => setTimeout(resolve, 2000));
  
  // 3. Order Completed (payment processed, ready to ship)
  await sendEvent('order.completed', {
    order_id: orderId,
    customer_email: customerEmail,
    payment_status: 'completed',
    total_amount: 119.97,
    currency: 'USD',
    completed_at: new Date().toISOString()
  }, {
    environment: 'tutorial',
    payment_method: 'credit_card'
  });
  
  console.log('=== Order Simulation Complete ===');
}

// Run simulation
simulateOrder().catch(console.error);
```

Install node-fetch:
```bash
npm install node-fetch@2
```

Run the simulation:
```bash
node ecommerce-simulator.js
```

## Step 7: Monitor the Integration

### Check Your Webhook Server
You should see logs like:
```
Email webhook received: order.created
Sending order confirmation to customer@example.com
Analytics webhook received: order.created  
Recording metric: orders_created = 1
Email webhook received: order.updated
Sending order update to customer@example.com
Analytics webhook received: order.updated
Recording metric: orders_updated = 1
Shipping webhook received: order.completed
Creating shipment for order order_1641234567890
Tracking number assigned: TRACK-1641234567891
```

### Check Hook0 Dashboard
1. Navigate to Events in your application
2. You should see 3 events with delivery details
3. Check Request Attempts to see delivery status
4. Review any failed deliveries and retry information

## Step 8: Handle Failures Gracefully

Add error handling to your webhook receivers:

```javascript
// Enhanced webhook handler with error handling
app.post('/webhooks/shipping', verifySignature('SHIPPING_SECRET'), async (req, res) => {
  try {
    const { event_type, payload } = req.body;
    
    if (event_type === 'order.completed') {
      // Simulate potential failure
      if (Math.random() < 0.2) { // 20% failure rate
        throw new Error('Shipping service temporarily unavailable');
      }
      
      console.log(`Creating shipment for order ${payload.order_id}`);
      
      // Simulate API call delay
      await new Promise(resolve => setTimeout(resolve, 1000));
      
      const trackingNumber = `TRACK-${Date.now()}`;
      console.log(`Tracking number assigned: ${trackingNumber}`);
    }
    
    res.json({ status: 'processed' });
  } catch (error) {
    console.error('Shipping webhook error:', error.message);
    // Return 500 to trigger Hook0 retry
    res.status(500).json({ error: error.message });
  }
});
```

## Step 9: Implement Idempotency

Add idempotency to handle duplicate events:

```javascript
// In-memory store for processed events (use Redis in production)
const processedEvents = new Set();

app.post('/webhooks/analytics', verifySignature('ANALYTICS_SECRET'), (req, res) => {
  const { event_id, event_type, payload } = req.body;
  
  // Check if we've already processed this event
  if (processedEvents.has(event_id)) {
    console.log(`Duplicate event ignored: ${event_id}`);
    return res.json({ status: 'already_processed' });
  }
  
  try {
    console.log(`Analytics webhook received: ${event_type}`);
    
    // Process the event
    const metrics = {
      'order.created': { metric: 'orders_created', value: 1 },
      'order.updated': { metric: 'orders_updated', value: 1 },
      'order.completed': { metric: 'orders_completed', value: payload.total_amount }
    };
    
    const metric = metrics[event_type];
    if (metric) {
      console.log(`Recording metric: ${metric.metric} = ${metric.value}`);
    }
    
    // Mark as processed
    processedEvents.add(event_id);
    
    res.json({ status: 'recorded' });
  } catch (error) {
    console.error('Analytics webhook error:', error.message);
    res.status(500).json({ error: error.message });
  }
});
```

## What You've Learned

✅ Built a complete webhook integration for an e-commerce system  
✅ Created multiple event types for different business events  
✅ Set up multiple webhook receivers with proper signature verification  
✅ Handled webhook failures and retries gracefully  
✅ Implemented idempotency to handle duplicate events  
✅ Monitored webhook deliveries through Hook0 dashboard  

## Best Practices Demonstrated

- **Signature Verification**: Always verify webhook signatures
- **Error Handling**: Return appropriate HTTP status codes
- **Idempotency**: Handle duplicate events gracefully
- **Monitoring**: Log webhook processing for debugging
- **Structured Payloads**: Use consistent event payload schemas

## Next Steps

- [Setting up Event Types and Subscriptions](./event-types-subscriptions.md)
- [Implementing Webhook Authentication](./webhook-authentication.md)
- [Debugging Failed Webhook Deliveries](../how-to-guides/debug-failed-webhooks.md)

## Production Considerations

### Security
- Use HTTPS endpoints in production
- Store secrets securely (environment variables, key management)
- Implement rate limiting on webhook endpoints
- Add authentication beyond signatures if needed

### Reliability
- Use persistent storage for idempotency tracking
- Implement proper error logging and alerting
- Set up monitoring for webhook endpoint health
- Consider implementing circuit breakers for external calls

### Performance
- Process webhooks asynchronously if possible
- Batch database operations
- Use connection pooling
- Monitor webhook processing times