---
title: Security
description: Security measures in Hook0
---

# Security

Hook0 uses TLS encryption and HTTPS protocol to protect against various types of attacks, such as man-in-the-middle and replay attacks. It also uses HMAC-SHA-256 to sign webhooks and stores events and webhooks deliveries for audit and incident resolution purposes.

## Communication Encryption

In order to secure the transmission of data between Hook0, customer's subscriptions and customer's applications, Hook0 uses TLS (Transport Layer Security) versions 1.2 and 1.3 for both the API and web application. TLS is a cryptographic protocol that ensures the confidentiality and integrity of data as it is transmitted over the internet. Additionally, the HTTPS (Hypertext Transfer Protocol Secure) protocol is required to further protect against potential attacks.

Nothing below TLS 1.2 is served, on any hostname.

The cipher suites built on CBC with a SHA-1 MAC are a different matter, and
scanners report them against Hook0 Cloud regularly. They are right. The
hostnames served through our CDN negotiate `ECDHE-ECDSA-AES128-SHA` under
TLS 1.2, and we keep them on purpose.

The reason is the traffic. Over a day, 99.7% of requests reach us over TLS 1.3
and a fraction of a percent over TLS 1.2. That fraction is not idle. It is
customer integrations calling the API and being served. Removing those suites
means either dropping TLS 1.2 itself, which stops those integrations at the
handshake with no warning they can act on, or a paid option on our CDN plan.
We would rather keep a working integration than improve a scan result. This
gets revisited when the traffic that depends on it goes away, not on a date we
would invent today. The origin our apex is served from refuses those suites
already, which is why one hostname behaves differently from the rest.

What holds alongside that, and what does not:

- TLS 1.0 and 1.1 are refused by the server, on every hostname.
- Suites offering no encryption, and suites offering no authentication, are
  refused.
- Encrypt-then-MAC (RFC 7366) is **not** negotiated on those CBC suites. They
  run MAC-then-Encrypt, which is the case RFC 9325 singles out, so what stands
  between them and a padding-oracle timing attack is the edge's constant-time
  verification rather than the protocol itself. That code is not ours and we
  cannot check it from outside, so we are not going to tell you it is fine.
- The SHA-1 in those suites is an HMAC. HMAC security does not rest on
  collision resistance, so the collision work published against SHA-1 does not
  carry over to it.

That posture is measured rather than asserted:
`tests-e2e/tests/website/tls-posture.spec.ts` discovers the hostnames the site
links to and, for each, offers those suites and an obsolete protocol version.
Every outcome has to come from the server, either a completed handshake or an
alert the server sent. A probe that offers suites the client itself cannot put on the
wire dies locally and reads exactly like a refusal, which is how an earlier
version of that file reported a clean posture it had never observed.

## Rate-limiting

Hook0 implements four rate limiters to control the flow of incoming requests. Three of them apply to every request. The global limiter caps the total number of requests per second that an instance can handle, while the per IP rate-limiter and the per token rate-limiter key their quota on the caller and on the token respectively. The fourth one guards a couple of endpoints only, and is described below.

:::note

The per token limiter is only applied to requests that are successfully authenticated.

:::

All incoming requests are processed by the first three limiters. By default, the global limiter allows more requests per second than the per IP limiter, which, in turn, allows more requests per second than the per token limiter.

The fourth limiter sits in front of the endpoints that send an email to an address the caller names, which today means `POST /api/v1/auth/begin-reset-password` and `POST /api/v1/auth/resend-verification-email`. It is keyed on the caller's IP address, and allows a burst of 5 calls with one call restored every 60 seconds.

Those two endpoints answer identically whatever address they are given, so a caller learns nothing about which addresses have an account. What is left worth abusing is the volume, because walking a list of addresses costs Hook0 an email every time, and the people behind those addresses receive mail they never asked for. The per IP limiter in front of the whole API is sized for API traffic, and a sweep of a few hundred addresses stays comfortably under it, which is why this fourth quota exists. Hook0 also holds a per-account cooldown and daily allowance in the database, but those bound what a single mailbox receives and see nothing of a caller walking thousands of distinct addresses.

If your instance sits behind a reverse proxy, set `REVERSE_PROXY_IPS` to the address of that proxy. Without it, every request arrives wearing the proxy's address and the whole internet shares one per IP quota. Set to a range wider than your proxy, it lets anyone inside that range announce whatever address they like and walk past both per IP limiters.

:::note

All four limiters can be customized or disabled according to your specific needs. The configuration variables are listed in the [configuration reference](../reference/configuration.md); you can also read them by running the API with the "--help" option.

:::

We recommend against disabling all four limiters, as this may pose a significant risk. Depending on your system's characteristics, it may be acceptable to disable one or more of the first three, particularly if your instance is not publicly accessible.

The fourth one deserves its own decision. The first three bound load, so switching one off costs you throughput you can measure and take back the day it hurts. `DISABLE_API_RATE_LIMITING_EMAIL` gives up something you cannot measure the same way, since it is what keeps an attacker from harvesting your user base one address at a time, and from spending your mail credit on strangers. Note that `DISABLE_API_RATE_LIMITING` switches it off along with the other three, which is easy to overlook. Wherever your sign-up and password reset forms are reachable, leave the email limiter on, even on an instance you consider private.

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
