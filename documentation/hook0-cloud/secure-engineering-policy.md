---
title: "Secure engineering policy"
slug: "secure-engineering-policy"
excerpt: ""
hidden: false
metadata: 
  image: []
  robots: "index"
createdAt: "Sat Jul 22 2023 12:36:38 GMT+0000 (Coordinated Universal Time)"
updatedAt: "Sat Jul 22 2023 12:36:38 GMT+0000 (Coordinated Universal Time)"
---
## Summary

The secure engineering policy defines rules and principles applied in design and engineering of systems, networks and infrastructure.

The policy applies to all network engineers, internal and external.

## Principles

- Network architecture and designs should always be peer reviewed
- The four eyes-principle should be applied when medium and high impact network changes take place
- Engineers should stay up-to-date of vulnerabilities in used networking technology (firewalls, routers, ...)
- Engineers should not have access to Sensitive data
- Software versions that no longer have security patches released are prohibited

### Cryptography and authentication

- **Password hashing**: Argon2 (state-of-the-art, memory-hard function resistant to GPU and ASIC attacks)
- **Authorization tokens**: Biscuit (capability-based, decentralized authorization tokens)
- **TLS implementation**: rustls with TLS 1.2+ minimum and post-quantum cryptography support
- **SMTP security**: TLS via rustls with AWS LC-RS cryptographic backend

### Monitoring and incident detection

- **Error tracking**: Sentry for real-time error monitoring and alerting
- **Distributed tracing**: OpenTelemetry for distributed traces and metrics with OTLP export
- **Uptime monitoring**: BetterUptime for external uptime monitoring and incident alerting
