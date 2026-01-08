# Reference

Technical reference documentation for Hook0. These information-oriented materials provide comprehensive details about APIs, configuration, schemas, and other technical specifications.

## AI Assistant Integration

### [MCP Server for AI Assistants](mcp-for-ia-assistant.md)
Control Hook0 using natural language with Claude, Cursor, Windsurf, or any MCP-compatible AI assistant.

**Includes:**
- Quick start guide for all major AI assistants
- Complete tool, resource, and prompt reference
- Example conversations and workflows
- Security best practices and token attenuation
- Troubleshooting guide

---

## API Documentation

### [API Reference](../openapi/intro)
Complete REST API documentation with all endpoints, parameters, and response formats.

**Includes:**
- Authentication endpoints
- Application management
- Event type operations
- Subscription management  
- Event creation and querying
- Webhook delivery status
- OpenAPI/Swagger specification

---

## Data Specifications

### [Event Schemas](event-schemas.md)
Standardized event payload schemas and validation rules.

**Includes:**
- Base event schema structure
- Common event type schemas (user, order, payment, etc.)
- Custom schema definition guidelines
- Validation rules and examples

---

### [Error Codes](error-codes.md)
Comprehensive list of all error codes with descriptions and resolution steps.

**Includes:**
- HTTP status codes
- Hook0-specific error codes
- Authentication errors
- Validation errors
- System errors

---

## Configuration

### [Configuration](configuration.md)
All configuration options for Hook0 server and worker processes.

**Includes:**
- Environment variables
- Configuration file options
- Database settings
- Security configuration
- Performance tuning parameters

---

## SDKs & Libraries

### [SDKs](sdk/)
Client libraries and SDKs for different programming languages.

**Available SDKs:**
- JavaScript/TypeScript SDK  
- Rust SDK (in development)

**Coming Soon:**
- Python SDK
- Go SDK
- PHP SDK
- Ruby SDK
- Java SDK
- .NET SDK

---

## Quick Reference Tables

### HTTP Status Codes
| Code | Meaning | Common Causes |
|------|---------|---------------|
| 200 | OK | Request successful |
| 201 | Created | Resource created successfully |
| 400 | Bad Request | Invalid request parameters |
| 401 | Unauthorized | Missing or invalid authentication |
| 403 | Forbidden | Insufficient permissions |
| 404 | Not Found | Resource does not exist |
| 422 | Unprocessable Entity | Validation errors |
| 500 | Internal Server Error | Server-side errors |

### Event Delivery Status
| Status | Description |
|--------|-------------|
| `waiting` | Scheduled for future delivery (retry backoff) |
| `pending` | Event queued for delivery |
| `inprogress` | Currently being delivered |
| `successful` | Successfully delivered |
| `failed` | Delivery failed (will retry) |

### Configuration Sections
| Section | Purpose |
|---------|---------|
| `database` | PostgreSQL connection settings |
| `server` | HTTP server configuration |
| `worker` | Background worker settings |
| `security` | Authentication and encryption |
| `logging` | Log levels and output |

---

## API Specification Formats

Hook0's API is documented in multiple formats:

- **[OpenAPI 3.0 Specification](../openapi/intro)** - Machine-readable API spec
- **Interactive Documentation** - Built-in Swagger UI at `/docs`
- **Postman Collection** - Ready-to-use API collection

---

*For implementation examples and step-by-step instructions, see [Tutorials](../tutorials/index.md) and [How-to Guides](../how-to-guides/index.md).*