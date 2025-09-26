# Getting Started with Hook0

This tutorial will guide you through setting up your first Hook0 project, creating an application, and sending your first webhook event. By the end, you'll have a working webhook integration.

## Prerequisites

- A Hook0 account (sign up at [hook0.com](https://www.hook0.com/))
- Basic understanding of HTTP APIs
- cURL or similar HTTP client
- Familiarity with [Hook0 Core Concepts](../explanation/what-is-hook0.md#core-concepts)

## Step 1: Create Your Organization

When you first sign up for Hook0, you'll need to create an organization. This serves as the top-level container for all your projects.

1. **Sign up** at [hook0.com](https://www.hook0.com/)
2. **Verify your email** address
3. **Create your organization**:
   - Choose a unique organization name
   - Add a description (optional)
   - Select your region

Your organization URL will be: `https://app.hook0.com/organizations/{org-id}`

## Step 2: Create Your First Application

Applications represent individual services or projects within your organization.

1. **Navigate to Applications** in the Hook0 dashboard
2. **Click "Create Application"**
3. **Fill in the details**:
   ```
   Name: My First App
   Description: Learning Hook0 basics
   ```
4. **Click "Create"**

You'll receive an application ID that looks like: `app_1234567890abcdef`

## Step 3: Get Your API Token  

To send events to Hook0, you need an API token.

1. **Go to Service Tokens** in your organization settings
2. **Click "Create Service Token"**
3. **Configure the token**:
   ```
   Name: Tutorial Token
   Permissions: 
   - application:read
   - application:write
   - event:send
   Applications: My First App
   ```
4. **Copy the token** - it looks like:
   ```
   biscuit:EoQKCAohCiEKIH0eTOWqO...
   ```

⚠️ **Important**: Save this token securely. It won't be shown again.

## Step 4: Create an Event Type

Event types define the structure of events your application can send.

### Using the Dashboard

1. **Navigate to Event Types** in your application
2. **Click "Create Event Type"**
3. **Define your event type**:
   ```
   Name: user.created
   Description: Triggered when a new user registers
   ```

### Using the API

```bash
curl -X POST "https://app.hook0.com/api/v1/event_types" \
  -H "Authorization: Bearer biscuit:YOUR_TOKEN_HERE" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "user.created",
    "description": "Triggered when a new user registers"
  }'
```

## Step 5: Create a Webhook Subscription

Subscriptions define where Hook0 should send webhook notifications.

For this tutorial, we'll use [webhook.site](https://webhook.site) to create a test endpoint:

1. **Visit [webhook.site](https://webhook.site)**
2. **Copy your unique URL** (e.g., `https://webhook.site/abc123`)

### Create the Subscription

```bash
curl -X POST "https://app.hook0.com/api/v1/subscriptions" \
  -H "Authorization: Bearer biscuit:YOUR_TOKEN_HERE" \
  -H "Content-Type: application/json" \
  -d '{
    "event_types": ["user.created"],
    "description": "Tutorial webhook endpoint",
    "target": {
      "type": "http",
      "method": "POST",
      "url": "https://webhook.site/YOUR_UNIQUE_ID",
      "headers": {
        "Content-Type": "application/json"
      }
    }
  }'
```

You'll receive a subscription ID: `sub_abcdef1234567890`

## Step 6: Send Your First Event

Now let's trigger a webhook by sending an event to Hook0:

```bash
curl -X POST "https://app.hook0.com/api/v1/event" \
  -H "Authorization: Bearer biscuit:YOUR_TOKEN_HERE" \
  -H "Content-Type: application/json" \
  -d '{
    "event_type": "user.created",
    "payload": {
      "user_id": 123,
      "email": "john.doe@example.com",
      "name": "John Doe",
      "created_at": "2024-01-15T10:30:00Z"
    },
    "labels": {
      "environment": "tutorial"
    }
  }'
```

### Expected Response

```json
{
  "event_id": "evt_1234567890abcdef",
  "status": "accepted",
  "message": "Event queued for delivery"
}
```

## Step 7: Verify Webhook Delivery

1. **Check webhook.site** - you should see a new request with:
   ```json
   {
     "event_id": "evt_1234567890abcdef",
     "event_type": "user.created", 
     "payload": {
       "user_id": 123,
       "email": "john.doe@example.com",
       "name": "John Doe",
       "created_at": "2024-01-15T10:30:00Z"
     },
     "timestamp": "2024-01-15T10:30:01Z",
     "labels": {
       "environment": "tutorial"
     }
   }
   ```

2. **Check the Hook0 dashboard** - navigate to Events to see:
   - Event details
   - Delivery status
   - Response codes
   - Retry attempts (if any)

## Step 8: Verify Webhook Signature

Hook0 signs all webhook deliveries with HMAC-SHA256. Let's verify the signature:

### Get Your Subscription Secret

```bash
curl "https://app.hook0.com/api/v1/subscriptions/{sub-id}" \
  -H "Authorization: Bearer biscuit:YOUR_TOKEN_HERE"
```

Note the `secret` field in the response.

### Verify Signature (JavaScript)

```javascript
const crypto = require('crypto');

function verifyHook0Signature(payload, signature, secret) {
    const expectedSignature = crypto
        .createHmac('sha256', secret)
        .update(payload)
        .digest('hex');
    
    return signature === `sha256=${expectedSignature}`;
}

// Example usage
const payload = '{"event_id":"evt_123",...}';
const signature = 'sha256=a1b2c3...'; // From Hook0-Signature header
const secret = 'your-subscription-secret';

const isValid = verifyHook0Signature(payload, signature, secret);
console.log('Signature valid:', isValid);
```

## What You've Learned

✅ Created a Hook0 organization and application  
✅ Generated API tokens for authentication  
✅ Defined event types for your application  
✅ Set up webhook subscriptions  
✅ Sent events through the Hook0 API  
✅ Verified webhook delivery and signatures  

## Next Steps

Now that you have the basics, try these advanced tutorials:

- [Building Your First Webhook Integration](./first-webhook-integration.md)
- [Setting up Event Types and Subscriptions](./event-types-subscriptions.md)
- [Implementing Webhook Authentication](./webhook-authentication.md)

## Common Issues

### Event Not Delivered
- Check subscription is enabled
- Verify event type matches subscription
- Check webhook endpoint is accessible
- Review logs in Hook0 dashboard

### Authentication Errors
- Ensure token has correct permissions
- Check token hasn't expired
- Verify Bearer token format
- Test with a fresh token

### Signature Verification Failed
- Use the raw request body for signature verification
- Check secret is correct
- Ensure consistent character encoding
- Verify HMAC algorithm (SHA256)

## API Reference

- [Events API](../openapi/intro)
- [Subscriptions API](../openapi/intro)
- [Event Types API](../openapi/intro)