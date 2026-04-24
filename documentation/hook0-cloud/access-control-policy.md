---
title: "Access control policy"
slug: "access-control-policy"
excerpt: ""
hidden: false
metadata: 
  image: []
  robots: "index"
createdAt: "Sat Jul 22 2023 07:03:28 GMT+0000 (Coordinated Universal Time)"
updatedAt: "Sat Jul 22 2023 07:19:19 GMT+0000 (Coordinated Universal Time)"
---
## Summary

The access control policy defines rules and principles upon which access rights and restrictions are configured.

The policy is applicable to all internal and external personnel.

## Principles

- Access is granted based upon the need-to-know/need-to-use principle.
- Based on the person's role, access to information/assets is granted.
- Access is granted/revoked upon management request. The user can request access himself, but then the request has to be approved by his/her manager first.
- As part of the HR on boarding process and HR off boarding process, access rights will be granted/revoked as well.
- Logical access lists should be reviewed by the system owner as defined in the Systems and (cloud) services overview, the frequency depends on the classification of the information:
  - Sensitive or Confidential: 3 months
  - Internal: 6 months
  - Public: 12 months
- 2FA (Two Factor Authentication) must be used for systems holding Sensitive information

## Multi-Factor Authentication (MFA) implementation status

- MFA is enforced for all infrastructure access: Clever Cloud (hosting), GitLab (source code), Stripe (billing), and all administrative tools
- MFA for individual Hook0 customer accounts is not yet implemented and is planned for a future release
- Until customer-facing MFA is available, strong password requirements (Argon2 hashing, minimum complexity) and session expiration provide baseline protection

## Role-Based Access Control (RBAC)

- The Hook0 platform implements RBAC using Biscuit capability-based authorization tokens
- Access to customer data follows the need-to-know principle
- Platform roles are defined per organization with granular permissions

## Requirements for user IDs

- User names are created by concatenating the user's first and last name, separated by a period, all in lower case (e.g. maurice.pasman)
- User names may not be shared or reused for systems holding Confidential or Sensitive information

## Requirements for password systems

- If the password was manually set by an administrator/system owner, the user must be required to change the password at first login
- The user should be enforced to use quality passwords (as per Password policy)
- The reuse of the last three passwords should be prevented
- The passwords should be stored in a secure way (encrypted/hashed and separated from other data)  
- Login information should be transmitted encrypted (using TLS)
