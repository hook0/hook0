---
title: "Logging policy"
slug: "logging-policy"
excerpt: ""
hidden: false
metadata: 
  image: []
  robots: "index"
createdAt: "Sat Jul 22 2023 12:59:44 GMT+0000 (Coordinated Universal Time)"
updatedAt: "Sat Jul 22 2023 12:59:44 GMT+0000 (Coordinated Universal Time)"
---
## Summary

The logging policy defines requirements for logging and monitoring.

The policy is applicable to all internal Systems and (cloud) services.

## Principles

### Logging

- Recording (special) PII in log files should be avoided
- Access to special PII should be logged
- Log files should be protected from deletion or modification
- All logging systems should be synchronized with the same NTP source
- Log files are kept at least 30 days

### Monitoring

Usage logs of access to special PII should be monitored by the respective system owner
