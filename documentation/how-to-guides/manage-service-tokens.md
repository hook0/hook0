# Managing Service Tokens

This guide covers creating and managing Hook0 service tokens for API access, including how to use token attenuation to follow the principle of least privilege.

## Quick Start (2 minutes)

1. Log in to [Hook0](https://app.hook0.com/)
2. Select your **Organization**
3. Click **Service Tokens** in the sidebar
4. Click **Create Service Token**
5. Name your token and copy it

For production environments, see [Token Attenuation](#token-attenuation) below.

## What is a Service Token?

A service token is a credential that authenticates API requests to Hook0 on behalf of your organization. Unlike user credentials, service tokens:

- Don't require interactive login
- Can be used in automated systems (CI/CD, scripts, AI assistants)
- Have organization-wide scope by default
- Can be restricted through attenuation

## Creating a Service Token

### Step 1: Access the Service Tokens Page

1. Log in to the Hook0 dashboard
2. Select your organization from the dropdown
3. Navigate to **Service Tokens** in the left sidebar

### Step 2: Create the Token

1. Click **Create Service Token**
2. Enter a descriptive name (e.g., "Production API", "Claude MCP", "CI Pipeline")
3. Click **Create**

:::warning Save Your Token
The full token is only shown **once**. Copy it immediately and store it securely. If you lose it, you'll need to create a new one.
:::

### Step 3: Configure Your Environment

Set the token as an environment variable:

```bash
export HOOK0_API_TOKEN="your-service-token-here"
```

Or use it in your application configuration.

---

## Token Attenuation

### What is Token Attenuation?

Token attenuation is a security feature that lets you create **restricted versions** of your service token. Think of it like giving someone a copy of your house key that only opens the front door, not the entire house.

```
+-------------------------------------------------------------------+
|                      Root Service Token                            |
|                    (Full organization access)                      |
+-------------------------------------------------------------------+
                              |
            +-----------------+-----------------+
            |                 |                 |
            v                 v                 v
+-------------------+ +-------------------+ +-------------------+
|  Attenuated #1    | |  Attenuated #2    | |  Attenuated #3    |
|  App: Order Svc   | |  App: User Events | |  All apps         |
|  Expires: 30 days | |  Expires: never   | |  Expires: 1 hour  |
+-------------------+ +-------------------+ +-------------------+
```

### Why Does This Matter?

Without attenuation, a service token gives **full access** to your entire organization:

- All applications
- All subscriptions
- All events
- Forever (no expiration)

This is risky because:

- **If the token leaks**, attackers have complete access to your webhook infrastructure
- **AI assistants see more data than necessary** for their specific task
- **No way to limit access** for specific use cases or team members
- **No automatic expiration** means forgotten tokens remain valid indefinitely

### How It Works

Hook0 uses [Biscuit tokens](https://www.biscuitsec.org/), a cryptographic token format that supports **offline attenuation**. This means:

1. **You don't need Hook0's permission** to create restricted tokens
2. **Restrictions can only be added**, never removed
3. **The original token remains unchanged**
4. **Attenuated tokens are cryptographically linked** to their parent

When you attenuate a token, you're essentially adding rules like:

- "This token can only access application X"
- "This token expires on date Y"

These rules are embedded in the token itself and verified by Hook0's API on every request.

### How to Attenuate Your Token

1. Go to **Service Tokens** in the Hook0 dashboard
2. Click **Show** on your token
3. Under **Attenuate your token**, select:
   - **Specific application** — Limit access to one app
   - **Expiration date** — Set an automatic expiry
4. Click **Generate** to create the attenuated token
5. Use the new token in place of the original

:::warning Important
If you revoke the **root token**, all tokens derived from it are automatically invalidated. This gives you a single kill switch for all related tokens.
:::

### Attenuation Use Cases

| Use Case | Recommended Attenuation |
|----------|------------------------|
| AI Assistant (Claude, Cursor) | Single app + 30-day expiration |
| CI/CD Pipeline | Single app + no expiration |
| Development/Testing | Test app only + 7-day expiration |
| Production Monitoring | Read-only app access + no expiration |
| Contractor Access | Specific app + project duration expiration |

---

## Best Practices

### 1. Use Separate Tokens per Environment

Create different tokens for development, staging, and production:

```bash
# Development
export HOOK0_API_TOKEN="dev-token-attenuated-to-dev-app"

# Staging
export HOOK0_API_TOKEN="staging-token-attenuated-to-staging-app"

# Production
export HOOK0_API_TOKEN="prod-token-attenuated-to-prod-app"
```

### 2. Always Attenuate for Third-Party Tools

When using tokens with AI assistants or external services:

1. Create an attenuated token limited to specific applications
2. Set an expiration date
3. Store the token securely in the tool's configuration

### 3. Use Descriptive Names

Name tokens clearly to identify their purpose:

- Good: "Claude MCP - Order Service - Expires 2024-06"
- Bad: "Token 1"

### 4. Rotate Tokens Regularly

For sensitive environments:

1. Create a new token
2. Update your applications to use the new token
3. Verify everything works
4. Revoke the old token

### 5. Monitor Token Usage

Review the Hook0 audit trail regularly to:

- Detect unauthorized access attempts
- Verify tokens are being used as expected
- Identify tokens that should be revoked

---

## Revoking Tokens

To revoke a service token:

1. Go to **Service Tokens** in the dashboard
2. Find the token to revoke
3. Click the **Delete** button
4. Confirm the deletion

:::danger Revocation is Immediate
Once revoked, the token and all its attenuated derivatives immediately stop working. Ensure your applications are updated before revoking.
:::

---

## Troubleshooting

### "Authentication failed" Error

**Possible causes:**

- Token was revoked
- Token expired (if attenuated with expiration)
- Token has extra whitespace
- Using attenuated token outside its allowed scope

**Solutions:**

1. Verify the token in the dashboard
2. Check if it has an expiration date
3. Ensure no extra spaces in configuration
4. Create a new attenuated token if needed

### "Forbidden" Error (403)

**Possible causes:**

- Attenuated token doesn't have access to the requested resource
- Token was attenuated to a different application

**Solutions:**

1. Check which application the token is attenuated to
2. Create a new attenuated token with correct scope

### Token Not Appearing in Dashboard

Service tokens belong to organizations, not users. Ensure you:

1. Selected the correct organization
2. Have admin permissions for the organization

---

## Related Resources

- **[API Reference](../openapi/intro)** - Complete API documentation
- **[MCP Server for AI Assistants](../reference/mcp-for-ia-assistant.md)** - Using tokens with AI assistants
- **[Configuration Reference](../reference/configuration.md)** - Environment variable setup

---

*Need help? Contact us through our [Discord community](https://www.hook0.com/community).*
