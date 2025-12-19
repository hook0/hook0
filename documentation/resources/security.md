---
title: Security
description: Security measures in Hook0
---

# Security

Hook0 uses TLS encryption and HTTPS protocol to protect against various types of attacks, such as man-in-the-middle and replay attacks. It also uses HMAC-SHA-256 to sign webhooks and stores events and webhooks deliveries for audit and incident resolution purposes.

## Communication Encryption

In order to secure the transmission of data between Hook0, customer's subscriptions and customer's applications, Hook0 uses TLS (Transport Layer Security) versions 1.2 and 1.3 for both the API and web application. TLS is a cryptographic protocol that ensures the confidentiality and integrity of data as it is transmitted over the internet. Additionally, the HTTPS (Hypertext Transfer Protocol Secure) protocol is required to further protect against potential attacks.

## Rate-limiting

Hook0 implements three rate limiters to control the flow of incoming requests. These include the global limiter, which limits the total number of requests per second that an instance can handle, as well as the per IP rate-limiter and per token rate-limiter.

:::note

The per token limiter is only applied to requests that are successfully authenticated.

:::

All incoming requests are processed by these three limiters. By default, the global limiter allows more requests per second than the per IP limiter, which, in turn, allows more requests per second than the per token limiter.

:::note

All three limiters can be customized or disabled according to your specific needs. You can access the configuration variables by running the API with the "--help" option or by reading the source code.

:::

We recommend against disabling all three limiters, as this may pose a significant risk. However, depending on your system's characteristics, it may be acceptable to disable one or more of them, particularly if your instance is not publicly accessible.

## Protection against attacks

Hook0 offers solutions to protect against various types of attacks that may attempt to compromise the security of communications between clients and servers. These solutions include protection against man-in-the-middle (MITM) attacks, in which an attacker intercepts and alters communications between two parties.

Hook0 also protects against forged request attacks, in which an attacker attempts to send unauthorized requests to the server, and replay attacks, in which an attacker captures and resends valid requests to the server in an attempt to trick it into performing unintended actions.

See more details in [Consuming Webhooks](/how-to-guides/secure-webhook-endpoints).

## Webhook security

Webhooks are a way for a server to send real-time notifications to a client when certain events occur. To ensure the integrity of these notifications, Hook0 uses an HMAC (Hash-based Message Authentication Code) signed with the SHA-256 (Secure Hash Algorithm) algorithm.

This helps to prevent an attacker from altering the content of the notification as it is transmitted over the internet.

See more details in [Consuming Webhooks](/how-to-guides/secure-webhook-endpoints).

## Event and webhook delivery storage

Hook0 stores events and webhook deliveries with HTTP responses or errors in order to assist with audit processes and to help resolve any issues that may arise.

This information can be useful for tracking the delivery of webhooks and for debugging any problems that may occur.

This feature is particularly helpful for new users who may be unfamiliar with the system and may need assistance with troubleshooting.
