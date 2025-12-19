---
title: "Cryptography policy"
slug: "cryptography-policy"
excerpt: ""
hidden: false
metadata: 
  image: []
  robots: "index"
createdAt: "Sat Jul 22 2023 12:33:30 GMT+0000 (Coordinated Universal Time)"
updatedAt: "Sat Jul 22 2023 12:33:30 GMT+0000 (Coordinated Universal Time)"
---
## Summary

This defines when, where and how cryptography is used in our organization, and how key management is done.

Technical staff is responsible for implementing the policy.

## Principles

Following the Information classification policy, encryption must be used to protect information classified as Confidential or Sensitive, at rest or in motion.

### Requirements for certificates

- The maximum duration for all certificates (signing, SSL/TLS) is 1 year
- The use of wildcard certificates is not allowed
- All certificates should have a key length of at least 2048 bits
- Certificates should be configured to be automatically renewed

### Requirements for keys

- Encryption keys (e.g. for encryption of backups or workstations) should be stored centrally

### Requirements for web services

- All public facing web sites are scanned each quarter using ssllabs.com, a score of "A" is considered minimum

### Requirements for email

- All email domains are scanned each quarter using mxtoolbox.com or internet.nl, critical problems must be resolved
