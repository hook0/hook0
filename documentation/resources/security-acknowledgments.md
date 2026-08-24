---
title: Security Acknowledgments
description: Researchers who reported a security vulnerability in the Hook0 API through our vulnerability disclosure policy.
keywords: [security acknowledgments, hall of fame, security researchers, responsible disclosure, hook0 security]
---

# Security acknowledgments

This page lists the researchers who reported a vulnerability to us through our [vulnerability disclosure policy](vulnerability-disclosure-policy.md) and whose report we acted on.

Severity plays no part in who gets an entry. A report we accept and fix earns one whatever its rating. The reward described in the policy is a separate matter, and it stays reserved for critical findings. If you would rather not be listed, tell us and we will leave you out.

## Researchers

- Eqan Chauhan, who reported that values a user controls were substituted into transactional emails as raw markup.
- tonghuaroot, who reported that IPv6 transition addresses could name a forbidden IPv4 address and get past the address check that guards webhook delivery.
- Sagar Kirola, who reported that a password reset link stayed usable once it had already reset a password, so whoever got hold of a spent link could set the password again.
- Nishant Lungare, who reported that the password reset endpoint answered one way for an address with an account and another way for an address without one, and that nothing bounded the number of messages it could be made to send.
