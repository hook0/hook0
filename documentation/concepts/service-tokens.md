---
title: Service tokens
description: API credentials for automated systems and programmatic access to Hook0
---

# Service tokens

A service token authenticates API requests to Hook0 on behalf of your [organization](organizations.md). Unlike user credentials (which require interactive login), service tokens are for automated systems and scripts.

## Key points

- Service tokens authenticate non-interactive API requests
- Tokens are scoped to an [organization](organizations.md) by default
- You can attenuate (restrict) tokens without contacting Hook0
- Revoking a root token invalidates all derived tokens

## Service tokens vs user credentials

```mermaid
flowchart TD
    A["User Login"]:::customer
    B["Service Token"]:::external
    C["Interactive<br/>Browser session<br/>MFA possible"]:::customer
    D["Non-interactive<br/>API calls<br/>Automated"]:::external

    A --> C
    B --> D

    classDef external fill:#dbeafe,stroke:#60a5fa,color:#1e3a5f
    classDef customer fill:#ffedd5,stroke:#fb923c,color:#7c2d12

    click B "/concepts/service-tokens" "Service Tokens"
```

Service tokens are the right choice for:

- CI/CD pipelines
- Backend services
- Scripts and automation
- AI assistants and MCP servers

## Biscuit token format

Hook0 uses [Biscuit tokens](https://www.biscuitsec.org/), a cryptographic token format with these properties:

- Offline attenuation: create restricted tokens without contacting Hook0
- Cryptographic verification: tokens are tamper-proof
- Cascading revocation: revoking a parent invalidates all children

## Attenuation

You can create restricted versions of a token without calling the Hook0 API:

```mermaid
flowchart TD
    Root["Root Token<br/>(full access)"]:::processing
    Root --> A["Attenuated Token A<br/>App: Orders, Expires: 30 days"]:::external
    Root --> B["Attenuated Token B<br/>App: Users, No expiration"]:::external
    Root --> C["Attenuated Token C<br/>App: Payments, Expires: 1 hour"]:::external

    classDef external fill:#dbeafe,stroke:#60a5fa,color:#1e3a5f
    classDef processing fill:#ede9fe,stroke:#a78bfa,color:#3b0764

    click Root "/how-to-guides/manage-service-tokens" "Manage Service Tokens"
```

Common restrictions:

- [Application](applications.md) scope: limit to specific [applications](applications.md)
- Expiration: automatic token expiry
- Operation limits: restrict to read-only operations

This is useful when sharing tokens with third-party services or temporary workers.

## Security practices

- Treat service tokens like passwords
- Always restrict tokens before giving them to third-party tools
- Rotate periodically: create new tokens and revoke old ones
- Monitor API calls made with each token

## What's next?

- [Managing Service Tokens](/how-to-guides/manage-service-tokens) - Create, attenuate, and revoke tokens
- [Organizations](organizations.md) - Understanding token scope
- [Applications](applications.md) - Restricting tokens to specific apps
