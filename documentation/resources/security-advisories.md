---
title: Security Advisories
description: Security advisories for Hook0, published after each vulnerability was fixed, with the affected code, the fix, and the reporter credited.
keywords: [security advisories, CVE, hook0 security, vulnerability, SSRF, webhook security, coordinated disclosure, GHSA]
---

# Security advisories

When a vulnerability affected users, we publish an advisory once the fix has
shipped. Each entry says what was wrong, which code was affected, what fixed it,
and who reported it.

Hook0 runs the same code on `app.hook0.com` and in a self-hosted deployment.
Hook0 Cloud is patched when the fix merges. A self-hosted deployment builds from
the repository, so the boundary is the fix commit. A build made before it is
affected, and updating to a build that includes it closes the issue.

Where a finding warrants one, a CVE is requested through GitHub Security
Advisories, where GitHub acts as a CVE Numbering Authority, and added to the
entry here once assigned. How to report a vulnerability, and what is in scope,
is on the [vulnerability disclosure policy](vulnerability-disclosure-policy.md)
page. Everyone whose report we act on is credited on the
[security acknowledgments](security-acknowledgments.md) page.

## Race condition bypasses the plan application limit

- **Reported by:** Sagar Kirola
- **Severity:** Low
- **Weakness:** CWE-362 (race condition), leading to CWE-770 (allocation without limits)
- **CVE:** to be assigned
- **Fixed:** commit `64f699f1`, 2026-08-25

Creating an application read the plan's application count and then inserted the
new application, with nothing held between the two steps. Requests sent at the
same moment each read the same count, each saw room under the limit, and each
inserted, so a tenant could hold more applications than the plan allows. The fix
keeps the count check and the insert it guards in one transaction.

## Password reset link stays usable after it has reset a password

- **Reported by:** Sagar Kirola, Eqan Chauhan
- **Severity:** Medium
- **Weakness:** CWE-640 (weak password recovery), CWE-613 (insufficient session expiration)
- **CVE:** to be assigned
- **Fixed:** commit `80fae0de`, 2026-08-24

A reset link was not invalidated once it had been used, so a link that had
already reset a password still set it again within its lifetime, and whoever
obtained a spent link could take over the account. Two related weaknesses sat in
the same flow: a link outlived the expiry its own email stated, and issuing a
newer link did not retire the earlier one. The fix binds each link to a nonce
held on the account row and rotates it on every write that sets a password, so
using a link, requesting a new one, or changing the password from account
settings all retire any outstanding links.

## Password reset endpoint reveals whether an address has an account

- **Reported by:** Nishant Lungare
- **Severity:** Low
- **Weakness:** CWE-204 (observable response discrepancy)
- **CVE:** to be assigned
- **Fixed:** commit `80fae0de`, 2026-08-24

The password reset endpoint answered one way for an address that had an account
and another for one that did not, which let an unauthenticated caller decide
whether an address was registered. The fix makes the endpoint answer the same
way in both cases.

## Server-side request forgery via IPv6 transition addresses

- **Reported by:** tonghuaroot
- **Severity:** High
- **Weakness:** CWE-918 (server-side request forgery)
- **CVE:** to be assigned
- **Fixed:** commit `3b27e932`, 2026-08-11

Hook0 delivers webhooks to a tenant-supplied URL and screens the target address
to keep delivery from reaching forbidden IPv4 ranges. The screen matched the
literal IPv4 form only, so an IPv6 transition address (an IPv4-mapped or
IPv4-compatible IPv6 address) naming the same forbidden host passed it, and the
worker connected. A tenant could use this to make the delivery worker reach
addresses on the internal network that the filter was meant to block. The fix
canonicalises the resolved address before screening.

## User-controlled values injected into transactional emails as raw markup

- **Reported by:** Eqan Chauhan
- **Severity:** Medium
- **Weakness:** CWE-116 (improper output encoding), CWE-79
- **CVE:** to be assigned
- **Fixed:** commit `d899f54b`, 2026-08-06

Values a user controls, such as a name substituted into a transactional email,
were placed into the email body without being escaped for the surrounding
markup, so a value carrying markup rendered as markup rather than text. The fix
escapes user-controlled values where they are substituted into the template.
