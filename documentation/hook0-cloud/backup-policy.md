---
title: "Backup policy"
slug: "backup-policy"
excerpt: ""
hidden: false
metadata: 
  image: []
  robots: "index"
createdAt: "Sat Jul 22 2023 07:13:29 GMT+0000 (Coordinated Universal Time)"
updatedAt: "Tue Jul 25 2023 12:47:14 GMT+0000 (Coordinated Universal Time)"
---
## Summary

The backup policy defines rules and principles on how backups are configured. 

The system administrator is responsible for implementing the policy.

## Principles

Hook0 backup strategy is based on the 7 principles:

- Coverage: all data is recorded
- Frequency: backup are run each day
- Separation: backups are stored in multi-region distributed system (S3-like) on CleverCloud fr-par (European Cloud Provider Company)
- History: each backup is persisted for 30 days
- Testing: backup are automatically and manually tested
- Security: backup are encrypted
- Integrity : backup integrity is ensured (e.g. an attacker can't change its content)

Each backup has:

- A name describing data contents
- A date of the data backup
- Storage location

Data restoration using data backups is tested every month to ensure that complete data restoration on a system separated from the test network (to avoid exposition of production data) is possible to ensure whether:

- Data restoration is possible
- Hook0 Internal Data backup procedure is practicable
- Hook0 Data backup procedures are documented properly
- Time required for data restoration meets the availability requirements

Backup and recovery documentation is reviewed and updated regularly to account for new technology, business changes, and migration of applications to alternative platforms.
