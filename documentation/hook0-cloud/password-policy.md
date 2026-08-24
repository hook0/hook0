---
title: "Password policy"
slug: "password-policy"
excerpt: ""
hidden: false
metadata: 
  image: []
  robots: "index"
createdAt: "Sat Jul 22 2023 12:56:59 GMT+0000 (Coordinated Universal Time)"
updatedAt: "Sat Jul 22 2023 13:39:13 GMT+0000 (Coordinated Universal Time)"
---
## Summary

The password policy defines requirements for passwords.

The policy is applicable to all internal and external personnel and Systems and (cloud) services holding information classified as Confidential or Sensitive.

## Principles

- Passwords should be strong: length is what buys strength, so prefer a long passphrase over a short password decorated with symbols
- Do not use the same password for more than one service or system
- Change the password at least twice per year
- Do not use variants of the old password (e.g. adding a number to the old password)
- The use of **Bitwarden password manager is mandatory**.

## What Hook0 enforces on account passwords

Every path that sets a password — registration, password reset and password
change — applies the same rules. Following
[NIST SP 800-63B](https://pages.nist.gov/800-63-3/sp800-63b.html), they are a
length floor plus blocklists rather than composition rules ("one uppercase, one
digit, one symbol"), which mostly push people towards predictable substitutions.

A password is refused when it is:

- shorter than the instance's minimum (`PASSWORD_MINIMUM_LENGTH`, 12 by default)
  or longer than 100 characters;
- the account's own email address or name — or built around either, unless what
  surrounds that fragment is still a secret of its own;
- among the ten thousand most common passwords, including disguised
  (`P@ssw0rd`, `1etmein`) and padded (`letmein2026!`, `2026letmein`) variants;
- a shorter unit typed several times (`abcdabcdabcd`), which clears the length
  floor while carrying almost no entropy;
- carrying a control character — a tab or a line break, usually pasted in by
  accident rather than typed.

Each refusal names the rule it broke on the password field. The first four
carry their own error identifier, listed in the
[error codes reference](../reference/error-codes.md); the last is reported as
malformed input, since it is about the characters the field accepts rather
than about the strength of the password.

Only the minimum length is configurable; the other rules always apply. Existing
passwords are never re-checked — the rules apply the next time a password is set.

## Replacing an existing password

Clearing the rules above is not the whole of it. Each of the two paths that
replaces a password an account already has carries a precondition of its own.

Changing the password from the account settings takes the current password
alongside the new one. Holding a session is not enough on its own, so a token
that leaks does not hand over the account. A wrong current password answers the
same 403 an unusable session answers, which keeps a caller from learning which
of the two it got wrong.

A reset link works once, and only the newest one works. The link carries a value
the account row holds, and every write that sets a password rotates that value,
so:

- a link that has already reset a password stops working;
- asking for another link retires the ones issued before it;
- changing the password from the account settings retires whatever links are
  still outstanding.

All three answer the expired-link error, listed in the [error codes
reference](../reference/error-codes.md). A link also expires 30 minutes after it
is issued, and the mail carrying it says so.

Reset mail is rationed per address, at one message per minute and five per
24 hours. Past either bound the endpoint answers exactly what it answers for an
address with no account, and sends nothing.

## Server-side password storage

- All user passwords are hashed using Argon2 with default parameters (memory-hard, resistant to GPU/ASIC attacks)
- Passwords are never stored in plaintext or with reversible encryption
- Each password is salted with a unique, randomly generated salt
