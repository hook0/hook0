# Getting Started with Hook0

This tutorial will guide you through creating an application and sending your first webhook event. By the end, you will have a working webhook integration.

## Prerequisites

- **For Self-Hosted**: Docker and Docker Compose installed. **🚨 You'll need to replace the API base URL with your own domain or http://localhost:8081 in the examples.**
- **For Cloud**: A Hook0 account at [hook0.com](https://www.hook0.com/)
- Basic understanding of HTTP APIs
- cURL or similar HTTP client
- Familiarity with [Hook0 Core Concepts](../explanation/what-is-hook0.md#core-concepts)

### Set Up Environment Variables

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

## Step 1: Start Hook0 (Self-Hosted Only)

Start Hook0 using Docker Compose:

```bash
# Clone the repository
git clone https://github.com/hook0/hook0.git
cd hook0

# Start all services
docker-compose up -d

# Wait for services to be ready
sleep 10

# Verify API is running (check swagger endpoint)
curl -s https://app.hook0.com/api/v1/swagger.json | head -c 100
```

Access the dashboard at http://localhost:8001 and create your first organization account.

:::tip Self-Hosted Email Verification
For self-hosted instances, after registering you need to verify your email:
1. Check Mailpit at **http://localhost:8025** for the verification email
2. Click the verification link in the email
3. Return to the dashboard and log in

Without email verification, you'll see an `AuthEmailNotVerified` error when trying to access the API.
:::

:::info Keep going?
At this point you have two choices:
- **Use the UI tutorial**, displayed on the dashboard, for a quick start. You can **stop the tutorial right here** and let the UI guides you.
- **Skip it and go to step 2**
:::


## Step 2: Create Your First Application

Applications represent individual services or projects within your organization.

1. **Select your organization** in the left sidebar.
2. At the end, **Click "Create new application"**
3. **Fill in the Application Name**: My First App
4. **Click "Create"**

In the header will be displayed an application ID (**App ID**) that looks like `b676db07-5a75-4359-a6ef-89c79706072e`. Please keep it, you'll need it for later.

## Step 3: Get Your API Token

To send events to Hook0, you need an API token.

1. **Go to API keys** the sidebar of your application.
2. **Click "Create new API Key"**
3. **Give it a name**
4. **Copy the token** - it looks like this: `49757726-4107-45d4-a262-e438d4f17ab4`
   
## Step 4: Create an Event Type

Event types define the structure of events your application can send.

### - Using the Dashboard:

1. **Navigate to Event Types** in your application
2. **Click "Create Event Type"**
3. **Define your event type**: `user.account.created`

### - OR using the API:

```bash
curl -X POST "$HOOK0_API/event_types" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "'"$APP_ID"'",
    "service": "user",
    "resource_type": "account",
    "verb": "created"
  }'
```

**Response:**
```json
{
  "service_name": "user",
  "resource_type_name": "account",
  "verb_name": "created",
  "event_type_name": "user.account.created"
}
```

This creates an event type named `user.account.created` (composed from `service.resource_type.verb`).

## Step 5: Create a Webhook Subscription

Subscriptions define where Hook0 should send webhook notifications.

For this tutorial, use [webhook.site](https://webhook.site) to create a test endpoint:

1. **Visit [webhook.site](https://webhook.site)**
2. **Copy your unique URL** (e.g., `https://webhook.site/abc123`)

### Create the Subscription

```bash
curl -X POST "$HOOK0_API/subscriptions" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "'"$APP_ID"'",
    "is_enabled": true,
    "event_types": ["user.account.created"],
    "description": "Tutorial webhook endpoint",
    "label_key": "environment",
    "label_value": "tutorial",
    "target": {
      "type": "http",
      "method": "POST",
      "url": "https://webhook.site/YOUR_UNIQUE_ID",
      "headers": {
        "X-Custom-Header": "my-value"
      }
    }
  }'
```

You'll receive a response with the subscription ID and secret:

```json
{
  "subscription_id": "{SUBSCRIPTION_ID}",
  "secret": "{SECRET}",
  ...
}
```

⚠️ **Important**: Save the `secret` - you'll need it to verify webhook signatures.

Keep the `subscription_id`, you'll need it in step 8.

## Step 6: Send Your First Event

Now let's trigger a webhook by sending an event to Hook0:

:::warning Labels Required
Every event must include at least one label. Labels are used to route events to the correct subscriptions. An empty `labels: {}` object will be rejected by the API.
:::

```bash
curl -X POST "$HOOK0_API/event" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "'"$APP_ID"'",
    "event_id": "'$(uuidgen)'",
    "event_type": "user.account.created",
    "payload": "{\"user_id\": 123, \"email\": \"john.doe@example.com\"}",
    "payload_content_type": "application/json",
    "labels": {
      "environment": "tutorial"
    },
    "occurred_at": "'$(date -u +"%Y-%m-%dT%H:%M:%SZ")'"
  }'
```

:::tip Event ID Format
The `event_id` **must be a valid UUID** (e.g., `550e8400-e29b-41d4-a716-446655440000`). Using `$(uuidgen)` generates this automatically. Non-UUID values will be rejected by the API.
:::

:::warning Payload Format
The `payload` field must be a **JSON-encoded string**, not a raw object. See [Understanding Payload Format](#understanding-payload-format) below for details.
:::

### Understanding Payload Format

A common source of confusion is the payload format. **The `payload` field must be a JSON-encoded string, not a JavaScript/JSON object.**

#### Why String Format?

Hook0 forwards payloads exactly as received without re-serialization. This ensures:
- No data transformation or modification
- Preserved formatting and whitespace
- Support for any content type (not just JSON)
- Zero overhead processing

#### Correct vs Incorrect Format

**✅ Correct** - String with escaped quotes:
```json
{
  "event_type": "user.account.created",
  "payload": "{\"user_id\": \"123\", \"email\": \"user@example.com\"}",
  "payload_content_type": "application/json"
}
```

**❌ Incorrect** - Raw object:
```json
{
  "event_type": "user.account.created",
  "payload": {"user_id": "123", "email": "user@example.com"},
  "payload_content_type": "application/json"
}
```
### Expected Response

```json
{
  "application_id": "{APP_ID}",
  "event_id": "{EVENT_ID}",
  "received_at": "2024-01-15T10:30:01Z"
}
```

## Step 7: Verify Webhook Delivery

1. **Check webhook.site** - you should see a new request containing your event payload.

2. **Check the Hook0 dashboard** - navigate to Request Attempts to see:
   - Event details
   - Delivery status
   - Retry attempts (if any)

## Step 8: Verify Webhook Signature

Hook0 signs all webhook deliveries with HMAC-SHA256. You should always verify signatures to ensure webhooks are authentic.

:::tip Forgot your application secret?
You can get it with:

```bash
curl "$HOOK0_API/subscriptions/$SUBSCRIPTION_ID?application_id=$APP_ID" \
  -H "Authorization: Bearer $HOOK0_TOKEN"
```
Note the `secret` field in the response.
:::


### Verify the Signature

See [Implementing Webhook Authentication](./webhook-authentication.md) for complete signature verification code in JavaScript, Python, and Go.

## 🎉 Congrats, you made it to the end!

### What You've Learned

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
- Verify event type and labels match subscription
- Check that the URL you specified as target in your subscription is accessible

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
