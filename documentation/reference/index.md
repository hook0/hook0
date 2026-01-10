# Reference

Technical specifications for Hook0. Use this reference when you need exact details about APIs, configuration options, data formats, or error codes.

## Quick Navigation

| I want to... | Go to |
|--------------|-------|
| Integrate with the API | [API Reference](../openapi/intro) |
| Use a client library | [SDKs](#sdks) |
| Configure Hook0 | [Configuration](configuration.md) |
| Understand an error | [Error Codes](error-codes.md) |
| Define event payloads | [Event Schemas](event-schemas.md) |
| Use AI assistants | [MCP Server](mcp.md) |

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

## Configuration

### [Configuration Reference](configuration.md)

All environment variables and configuration options for Hook0 server and worker processes.

---

## Data Formats

### [Event Schemas](event-schemas.md)

Standardized payload structures and validation rules for your event types.

### [Error Codes](error-codes.md)

Complete list of HTTP status codes and Hook0-specific error codes with resolution steps.

---

## AI Assistant Integration

### [MCP Server for AI Assistants](mcp.md)

Control Hook0 using natural language with Claude, Cursor, Windsurf, or any MCP-compatible AI assistant.

---

## Related Documentation

- **[Tutorials](../tutorials/index.md)** — Step-by-step learning guides
- **[How-to Guides](../how-to-guides/index.md)** — Task-oriented problem solving
- **[Explanation](../explanation/index.md)** — Conceptual understanding
