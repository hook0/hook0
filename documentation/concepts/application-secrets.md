---
title: Application Secrets
description: Cryptographic tokens for signing webhook payloads in Hook0
---

# Application Secrets

An application secret is a cryptographic token used to sign webhook payloads. When Hook0 delivers a webhook, it uses the secret to generate an HMAC signature, allowing recipients to verify the payload authenticity and integrity.

## Key Points

- Each [application](applications.md) can have multiple secrets for key rotation
- Secrets are used to sign webhook payloads with HMAC-SHA256
- Consumers verify signatures to ensure payload integrity
- Revoking a secret immediately invalidates all webhooks signed with it

## Why Signatures Matter

Without signature verification, webhook endpoints are vulnerable to:

- **Spoofing** - Attackers sending fake webhooks
- **Tampering** - Payload modification in transit
- **Replay attacks** - Resending captured webhooks

Signature verification ensures:

1. The webhook originates from Hook0
2. The payload hasn't been modified
3. The webhook is fresh (timestamp validation)

## How Signing Works

```
Event Payload + Timestamp
         |
         v
+-------------------+
|   HMAC-SHA256     |
|   (secret key)    |
+-------------------+
         |
         v
    Signature
         |
         v
+-------------------+
|  hook0-signature  |
|  header added     |
+-------------------+
         |
         v
   Webhook Sent
```

The signature header contains:
- **Timestamp** - When the signature was generated
- **Signature** - HMAC-SHA256 hash of timestamp + payload

## Secret Rotation

To rotate secrets without downtime:

1. Create a new secret
2. Update consumers to accept both secrets
3. Wait for in-flight webhooks to complete
4. Revoke the old secret

This ensures zero-downtime rotation while maintaining security.

## Security Considerations

- **Store securely** - Treat secrets like passwords
- **Never expose** - Don't log or display secrets
- **Rotate periodically** - Create new secrets regularly
- **Revoke compromised** - Immediately revoke leaked secrets

:::warning Save the Token
The secret is displayed only once at creation time. Store it securely before leaving the page.
:::

## What's Next?

- [Secure Webhook Endpoints](/how-to-guides/secure-webhook-endpoints) - Complete verification guide
- [Applications](applications.md) - Managing your applications
- [Subscriptions](subscriptions.md) - Configuring webhook delivery
