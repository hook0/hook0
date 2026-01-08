# MCP Server for AI Assistants

Control your Hook0 webhook infrastructure using natural language with Claude, Cursor, Windsurf, or any MCP-compatible AI assistant.

## What is MCP?

The [Model Context Protocol (MCP)](https://modelcontextprotocol.io/) is an open standard that enables AI assistants to interact with external tools and data sources. Hook0's MCP server exposes your webhook infrastructure as a set of tools that AI assistants can use on your behalf.

**With Hook0 MCP, you can:**

- List and inspect applications, events, and subscriptions
- Create new webhook subscriptions through conversation
- Debug failed deliveries and retry them instantly
- Send test events without writing code
- Manage your entire webhook infrastructure using plain English

---

## Quick Start

### 1. Install the MCP Server

```bash
# Install via Cargo (Rust package manager)
cargo install hook0-mcp

# Verify installation
hook0-mcp --version
```

:::tip Don't have Rust installed?
Install Rust first: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
:::

### 2. Create a Service Token

Service tokens authenticate the MCP server with your Hook0 organization.

1. Log in to [Hook0](https://app.hook0.com/)
2. Select your **Organization** from the dropdown
3. Click **Service Tokens** in the left sidebar
4. Click **Create Service Token**
5. Name it (e.g., "Claude MCP" or "Cursor AI")
6. **Copy the token** — this is your `HOOK0_API_TOKEN`

:::warning Security Best Practice
Service tokens have full access to your organization. For production environments, consider using [token attenuation](#token-attenuation) to restrict permissions to specific applications.
:::

### 3. Configure Your AI Assistant

Choose your AI assistant below and add the configuration:

#### Claude Desktop

**macOS:** `~/Library/Application Support/Claude/claude_desktop_config.json`
**Windows:** `%APPDATA%\Claude\claude_desktop_config.json`
**Linux:** `~/.config/Claude/claude_desktop_config.json`

```json
{
  "mcpServers": {
    "hook0": {
      "command": "hook0-mcp",
      "env": {
        "HOOK0_API_TOKEN": "your-service-token-here"
      }
    }
  }
}
```

#### Cursor

**Settings → Features → MCP Servers → Add Server**

```json
{
  "hook0": {
    "command": "hook0-mcp",
    "env": {
      "HOOK0_API_TOKEN": "your-service-token-here"
    }
  }
}
```

#### Windsurf

**Settings → Cascade → MCP Servers**

```json
{
  "mcpServers": {
    "hook0": {
      "command": "hook0-mcp",
      "env": {
        "HOOK0_API_TOKEN": "your-service-token-here"
      }
    }
  }
}
```

#### Cline (VS Code Extension)

Open Cline settings and add to MCP servers:

```json
{
  "hook0": {
    "command": "hook0-mcp",
    "env": {
      "HOOK0_API_TOKEN": "your-service-token-here"
    }
  }
}
```

### 4. Start Using It

Restart your AI assistant, then try these prompts:

> "List my Hook0 applications"

> "Show me the last 10 events for my Order Notifications app"

> "Create a webhook subscription to send user.created events to https://api.example.com/webhooks"

> "Why did my last webhook delivery fail?"

---

## Configuration Reference

| Environment Variable | Default | Description |
|---------------------|---------|-------------|
| `HOOK0_API_TOKEN` | *required* | Your Hook0 service token |
| `HOOK0_API_URL` | `https://app.hook0.com` | Hook0 API base URL (change for self-hosted) |
| `HOOK0_READ_ONLY` | `false` | Restrict to read-only operations |
| `MCP_TRANSPORT` | `stdio` | Transport protocol: `stdio` or `sse` |
| `MCP_SSE_PORT` | `3000` | Port for SSE transport mode |

### Self-Hosted Configuration

For self-hosted Hook0 instances, set the API URL:

```json
{
  "mcpServers": {
    "hook0": {
      "command": "hook0-mcp",
      "env": {
        "HOOK0_API_TOKEN": "your-token",
        "HOOK0_API_URL": "https://hook0.your-domain.com"
      }
    }
  }
}
```

### Read-Only Mode

Enable read-only mode for safe observability access. Only query operations are available — no modifications possible:

```json
{
  "mcpServers": {
    "hook0": {
      "command": "hook0-mcp",
      "env": {
        "HOOK0_API_TOKEN": "your-token",
        "HOOK0_READ_ONLY": "true"
      }
    }
  }
}
```

This is recommended for:
- Shared team environments
- Monitoring and debugging without risk
- Demo or training scenarios

---

## Available Tools

The MCP server provides 17 tools organized by operation type. In read-only mode, only read operations are available.

### Read Operations

| Tool | Description | Example Prompt |
|------|-------------|----------------|
| `list_organizations` | List all accessible organizations | "Show my organizations" |
| `list_applications` | List applications in an organization | "What apps do I have?" |
| `get_application` | Get detailed application info | "Show details for app X" |
| `list_event_types` | List event types for an application | "What event types are registered?" |
| `list_subscriptions` | List webhook subscriptions | "Show all my webhooks" |
| `get_subscription` | Get subscription details | "Show webhook configuration for..." |
| `list_events` | List events for an application | "Show recent events" |
| `get_event` | Get event details with payload | "Show event abc123" |
| `list_request_attempts` | List delivery attempts | "Show delivery history for event X" |

### Write Operations

| Tool | Description | Example Prompt |
|------|-------------|----------------|
| `create_application` | Create a new application | "Create an app called Order Service" |
| `delete_application` | Delete an application | "Delete the test application" |
| `create_event_type` | Register a new event type | "Add event type order.completed" |
| `create_subscription` | Create a webhook subscription | "Create a webhook to https://..." |
| `update_subscription` | Update subscription settings | "Disable the webhook for..." |
| `delete_subscription` | Delete a subscription | "Remove the staging webhook" |
| `ingest_event` | Send a new event | "Send a test user.created event" |
| `retry_delivery` | Retry a failed delivery | "Retry the failed delivery for event X" |

---

## Resources

MCP resources provide direct access to Hook0 data through URI-based navigation. Your AI assistant can browse these like a file system.

| Resource URI | Description |
|--------------|-------------|
| `hook0://organizations` | List all organizations |
| `hook0://applications` | List all applications |
| `hook0://applications/{id}` | Single application details |
| `hook0://applications/{id}/events` | Events for an application |
| `hook0://applications/{id}/subscriptions` | Subscriptions for an application |
| `hook0://applications/{id}/event_types` | Event types for an application |
| `hook0://events/{id}` | Single event details |
| `hook0://events/{id}/attempts` | Delivery attempts for an event |

---

## Guided Prompts

The MCP server includes 3 guided prompts that walk you through common workflows step-by-step.

### `create_webhook_subscription`

Interactive guide to create a new webhook subscription.

**What it does:**
1. Helps you select the target application
2. Guides you through event type selection
3. Configures the webhook endpoint URL
4. Sets up any additional options

**Example:**
> "I want to create a webhook subscription"

### `debug_event_delivery`

Troubleshoot webhook delivery issues systematically.

**What it does:**
1. Examines the event details and payload
2. Reviews all delivery attempts
3. Analyzes HTTP status codes and response times
4. Suggests fixes based on common issues

**Example:**
> "Help me debug why my webhooks are failing"

### `setup_application`

Complete guide for initial application setup.

**What it does:**
1. Creates a new application
2. Registers your event types
3. Sets up your first subscription
4. Sends a test event

**Example:**
> "I'm new to Hook0, help me set up my first application"

---

## Example Conversations

### Listing and Inspecting

```
You: Show me my applications

Claude: I'll list your Hook0 applications.

[Uses list_applications tool]

You have 3 applications:

| Name | ID | Created |
|------|-----|---------|
| Order Service | app_abc123 | 2024-01-15 |
| User Events | app_def456 | 2024-02-20 |
| Inventory | app_ghi789 | 2024-03-10 |

Which application would you like to explore?
```

### Creating a Subscription

```
You: Create a webhook for order events to https://api.myapp.com/webhooks

Claude: I'll create a webhook subscription. Let me first check your applications...

[Uses list_applications tool]
[Uses list_event_types tool]

I found your "Order Service" application with these event types:
- order.created
- order.updated
- order.completed
- order.cancelled

Which event types should trigger this webhook? Or should I subscribe to all of them?

You: All order events

Claude: [Uses create_subscription tool]

Done! Created webhook subscription:
- **Target:** https://api.myapp.com/webhooks
- **Events:** order.* (all order events)
- **Status:** Enabled
- **Secret:** wh_sec_xxx... (save this for signature verification)

The webhook is now active and will receive events immediately.
```

### Debugging Failed Deliveries

```
You: Why are my webhooks failing?

Claude: I'll investigate your recent delivery failures.

[Uses list_applications tool]
[Uses list_events tool with failed status]
[Uses list_request_attempts tool]

I found 3 failed deliveries in the last hour:

| Event | Status | Error |
|-------|--------|-------|
| evt_123 | 503 | Service Unavailable |
| evt_124 | 503 | Service Unavailable |
| evt_125 | Timeout | No response in 30s |

All failures are for your endpoint `https://api.example.com/webhooks`.
The 503 errors suggest your server is overloaded or down.

**Recommended actions:**
1. Check your server health and logs
2. Verify the endpoint is accessible: `curl https://api.example.com/webhooks`
3. Once fixed, I can retry these deliveries for you

You: The server is back up, retry them

Claude: [Uses retry_delivery tool x3]

All 3 deliveries have been retried:
- evt_123: ✅ 200 OK
- evt_124: ✅ 200 OK
- evt_125: ✅ 200 OK

All webhooks are now delivered successfully!
```

### Sending Test Events

```
You: Send a test event to check my webhook

Claude: I'll send a test event. Which application and event type?

You: Order Service, order.created

Claude: [Uses ingest_event tool]

Test event sent:
- **Event ID:** evt_test_abc123
- **Type:** order.created
- **Payload:** {"test": true, "timestamp": "2024-01-15T10:30:00Z"}

[Uses list_request_attempts tool]

Delivery status:
- **Subscription:** Production Webhook
- **Status:** ✅ 200 OK
- **Response time:** 145ms

Your webhook is working correctly!
```

---

## Security Best Practices

### Service Token Management

Service tokens have full organization access by default. For production use, follow these security practices:

1. **[Use token attenuation](../how-to-guides/manage-service-tokens.md#token-attenuation)** to restrict access to specific applications
2. **[Set expiration dates](../how-to-guides/manage-service-tokens.md#how-to-attenuate-your-token)** for tokens used in temporary contexts
3. **[Create separate tokens](../how-to-guides/manage-service-tokens.md#best-practices)** for different environments and team members

For the complete guide, see **[Managing Service Tokens](../how-to-guides/manage-service-tokens.md)**.

:::warning Security Best Practice
When using MCP with AI assistants, always [attenuate your token](../how-to-guides/manage-service-tokens.md#token-attenuation) to limit access to only the applications the assistant needs.
:::

### Environment-Specific Configuration

**Development** (full access to test app):
```json
{
  "env": {
    "HOOK0_API_TOKEN": "dev-token-attenuated-to-test-app",
    "HOOK0_READ_ONLY": "false"
  }
}
```

**Production Monitoring** (read-only):
```json
{
  "env": {
    "HOOK0_API_TOKEN": "prod-token-attenuated",
    "HOOK0_READ_ONLY": "true"
  }
}
```

---

## Troubleshooting

### "Tool not found" Error

**Cause:** The MCP server isn't running or not configured correctly.

**Fix:**
1. Verify installation: `hook0-mcp --version`
2. Check the config file path is correct for your OS
3. Restart your AI assistant after configuration changes

### "Authentication failed" Error

**Cause:** Invalid or expired service token.

**Fix:**
1. Verify your token in the Hook0 dashboard
2. Check the token hasn't been revoked
3. Ensure no extra whitespace in the config file
4. For attenuated tokens, verify they haven't expired

### "Read-only mode" Warning

**Cause:** `HOOK0_READ_ONLY=true` is set but you're trying a write operation.

**Fix:**
- Set `HOOK0_READ_ONLY=false` if you need write access
- Or use a different configuration for write operations

### MCP Server Not Starting

**Cause:** Various startup issues.

**Debug steps:**
```bash
# Run manually to see errors
HOOK0_API_TOKEN=your-token hook0-mcp

# Check if another process is using the port (SSE mode)
lsof -i :3000
```

### No Tools Appearing

**Cause:** AI assistant hasn't loaded the MCP server.

**Fix:**
1. Completely quit and restart your AI assistant
2. Check for JSON syntax errors in the config file
3. Verify the `command` path is correct (try absolute path)

---

## Advanced Usage

### SSE Transport Mode

For web-based integrations or scenarios where stdio isn't available:

```bash
HOOK0_API_TOKEN=your-token MCP_TRANSPORT=sse MCP_SSE_PORT=3001 hook0-mcp
```

The server will listen on `http://localhost:3001/sse` for MCP connections.

### Programmatic Integration

The Hook0 MCP server can be used programmatically in your own applications:

```rust
use hook0_mcp::{Hook0McpServer, Hook0Client, Config};

let config = Config::from_env()?;
let client = Hook0Client::new(&config);
let server = Hook0McpServer::new(client, config.read_only);

// Use with rmcp runtime
```

### Docker Deployment

```dockerfile
FROM rust:latest
RUN cargo install hook0-mcp
ENV HOOK0_API_TOKEN=your-token
CMD ["hook0-mcp"]
```

---

## FAQ

### Is my data sent to the AI?

The AI assistant only sees the data returned by the tools it calls. Hook0 does not send your data to any third party — the MCP server runs locally on your machine and communicates directly with the Hook0 API.

### Can I use this with ChatGPT?

Currently, MCP is primarily supported by Claude (Anthropic), Cursor, Windsurf, and other compatible assistants. ChatGPT does not natively support MCP, but may in the future.

### What happens if I delete something by accident?

Hook0 maintains audit logs of all operations. While deleted resources cannot be automatically restored, you can contact support with the audit trail information.

### Can multiple team members use MCP?

Yes! Each team member should create their own service token. This ensures proper audit trails and allows individual token revocation if needed.

### Does this work with self-hosted Hook0?

Yes. Set `HOOK0_API_URL` to your self-hosted instance URL.

---

## Support

- **Documentation:** [Hook0 Docs](https://documentation.hook0.com/)
- **GitHub Issues:** [Report Issues](https://github.com/hook0/hook0/issues)
- **Community:** [Join Discord](https://www.hook0.com/community)
- **Crates.io:** [hook0-mcp](https://crates.io/crates/hook0-mcp)

---

*This page documents the Hook0 MCP Server v0.1.0+*
