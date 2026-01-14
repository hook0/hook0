---
title: Service Tokens
description: API credentials for automated systems and programmatic access to Hook0
---

# Service Tokens

A service token is a credential that authenticates API requests to Hook0 on behalf of your [organization](organizations.md). Unlike user credentials that require interactive login, service tokens are designed for automated systems and programmatic access.

## Key Points

- Service tokens authenticate non-interactive API requests
- Tokens are scoped to an [organization](organizations.md) by default
- Tokens can be attenuated (restricted) without contacting Hook0
- Revoking a root token invalidates all derived tokens

## Service Tokens vs User Credentials

```
User Login                          Service Token
    |                                     |
    v                                     v
+------------------+              +------------------+
|  Interactive     |              |  Non-interactive |
|  Browser session |              |  API calls       |
|  MFA possible    |              |  Automated       |
+------------------+              +------------------+
```

Service tokens are ideal for:

- CI/CD pipelines
- Backend services
- Scripts and automation
- AI assistants and MCP servers

## Biscuit Token Format

Hook0 uses [Biscuit tokens](https://www.biscuitsec.org/), a modern cryptographic token format with powerful security properties:

- **Offline attenuation** - Create restricted tokens without contacting Hook0
- **Cryptographic verification** - Tokens are tamper-proof
- **Cascading revocation** - Revoking a parent invalidates all children

## Attenuation

Attenuation lets you create restricted versions of a token:

```
Root Token (full access)
    |
    +-- Attenuated Token A (App: Orders, Expires: 30 days)
    |
    +-- Attenuated Token B (App: Users, No expiration)
    |
    +-- Attenuated Token C (App: Payments, Expires: 1 hour)
```

Common restrictions include:

- **[Application](applications.md) scope** - Limit to specific [applications](applications.md)
- **Expiration** - Automatic token expiry
- **Operation limits** - Restrict to read-only operations

This is particularly useful when sharing tokens with third-party services or temporary workers.

## Security Best Practices

- **Store securely** - Treat service tokens like passwords
- **Use attenuation** - Always restrict tokens for third-party tools
- **Rotate regularly** - Create new tokens and revoke old ones periodically
- **Audit usage** - Monitor API calls made with each token

## What's Next?

- [Managing Service Tokens](/how-to-guides/manage-service-tokens) - Create, attenuate, and revoke tokens
- [Organizations](organizations.md) - Understanding token scope
- [Applications](applications.md) - Restricting tokens to specific apps
